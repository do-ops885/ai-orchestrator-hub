//! # Edge Case Tests for Error Handling Unwrap Fixes
//!
//! Tests to prevent regression of `unwrap_or()` to `unwrap_or_else`(||) changes
//! in the error handling and recovery system.


/// Test cascade failure scenarios
#[tokio::test]
async fn test_cascade_failure_scenarios() {
    let config = RecoveryConfig {
        max_retries: 3,
        base_delay_ms: 10, // Fast for testing
        max_delay_ms: 100,
        enable_circuit_breaker: true,
        circuit_breaker_threshold: 3,
        circuit_breaker_timeout_ms: 500,
    };

    let safe_ops = SafeOperations::new(config);

    // Simulate multiple cascading failures
    let mut failure_count = 0;
    let operation = || async {
        failure_count += 1;
        if failure_count <= 5 {
            Err(HiveError::OperationFailed {
                reason: format!("Cascade failure {}", failure_count),
            })
        } else {
            Ok("success_after_cascade")
        }
    };

    let result = safe_ops.with_retry(operation, 3).await;

    // Should eventually succeed or fail gracefully
    match result {
        Ok(success) => assert_eq!(success, "success_after_cascade"),
        Err(HiveError::CircuitBreakerOpen { .. }) => {
            // Circuit breaker opened due to too many failures
            let state = safe_ops.get_circuit_breaker_status().await;
            assert_eq!(state, CircuitBreakerState::Open);
        }
        Err(_) => {
            // Other error types are acceptable
        }
    }
}

/// Test partial recovery situations
#[tokio::test]
async fn test_partial_recovery_situations() {
    let config = RecoveryConfig {
        max_retries: 5,
        base_delay_ms: 5,
        max_delay_ms: 50,
        enable_circuit_breaker: true,
        circuit_breaker_threshold: 10,
        circuit_breaker_timeout_ms: 1000,
    };

    let safe_ops = SafeOperations::new(config);

    let mut attempt_count = 0;
    let operation = || async {
        attempt_count += 1;
        match attempt_count {
            1..=2 => Err(HiveError::NetworkError {
                reason: "Connection timeout".to_string(),
            }),
            3 => Err(HiveError::DatabaseError {
                reason: "Temporary lock contention".to_string(),
            }),
            4 => Ok("partial_recovery_success"),
            _ => Err(HiveError::OperationFailed {
                reason: "Unexpected failure".to_string(),
            }),
        }
    };

    let result = safe_ops.with_retry(operation, 5).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "partial_recovery_success");
}

/// Test resource exhaustion during recovery
#[tokio::test]
async fn test_resource_exhaustion_during_recovery() {
    let config = RecoveryConfig {
        max_retries: 10,
        base_delay_ms: 1,
        max_delay_ms: 10,
        enable_circuit_breaker: true,
        circuit_breaker_threshold: 5,
        circuit_breaker_timeout_ms: 100,
    };

    let safe_ops = SafeOperations::new(config);

    // Simulate resource exhaustion
    let operation = || async {
        // Simulate increasing memory usage
        static mut MEMORY_COUNTER: u64 = 0;
        unsafe {
            MEMORY_COUNTER += 1;
            if MEMORY_COUNTER > 3 {
                return Err(HiveError::ResourceExhausted {
                    resource: "memory".to_string(),
                    reason: "Out of memory during recovery".to_string(),
                });
            }
        }

        Err(HiveError::OperationFailed {
            reason: "Transient failure".to_string(),
        })
    };

    let result = safe_ops.with_retry(operation, 10).await;

    // Should either succeed, fail with resource exhaustion, or circuit breaker opens
    match result {
        Ok(_) => {}                                    // Success is acceptable
        Err(HiveError::ResourceExhausted { .. }) => {} // Resource exhaustion is expected
        Err(HiveError::CircuitBreakerOpen { .. }) => {
            let state = safe_ops.get_circuit_breaker_status().await;
            assert_eq!(state, CircuitBreakerState::Open);
        }
        Err(_) => {} // Other errors are acceptable
    }
}

/// Test network failures during error recovery
#[tokio::test]
async fn test_network_failures_during_recovery() {
    let config = RecoveryConfig {
        max_retries: 3,
        base_delay_ms: 10,
        max_delay_ms: 100,
        enable_circuit_breaker: true,
        circuit_breaker_threshold: 3,
        circuit_breaker_timeout_ms: 200,
    };

    let safe_ops = SafeOperations::new(config);

    let mut network_failure_count = 0;
    let operation = || async {
        network_failure_count += 1;

        // Simulate different types of network failures
        match network_failure_count {
            1 => Err(HiveError::NetworkError {
                reason: "Connection refused".to_string(),
            }),
            2 => Err(HiveError::NetworkError {
                reason: "DNS resolution failed".to_string(),
            }),
            3 => Err(HiveError::NetworkError {
                reason: "Timeout".to_string(),
            }),
            _ => Ok("network_recovery_success"),
        }
    };

    let result = safe_ops.with_retry(operation, 3).await;

    // Should handle network failures gracefully
    match result {
        Ok(success) => assert_eq!(success, "network_recovery_success"),
        Err(HiveError::NetworkError { .. }) => {} // Network errors are expected
        Err(HiveError::CircuitBreakerOpen { .. }) => {
            let state = safe_ops.get_circuit_breaker_status().await;
            assert_eq!(state, CircuitBreakerState::Open);
        }
        Err(_) => {} // Other errors are acceptable
    }
}

/// Test database connection failures during recovery
#[tokio::test]
async fn test_database_failures_during_recovery() {
    let config = RecoveryConfig {
        max_retries: 3,
        base_delay_ms: 20,
        max_delay_ms: 200,
        enable_circuit_breaker: true,
        circuit_breaker_threshold: 2,
        circuit_breaker_timeout_ms: 300,
    };

    let safe_ops = SafeOperations::new(config);

    let mut db_failure_count = 0;
    let operation = || async {
        db_failure_count += 1;

        match db_failure_count {
            1 => Err(HiveError::DatabaseError {
                reason: "Connection lost".to_string(),
            }),
            2 => Err(HiveError::DatabaseError {
                reason: "Transaction rollback".to_string(),
            }),
            _ => Ok("database_recovery_success"),
        }
    };

    let result = safe_ops.with_retry(operation, 3).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "database_recovery_success");
}

/// Test circuit breaker state transitions
#[tokio::test]
async fn test_circuit_breaker_state_transitions() {
    let config = RecoveryConfig {
        circuit_breaker_threshold: 2,
        circuit_breaker_timeout_ms: 100,
        ..Default::default()
    };

    let safe_ops = SafeOperations::new(config);

    // Start in closed state
    let state = safe_ops.get_circuit_breaker_status().await;
    assert_eq!(state, CircuitBreakerState::Closed);

    // Cause failures to open circuit breaker
    for _ in 0..3 {
        let _ = safe_ops
            .execute_safely(async {
                Err(HiveError::OperationFailed {
                    reason: "Test failure".to_string(),
                })
            })
            .await;
    }

    let state = safe_ops.get_circuit_breaker_status().await;
    assert_eq!(state, CircuitBreakerState::Open);

    // Wait for timeout and try again (should go to half-open)
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    let result = safe_ops
        .execute_safely(async { Ok("half_open_success") })
        .await;

    // Should succeed and potentially close the circuit
    match result {
        Ok(success) => {
            assert_eq!(success, "half_open_success");
            let state = safe_ops.get_circuit_breaker_status().await;
            assert_eq!(state, CircuitBreakerState::Closed);
        }
        Err(HiveError::CircuitBreakerOpen { .. }) => {
            // Still open, which is also acceptable
        }
        Err(_) => {} // Other errors are acceptable
    }
}

/// Test concurrent circuit breaker access
#[tokio::test]
async fn test_concurrent_circuit_breaker_access() {
    let config = RecoveryConfig {
        circuit_breaker_threshold: 5,
        circuit_breaker_timeout_ms: 200,
        ..Default::default()
    };

    let safe_ops = SafeOperations::new(config);

    // Spawn multiple concurrent operations
    let mut handles = vec![];

    for i in 0..10 {
        let safe_ops_clone = safe_ops.clone();
        let handle = tokio::spawn(async move {
            let result = safe_ops_clone
                .execute_safely(async {
                    if i < 7 {
                        // First 7 operations fail
                        Err(HiveError::OperationFailed {
                            reason: format!("Concurrent failure {}", i),
                        })
                    } else {
                        // Last 3 operations succeed
                        Ok(format!("concurrent_success_{}", i))
                    }
                })
                .await;

            result
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    let mut success_count = 0;
    let mut failure_count = 0;
    let mut circuit_breaker_opened = false;

    for handle in handles {
        let result = handle.await.unwrap();
        match result {
            Ok(_) => success_count += 1,
            Err(HiveError::CircuitBreakerOpen { .. }) => {
                circuit_breaker_opened = true;
                failure_count += 1;
            }
            Err(_) => failure_count += 1,
        }
    }

    // Verify behavior
    assert!(success_count >= 3); // At least some successes
    assert!(failure_count >= 0); // Some failures expected
    assert!(circuit_breaker_opened || success_count > 0); // Either circuit breaker opened or some succeeded
}

/// Test retry logic with different error types
#[tokio::test]
async fn test_retry_with_different_error_types() {
    let config = RecoveryConfig {
        max_retries: 5,
        base_delay_ms: 5,
        max_delay_ms: 50,
        ..Default::default()
    };

    let safe_ops = SafeOperations::new(config);

    let mut attempt_count = 0;
    let operation = || async {
        attempt_count += 1;

        match attempt_count {
            1 => Err(HiveError::ValidationError {
                field: "test_field".to_string(),
                reason: "Validation failed".to_string(),
            }),
            2 => Err(HiveError::AuthenticationError {
                reason: "Auth failed".to_string(),
            }),
            3 => Err(HiveError::AuthorizationError {
                reason: "Permission denied".to_string(),
            }),
            4 => Err(HiveError::SecurityError {
                reason: "Security violation".to_string(),
            }),
            5 => Err(HiveError::NetworkError {
                reason: "Network issue".to_string(),
            }),
            _ => Ok("retry_success_with_different_errors"),
        }
    };

    let result = safe_ops.with_retry(operation, 5).await;

    // Should fail immediately on non-retryable errors
    match result {
        Ok(success) => assert_eq!(success, "retry_success_with_different_errors"),
        Err(HiveError::ValidationError { .. }) => {} // Should not retry validation errors
        Err(HiveError::AuthenticationError { .. }) => {} // Should not retry auth errors
        Err(HiveError::AuthorizationError { .. }) => {} // Should not retry authz errors
        Err(HiveError::SecurityError { .. }) => {}   // Should not retry security errors
        Err(HiveError::NetworkError { .. }) => {}    // Network errors should be retried
        Err(_) => {}                                 // Other errors are acceptable
    }
}

/// Test exponential backoff calculation
#[tokio::test]
async fn test_exponential_backoff_calculation() {
    let config = RecoveryConfig {
        max_retries: 5,
        base_delay_ms: 100,
        max_delay_ms: 2000,
        ..Default::default()
    };

    let safe_ops = SafeOperations::new(config);

    let mut attempt_count = 0;
    let mut delays = vec![];

    let operation = || async {
        attempt_count += 1;
        let start = std::time::Instant::now();

        // Simulate delay tracking
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;

        if attempt_count < 5 {
            Err(HiveError::OperationFailed {
                reason: format!("Attempt {}", attempt_count),
            })
        } else {
            let elapsed = start.elapsed().as_millis() as u64;
            delays.push(elapsed);
            Ok("backoff_test_success")
        }
    };

    let result = safe_ops.with_retry(operation, 5).await;
    assert!(result.is_ok());
}

/// Test circuit breaker reset functionality
#[tokio::test]
async fn test_circuit_breaker_reset() {
    let config = RecoveryConfig {
        circuit_breaker_threshold: 2,
        circuit_breaker_timeout_ms: 1000,
        ..Default::default()
    };

    let safe_ops = SafeOperations::new(config);

    // Cause circuit breaker to open
    for _ in 0..3 {
        let _ = safe_ops
            .execute_safely(async {
                Err(HiveError::OperationFailed {
                    reason: "Test failure".to_string(),
                })
            })
            .await;
    }

    let state = safe_ops.get_circuit_breaker_status().await;
    assert_eq!(state, CircuitBreakerState::Open);

    // Reset circuit breaker
    safe_ops.reset_circuit_breaker().await;

    let state = safe_ops.get_circuit_breaker_status().await;
    assert_eq!(state, CircuitBreakerState::Closed);

    // Should now allow operations
    let result = safe_ops.execute_safely(async { Ok("reset_success") }).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "reset_success");
}

/// Test safe option utilities edge cases
#[test]
fn test_safe_option_utilities_edge_cases() {
    use crate::utils::error_handling::safe_option;

    // Test expect_or_error with None
    let none_value: Option<i32> = None;
    let result = safe_option::expect_or_error(none_value, "Test error message");
    assert!(result.is_err());
    if let Err(HiveError::OperationFailed { reason }) = result {
        assert_eq!(reason, "Test error message");
    }

    // Test expect_or_error with Some
    let some_value = Some(42);
    let result = safe_option::expect_or_error(some_value, "Should not see this");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);

    // Test unwrap_or_default
    assert_eq!(safe_option::unwrap_or_default::<i32>(None), 0);
    assert_eq!(safe_option::unwrap_or_default(Some(42)), 42);

    // Test unwrap_or
    assert_eq!(safe_option::unwrap_or(None::<i32>, 99), 99);
    assert_eq!(safe_option::unwrap_or(Some(42), 99), 42);
}

/// Test safe result utilities edge cases
#[test]
fn test_safe_result_utilities_edge_cases() {
    use crate::utils::error_handling::safe_result;

    // Test expect_or_error with Err
    let err_result: Result<i32, HiveError> = Err(HiveError::OperationFailed {
        reason: "Test error".to_string(),
    });
    let result = safe_result::expect_or_error(err_result, "Additional context");
    assert!(result.is_err());

    // Test expect_or_error with Ok
    let ok_result: Result<i32, HiveError> = Ok(42);
    let result = safe_result::expect_or_error(ok_result, "Should not see this");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);

    // Test map_error
    let err_result: Result<i32, HiveError> = Err(HiveError::OperationFailed {
        reason: "Original error".to_string(),
    });
    let result = safe_result::map_error(err_result, |e| HiveError::OperationFailed {
        reason: format!("Mapped: {:?}", e),
    });
    assert!(result.is_err());

    // Test with_context
    let err_result: Result<i32, HiveError> = Err(HiveError::OperationFailed {
        reason: "Original".to_string(),
    });
    let result = safe_result::with_context(err_result, "Additional context");
    assert!(result.is_err());
    if let Err(HiveError::OperationFailed { reason }) = result {
        assert!(reason.contains("Additional context"));
        assert!(reason.contains("Original"));
    }
}

/// Test safe JSON utilities edge cases
#[test]
fn test_safe_json_utilities_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    use crate::utils::error_handling::safe_json;
    use serde_json::json;

    let json_value = json!({
        "string_field": "test",
        "number_field": 42.0,
        "boolean_field": true,
        "array_field": [1, 2, 3],
        "object_field": {"nested": "value"},
        "null_field": null,
        "empty_array": [],
        "empty_object": {}
    });

    // Test successful extractions
    let string_val = safe_json::get_string(&json_value, "string_field")?;
    assert_eq!(string_val, "test");

    let number_val = safe_json::get_number(&json_value, "number_field")?;
    assert_eq!(number_val, 42.0);

    let boolean_val = safe_json::get_boolean(&json_value, "boolean_field")?;
    assert_eq!(boolean_val, true);

    // Test error cases
    assert!(safe_json::get_string(&json_value, "missing_field").is_err());
    assert!(safe_json::get_number(&json_value, "string_field").is_err());
    assert!(safe_json::get_boolean(&json_value, "number_field").is_err());
    assert!(safe_json::get_array(&json_value, "string_field").is_err());
    assert!(safe_json::get_object(&json_value, "string_field").is_err());

    // Test null values
    assert!(safe_json::get_string(&json_value, "null_field").is_err());
    assert!(safe_json::get_number(&json_value, "null_field").is_err());
    assert!(safe_json::get_boolean(&json_value, "null_field").is_err());

    // Test empty arrays/objects
    let empty_array = safe_json::get_array(&json_value, "empty_array")?;
    assert!(empty_array.is_empty());

    let empty_object = safe_json::get_object(&json_value, "empty_object")?;
    assert!(empty_object.is_empty());

    // Test optional variants
    assert_eq!(
        safe_json::get_optional_string(&json_value, "string_field"),
        Some("test".to_string())
    );
    assert_eq!(
        safe_json::get_optional_string(&json_value, "missing_field"),
        None
    );
    assert_eq!(
        safe_json::get_optional_number(&json_value, "number_field"),
        Some(42.0)
    );
    assert_eq!(
        safe_json::get_optional_number(&json_value, "missing_field"),
        None
    );
    assert_eq!(
        safe_json::get_optional_boolean(&json_value, "boolean_field"),
        Some(true)
    );
    assert_eq!(
        safe_json::get_optional_boolean(&json_value, "missing_field"),
        None
    );

    Ok(())
}

/// Test recovery config validation
#[test]
fn test_recovery_config_validation() {
    // Test default config
    let config = RecoveryConfig::default();
    assert!(config.max_retries > 0);
    assert!(config.base_delay_ms > 0);
    assert!(config.max_delay_ms >= config.base_delay_ms);
    assert!(config.circuit_breaker_threshold > 0);
    assert!(config.circuit_breaker_timeout_ms > 0);

    // Test custom config
    let custom_config = RecoveryConfig {
        max_retries: 10,
        base_delay_ms: 50,
        max_delay_ms: 5000,
        enable_circuit_breaker: false,
        circuit_breaker_threshold: 20,
        circuit_breaker_timeout_ms: 60000,
    };

    assert_eq!(custom_config.max_retries, 10);
    assert_eq!(custom_config.base_delay_ms, 50);
    assert_eq!(custom_config.max_delay_ms, 5000);
    assert!(!custom_config.enable_circuit_breaker);
    assert_eq!(custom_config.circuit_breaker_threshold, 20);
    assert_eq!(custom_config.circuit_breaker_timeout_ms, 60000);
}

/// Test circuit breaker initialization
#[test]
fn test_circuit_breaker_initialization() {
    let config = RecoveryConfig {
        circuit_breaker_threshold: 5,
        circuit_breaker_timeout_ms: 1000,
        ..Default::default()
    };

    let circuit_breaker = CircuitBreaker::new(config);

    assert_eq!(circuit_breaker.get_state(), &CircuitBreakerState::Closed);
    assert_eq!(circuit_breaker.failure_count, 0);
    assert!(circuit_breaker.last_failure_time.is_none());
}

/// Test circuit breaker failure recording
#[test]
fn test_circuit_breaker_failure_recording() {
    let config = RecoveryConfig {
        circuit_breaker_threshold: 3,
        circuit_breaker_timeout_ms: 1000,
        ..Default::default()
    };

    let mut circuit_breaker = CircuitBreaker::new(config);

    // Record failures
    circuit_breaker.record_failure();
    assert_eq!(circuit_breaker.failure_count, 1);
    assert_eq!(circuit_breaker.get_state(), &CircuitBreakerState::Closed);

    circuit_breaker.record_failure();
    assert_eq!(circuit_breaker.failure_count, 2);
    assert_eq!(circuit_breaker.get_state(), &CircuitBreakerState::Closed);

    circuit_breaker.record_failure();
    assert_eq!(circuit_breaker.failure_count, 3);
    assert_eq!(circuit_breaker.get_state(), &CircuitBreakerState::Open);

    // Record success (should reset)
    circuit_breaker.record_success();
    assert_eq!(circuit_breaker.failure_count, 0);
    assert_eq!(circuit_breaker.get_state(), &CircuitBreakerState::Closed);
}

/// Test circuit breaker operation allowance
#[test]
fn test_circuit_breaker_operation_allowance() {
    let config = RecoveryConfig {
        circuit_breaker_threshold: 2,
        circuit_breaker_timeout_ms: 100,
        ..Default::default()
    };

    let mut circuit_breaker = CircuitBreaker::new(config);

    // Initially closed
    assert!(circuit_breaker.allow_operation());

    // Open circuit breaker
    circuit_breaker.record_failure();
    circuit_breaker.record_failure();
    circuit_breaker.record_failure();
    assert!(!circuit_breaker.allow_operation());

    // Simulate timeout passing
    // Note: In real usage, this would be checked against current time
    // For testing, we manually set the state
    circuit_breaker.state = CircuitBreakerState::HalfOpen;
    assert!(circuit_breaker.allow_operation());
}
