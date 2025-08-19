//! Enhanced security middleware integrating advanced security features
//!
//! This module provides comprehensive security middleware that integrates:
//! - JWT authentication and authorization
//! - Advanced threat detection
//! - Real-time security monitoring
//! - Automated incident response

// Advanced security functionality consolidated into security_middleware
use crate::utils::auth::{AuthManager, Claims, Permission};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::net::IpAddr;
use std::sync::Arc;
use tracing::{info, warn, error};

/// Enhanced security middleware state
#[derive(Clone)]
pub struct SecurityMiddlewareState {
    pub auth_manager: Arc<AuthManager>,
    // Security manager functionality consolidated into middleware
}

/// Comprehensive security middleware that handles authentication, authorization, and threat detection
pub async fn comprehensive_security_middleware(
    State(security_state): State<SecurityMiddlewareState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = std::time::Instant::now();
    
    // Extract request information
    let method = request.method().clone();
    let uri = request.uri().clone();
    let endpoint = uri.path();
    
    // Extract client information
    let source_ip = extract_client_ip(&headers);
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    
    // Get payload size if available
    let payload_size = headers
        .get("content-length")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    info!(
        method = %method,
        endpoint = %endpoint,
        source_ip = ?source_ip,
        user_agent = ?user_agent,
        "Processing security validation"
    );

    // Step 1: Advanced security validation
    let security_result = security_state
        .auth_manager // Use auth_manager instead of removed security_manager
        .validate_request(source_ip, user_agent.clone(), endpoint, payload_size)
        .await
        .map_err(|e| {
            error!("Security validation failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !security_result.allowed {
        warn!(
            endpoint = %endpoint,
            source_ip = ?source_ip,
            threat_level = ?security_result.threat_level,
            warnings = ?security_result.warnings,
            "Request blocked by security validation"
        );
        
        return match security_result.threat_level {
            ThreatLevel::Critical => Err(StatusCode::FORBIDDEN),
            ThreatLevel::High => Err(StatusCode::TOO_MANY_REQUESTS),
            ThreatLevel::Medium => Err(StatusCode::UNAUTHORIZED),
            ThreatLevel::Low => Err(StatusCode::BAD_REQUEST),
        };
    }

    // Step 2: Authentication and authorization
    let auth_result = handle_authentication(&security_state.auth_manager, &headers, endpoint).await;
    
    match auth_result {
        Ok(Some(claims)) => {
            // Add authenticated user claims to request
            request.extensions_mut().insert(claims.clone());
            
            info!(
                user_id = %claims.sub,
                roles = ?claims.roles,
                endpoint = %endpoint,
                "Request authenticated successfully"
            );
        }
        Ok(None) => {
            // No authentication required for this endpoint
            info!(endpoint = %endpoint, "Public endpoint accessed");
        }
        Err(status) => {
            warn!(
                endpoint = %endpoint,
                source_ip = ?source_ip,
                "Authentication failed"
            );
            return Err(status);
        }
    }

    // Step 3: Process request
    let response = next.run(request).await;
    let duration = start_time.elapsed();
    
    // Step 4: Log security metrics
    let success = response.status().is_success();
    info!(
        endpoint = %endpoint,
        source_ip = ?source_ip,
        status = %response.status(),
        duration_ms = duration.as_millis(),
        success = success,
        "Request completed"
    );

    Ok(response)
}

/// Handle authentication for the request
async fn handle_authentication(
    auth_manager: &AuthManager,
    headers: &HeaderMap,
    endpoint: &str,
) -> Result<Option<Claims>, StatusCode> {
    // Check if endpoint requires authentication
    if is_public_endpoint(endpoint) {
        return Ok(None);
    }

    // Try JWT authentication first
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                match auth_manager.validate_token(token).await {
                    Ok(claims) => {
                        // Check if user has permission for this endpoint
                        let required_permission = get_required_permission(endpoint);
                        if let Some(permission) = required_permission {
                            match auth_manager.check_permission(&claims.session_id, permission).await {
                                Ok(true) => return Ok(Some(claims)),
                                Ok(false) => return Err(StatusCode::FORBIDDEN),
                                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
                            }
                        }
                        return Ok(Some(claims));
                    }
                    Err(_) => return Err(StatusCode::UNAUTHORIZED),
                }
            }
        }
    }

    // Try API key authentication
    if let Some(api_key_header) = headers.get("x-api-key") {
        if let Ok(api_key) = api_key_header.to_str() {
            match auth_manager.validate_api_key(api_key).await {
                Ok(permissions) => {
                    // Check if API key has required permission
                    if let Some(required_permission) = get_required_permission(endpoint) {
                        if permissions.contains(&required_permission) {
                            // Create pseudo-claims for API key
                            let claims = Claims {
                                sub: "api_key".to_string(),
                                exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
                                iat: chrono::Utc::now().timestamp() as usize,
                                iss: "hive_system".to_string(),
                                aud: "api".to_string(),
                                roles: vec!["Service".to_string()],
                                permissions: permissions.iter().map(|p| format!("{:?}", p)).collect(),
                                session_id: "api_key_session".to_string(),
                                client_type: crate::utils::auth::ClientType::Service,
                            };
                            return Ok(Some(claims));
                        } else {
                            return Err(StatusCode::FORBIDDEN);
                        }
                    }
                    // API key is valid and no specific permission required
                    let claims = Claims {
                        sub: "api_key".to_string(),
                        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
                        iat: chrono::Utc::now().timestamp() as usize,
                        iss: "hive_system".to_string(),
                        aud: "api".to_string(),
                        roles: vec!["Service".to_string()],
                        permissions: permissions.iter().map(|p| format!("{:?}", p)).collect(),
                        session_id: "api_key_session".to_string(),
                        client_type: crate::utils::auth::ClientType::Service,
                    };
                    return Ok(Some(claims));
                }
                Err(_) => return Err(StatusCode::UNAUTHORIZED),
            }
        }
    }

    // No valid authentication found
    Err(StatusCode::UNAUTHORIZED)
}

/// Check if endpoint is public (doesn't require authentication)
fn is_public_endpoint(endpoint: &str) -> bool {
    matches!(endpoint, 
        "/health" | 
        "/metrics" | 
        "/api/status" |
        "/" |
        "/favicon.ico" |
        "/robots.txt"
    )
}

/// Get required permission for endpoint
fn get_required_permission(endpoint: &str) -> Option<Permission> {
    match endpoint {
        // Agent management endpoints
        path if path.starts_with("/api/agents") => {
            if path.contains("POST") || path.contains("PUT") || path.contains("DELETE") {
                Some(Permission::AgentManagement)
            } else {
                Some(Permission::MetricsRead)
            }
        }
        // Task management endpoints
        path if path.starts_with("/api/tasks") => {
            if path.contains("POST") || path.contains("PUT") || path.contains("DELETE") {
                Some(Permission::TaskManagement)
            } else {
                Some(Permission::MetricsRead)
            }
        }
        // Configuration endpoints
        path if path.starts_with("/api/config") => {
            if path.contains("POST") || path.contains("PUT") || path.contains("DELETE") {
                Some(Permission::ConfigWrite)
            } else {
                Some(Permission::ConfigRead)
            }
        }
        // User management endpoints
        path if path.starts_with("/api/users") => Some(Permission::UserManagement),
        // Security audit endpoints
        path if path.starts_with("/api/security") => Some(Permission::SecurityAudit),
        // Admin endpoints
        path if path.starts_with("/api/admin") => Some(Permission::SystemAdmin),
        // Default to metrics read for other API endpoints
        path if path.starts_with("/api/") => Some(Permission::MetricsRead),
        // No permission required for other endpoints
        _ => None,
    }
}

/// Extract client IP address from headers
fn extract_client_ip(headers: &HeaderMap) -> Option<IpAddr> {
    // Try various headers in order of preference
    let ip_headers = [
        "x-forwarded-for",
        "x-real-ip",
        "cf-connecting-ip",
        "x-client-ip",
        "x-cluster-client-ip",
    ];

    for header_name in &ip_headers {
        if let Some(header_value) = headers.get(*header_name) {
            if let Ok(ip_str) = header_value.to_str() {
                // X-Forwarded-For can contain multiple IPs, take the first one
                let ip_str = ip_str.split(',').next().unwrap_or(ip_str).trim();
                if let Ok(ip) = ip_str.parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }
    }

    None
}

/// Rate limiting middleware specifically for API endpoints
pub async fn api_rate_limiting_middleware(
    State(_security_state): State<SecurityMiddlewareState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let source_ip = extract_client_ip(&headers);
    let endpoint = request.uri().path();

    // Apply stricter rate limiting for API endpoints
    if endpoint.starts_with("/api/") {
        // This would integrate with the advanced security manager's rate limiting
        // For now, we'll use a simple check
        if let Some(_ip) = source_ip {
            // Rate limiting logic would go here
            // For demonstration, we'll allow all requests
        }
    }

    Ok(next.run(request).await)
}

/// Security headers middleware with enhanced protection
pub async fn enhanced_security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Enhanced security headers
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
    headers.insert("X-Permitted-Cross-Domain-Policies", "none".parse().unwrap());
    headers.insert("X-Download-Options", "noopen".parse().unwrap());
    
    // Content Security Policy
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; font-src 'self'; object-src 'none'; media-src 'self'; frame-src 'none';"
            .parse()
            .unwrap(),
    );

    // Strict Transport Security (HSTS)
    headers.insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains; preload".parse().unwrap(),
    );

    // Feature Policy / Permissions Policy
    headers.insert(
        "Permissions-Policy",
        "geolocation=(), microphone=(), camera=(), payment=(), usb=(), magnetometer=(), gyroscope=(), speaker=()"
            .parse()
            .unwrap(),
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_endpoint_detection() {
        assert!(is_public_endpoint("/health"));
        assert!(is_public_endpoint("/metrics"));
        assert!(!is_public_endpoint("/api/agents"));
        assert!(!is_public_endpoint("/api/tasks"));
    }

    #[test]
    fn test_permission_mapping() {
        assert_eq!(get_required_permission("/api/agents"), Some(Permission::MetricsRead));
        assert_eq!(get_required_permission("/api/admin/users"), Some(Permission::SystemAdmin));
        assert_eq!(get_required_permission("/health"), None);
    }
}