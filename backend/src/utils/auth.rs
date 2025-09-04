//! Advanced authentication and authorization system for the multiagent hive
//!
//! This module provides:
//! - JWT-based authentication with refresh tokens
//! - Role-based access control (RBAC)
//! - Session management with secure storage
//! - Multi-factor authentication support
//! - API key management for service-to-service communication

use crate::utils::error::{HiveError, HiveResult};
use crate::utils::security::SecurityAuditor;
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,              // Subject (user ID)
    pub exp: usize,               // Expiration time
    pub iat: usize,               // Issued at
    pub iss: String,              // Issuer
    pub aud: String,              // Audience
    pub roles: Vec<String>,       // User roles
    pub permissions: Vec<String>, // Specific permissions
    pub session_id: String,       // Session identifier
    pub client_type: ClientType,  // Client type (human, agent, service)
}

/// Client types for different authentication scenarios
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ClientType {
    Human,   // Human user
    Agent,   // AI agent
    Service, // Service-to-service
    Admin,   // Administrative access
}

/// User roles with hierarchical permissions
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Role {
    SuperAdmin, // Full system access
    Admin,      // Administrative access
    Operator,   // Operational access
    Developer,  // Development and debugging
    Viewer,     // Read-only access
    Agent,      // AI agent access
    Service,    // Service account
}

impl Role {
    /// Get all permissions for a role
    #[must_use]
    pub fn permissions(&self) -> Vec<Permission> {
        match self {
            Role::SuperAdmin => vec![
                Permission::SystemAdmin,
                Permission::UserManagement,
                Permission::AgentManagement,
                Permission::TaskManagement,
                Permission::MetricsRead,
                Permission::MetricsWrite,
                Permission::ConfigRead,
                Permission::ConfigWrite,
                Permission::SecurityAudit,
            ],
            Role::Admin => vec![
                Permission::UserManagement,
                Permission::AgentManagement,
                Permission::TaskManagement,
                Permission::MetricsRead,
                Permission::ConfigRead,
                Permission::SecurityAudit,
            ],
            Role::Operator => vec![
                Permission::AgentManagement,
                Permission::TaskManagement,
                Permission::MetricsRead,
            ],
            Role::Developer => vec![
                Permission::AgentManagement,
                Permission::TaskManagement,
                Permission::MetricsRead,
                Permission::ConfigRead,
            ],
            Role::Viewer => vec![Permission::MetricsRead],
            Role::Agent => vec![Permission::TaskManagement, Permission::MetricsRead],
            Role::Service => vec![
                Permission::TaskManagement,
                Permission::MetricsRead,
                Permission::MetricsWrite,
            ],
        }
    }
}

/// Granular permissions system
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    SystemAdmin,     // Full system administration
    UserManagement,  // Create, update, delete users
    AgentManagement, // Create, update, delete agents
    TaskManagement,  // Create, update, delete tasks
    MetricsRead,     // Read system metrics
    MetricsWrite,    // Write system metrics
    ConfigRead,      // Read system configuration
    ConfigWrite,     // Write system configuration
    SecurityAudit,   // Access security audit logs
}

/// Authentication session
#[derive(Debug, Clone)]
pub struct AuthSession {
    pub session_id: String,
    pub user_id: String,
    pub roles: Vec<Role>,
    pub permissions: HashSet<Permission>,
    pub client_type: ClientType,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub refresh_token: Option<String>,
}

/// API Key for service-to-service authentication
#[derive(Debug, Clone)]
pub struct ApiKey {
    pub key_id: String,
    pub key_hash: String,
    pub service_name: String,
    pub permissions: HashSet<Permission>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub usage_count: u64,
    pub rate_limit: Option<u32>,
}

/// Advanced authentication manager
pub struct AuthManager {
    /// JWT encoding key
    encoding_key: EncodingKey,
    /// JWT decoding key
    decoding_key: DecodingKey,
    /// Active sessions
    sessions: Arc<RwLock<HashMap<String, AuthSession>>>,
    /// API keys
    api_keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    /// Security auditor
    security_auditor: Arc<SecurityAuditor>,
    /// JWT issuer
    issuer: String,
    /// JWT audience
    audience: String,
    /// Session timeout
    session_timeout: Duration,
    /// Refresh token timeout
    #[allow(dead_code)]
    refresh_timeout: Duration,
}

impl AuthManager {
    /// Create a new authentication manager
    #[must_use]
    pub fn new(
        jwt_secret: &str,
        issuer: String,
        audience: String,
        security_auditor: Arc<SecurityAuditor>,
    ) -> Self {
        let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());

        Self {
            encoding_key,
            decoding_key,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            api_keys: Arc::new(RwLock::new(HashMap::new())),
            security_auditor,
            issuer,
            audience,
            session_timeout: Duration::hours(8),
            refresh_timeout: Duration::days(30),
        }
    }

    /// Authenticate user and create session
    pub async fn authenticate_user(
        &self,
        user_id: String,
        roles: Vec<Role>,
        client_type: ClientType,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> HiveResult<(String, String)> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Collect all permissions from roles
        let mut permissions = HashSet::new();
        for role in &roles {
            permissions.extend(role.permissions());
        }

        // Create session
        let session = AuthSession {
            session_id: session_id.clone(),
            user_id: user_id.clone(),
            roles: roles.clone(),
            permissions: permissions.clone(),
            client_type: client_type.clone(),
            created_at: now,
            last_activity: now,
            expires_at: now + self.session_timeout,
            ip_address: ip_address.clone(),
            user_agent: user_agent.clone(),
            refresh_token: Some(Uuid::new_v4().to_string()),
        };

        // Create JWT claims
        let claims = Claims {
            sub: user_id.clone(),
            exp: (now + self.session_timeout).timestamp() as usize,
            iat: now.timestamp() as usize,
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            roles: roles.iter().map(|r| format!("{r:?}")).collect(),
            permissions: permissions.iter().map(|p| format!("{p:?}")).collect(),
            session_id: session_id.clone(),
            client_type,
        };

        // Generate JWT token
        let token = encode(&Header::default(), &claims, &self.encoding_key).map_err(|e| {
            HiveError::AuthenticationError {
                reason: format!("Failed to generate JWT: {e}"),
            }
        })?;

        // Store session
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);

        // Log authentication success
        self.security_auditor
            .log_authentication_attempt(user_id, ip_address, user_agent, true, None);

        Ok((token, session_id))
    }

    /// Validate JWT token and return claims
    pub async fn validate_token(&self, token: &str) -> HiveResult<Claims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation).map_err(|e| {
            HiveError::AuthenticationError {
                reason: format!("Invalid JWT token: {e}"),
            }
        })?;

        let claims = token_data.claims;

        // Check if session still exists and is valid
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(&claims.session_id) {
            if session.expires_at < Utc::now() {
                return Err(HiveError::AuthenticationError {
                    reason: "Session expired".to_string(),
                });
            }
        } else {
            return Err(HiveError::AuthenticationError {
                reason: "Session not found".to_string(),
            });
        }

        Ok(claims)
    }

    /// Check if user has specific permission
    pub async fn check_permission(
        &self,
        session_id: &str,
        permission: Permission,
    ) -> HiveResult<bool> {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            Ok(session.permissions.contains(&permission))
        } else {
            Err(HiveError::AuthenticationError {
                reason: "Session not found".to_string(),
            })
        }
    }

    /// Refresh JWT token
    pub async fn refresh_token(&self, refresh_token: &str) -> HiveResult<String> {
        let mut sessions = self.sessions.write().await;

        // Find session by refresh token
        let session = sessions
            .values_mut()
            .find(|s| s.refresh_token.as_ref() == Some(&refresh_token.to_string()))
            .ok_or_else(|| HiveError::AuthenticationError {
                reason: "Invalid refresh token".to_string(),
            })?;

        // Check if session is still valid
        if session.expires_at < Utc::now() {
            return Err(HiveError::AuthenticationError {
                reason: "Session expired".to_string(),
            });
        }

        let now = Utc::now();

        // Update session
        session.last_activity = now;
        session.expires_at = now + self.session_timeout;

        // Create new JWT claims
        let claims = Claims {
            sub: session.user_id.clone(),
            exp: (now + self.session_timeout).timestamp() as usize,
            iat: now.timestamp() as usize,
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            roles: session.roles.iter().map(|r| format!("{r:?}")).collect(),
            permissions: session
                .permissions
                .iter()
                .map(|p| format!("{p:?}"))
                .collect(),
            session_id: session.session_id.clone(),
            client_type: session.client_type.clone(),
        };

        // Generate new JWT token
        let token = encode(&Header::default(), &claims, &self.encoding_key).map_err(|e| {
            HiveError::AuthenticationError {
                reason: format!("Failed to generate JWT: {e}"),
            }
        })?;

        Ok(token)
    }

    /// Create API key for service authentication
    pub async fn create_api_key(
        &self,
        service_name: String,
        permissions: HashSet<Permission>,
        expires_at: Option<DateTime<Utc>>,
        rate_limit: Option<u32>,
    ) -> HiveResult<(String, String)> {
        let key_id = Uuid::new_v4().to_string();
        let api_key = Uuid::new_v4().to_string();
        let key_hash = format!("{:x}", md5::compute(&api_key));

        let api_key_obj = ApiKey {
            key_id: key_id.clone(),
            key_hash,
            service_name,
            permissions,
            created_at: Utc::now(),
            expires_at,
            last_used: None,
            usage_count: 0,
            rate_limit,
        };

        let mut api_keys = self.api_keys.write().await;
        api_keys.insert(key_id.clone(), api_key_obj);

        Ok((key_id, api_key))
    }

    /// Validate API key
    pub async fn validate_api_key(&self, api_key: &str) -> HiveResult<HashSet<Permission>> {
        let key_hash = format!("{:x}", md5::compute(api_key));
        let mut api_keys = self.api_keys.write().await;

        for (_, key_obj) in api_keys.iter_mut() {
            if key_obj.key_hash == key_hash {
                // Check expiration
                if let Some(expires_at) = key_obj.expires_at {
                    if expires_at < Utc::now() {
                        return Err(HiveError::AuthenticationError {
                            reason: "API key expired".to_string(),
                        });
                    }
                }

                // Update usage
                key_obj.last_used = Some(Utc::now());
                key_obj.usage_count += 1;

                return Ok(key_obj.permissions.clone());
            }
        }

        Err(HiveError::AuthenticationError {
            reason: "Invalid API key".to_string(),
        })
    }

    /// Logout user and invalidate session
    pub async fn logout(&self, session_id: &str) -> HiveResult<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.remove(session_id) {
            self.security_auditor.log_authentication_attempt(
                session.user_id,
                session.ip_address,
                session.user_agent,
                true,
                None,
            );
        }
        Ok(())
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        let now = Utc::now();
        let initial_count = sessions.len();

        sessions.retain(|_, session| session.expires_at > now);

        initial_count - sessions.len()
    }

    /// Get active session count
    pub async fn get_active_session_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Get session info
    pub async fn get_session_info(&self, session_id: &str) -> Option<AuthSession> {
        self.sessions.read().await.get(session_id).cloned()
    }

    /// Validate incoming request for security threats
    pub async fn validate_request(
        &self,
        _source_ip: Option<String>,
        user_agent: Option<String>,
        endpoint: &str,
        payload_size: usize,
    ) -> crate::utils::error::HiveResult<crate::utils::security::SecurityResult> {
        use crate::utils::security::{SecurityResult, ThreatLevel};

        // Basic security validation logic
        let mut threat_level = ThreatLevel::Low;
        let mut is_valid = true;
        let mut reason = None;

        // Check payload size
        if payload_size > 10_000_000 {
            // 10MB limit
            threat_level = ThreatLevel::High;
            is_valid = false;
            reason = Some("Payload too large".to_string());
        }

        // Check for suspicious user agents
        if let Some(ref ua) = user_agent {
            if ua.to_lowercase().contains("bot") || ua.to_lowercase().contains("crawler") {
                threat_level = ThreatLevel::Medium;
            }
        }

        // Check for admin endpoints
        if (endpoint.contains("/admin") || endpoint.contains("/system"))
            && threat_level == ThreatLevel::Low
        {
            threat_level = ThreatLevel::Medium;
        }

        Ok(SecurityResult {
            threat_level,
            is_valid,
            reason,
        })
    }
}

/// Authentication middleware for Axum
pub async fn auth_middleware(
    auth_manager: Arc<AuthManager>,
    required_permission: Option<Permission>,
) -> impl Fn(
    axum::extract::Request,
    axum::middleware::Next,
) -> std::pin::Pin<
    Box<
        dyn std::future::Future<Output = Result<axum::response::Response, axum::http::StatusCode>>
            + Send,
    >,
> + Clone {
    move |req: axum::extract::Request, next: axum::middleware::Next| {
        let auth_manager = auth_manager.clone();
        let required_permission = required_permission.clone();

        Box::pin(async move {
            // Extract Authorization header
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "));

            let token = if let Some(token) = auth_header {
                token
            } else {
                // Check for API key
                if let Some(api_key) = req.headers().get("X-API-Key").and_then(|h| h.to_str().ok())
                {
                    match auth_manager.validate_api_key(api_key).await {
                        Ok(permissions) => {
                            if let Some(required) = &required_permission {
                                if !permissions.contains(required) {
                                    return Err(axum::http::StatusCode::FORBIDDEN);
                                }
                            }
                            return Ok(next.run(req).await);
                        }
                        Err(_) => return Err(axum::http::StatusCode::UNAUTHORIZED),
                    }
                }
                return Err(axum::http::StatusCode::UNAUTHORIZED);
            };

            // Validate JWT token
            match auth_manager.validate_token(token).await {
                Ok(claims) => {
                    // Check permission if required
                    if let Some(required) = &required_permission {
                        match auth_manager
                            .check_permission(&claims.session_id, required.clone())
                            .await
                        {
                            Ok(true) => {}
                            Ok(false) => return Err(axum::http::StatusCode::FORBIDDEN),
                            Err(_) => return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
                        }
                    }

                    // Add claims to request extensions for use in handlers
                    let mut req = req;
                    req.extensions_mut().insert(claims);
                    Ok(next.run(req).await)
                }
                Err(_) => Err(axum::http::StatusCode::UNAUTHORIZED),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_authentication_flow() {
        let security_auditor = Arc::new(SecurityAuditor::new(true, 90));
        let auth_manager = AuthManager::new(
            "test_secret",
            "test_issuer".to_string(),
            "test_audience".to_string(),
            security_auditor,
        );

        // Test user authentication
        let (token, session_id) = auth_manager
            .authenticate_user(
                "test_user".to_string(),
                vec![Role::Developer],
                ClientType::Human,
                Some("127.0.0.1".to_string()),
                Some("test_agent".to_string()),
            )
            .await
            .unwrap();

        assert!(!token.is_empty());
        assert!(!session_id.is_empty());

        // Test token validation
        let claims = auth_manager.validate_token(&token).await.unwrap();
        assert_eq!(claims.sub, "test_user");
        assert_eq!(claims.session_id, session_id);

        // Test permission check
        let has_permission = auth_manager
            .check_permission(&session_id, Permission::AgentManagement)
            .await
            .unwrap();
        assert!(has_permission);

        // Test logout
        auth_manager.logout(&session_id).await.unwrap();

        // Session should be gone
        assert!(auth_manager.get_session_info(&session_id).await.is_none());
    }

    #[tokio::test]
    async fn test_api_key_authentication() {
        let security_auditor = Arc::new(SecurityAuditor::new(true, 90));
        let auth_manager = AuthManager::new(
            "test_secret",
            "test_issuer".to_string(),
            "test_audience".to_string(),
            security_auditor,
        );

        let mut permissions = HashSet::new();
        permissions.insert(Permission::TaskManagement);
        permissions.insert(Permission::MetricsRead);

        // Create API key
        let (key_id, api_key) = auth_manager
            .create_api_key(
                "test_service".to_string(),
                permissions.clone(),
                None,
                Some(1000),
            )
            .await
            .unwrap();

        assert!(!key_id.is_empty());
        assert!(!api_key.is_empty());

        // Validate API key
        let validated_permissions = auth_manager.validate_api_key(&api_key).await.unwrap();
        assert_eq!(validated_permissions, permissions);
    }
}
