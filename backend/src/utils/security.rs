//! Security utilities and audit logging for the multiagent hive system
//!
//! This module provides comprehensive security features including:
//! - Input sanitization and validation
//! - Security audit logging
//! - Authentication and authorization helpers
//! - Security headers and CORS configuration

use crate::utils::error::{HiveError, HiveResult};
use crate::utils::structured_logging::{SecurityEventDetails, SecurityEventType, StructuredLogger};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Threat level assessment for security validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Security validation result
#[derive(Debug, Clone)]
pub struct SecurityResult {
    pub threat_level: ThreatLevel,
    pub is_valid: bool,
    pub reason: Option<String>,
}

/// Security audit logger for tracking security-related events
#[derive(Debug, Clone)]
pub struct SecurityAuditor {
    /// Enable audit logging
    enabled: bool,
}

impl SecurityAuditor {
    /// Create a new security auditor
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Log a security event with comprehensive details
    pub fn log_security_event(
        &self,
        event_type: &SecurityEventType,
        client_id: String,
        endpoint: String,
        user_agent: Option<String>,
        ip_address: Option<String>,
        additional_info: HashMap<String, String>,
    ) {
        if !self.enabled {
            return;
        }

        let details = SecurityEventDetails {
            client_id,
            endpoint,
            user_agent,
            ip_address,
            timestamp: chrono::Utc::now(),
            additional_info,
        };

        StructuredLogger::log_security_event(&event_type, &details);
    }

    /// Log authentication attempt
    pub fn log_authentication_attempt(
        &self,
        client_id: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
        success: bool,
        failure_reason: Option<String>,
    ) {
        let mut additional_info = HashMap::new();
        additional_info.insert("success".to_string(), success.to_string());
        if let Some(reason) = failure_reason {
            additional_info.insert("failure_reason".to_string(), reason);
        }

        self.log_security_event(
            if success {
                SecurityEventType::AuthenticationSuccess
            } else {
                SecurityEventType::UnauthorizedAccess
            },
            client_id,
            "authentication".to_string(),
            user_agent,
            ip_address,
            additional_info,
        );
    }

    /// Log authorization check
    pub fn log_authorization_check(
        &self,
        client_id: String,
        resource: String,
        action: String,
        granted: bool,
        reason: Option<String>,
    ) {
        let mut additional_info = HashMap::new();
        additional_info.insert("resource".to_string(), resource.clone());
        additional_info.insert("action".to_string(), action);
        additional_info.insert("granted".to_string(), granted.to_string());
        if let Some(reason) = reason {
            additional_info.insert("reason".to_string(), reason);
        }

        self.log_security_event(
            if granted {
                SecurityEventType::AuthenticationSuccess
            } else {
                SecurityEventType::UnauthorizedAccess
            },
            client_id,
            resource,
            None,
            None,
            additional_info,
        );
    }

    /// Log suspicious activity
    pub fn log_suspicious_activity(
        &self,
        client_id: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
        activity_type: String,
        severity: String,
        details: HashMap<String, String>,
    ) {
        let mut additional_info = details;
        additional_info.insert("activity_type".to_string(), activity_type);
        additional_info.insert("severity".to_string(), severity);

        self.log_security_event(
            SecurityEventType::SuspiciousActivity,
            client_id,
            "suspicious_activity".to_string(),
            user_agent,
            ip_address,
            additional_info,
        );
    }

    /// Log data access
    pub fn log_data_access(
        &self,
        client_id: String,
        resource: String,
        action: String,
        success: bool,
        data_classification: Option<String>,
    ) {
        let mut additional_info = HashMap::new();
        additional_info.insert("action".to_string(), action);
        additional_info.insert("success".to_string(), success.to_string());
        if let Some(classification) = data_classification {
            additional_info.insert("data_classification".to_string(), classification);
        }

        self.log_security_event(
            if success {
                SecurityEventType::AuthenticationSuccess
            } else {
                SecurityEventType::UnauthorizedAccess
            },
            client_id,
            resource,
            None,
            None,
            additional_info,
        );
    }
}

/// Security configuration for the hive system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct SecurityConfig {
    /// Enable security audit logging
    pub audit_logging_enabled: bool,
    /// Audit log retention period in days
    pub audit_retention_days: u32,
    /// Enable rate limiting
    pub rate_limiting_enabled: bool,
    /// Rate limit per minute
    pub rate_limit_per_minute: u32,
    /// Enable CORS
    pub cors_enabled: bool,
    /// Allowed CORS origins
    pub cors_origins: Vec<String>,
    /// Enable security headers
    pub security_headers_enabled: bool,
    /// Session timeout in minutes
    pub session_timeout_minutes: u32,
    /// Maximum login attempts before lockout
    pub max_login_attempts: u32,
    /// Account lockout duration in minutes
    pub lockout_duration_minutes: u32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            audit_logging_enabled: true,
            audit_retention_days: 90,
            rate_limiting_enabled: true,
            rate_limit_per_minute: 1000,
            cors_enabled: true,
            cors_origins: vec!["http://localhost:3000".to_string()],
            security_headers_enabled: true,
            session_timeout_minutes: 60,
            max_login_attempts: 5,
            lockout_duration_minutes: 15,
        }
    }
}

/// Input sanitization utilities
pub struct InputSanitizer;

impl InputSanitizer {
    /// Sanitize string input to prevent injection attacks
    #[must_use]
    pub fn sanitize_string(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || "-_.,!?".contains(*c))
            .collect::<String>()
            .trim()
            .to_string()
    }

    /// Sanitize and validate email address
    pub fn sanitize_email(email: &str) -> HiveResult<String> {
        let sanitized = email.trim().to_lowercase();

        // Basic email validation
        if !sanitized.contains('@') || !sanitized.contains('.') {
            return Err(HiveError::ValidationError {
                field: "email".to_string(),
                reason: "Invalid email format".to_string(),
            });
        }

        // Check for suspicious patterns
        if sanitized.contains("script") || sanitized.contains('<') || sanitized.contains('>') {
            return Err(HiveError::ValidationError {
                field: "email".to_string(),
                reason: "Email contains suspicious content".to_string(),
            });
        }

        Ok(sanitized)
    }

    /// Sanitize URL input
    pub fn sanitize_url(url: &str) -> HiveResult<String> {
        let sanitized = url.trim();

        // Basic URL validation
        if !sanitized.starts_with("http://") && !sanitized.starts_with("https://") {
            return Err(HiveError::ValidationError {
                field: "url".to_string(),
                reason: "URL must start with http:// or https://".to_string(),
            });
        }

        // Check for suspicious patterns
        if sanitized.contains("javascript:") || sanitized.contains("data:") {
            return Err(HiveError::ValidationError {
                field: "url".to_string(),
                reason: "URL contains suspicious protocol".to_string(),
            });
        }

        Ok(sanitized.to_string())
    }

    /// Validate and sanitize UUID
    pub fn validate_uuid(uuid_str: &str) -> HiveResult<Uuid> {
        Uuid::parse_str(uuid_str.trim()).map_err(|_| HiveError::ValidationError {
            field: "uuid".to_string(),
            reason: "Invalid UUID format".to_string(),
        })
    }
}

/// Security headers configuration
pub struct SecurityHeaders;

impl SecurityHeaders {
    /// Get default security headers
    #[must_use]
    pub fn default_headers() -> HashMap<String, String> {
        let mut headers = HashMap::new();

        // Prevent clickjacking
        headers.insert("X-Frame-Options".to_string(), "DENY".to_string());

        // Prevent MIME type sniffing
        headers.insert("X-Content-Type-Options".to_string(), "nosniff".to_string());

        // Enable XSS protection
        headers.insert("X-XSS-Protection".to_string(), "1; mode=block".to_string());

        // Strict transport security
        headers.insert(
            "Strict-Transport-Security".to_string(),
            "max-age=31536000; includeSubDomains".to_string(),
        );

        // Content security policy
        headers.insert(
            "Content-Security-Policy".to_string(),
            "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'".to_string(),
        );

        // Referrer policy
        headers.insert(
            "Referrer-Policy".to_string(),
            "strict-origin-when-cross-origin".to_string(),
        );

        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_sanitization() {
        assert_eq!(
            InputSanitizer::sanitize_string("Hello <script>alert('xss')</script> World"),
            "Hello scriptalertxss/script World"
        );

        assert_eq!(
            InputSanitizer::sanitize_string("Normal text with spaces"),
            "Normal text with spaces"
        );
    }

    #[test]
    fn test_email_validation() {
        assert!(InputSanitizer::sanitize_email("test@example.com").is_ok());
        assert!(InputSanitizer::sanitize_email("invalid-email").is_err());
        assert!(InputSanitizer::sanitize_email("test<script>@example.com").is_err());
    }

    #[test]
    fn test_url_validation() {
        assert!(InputSanitizer::sanitize_url("https://example.com").is_ok());
        assert!(InputSanitizer::sanitize_url("javascript:alert('xss')").is_err());
        assert!(InputSanitizer::sanitize_url("ftp://example.com").is_err());
    }

    #[test]
    fn test_uuid_validation() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(InputSanitizer::validate_uuid(valid_uuid).is_ok());
        assert!(InputSanitizer::validate_uuid("invalid-uuid").is_err());
    }
}
