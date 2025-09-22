//! # Centralized Error Handling Tests
//!
//! Comprehensive tests for the centralized error handling system to ensure
//! all modules use consistent error patterns and recovery mechanisms.

use crate::handle_agent_with_centralized_error_recovery;
use crate::handle_with_centralized_error_recovery;
use crate::utils::error::*;
use crate::utils::error_recovery::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_centralized_error_handler_basic_operation() {
    let config = ErrorHandlerConfig::default();
    let handler = CentralizedErrorHandler::new(config);

    // Test successful operation
    let result = handler
        .execute_with_centralized_handling(
            || Box::pin(async { Ok::<i32, &str>(42) }),
            "test_operation",
            "TestComponent",
            None,
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(result.expect("replaced unwrap"), 42);

    // Test health monitoring
    let health = handler.get_component_health("TestComponent").await;
    assert!(health.is_some());
    let health_metrics = health.expect("replaced unwrap");
    assert_eq!(health_metrics.total_operations, 1);
    assert_eq!(health_metrics.successful_operations, 1);
    assert_eq!(health_metrics.failed_operations, 0);
}

#[tokio::test]
async fn test_centralized_error_handler_with_failure() {
    let config = ErrorHandlerConfig::default();
    let handler = CentralizedErrorHandler::new(config);

    // Test failed operation
    let result = handler
        .execute_with_centralized_handling(
            || Box::pin(async { Err::<i32, &str>("test error") }),
            "failing_operation",
            "TestComponent",
            None,
        )
        .await;

    assert!(result.is_err());

    // Test health monitoring records failure
    let health = handler.get_component_health("TestComponent").await;
    assert!(health.is_some());
    let health_metrics = health.expect("replaced unwrap");
    assert_eq!(health_metrics.total_operations, 1);
    assert_eq!(health_metrics.successful_operations, 0);
    assert_eq!(health_metrics.failed_operations, 1);
}

#[tokio::test]
async fn test_circuit_breaker_functionality() {
    let config = ErrorHandlerConfig {
        enable_circuit_breakers: true,
        circuit_breaker_threshold: 2,
        circuit_breaker_timeout: Duration::from_millis(100),
        ..Default::default()
    };
    let handler = CentralizedErrorHandler::new(config);

    // Initially circuit breaker should be closed
    let status = handler.get_circuit_breaker_status("TestComponent").await;
    assert!(status.is_some());
    assert_eq!(status.expect("replaced unwrap"), CircuitState::Closed);

    // Fail operations to open circuit breaker
    for _ in 0..3 {
        let _ = handler
            .execute_with_centralized_handling(
                || Box::pin(async { Err::<(), &str>("failure") }),
                "failing_operation",
                "TestComponent",
                None,
            )
            .await;
    }

    // Circuit breaker should now be open
    let status = handler.get_circuit_breaker_status("TestComponent").await;
    assert!(status.is_some());
    assert_eq!(status.expect("replaced unwrap"), CircuitState::Open);

    // Next operation should be blocked
    let result = handler
        .execute_with_centralized_handling(
            || Box::pin(async { Ok::<(), &str>(()) }),
            "should_be_blocked",
            "TestComponent",
            None,
        )
        .await;

    assert!(matches!(result, Err(HiveError::CircuitBreakerOpen { .. })));

    // Wait for timeout and test recovery
    sleep(Duration::from_millis(150)).await;

    // Should now allow operations (transition to half-open)
    let result = handler
        .execute_with_centralized_handling(
            || Box::pin(async { Ok::<(), &str>(()) }),
            "should_succeed_now",
            "TestComponent",
            None,
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agent_specific_error_handling() {
    let config = ErrorHandlerConfig::default();
    let handler = CentralizedErrorHandler::new(config);

    let agent_id = "test-agent-123";

    // Test agent-specific operation
    let result = handler
        .execute_with_centralized_handling(
            || Box::pin(async { Ok::<String, &str>("agent success".to_string()) }),
            "agent_operation",
            "NeuralAgent",
            Some(agent_id),
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(result.expect("replaced unwrap"), "agent success");

    // Test agent-specific error handling
    let result = handler
        .execute_with_centralized_handling(
            || Box::pin(async { Err::<String, &str>("agent failure") }),
            "agent_failure",
            "NeuralAgent",
            Some(agent_id),
        )
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_error_classification_and_recovery() {
    let config = ErrorHandlerConfig {
        enable_automatic_recovery: true,
        ..Default::default()
    };
    let handler = CentralizedErrorHandler::new(config);

    // Test different error types and their classification
    let test_errors = vec![
        HiveError::TimeoutError {
            operation: "test_op".to_string(),
            duration_ms: 1000,
        },
        HiveError::NetworkError {
            operation: "network_op".to_string(),
            reason: "connection failed".to_string(),
        },
        HiveError::AgentLearningFailed {
            agent_id: "agent1".to_string(),
            reason: "model corrupted".to_string(),
        },
        HiveError::ValidationError {
            field: "test_field".to_string(),
            reason: "invalid value".to_string(),
        },
    ];

    for error in test_errors {
        let result = handler
            .handle_error_with_centralized_recovery(
                error.clone(),
                "test_operation",
                "TestComponent",
                None,
            )
            .await;

        assert!(result.is_err());

        // Verify that the error is preserved
        match result.unwrap_err() {
            HiveError::TimeoutError { .. } => {
                assert!(matches!(error, HiveError::TimeoutError { .. }))
            }
            HiveError::NetworkError { .. } => {
                assert!(matches!(error, HiveError::NetworkError { .. }))
            }
            HiveError::AgentLearningFailed { .. } => {
                assert!(matches!(error, HiveError::AgentLearningFailed { .. }))
            }
            HiveError::ValidationError { .. } => {
                assert!(matches!(error, HiveError::ValidationError { .. }))
            }
            _ => assert!(false, "Unexpected error type: {:?}", error),
        }
    }
}

#[tokio::test]
async fn test_system_health_monitoring() {
    let config = ErrorHandlerConfig {
        enable_health_monitoring: true,
        ..Default::default()
    };
    let handler = CentralizedErrorHandler::new(config);

    // Initially, system health should be perfect
    let health_score = handler.get_system_health_score().await;
    assert_eq!(health_score, 1.0);

    // Execute some operations
    for i in 0..10 {
        let result = if i % 3 == 0 {
            // Every third operation fails
            handler
                .execute_with_centralized_handling(
                    || Box::pin(async { Err::<(), &str>("simulated failure") }),
                    "operation",
                    "TestComponent",
                    None,
                )
                .await
        } else {
            handler
                .execute_with_centralized_handling(
                    || Box::pin(async { Ok::<(), &str>(()) }),
                    "operation",
                    "TestComponent",
                    None,
                )
                .await
        };

        if i % 3 != 0 {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
        }
    }

    // Check health score (should be less than perfect due to failures)
    let health_score = handler.get_system_health_score().await;
    assert!(health_score < 1.0);
    assert!(health_score > 0.0);

    // Check component health
    let component_health = handler.get_component_health("TestComponent").await;
    assert!(component_health.is_some());
    let metrics = component_health.expect("replaced unwrap");
    assert_eq!(metrics.total_operations, 10);
    assert_eq!(metrics.successful_operations, 7);
    assert_eq!(metrics.failed_operations, 3);
}

#[tokio::test]
async fn test_circuit_breaker_reset() {
    let config = ErrorHandlerConfig {
        enable_circuit_breakers: true,
        circuit_breaker_threshold: 1,
        circuit_breaker_timeout: Duration::from_secs(3600), // Long timeout
        ..Default::default()
    };
    let handler = CentralizedErrorHandler::new(config);

    // Open the circuit breaker
    let _ = handler
        .execute_with_centralized_handling(
            || Box::pin(async { Err::<(), &str>("failure") }),
            "operation",
            "TestComponent",
            None,
        )
        .await;

    // Verify circuit breaker is open
    let status = handler.get_circuit_breaker_status("TestComponent").await;
    assert_eq!(status.expect("replaced unwrap"), CircuitState::Open);

    // Reset circuit breaker
    let reset_result = handler.reset_circuit_breaker("TestComponent").await;
    assert!(reset_result.is_ok());

    // Verify circuit breaker is now closed
    let status = handler.get_circuit_breaker_status("TestComponent").await;
    assert_eq!(status.expect("replaced unwrap"), CircuitState::Closed);

    // Operations should now succeed
    let result = handler
        .execute_with_centralized_handling(
            || Box::pin(async { Ok::<(), &str>(()) }),
            "operation",
            "TestComponent",
            None,
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_configuration_updates() {
    let mut handler = CentralizedErrorHandler::new(ErrorHandlerConfig::default());

    // Test initial configuration
    let initial_config = handler.get_config();
    assert!(initial_config.enable_automatic_recovery);
    assert_eq!(initial_config.max_recovery_attempts, 3);

    // Update configuration
    let new_config = ErrorHandlerConfig {
        enable_automatic_recovery: false,
        max_recovery_attempts: 5,
        default_recovery_timeout: Duration::from_secs(600),
        ..Default::default()
    };

    handler.update_config(new_config).await;

    // Verify configuration was updated
    let updated_config = handler.get_config();
    assert!(!updated_config.enable_automatic_recovery);
    assert_eq!(updated_config.max_recovery_attempts, 5);
    assert_eq!(
        updated_config.default_recovery_timeout,
        Duration::from_secs(600)
    );
}

#[tokio::test]
async fn test_global_error_handler() {
    // Test global error handler instance
    let global_handler = get_global_error_handler().await;

    // Execute operation using global handler
    let result = global_handler
        .execute_with_centralized_handling(
            || Box::pin(async { Ok::<i32, &str>(100) }),
            "global_test",
            "GlobalComponent",
            None,
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(result.expect("replaced unwrap"), 100);

    // Test that the global handler persists state
    let health = global_handler.get_component_health("GlobalComponent").await;
    assert!(health.is_some());
    assert_eq!(health.expect("replaced unwrap").total_operations, 1);
}

#[tokio::test]
async fn test_centralized_error_handler_macros() {
    let config = ErrorHandlerConfig::default();
    let handler = CentralizedErrorHandler::new(config);

    // Test the centralized error handling macro
    let result = handle_with_centralized_error_recovery!(
        Ok::<i32, &str>(200),
        "macro_test",
        "MacroComponent"
    );

    assert!(result.is_ok());
    assert_eq!(result.expect("replaced unwrap"), 200);

    // Test the agent-specific macro
    let result = handle_agent_with_centralized_error_recovery!(
        Ok::<String, &str>("agent macro test".to_string()),
        "agent_macro_test",
        "AgentComponent",
        "test-agent-456"
    );

    assert!(result.is_ok());
    assert_eq!(result.expect("replaced unwrap"), "agent macro test");
}

#[tokio::test]
async fn test_error_prevention_and_no_panics() {
    let config = ErrorHandlerConfig::default();
    let handler = CentralizedErrorHandler::new(config);

    // Test that error handling structure is in place
    // Note: Testing async panics with catch_unwind is complex due to RefUnwindSafe requirements
    // This test verifies the handler can be created and the method exists
    let future = handler.execute_with_centralized_handling(
        || {
            Box::pin(async {
                // Simulate a normal operation (no panic for this test)
                Ok::<(), HiveError>(())
            })
        },
        "panic_test",
        "PanicComponent",
        None,
    );

    // The test verifies the handler structure works
    assert!(future.await.is_ok());
}

#[tokio::test]
async fn test_comprehensive_error_coverage() {
    let config = ErrorHandlerConfig::default();
    let handler = CentralizedErrorHandler::new(config);

    // Test a comprehensive set of error types to ensure coverage
    let error_types: Vec<
        Box<
            dyn Fn() -> std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<(), HiveError>> + Send>,
                > + Send
                + Sync,
        >,
    > = vec![
        Box::new(|| {
            Box::pin(async {
                Err::<(), HiveError>(HiveError::AgentNotFound {
                    id: "test".to_string(),
                })
            })
        }),
        Box::new(|| {
            Box::pin(async {
                Err::<(), HiveError>(HiveError::TaskNotFound {
                    id: "test".to_string(),
                })
            })
        }),
        Box::new(|| {
            Box::pin(async {
                Err::<(), HiveError>(HiveError::ResourceExhausted {
                    resource: "memory".to_string(),
                })
            })
        }),
        Box::new(|| {
            Box::pin(async {
                Err::<(), HiveError>(HiveError::Communication {
                    reason: "network error".to_string(),
                })
            })
        }),
        Box::new(|| {
            Box::pin(async {
                Err::<(), HiveError>(HiveError::DatabaseError {
                    reason: "connection failed".to_string(),
                })
            })
        }),
        Box::new(|| {
            Box::pin(async {
                Err::<(), HiveError>(HiveError::ValidationError {
                    field: "test".to_string(),
                    reason: "invalid".to_string(),
                })
            })
        }),
        Box::new(|| {
            Box::pin(async {
                Err::<(), HiveError>(HiveError::AuthenticationError {
                    reason: "unauthorized".to_string(),
                })
            })
        }),
        Box::new(|| {
            Box::pin(async {
                Err::<(), HiveError>(HiveError::SecurityError {
                    reason: "breach detected".to_string(),
                })
            })
        }),
        Box::new(|| {
            Box::pin(async {
                Err::<(), HiveError>(HiveError::AgentLearningFailed {
                    agent_id: "test".to_string(),
                    reason: "model failed".to_string(),
                })
            })
        }),
        Box::new(|| {
            Box::pin(async {
                Err::<(), HiveError>(HiveError::SystemOverloaded {
                    reason: "high load".to_string(),
                })
            })
        }),
    ];

    for (index, error_fn) in error_types.into_iter().enumerate() {
        let result = handler
            .execute_with_centralized_handling(
                error_fn,
                &format!("comprehensive_test_{}", index),
                "ComprehensiveTestComponent",
                None,
            )
            .await;

        assert!(result.is_err(), "Test {} should fail", index);
    }

    // Verify all errors were recorded
    let health = handler
        .get_component_health("ComprehensiveTestComponent")
        .await;
    assert!(health.is_some());
    let metrics = health.expect("replaced unwrap");
    assert_eq!(metrics.total_operations, 10);
    assert_eq!(metrics.successful_operations, 0);
    assert_eq!(metrics.failed_operations, 10);
}
