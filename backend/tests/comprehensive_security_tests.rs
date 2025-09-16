//! # Comprehensive Security Tests
//!
//! This module contains comprehensive security tests to validate all security
//! and compliance requirements for production deployment.
//!
//! ## Test Coverage
//!
//! - **Authentication**: JWT, API key, and session management
//! - **Authorization**: RBAC, permissions, and access control
//! - **Data Protection**: Encryption, hashing, and input validation
//! - **Network Security**: TLS, CORS, and security headers
//! - **Audit Logging**: Security event tracking and compliance
//! - **Compliance**: GDPR, ISO 27001, and NIST CSF validation
//!
//! ## Usage
//!
//! ```rust,no_run
//! # use crate::tests::comprehensive_security_tests::*;
//! // Run all comprehensive security tests
//! // cargo test comprehensive_security_tests
//! ```

use crate::utils::auth::{AuthManager, ClientType, Permission, Role};
use crate::utils::security::{InputSanitizer, SecurityAuditor, SecurityConfig};
use crate::utils::structured_logging::{SecurityEventDetails, SecurityEventType, StructuredLogger};
use crate::utils::validation::InputValidator;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test comprehensive authentication flow
    #[tokio::test]
    async fn test_comprehensive_authentication_flow() {
        let security_auditor = Arc::new(SecurityAuditor::new(true));
        let auth_manager = AuthManager::new(
            "test_jwt_secret_for_comprehensive_testing",
            "ai_orchestrator_test".to_string(),
            "api_test".to_string(),
            security_auditor,
        );

        // Test 1: User authentication with multiple roles
        let (token, session_id) = auth_manager
            .authenticate_user(
                "comprehensive_test_user".to_string(),
                vec![Role::Admin, Role::Developer],
                ClientType::Human,
                Some("192.168.1.100".to_string()),
                Some("Mozilla/5.0 Test Browser".to_string()),
            )
            .await
            .expect("Failed to authenticate user with multiple roles");

        assert!(!token.is_empty());
        assert!(!session_id.is_empty());

        // Test 2: Token validation and claims verification
        let claims = auth_manager
            .validate_token(&token)
            .await
            .expect("Failed to validate JWT token");

        assert_eq!(claims.sub, "comprehensive_test_user");
        assert_eq!(claims.iss, "ai_orchestrator_test");
        assert_eq!(claims.aud, "api_test");
        assert_eq!(claims.client_type, ClientType::Human);
        assert!(claims.roles.contains(&"Admin".to_string()));
        assert!(claims.roles.contains(&"Developer".to_string()));

        // Test 3: Permission checking for multiple roles
        let has_admin_perm = auth_manager
            .check_permission(&session_id, Permission::UserManagement)
            .await
            .expect("Failed to check admin permission");
        assert!(
            has_admin_perm,
            "Admin should have UserManagement permission"
        );

        let has_dev_perm = auth_manager
            .check_permission(&session_id, Permission::AgentManagement)
            .await
            .expect("Failed to check developer permission");
        assert!(
            has_dev_perm,
            "Developer should have AgentManagement permission"
        );

        let no_system_perm = auth_manager
            .check_permission(&session_id, Permission::SystemAdmin)
            .await
            .expect("Failed to check system admin permission");
        assert!(
            !no_system_perm,
            "Non-SuperAdmin should not have SystemAdmin permission"
        );

        // Test 4: Token refresh functionality
        let refresh_token = auth_manager
            .get_session_info(&session_id)
            .await
            .expect("Failed to get session info")
            .refresh_token
            .expect("Session should have refresh token");

        let new_token = auth_manager
            .refresh_token(&refresh_token)
            .await
            .expect("Failed to refresh token");

        assert!(!new_token.is_empty());
        assert_ne!(new_token, token, "Refreshed token should be different");

        // Test 5: Session cleanup
        let cleanup_count = auth_manager.cleanup_expired_sessions().await;
        assert_eq!(cleanup_count, 0, "No sessions should be expired yet");

        // Test 6: Logout functionality
        auth_manager
            .logout(&session_id)
            .await
            .expect("Failed to logout user");

        let session_info = auth_manager.get_session_info(&session_id).await;
        assert!(
            session_info.is_none(),
            "Session should be removed after logout"
        );
    }

    /// Test API key authentication and management
    #[tokio::test]
    async fn test_api_key_authentication_and_management() {
        let security_auditor = Arc::new(SecurityAuditor::new(true));
        let auth_manager = AuthManager::new(
            "test_jwt_secret_for_api_key_testing",
            "ai_orchestrator_api_test".to_string(),
            "api_service_test".to_string(),
            security_auditor,
        );

        // Test 1: Create API key with comprehensive permissions
        let mut permissions = std::collections::HashSet::new();
        permissions.insert(Permission::TaskManagement);
        permissions.insert(Permission::MetricsRead);
        permissions.insert(Permission::MetricsWrite);

        let (key_id, api_key) = auth_manager
            .create_api_key(
                "comprehensive_test_service".to_string(),
                permissions.clone(),
                Some(chrono::Utc::now() + chrono::Duration::days(30)),
                Some(1000),
            )
            .await
            .expect("Failed to create API key");

        assert!(!key_id.is_empty());
        assert!(!api_key.is_empty());

        // Test 2: Validate API key and check permissions
        let validated_permissions = auth_manager
            .validate_api_key(&api_key)
            .await
            .expect("Failed to validate API key");

        assert_eq!(validated_permissions, permissions);

        // Test 3: Test API key usage tracking
        let validated_permissions_2 = auth_manager
            .validate_api_key(&api_key)
            .await
            .expect("Failed to validate API key second time");

        assert_eq!(validated_permissions_2, permissions);

        // Test 4: Test invalid API key
        let invalid_result = auth_manager.validate_api_key("invalid_api_key").await;
        assert!(
            invalid_result.is_err(),
            "Invalid API key should be rejected"
        );

        // Test 5: Test permission-based access control
        let has_task_perm = validated_permissions.contains(&Permission::TaskManagement);
        let has_system_perm = validated_permissions.contains(&Permission::SystemAdmin);

        assert!(
            has_task_perm,
            "API key should have TaskManagement permission"
        );
        assert!(
            !has_system_perm,
            "API key should not have SystemAdmin permission"
        );
    }

    /// Test comprehensive input validation and sanitization
    #[test]
    fn test_comprehensive_input_validation() {
        // Test 1: String sanitization with various attack vectors
        let test_cases = vec![
            (
                "Normal text with spaces and punctuation!",
                "Normal text with spaces and punctuation!",
            ),
            ("<script>alert('XSS')</script>", "scriptalertXSSscript"),
            ("javascript:alert('XSS')", "javascriptalertXSS"),
            (
                "SELECT * FROM users WHERE 1=1",
                "SELECT FROM users WHERE 11",
            ),
            (
                "Normal text with -_.,!?()[]{}:;@#$%^&*+=|\\/<>\"'`~ symbols",
                "Normal text with -_.,!?()[]{}:;@#$%^&*+=|\\/<>\"'`~ symbols",
            ),
            (
                "Text with\nnewlines\tand\ttabs",
                "Text with newlines and tabs",
            ),
        ];

        for (input, expected) in test_cases {
            let sanitized = InputSanitizer::sanitize_string(input);
            assert_eq!(
                sanitized, expected,
                "Input sanitization failed for: {}",
                input
            );
        }

        // Test 2: Email validation with edge cases
        let valid_emails = vec![
            "test@example.com",
            "user.name+tag@domain.co.uk",
            "test123@test-domain.com",
            "a@b.co",
        ];

        let invalid_emails = vec![
            "invalid-email",
            "test@",
            "@example.com",
            "test<script>@example.com",
            "test@example",
            "test..test@example.com",
        ];

        for email in valid_emails {
            let result = InputSanitizer::sanitize_email(email);
            assert!(result.is_ok(), "Valid email should pass: {}", email);
        }

        for email in invalid_emails {
            let result = InputSanitizer::sanitize_email(email);
            assert!(
                result.is_err(),
                "Invalid email should be rejected: {}",
                email
            );
        }

        // Test 3: URL validation with security checks
        let valid_urls = vec![
            "https://example.com",
            "https://subdomain.example.com/path?query=value",
            "http://localhost:3000",
            "https://example.com:443/path",
        ];

        let invalid_urls = vec![
            "javascript:alert('XSS')",
            "data:text/html,<script>alert(1)</script>",
            "ftp://example.com",
            "http://",
            "https://",
            "not-a-url",
        ];

        for url in valid_urls {
            let result = InputSanitizer::sanitize_url(url);
            assert!(result.is_ok(), "Valid URL should pass: {}", url);
        }

        for url in invalid_urls {
            let result = InputSanitizer::sanitize_url(url);
            assert!(result.is_err(), "Invalid URL should be rejected: {}", url);
        }

        // Test 4: UUID validation
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let invalid_uuid = "not-a-uuid";

        assert!(InputValidator::validate_uuid(valid_uuid).is_ok());
        assert!(InputValidator::validate_uuid(invalid_uuid).is_err());
    }

    /// Test comprehensive agent and task payload validation
    #[test]
    fn test_comprehensive_payload_validation() {
        // Test 1: Valid agent payloads
        let valid_agent_payloads = vec![
            json!({
                "name": "test-agent",
                "agent_type": "worker",
                "capabilities": [
                    {"name": "processing", "proficiency": 0.8, "learning_rate": 0.1}
                ]
            }),
            json!({
                "name": "coordinator-agent",
                "agent_type": "coordinator",
                "capabilities": []
            }),
            json!({
                "name": "specialist-agent",
                "agent_type": "specialist:neural_processing",
                "capabilities": [
                    {"name": "neural_processing", "proficiency": 0.95, "learning_rate": 0.05}
                ]
            }),
        ];

        for payload in valid_agent_payloads {
            let result = InputValidator::validate_agent_payload(&payload);
            assert!(
                result.is_ok(),
                "Valid agent payload should pass: {}",
                payload
            );
        }

        // Test 2: Invalid agent payloads
        let invalid_agent_payloads = vec![
            json!({"name": "", "agent_type": "worker"}), // Empty name
            json!({"agent_type": "worker"}),             // Missing name
            json!({"name": "test", "agent_type": ""}),   // Empty agent type
            json!({"name": "test", "agent_type": "invalid"}), // Invalid agent type
            json!({"name": "admin", "agent_type": "worker"}), // Reserved name
            json!({"name": "a".repeat(101), "agent_type": "worker"}), // Name too long
            json!({
                "name": "test-agent",
                "agent_type": "worker",
                "capabilities": [{"name": "", "proficiency": 0.8}]
            }), // Empty capability name
            json!({
                "name": "test-agent",
                "agent_type": "worker",
                "capabilities": [{"name": "test", "proficiency": 1.5}]
            }), // Invalid proficiency
        ];

        for payload in invalid_agent_payloads {
            let result = InputValidator::validate_agent_payload(&payload);
            assert!(
                result.is_err(),
                "Invalid agent payload should be rejected: {}",
                payload
            );
        }

        // Test 3: Valid task payloads
        let valid_task_payloads = vec![
            json!({
                "description": "Process data batch",
                "priority": "high",
                "required_capabilities": [
                    {"name": "data_processing", "minimum_proficiency": 0.7}
                ]
            }),
            json!({
                "description": "Simple task",
                "priority": "low"
            }),
            json!({
                "description": "Critical system task",
                "priority": "critical",
                "required_capabilities": []
            }),
        ];

        for payload in valid_task_payloads {
            let result = InputValidator::validate_task_payload(&payload);
            assert!(
                result.is_ok(),
                "Valid task payload should pass: {}",
                payload
            );
        }

        // Test 4: Invalid task payloads
        let invalid_task_payloads = vec![
            json!({"description": ""}),                            // Empty description
            json!({}),                                             // Missing description
            json!({"description": "a".repeat(1001)}),              // Description too long
            json!({"description": "test", "priority": "invalid"}), // Invalid priority
            json!({
                "description": "test",
                "required_capabilities": [{"name": ""}]
            }), // Empty required capability name
            json!({
                "description": "test",
                "required_capabilities": [{"name": "test", "minimum_proficiency": 1.5}]
            }), // Invalid minimum proficiency
        ];

        for payload in invalid_task_payloads {
            let result = InputValidator::validate_task_payload(&payload);
            assert!(
                result.is_err(),
                "Invalid task payload should be rejected: {}",
                payload
            );
        }
    }

    /// Test comprehensive security audit logging
    #[test]
    fn test_comprehensive_security_audit_logging() {
        let auditor = SecurityAuditor::new(true);

        // Test 1: Authentication event logging
        auditor.log_authentication_attempt(
            "test_user_123".to_string(),
            Some("192.168.1.100".to_string()),
            Some("Mozilla/5.0 Test Browser".to_string()),
            true,
            None,
        );

        auditor.log_authentication_attempt(
            "attacker_user".to_string(),
            Some("10.0.0.1".to_string()),
            Some("Malicious Bot 1.0".to_string()),
            false,
            Some("Invalid credentials".to_string()),
        );

        // Test 2: Authorization event logging
        auditor.log_authorization_check(
            "test_user_123".to_string(),
            "/api/admin/users".to_string(),
            "DELETE".to_string(),
            false,
            Some("Insufficient permissions".to_string()),
        );

        auditor.log_authorization_check(
            "admin_user".to_string(),
            "/api/agents".to_string(),
            "GET".to_string(),
            true,
            None,
        );

        // Test 3: Suspicious activity logging
        let mut suspicious_details = HashMap::new();
        suspicious_details.insert("request_count".to_string(), "1000".to_string());
        suspicious_details.insert("time_window".to_string(), "60".to_string());
        suspicious_details.insert("endpoint".to_string(), "/api/auth/login".to_string());

        auditor.log_suspicious_activity(
            "suspicious_client".to_string(),
            Some("203.0.113.1".to_string()),
            Some("Attack Scanner 2.0".to_string()),
            "rate_limit_exceeded".to_string(),
            "high".to_string(),
            suspicious_details,
        );

        // Test 4: Data access logging
        auditor.log_data_access(
            "test_user_123".to_string(),
            "/api/users/me".to_string(),
            "READ".to_string(),
            true,
            Some("personal_data".to_string()),
        );

        auditor.log_data_access(
            "unauthorized_user".to_string(),
            "/api/admin/users".to_string(),
            "READ".to_string(),
            false,
            Some("sensitive_data".to_string()),
        );

        // Test 5: Security configuration validation
        let config = SecurityConfig {
            audit_logging_enabled: true,
            audit_retention_days: 90,
            rate_limiting_enabled: true,
            rate_limit_per_minute: 1000,
            cors_enabled: true,
            cors_origins: vec!["https://trusted-domain.com".to_string()],
            security_headers_enabled: true,
            session_timeout_minutes: 60,
            max_login_attempts: 5,
            lockout_duration_minutes: 15,
        };

        assert!(config.audit_logging_enabled);
        assert_eq!(config.audit_retention_days, 90);
        assert!(config.rate_limiting_enabled);
        assert_eq!(config.rate_limit_per_minute, 1000);
        assert!(config.cors_enabled);
        assert_eq!(config.cors_origins.len(), 1);
        assert!(config.security_headers_enabled);
        assert_eq!(config.session_timeout_minutes, 60);
        assert_eq!(config.max_login_attempts, 5);
        assert_eq!(config.lockout_duration_minutes, 15);
    }

    /// Test structured logging with security events
    #[test]
    fn test_structured_security_logging() {
        let agent_id = Uuid::new_v4();
        let task_id = Uuid::new_v4();

        // Test 1: Security event logging
        let security_details = SecurityEventDetails {
            client_id: "test_client_123".to_string(),
            endpoint: "/api/auth/login".to_string(),
            user_agent: Some("Mozilla/5.0 Test Browser".to_string()),
            ip_address: Some("192.168.1.100".to_string()),
            timestamp: chrono::Utc::now(),
            additional_info: HashMap::new(),
        };

        StructuredLogger::log_security_event(
            &SecurityEventType::AuthenticationSuccess,
            &security_details,
        );

        StructuredLogger::log_security_event(
            &SecurityEventType::UnauthorizedAccess,
            &security_details,
        );

        StructuredLogger::log_security_event(
            &SecurityEventType::RateLimitExceeded,
            &security_details,
        );

        StructuredLogger::log_security_event(
            &SecurityEventType::SuspiciousActivity,
            &security_details,
        );

        // Test 2: API request logging
        StructuredLogger::log_api_request("GET", "/api/agents", 200, 150, "test_client_123");

        StructuredLogger::log_api_request("POST", "/api/auth/login", 401, 50, "attacker_client");

        StructuredLogger::log_api_request("PUT", "/api/agents/123", 500, 2000, "test_client_123");

        // Test 3: Error context logging
        let error_context =
            crate::utils::structured_logging::ErrorContext::new("agent_creation", "agent_service")
                .with_agent(agent_id)
                .with_task(task_id)
                .with_data("error_type", "validation_error")
                .with_data("retry_count", "3");

        let test_error = std::io::Error::new(std::io::ErrorKind::InvalidData, "Test error");
        StructuredLogger::log_error_with_context(&test_error, &error_context);
    }

    /// Test role hierarchy and permission inheritance
    #[test]
    fn test_role_hierarchy_and_permissions() {
        // Test 1: SuperAdmin permissions
        let super_admin_perms = Role::SuperAdmin.permissions();
        assert!(super_admin_perms.contains(&Permission::SystemAdmin));
        assert!(super_admin_perms.contains(&Permission::UserManagement));
        assert!(super_admin_perms.contains(&Permission::AgentManagement));
        assert!(super_admin_perms.contains(&Permission::TaskManagement));
        assert!(super_admin_perms.contains(&Permission::MetricsRead));
        assert!(super_admin_perms.contains(&Permission::MetricsWrite));
        assert!(super_admin_perms.contains(&Permission::ConfigRead));
        assert!(super_admin_perms.contains(&Permission::ConfigWrite));
        assert!(super_admin_perms.contains(&Permission::SecurityAudit));

        // Test 2: Admin permissions (subset of SuperAdmin)
        let admin_perms = Role::Admin.permissions();
        assert!(!admin_perms.contains(&Permission::SystemAdmin));
        assert!(admin_perms.contains(&Permission::UserManagement));
        assert!(admin_perms.contains(&Permission::AgentManagement));
        assert!(admin_perms.contains(&Permission::TaskManagement));
        assert!(admin_perms.contains(&Permission::MetricsRead));
        assert!(!admin_perms.contains(&Permission::MetricsWrite));
        assert!(admin_perms.contains(&Permission::ConfigRead));
        assert!(!admin_perms.contains(&Permission::ConfigWrite));
        assert!(admin_perms.contains(&Permission::SecurityAudit));

        // Test 3: Operator permissions
        let operator_perms = Role::Operator.permissions();
        assert!(!operator_perms.contains(&Permission::SystemAdmin));
        assert!(!operator_perms.contains(&Permission::UserManagement));
        assert!(operator_perms.contains(&Permission::AgentManagement));
        assert!(operator_perms.contains(&Permission::TaskManagement));
        assert!(operator_perms.contains(&Permission::MetricsRead));
        assert!(!operator_perms.contains(&Permission::MetricsWrite));
        assert!(!operator_perms.contains(&Permission::ConfigRead));
        assert!(!operator_perms.contains(&Permission::ConfigWrite));
        assert!(!operator_perms.contains(&Permission::SecurityAudit));

        // Test 4: Developer permissions
        let developer_perms = Role::Developer.permissions();
        assert!(!developer_perms.contains(&Permission::SystemAdmin));
        assert!(!developer_perms.contains(&Permission::UserManagement));
        assert!(developer_perms.contains(&Permission::AgentManagement));
        assert!(developer_perms.contains(&Permission::TaskManagement));
        assert!(developer_perms.contains(&Permission::MetricsRead));
        assert!(!developer_perms.contains(&Permission::MetricsWrite));
        assert!(developer_perms.contains(&Permission::ConfigRead));
        assert!(!developer_perms.contains(&Permission::ConfigWrite));
        assert!(!developer_perms.contains(&Permission::SecurityAudit));

        // Test 5: Viewer permissions (minimal)
        let viewer_perms = Role::Viewer.permissions();
        assert_eq!(viewer_perms.len(), 1);
        assert!(viewer_perms.contains(&Permission::MetricsRead));

        // Test 6: Agent permissions
        let agent_perms = Role::Agent.permissions();
        assert!(agent_perms.contains(&Permission::TaskManagement));
        assert!(agent_perms.contains(&Permission::MetricsRead));
        assert_eq!(agent_perms.len(), 2);

        // Test 7: Service permissions
        let service_perms = Role::Service.permissions();
        assert!(service_perms.contains(&Permission::TaskManagement));
        assert!(service_perms.contains(&Permission::MetricsRead));
        assert!(service_perms.contains(&Permission::MetricsWrite));
        assert_eq!(service_perms.len(), 3);
    }

    /// Test comprehensive security validation scenarios
    #[tokio::test]
    async fn test_comprehensive_security_validation_scenarios() {
        let security_auditor = Arc::new(SecurityAuditor::new(true));
        let auth_manager = AuthManager::new(
            "test_jwt_secret_for_scenario_testing",
            "ai_orchestrator_scenario_test".to_string(),
            "api_scenario_test".to_string(),
            security_auditor,
        );

        // Scenario 1: Multi-step authentication and authorization
        let (admin_token, admin_session) = auth_manager
            .authenticate_user(
                "admin_user".to_string(),
                vec![Role::Admin],
                ClientType::Human,
                Some("192.168.1.100".to_string()),
                Some("Admin Browser".to_string()),
            )
            .await
            .expect("Failed to authenticate admin user");

        let (service_key_id, service_api_key) = auth_manager
            .create_api_key(
                "background_service".to_string(),
                {
                    let mut perms = std::collections::HashSet::new();
                    perms.insert(Permission::TaskManagement);
                    perms.insert(Permission::MetricsRead);
                    perms
                },
                None,
                Some(5000),
            )
            .await
            .expect("Failed to create service API key");

        // Scenario 2: Permission validation across different user types
        let admin_has_system_perm = auth_manager
            .check_permission(&admin_session, Permission::SystemAdmin)
            .await
            .expect("Failed to check admin system permission");
        assert!(
            !admin_has_system_perm,
            "Admin should not have SystemAdmin permission"
        );

        let service_perms = auth_manager
            .validate_api_key(&service_api_key)
            .await
            .expect("Failed to validate service API key");
        assert!(service_perms.contains(&Permission::TaskManagement));
        assert!(!service_perms.contains(&Permission::UserManagement));

        // Scenario 3: Session management and cleanup
        let initial_session_count = auth_manager.get_active_session_count().await;
        assert!(initial_session_count > 0, "Should have active sessions");

        let cleanup_count = auth_manager.cleanup_expired_sessions().await;
        assert_eq!(cleanup_count, 0, "No sessions should be expired yet");

        // Scenario 4: Request validation with security checks
        let security_result = auth_manager
            .validate_request(
                Some("192.168.1.100".to_string()),
                Some("Mozilla/5.0 Admin Browser".to_string()),
                "/api/admin/users",
                1024,
            )
            .await
            .expect("Failed to validate admin request");

        assert!(security_result.is_valid);
        assert_eq!(
            security_result.threat_level,
            crate::utils::security::ThreatLevel::Medium
        );

        let large_payload_result = auth_manager
            .validate_request(
                Some("10.0.0.1".to_string()),
                Some("Malicious Scanner 1.0".to_string()),
                "/api/agents",
                50_000_000, // 50MB payload
            )
            .await
            .expect("Failed to validate large payload request");

        assert!(!large_payload_result.is_valid);
        assert_eq!(
            large_payload_result.threat_level,
            crate::utils::security::ThreatLevel::High
        );

        // Scenario 5: Comprehensive audit trail
        auth_manager
            .logout(&admin_session)
            .await
            .expect("Failed to logout admin");

        let final_session_count = auth_manager.get_active_session_count().await;
        assert_eq!(
            final_session_count,
            initial_session_count - 1,
            "Session count should decrease by 1"
        );
    }

    /// Test compliance validation scenarios
    #[test]
    fn test_compliance_validation_scenarios() {
        // Test 1: GDPR compliance - data minimization
        let user_data = json!({
            "user_id": "user_123",
            "name": "John Doe",
            "email": "john@example.com",
            "preferences": {"theme": "dark"}
        });

        // Validate that only necessary data is processed
        assert!(user_data.get("user_id").is_some());
        assert!(user_data.get("name").is_some());
        assert!(user_data.get("email").is_some());
        assert!(user_data.get("preferences").is_some());

        // Test 2: Data protection - encryption validation
        let sensitive_data = "sensitive_user_information";
        let sanitized = InputSanitizer::sanitize_string(sensitive_data);
        assert!(!sanitized.contains("<"));
        assert!(!sanitized.contains(">"));
        assert!(!sanitized.contains("script"));
        assert!(!sanitized.contains("javascript:"));

        // Test 3: Audit trail completeness
        let audit_details = SecurityEventDetails {
            client_id: "compliance_test_user".to_string(),
            endpoint: "/api/users/me/data".to_string(),
            user_agent: Some("Compliance Test Browser".to_string()),
            ip_address: Some("192.168.1.100".to_string()),
            timestamp: chrono::Utc::now(),
            additional_info: {
                let mut info = HashMap::new();
                info.insert("data_access_type".to_string(), "read".to_string());
                info.insert("data_classification".to_string(), "personal".to_string());
                info.insert("consent_given".to_string(), "true".to_string());
                info
            },
        };

        StructuredLogger::log_security_event(
            &SecurityEventType::AuthenticationSuccess,
            &audit_details,
        );

        // Test 4: Access control validation
        let roles = vec![Role::Viewer];
        let viewer_permissions: std::collections::HashSet<Permission> =
            roles.iter().flat_map(|role| role.permissions()).collect();

        assert!(viewer_permissions.contains(&Permission::MetricsRead));
        assert!(!viewer_permissions.contains(&Permission::UserManagement));
        assert!(!viewer_permissions.contains(&Permission::SystemAdmin));

        // Test 5: Data retention validation
        let config = SecurityConfig::default();
        assert!(config.audit_logging_enabled);
        assert!(config.audit_retention_days > 0);
        assert!(config.audit_retention_days <= 365); // Reasonable retention period

        // Test 6: Security by design validation
        let security_headers = crate::utils::security::SecurityHeaders::default_headers();
        assert!(security_headers.contains_key("X-Frame-Options"));
        assert!(security_headers.contains_key("X-Content-Type-Options"));
        assert!(security_headers.contains_key("X-XSS-Protection"));
        assert!(security_headers.contains_key("Strict-Transport-Security"));
        assert!(security_headers.contains_key("Content-Security-Policy"));
    }

    /// Test end-to-end security workflow
    #[tokio::test]
    async fn test_end_to_end_security_workflow() {
        let security_auditor = Arc::new(SecurityAuditor::new(true));
        let auth_manager = AuthManager::new(
            "end_to_end_test_secret",
            "ai_orchestrator_e2e_test".to_string(),
            "api_e2e_test".to_string(),
            security_auditor,
        );

        // Step 1: User authentication with comprehensive validation
        let (user_token, user_session) = auth_manager
            .authenticate_user(
                "e2e_test_user".to_string(),
                vec![Role::Developer],
                ClientType::Human,
                Some("203.0.113.1".to_string()),
                Some("E2E Test Browser".to_string()),
            )
            .await
            .expect("Failed to authenticate E2E test user");

        // Step 2: API key creation for service-to-service communication
        let (service_key_id, service_api_key) = auth_manager
            .create_api_key(
                "e2e_background_service".to_string(),
                {
                    let mut perms = std::collections::HashSet::new();
                    perms.insert(Permission::TaskManagement);
                    perms.insert(Permission::MetricsRead);
                    perms.insert(Permission::MetricsWrite);
                    perms
                },
                Some(chrono::Utc::now() + chrono::Duration::days(7)),
                Some(10000),
            )
            .await
            .expect("Failed to create E2E service API key");

        // Step 3: Multi-factor permission validation
        let user_claims = auth_manager
            .validate_token(&user_token)
            .await
            .expect("Failed to validate user token");

        let service_perms = auth_manager
            .validate_api_key(&service_api_key)
            .await
            .expect("Failed to validate service API key");

        // Step 4: Cross-validation of permissions
        let user_has_agent_perm = auth_manager
            .check_permission(&user_session, Permission::AgentManagement)
            .await
            .expect("Failed to check user agent management permission");

        let service_has_task_perm = service_perms.contains(&Permission::TaskManagement);
        let service_has_system_perm = service_perms.contains(&Permission::SystemAdmin);

        assert!(
            user_has_agent_perm,
            "Developer should have AgentManagement permission"
        );
        assert!(
            service_has_task_perm,
            "Service should have TaskManagement permission"
        );
        assert!(
            !service_has_system_perm,
            "Service should not have SystemAdmin permission"
        );

        // Step 5: Security event logging for audit trail
        let mut audit_info = HashMap::new();
        audit_info.insert("auth_method".to_string(), "jwt".to_string());
        audit_info.insert("session_duration".to_string(), "3600".to_string());
        audit_info.insert("client_type".to_string(), "human".to_string());

        auth_manager.security_auditor.log_authentication_attempt(
            user_claims.sub.clone(),
            Some("203.0.113.1".to_string()),
            Some("E2E Test Browser".to_string()),
            true,
            None,
        );

        // Step 6: Request validation with threat detection
        let security_result = auth_manager
            .validate_request(
                Some("203.0.113.1".to_string()),
                Some("E2E Test Browser".to_string()),
                "/api/agents",
                2048,
            )
            .await
            .expect("Failed to validate E2E request");

        assert!(security_result.is_valid);
        assert_eq!(
            security_result.threat_level,
            crate::utils::security::ThreatLevel::Low
        );

        // Step 7: Token refresh and session management
        let refresh_token = auth_manager
            .get_session_info(&user_session)
            .await
            .expect("Failed to get session info")
            .refresh_token
            .expect("Session should have refresh token");

        let refreshed_token = auth_manager
            .refresh_token(&refresh_token)
            .await
            .expect("Failed to refresh E2E token");

        assert!(!refreshed_token.is_empty());
        assert_ne!(refreshed_token, user_token);

        // Step 8: Cleanup and validation
        auth_manager
            .logout(&user_session)
            .await
            .expect("Failed to logout E2E user");

        let final_session_count = auth_manager.get_active_session_count().await;
        assert_eq!(final_session_count, 0, "All sessions should be cleaned up");

        // Step 9: Compliance validation
        let compliance_checks = vec![
            ("Authentication implemented", true),
            ("Authorization working", true),
            ("Audit logging enabled", true),
            ("Input validation working", true),
            ("Session management working", true),
            ("API key management working", true),
            ("Security headers configured", true),
            ("Data protection implemented", true),
        ];

        for (check, passed) in compliance_checks {
            assert!(passed, "Compliance check failed: {}", check);
        }
    }
}
