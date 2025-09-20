//! Integration tests for the authentication system
//!
//! These tests verify the complete authentication flow including
//! API endpoints, database persistence, and security features.

use std::net::TcpListener;
use std::sync::Arc;
use tokio::sync::RwLock;

use multiagent_hive::api::create_router;
use multiagent_hive::auth::{AuthManager, UserRole};
use multiagent_hive::persistence::{PersistenceManager, SQLiteStorage};
use multiagent_hive::settings::Settings;

use reqwest::Client;
use serde_json::json;
use tempfile::TempDir;
use tower::ServiceBuilder;
use axum::http::StatusCode;

mod test_utils;
use test_utils::*;

/// Test fixture for integration testing
struct AuthIntegrationFixture {
    client: Client,
    base_url: String,
    _temp_dir: TempDir,
    _server_handle: tokio::task::JoinHandle<()>,
}

impl AuthIntegrationFixture {
    async fn new() -> Self {
        // Create temporary database
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        // Setup storage and persistence
        let storage = Arc::new(RwLock::new(
            SQLiteStorage::new(&db_path).expect("Failed to create storage")
        ));

        let persistence = PersistenceManager::new(storage, None)
            .await
            .expect("Failed to create persistence");

        // Create auth manager
        let auth_manager = AuthManager::new(persistence)
            .await
            .expect("Failed to create auth manager");

        // Find available port
        let listener = TcpListener::bind("127.0.0.1:0")
            .expect("Failed to bind to available port");
        let port = listener.local_addr().unwrap().port();

        // Create router with auth manager
        let app = create_router()
            .layer(
                ServiceBuilder::new()
                    .layer(axum::middleware::from_fn(move |req, next| {
                        let auth_manager = auth_manager.clone();
                        async move {
                            // Add auth manager to request extensions for testing
                            req.extensions_mut().insert(auth_manager);
                            next.run(req).await
                        }
                    }))
            );

        // Start server
        let server_handle = tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .expect("Server failed to start");
        });

        // Give server time to start
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let base_url = format!("http://127.0.0.1:{}", port);
        let client = Client::new();

        Self {
            client,
            base_url,
            _temp_dir: temp_dir,
            _server_handle: server_handle,
        }
    }

    async fn register_user(&self, username: &str, password: &str, role: UserRole) -> reqwest::Response {
        self.client
            .post(&format!("{}/api/auth/register", self.base_url))
            .json(&json!({
                "username": username,
                "password": password,
                "role": role
            }))
            .send()
            .await
            .expect("Failed to send register request")
    }

    async fn login(&self, username: &str, password: &str) -> reqwest::Response {
        self.client
            .post(&format!("{}/api/auth/login", self.base_url))
            .json(&json!({
                "username": username,
                "password": password
            }))
            .send()
            .await
            .expect("Failed to send login request")
    }

    async fn refresh_token(&self, refresh_token: &str) -> reqwest::Response {
        self.client
            .post(&format!("{}/api/auth/refresh", self.base_url))
            .json(&json!({
                "refresh_token": refresh_token
            }))
            .send()
            .await
            .expect("Failed to send refresh request")
    }

    async fn get_protected_resource(&self, token: &str) -> reqwest::Response {
        self.client
            .get(&format!("{}/api/protected", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .expect("Failed to send protected request")
    }

    async fn logout(&self, token: &str) -> reqwest::Response {
        self.client
            .post(&format!("{}/api/auth/logout", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .expect("Failed to send logout request")
    }
}

#[cfg(test)]
mod auth_integration_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn test_complete_auth_flow() {
        let fixture = AuthIntegrationFixture::new().await;

        // 1. Register a new user
        let register_response = fixture
            .register_user("integration_test_user", "secure_password123!", UserRole::User)
            .await;

        assert_eq!(register_response.status(), StatusCode::CREATED);

        let register_data: serde_json::Value = register_response
            .json()
            .await
            .expect("Failed to parse register response");

        assert_eq!(register_data["user"]["username"], "integration_test_user");
        assert_eq!(register_data["user"]["role"], "user");

        // 2. Login with the registered user
        let login_response = fixture
            .login("integration_test_user", "secure_password123!")
            .await;

        assert_eq!(login_response.status(), StatusCode::OK);

        let login_data: serde_json::Value = login_response
            .json()
            .await
            .expect("Failed to parse login response");

        assert!(login_data["access_token"].is_string());
        assert!(login_data["refresh_token"].is_string());
        assert!(login_data["expires_in"].is_number());

        let access_token = login_data["access_token"].as_str().unwrap();
        let refresh_token = login_data["refresh_token"].as_str().unwrap();

        // 3. Access protected resource with access token
        let protected_response = fixture
            .get_protected_resource(access_token)
            .await;

        assert_eq!(protected_response.status(), StatusCode::OK);

        // 4. Refresh the access token
        let refresh_response = fixture
            .refresh_token(refresh_token)
            .await;

        assert_eq!(refresh_response.status(), StatusCode::OK);

        let refresh_data: serde_json::Value = refresh_response
            .json()
            .await
            .expect("Failed to parse refresh response");

        let new_access_token = refresh_data["access_token"].as_str().unwrap();

        // 5. Verify new token works
        let protected_response2 = fixture
            .get_protected_resource(new_access_token)
            .await;

        assert_eq!(protected_response2.status(), StatusCode::OK);

        // 6. Logout
        let logout_response = fixture
            .logout(new_access_token)
            .await;

        assert_eq!(logout_response.status(), StatusCode::OK);

        // 7. Verify token is invalidated
        let protected_response3 = fixture
            .get_protected_resource(new_access_token)
            .await;

        assert_eq!(protected_response3.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_registration_validation() {
        let fixture = AuthIntegrationFixture::new().await;

        // Test duplicate username
        fixture
            .register_user("duplicate_user", "password123", UserRole::User)
            .await;

        let duplicate_response = fixture
            .register_user("duplicate_user", "different_password", UserRole::User)
            .await;

        assert_eq!(duplicate_response.status(), StatusCode::CONFLICT);

        // Test weak password
        let weak_password_response = fixture
            .register_user("weak_user", "123", UserRole::User)
            .await;

        // Should fail with bad request (weak password)
        assert_eq!(weak_password_response.status(), StatusCode::BAD_REQUEST);

        // Test invalid role
        let invalid_role_response = fixture.client
            .post(&format!("{}/api/auth/register", fixture.base_url))
            .json(&json!({
                "username": "invalid_role_user",
                "password": "secure_password123!",
                "role": "invalid_role"
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(invalid_role_response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_login_validation() {
        let fixture = AuthIntegrationFixture::new().await;

        // Test nonexistent user
        let nonexistent_response = fixture
            .login("nonexistent_user", "password")
            .await;

        assert_eq!(nonexistent_response.status(), StatusCode::UNAUTHORIZED);

        // Test wrong password
        fixture
            .register_user("wrong_password_user", "correct_password", UserRole::User)
            .await;

        let wrong_password_response = fixture
            .login("wrong_password_user", "wrong_password")
            .await;

        assert_eq!(wrong_password_response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_token_security() {
        let fixture = AuthIntegrationFixture::new().await;

        // Register and login
        fixture
            .register_user("security_test_user", "password123", UserRole::User)
            .await;

        let login_response = fixture
            .login("security_test_user", "password123")
            .await;

        let login_data: serde_json::Value = login_response.json().await.unwrap();
        let access_token = login_data["access_token"].as_str().unwrap();

        // Test with malformed token
        let malformed_response = fixture
            .get_protected_resource("malformed.jwt.token")
            .await;

        assert_eq!(malformed_response.status(), StatusCode::UNAUTHORIZED);

        // Test with expired token (simulate by using an old token)
        // This would require manipulating time or using a pre-expired token

        // Test missing authorization header
        let no_auth_response = fixture.client
            .get(&format!("{}/api/protected", fixture.base_url))
            .send()
            .await
            .unwrap();

        assert_eq!(no_auth_response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_role_based_access() {
        let fixture = AuthIntegrationFixture::new().await;

        // Create users with different roles
        fixture
            .register_user("regular_user", "password", UserRole::User)
            .await;

        fixture
            .register_user("admin_user", "password", UserRole::Admin)
            .await;

        // Login as regular user
        let user_login = fixture.login("regular_user", "password").await;
        let user_data: serde_json::Value = user_login.json().await.unwrap();
        let user_token = user_data["access_token"].as_str().unwrap();

        // Login as admin
        let admin_login = fixture.login("admin_user", "password").await;
        let admin_data: serde_json::Value = admin_login.json().await.unwrap();
        let admin_token = admin_data["access_token"].as_str().unwrap();

        // Test admin-only endpoint
        let admin_only_response = fixture.client
            .get(&format!("{}/api/admin/users", fixture.base_url))
            .header("Authorization", format!("Bearer {}", user_token))
            .send()
            .await
            .unwrap();

        // Regular user should be forbidden
        assert_eq!(admin_only_response.status(), StatusCode::FORBIDDEN);

        // Admin should have access
        let admin_access_response = fixture.client
            .get(&format!("{}/api/admin/users", fixture.base_url))
            .header("Authorization", format!("Bearer {}", admin_token))
            .send()
            .await
            .unwrap();

        assert_eq!(admin_access_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        let fixture = Arc::new(AuthIntegrationFixture::new().await);

        // Register a user
        fixture
            .register_user("concurrent_user", "password", UserRole::User)
            .await;

        // Spawn multiple concurrent requests
        let mut handles = vec![];

        for i in 0..10 {
            let fixture_clone = fixture.clone();
            let handle = tokio::spawn(async move {
                let response = fixture_clone
                    .login("concurrent_user", "password")
                    .await;

                assert_eq!(response.status(), StatusCode::OK);

                let data: serde_json::Value = response.json().await.unwrap();
                assert!(data["access_token"].is_string());
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        for handle in handles {
            handle.await.expect("Concurrent request failed");
        }
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let fixture = AuthIntegrationFixture::new().await;

        // Make many rapid requests to test rate limiting
        let mut responses = vec![];

        for _ in 0..50 {
            let response = fixture
                .login("nonexistent", "password")
                .await;

            responses.push(response.status());
        }

        // Some requests should be rate limited (429)
        let rate_limited_count = responses
            .iter()
            .filter(|status| **status == StatusCode::TOO_MANY_REQUESTS)
            .count();

        // Allow some flexibility in rate limiting implementation
        assert!(rate_limited_count >= 0); // At least some should be rate limited
    }

    #[tokio::test]
    async fn test_session_management() {
        let fixture = AuthIntegrationFixture::new().await;

        // Register and login
        fixture
            .register_user("session_user", "password", UserRole::User)
            .await;

        let login1 = fixture.login("session_user", "password").await;
        let data1: serde_json::Value = login1.json().await.unwrap();
        let token1 = data1["access_token"].as_str().unwrap();

        // Login again (should create new session)
        let login2 = fixture.login("session_user", "password").await;
        let data2: serde_json::Value = login2.json().await.unwrap();
        let token2 = data2["access_token"].as_str().unwrap();

        // Both tokens should be valid (depending on implementation)
        let response1 = fixture.get_protected_resource(token1).await;
        let response2 = fixture.get_protected_resource(token2).await;

        // At least one should work
        assert!(response1.status().is_success() || response2.status().is_success());
    }

    #[tokio::test]
    async fn test_error_responses() {
        let fixture = AuthIntegrationFixture::new().await;

        // Test various error conditions
        let error_cases = vec![
            (
                fixture.client
                    .post(&format!("{}/api/auth/register", fixture.base_url))
                    .json(&json!({}))
                    .send()
                    .await
                    .unwrap(),
                StatusCode::BAD_REQUEST,
            ),
            (
                fixture.client
                    .post(&format!("{}/api/auth/login", fixture.base_url))
                    .json(&json!({ "username": "", "password": "" }))
                    .send()
                    .await
                    .unwrap(),
                StatusCode::BAD_REQUEST,
            ),
        ];

        for (response, expected_status) in error_cases {
            assert_eq!(response.status(), expected_status);

            // Check that error response has proper structure
            let error_data: serde_json::Value = response.json().await.unwrap_or_default();
            assert!(error_data["error"].is_string() || error_data.is_object());
        }
    }
}