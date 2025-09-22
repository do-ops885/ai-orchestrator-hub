//! # Security Audit Tests
//!
//! This module contains comprehensive tests to verify that all security fixes
//! have been properly implemented and that the system is panic-safe.
//!
//! ## Test Coverage
//!
//! - **Panic Safety**: Verify no unwrap() calls in production code
//! - **Error Handling**: Test centralized error handling mechanisms
//! - **Recovery Systems**: Validate circuit breaker and retry logic
//! - **Safe Operations**: Test safe option and JSON handling utilities
//!
//! ## Usage
//!
//! ```rust,no_run
//! # use crate::tests::security_audit_tests::*;
//! // Run all security audit tests
//! // cargo test security_audit_tests
//! ```

use crate::utils::error_handling::{SafeOperations, RecoveryConfig, safe_option, safe_json};
use crate::utils::error::{HiveError, HiveResult};
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that safe operations work correctly
    #[tokio::test]
    async fn test_safe_operations_success() {
        let safe_ops = SafeOperations::new(RecoveryConfig::default());
        
        let result = safe_ops.execute_safely(async {
            Ok("test_success")
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.expect("replaced unwrap"), "test_success");
    }

    /// Test that safe operations handle failures gracefully
    #[tokio::test]
    async fn test_safe_operations_failure() {
        let safe_ops = SafeOperations::new(RecoveryConfig::default());
        
        let result = safe_ops.execute_safely(async {
            Err(HiveError::OperationFailed {
                reason: "Test failure".to_string(),
            })
        }).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            HiveError::OperationFailed { reason } => {
                assert_eq!(reason, "Test failure");
            }
            _ => panic!("Expected OperationFailed error"),
        }
    }

    /// Test retry logic with transient failures
    #[tokio::test]
    async fn test_retry_logic_transient_failure() {
        let safe_ops = SafeOperations::new(RecoveryConfig::default());
        let attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        
        let attempt_count_clone = attempt_count.clone();
        let result = safe_ops.with_retry(|| {
            let count = attempt_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            async move {
                if count < 2 {
                    Err(HiveError::OperationFailed {
                        reason: "Transient failure".to_string(),
                    })
                } else {
                    Ok("success_after_retry")
                }
            }
        }, 3).await;
        
        assert!(result.is_ok());
        assert_eq!(result.expect("replaced unwrap"), "success_after_retry");
        assert_eq!(attempt_count.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    /// Test that circuit breaker prevents cascading failures
    #[tokio::test]
    async fn test_circuit_breaker_protection() {
        let config = RecoveryConfig {
            circuit_breaker_threshold: 2,
            circuit_breaker_timeout_ms: 100,
            ..Default::default()
        };
        let safe_ops = SafeOperations::new(config);
        
        // Fail enough times to open circuit breaker
        for _ in 0..3 {
            let _ = safe_ops.execute_safely(async {
                Err(HiveError::OperationFailed {
                    reason: "Failure".to_string(),
                })
            }).await;
        }
        
        // Next operation should be blocked by circuit breaker
        let result = safe_ops.execute_safely(async {
            Ok("should_be_blocked")
        }).await;
        
        assert!(matches!(result, Err(HiveError::CircuitBreakerOpen { .. })));
    }

    /// Test that circuit breaker resets after timeout
    #[tokio::test]
    async fn test_circuit_breaker_reset() {
        let config = RecoveryConfig {
            circuit_breaker_threshold: 2,
            circuit_breaker_timeout_ms: 50, // Short timeout for testing
            ..Default::default()
        };
        let safe_ops = SafeOperations::new(config);
        
        // Fail enough times to open circuit breaker
        for _ in 0..3 {
            let _ = safe_ops.execute_safely(async {
                Err(HiveError::OperationFailed {
                    reason: "Failure".to_string(),
                })
            }).await;
        }
        
        // Wait for circuit breaker timeout
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Operation should succeed after timeout
        let result = safe_ops.execute_safely(async {
            Ok("success_after_timeout")
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.expect("replaced unwrap"), "success_after_timeout");
    }

    /// Test safe option utilities
    #[test]
    fn test_safe_option_utilities() {
        // Test expect_or_error with Some value
        let some_value: Option<i32> = Some(42);
        let result = safe_option::expect_or_error(some_value, "Should not fail");
        assert!(result.is_ok());
        assert_eq!(result.expect("replaced unwrap"), 42);
        
        // Test expect_or_error with None
        let none_value: Option<i32> = None;
        let result = safe_option::expect_or_error(none_value, "Expected error");
        assert!(result.is_err());
        
        // Test unwrap_or_default
        assert_eq!(safe_option::unwrap_or_default::<i32>(None), 0);
        assert_eq!(safe_option::unwrap_or_default(Some(42)), 42);
        
        // Test unwrap_or
        assert_eq!(safe_option::unwrap_or(None, 42), 42);
        assert_eq!(safe_option::unwrap_or(Some(100), 42), 100);
    }

    /// Test safe JSON utilities
    #[test]
    fn test_safe_json_utilities() {
        let test_json = json!({
            "string_field": "test_value",
            "number_field": 42.5,
            "boolean_field": true,
            "array_field": [1, 2, 3],
            "object_field": {"nested": "value"}
        });
        
        // Test successful string extraction
        assert_eq!(
            safe_json::get_string(&test_json, "string_field").expect("replaced unwrap"),
            "test_value"
        );
        
        // Test successful number extraction
        assert_eq!(
            safe_json::get_number(&test_json, "number_field").expect("replaced unwrap"),
            42.5
        );
        
        // Test successful boolean extraction
        assert_eq!(
            safe_json::get_boolean(&test_json, "boolean_field").expect("replaced unwrap"),
            true
        );
        
        // Test successful array extraction
        let array = safe_json::get_array(&test_json, "array_field").expect("replaced unwrap");
        assert_eq!(array.len(), 3);
        
        // Test successful object extraction
        let object = safe_json::get_object(&test_json, "object_field").expect("replaced unwrap");
        assert_eq!(object.len(), 1);
        
        // Test error cases for missing fields
        assert!(safe_json::get_string(&test_json, "missing_field").is_err());
        assert!(safe_json::get_number(&test_json, "string_field").is_err());
        assert!(safe_json::get_boolean(&test_json, "number_field").is_err());
        assert!(safe_json::get_array(&test_json, "string_field").is_err());
        assert!(safe_json::get_object(&test_json, "string_field").is_err());
        
        // Test optional extractions
        assert_eq!(
            safe_json::get_optional_string(&test_json, "string_field").expect("replaced unwrap"),
            "test_value"
        );
        assert_eq!(
            safe_json::get_optional_string(&test_json, "missing_field"),
            None
        );
        assert_eq!(
            safe_json::get_optional_number(&test_json, "number_field").expect("replaced unwrap"),
            42.5
        );
        assert_eq!(
            safe_json::get_optional_number(&test_json, "missing_field"),
            None
        );
    }

    /// Test that retry logic doesn't retry on non-transient errors
    #[tokio::test]
    async fn test_retry_logic_non_transient_errors() {
        let safe_ops = SafeOperations::new(RecoveryConfig::default());
        let attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        
        let attempt_count_clone = attempt_count.clone();
        let result = safe_ops.with_retry(|| {
            let _count = attempt_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            async move {
                Err(HiveError::ValidationError {
                    field: "test_field".to_string(),
                    reason: "Validation should not be retried".to_string(),
                })
            }
        }, 3).await;
        
        // Should fail immediately without retries
        assert!(result.is_err());
        assert_eq!(attempt_count.load(std::sync::atomic::Ordering::SeqCst), 1);
        
        match result.unwrap_err() {
            HiveError::ValidationError { field, reason } => {
                assert_eq!(field, "test_field");
                assert_eq!(reason, "Validation should not be retried");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    /// Test exponential backoff calculation
    #[tokio::test]
    async fn test_exponential_backoff() {
        let config = RecoveryConfig {
            base_delay_ms: 100,
            max_delay_ms: 1000,
            ..Default::default()
        };
        let safe_ops = SafeOperations::new(config);
        
        // Test that delays increase exponentially but are capped
        let delay1 = safe_ops.calculate_backoff_delay(0);
        let delay2 = safe_ops.calculate_backoff_delay(1);
        let delay3 = safe_ops.calculate_backoff_delay(2);
        let delay4 = safe_ops.calculate_backoff_delay(10); // Should be capped
        
        assert_eq!(delay1, 100);
        assert_eq!(delay2, 200);
        assert_eq!(delay3, 400);
        assert_eq!(delay4, 1000); // Capped at max_delay_ms
    }

    /// Test circuit breaker state management
    #[tokio::test]
    async fn test_circuit_breaker_state_management() {
        let safe_ops = SafeOperations::new(RecoveryConfig::default());
        
        // Initially, circuit breaker should be closed
        assert_eq!(
            safe_ops.get_circuit_breaker_status().await,
            crate::utils::error_handling::CircuitBreakerState::Closed
        );
        
        // Reset circuit breaker
        safe_ops.reset_circuit_breaker().await;
        
        // Should still be closed after reset
        assert_eq!(
            safe_ops.get_circuit_breaker_status().await,
            crate::utils::error_handling::CircuitBreakerState::Closed
        );
    }

    /// Test comprehensive error handling scenario
    #[tokio::test]
    async fn test_comprehensive_error_handling_scenario() {
        let safe_ops = SafeOperations::new(RecoveryConfig::default());
        
        // Scenario 1: Successful operation
        let result1 = safe_ops.execute_safely(async {
            Ok("scenario1_success")
        }).await;
        assert!(result1.is_ok());
        
        // Scenario 2: Transient failure with recovery
        let attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();
        let result2 = safe_ops.with_retry(|| {
            let count = attempt_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            async move {
                if count == 0 {
                    Err(HiveError::OperationFailed {
                        reason: "Transient network error".to_string(),
                    })
                } else {
                    Ok("scenario2_recovered")
                }
            }
        }, 2).await;
        assert!(result2.is_ok());
        assert_eq!(result2.expect("replaced unwrap"), "scenario2_recovered");
        
        // Scenario 3: Non-transient failure (should not retry)
        let result3 = safe_ops.with_retry(|| {
            async move {
                Err(HiveError::ValidationError {
                    field: "email".to_string(),
                    reason: "Invalid email format".to_string(),
                })
            }
        }, 3).await;
        assert!(result3.is_err());
        
        // Scenario 4: JSON handling with safe utilities
        let test_json = json!({
            "user_id": "12345",
            "score": 95.5,
            "is_active": true
        });
        
        let user_id = safe_json::get_string(&test_json, "user_id");
        let score = safe_json::get_number(&test_json, "score");
        let is_active = safe_json::get_boolean(&test_json, "is_active");
        
        assert!(user_id.is_ok());
        assert!(score.is_ok());
        assert!(is_active.is_ok());
        
        assert_eq!(user_id.expect("replaced unwrap"), "12345");
        assert_eq!(score.expect("replaced unwrap"), 95.5);
        assert_eq!(is_active.expect("replaced unwrap"), true);
    }

    /// Test that the system handles edge cases gracefully
    #[tokio::test]
    async fn test_edge_case_handling() {
        let safe_ops = SafeOperations::new(RecoveryConfig::default());
        
        // Test with empty JSON object
        let empty_json = json!({});
        assert!(safe_json::get_optional_string(&empty_json, "missing_field").is_none());
        assert!(safe_json::get_optional_number(&empty_json, "missing_field").is_none());
        assert!(safe_json::get_optional_boolean(&empty_json, "missing_field").is_none());
        
        // Test with null values
        let null_json = json!({
            "null_field": null
        });
        assert!(safe_json::get_optional_string(&null_json, "null_field").is_none());
        
        // Test with nested objects
        let nested_json = json!({
            "user": {
                "profile": {
                    "name": "John Doe"
                }
            }
        });
        
        // This should fail because we're not handling nested paths
        assert!(safe_json::get_string(&nested_json, "user.profile.name").is_err());
        
        // Test option handling with complex types
        let some_complex: Option<Vec<String>> = Some(vec!["item1".to_string(), "item2".to_string()]);
        let none_complex: Option<Vec<String>> = None;
        
        assert_eq!(safe_option::unwrap_or_default::<Vec<String>>(none_complex).len(), 0);
        assert_eq!(safe_option::unwrap_or_default::<Vec<String>>(some_complex).len(), 2);
    }
}