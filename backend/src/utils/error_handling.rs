//! # Centralized Error Handling and Recovery System
//!
//! This module provides comprehensive error handling, recovery mechanisms,
//! and panic-safe operations for the AI Orchestrator Hub.
//!
//! ## Features
//!
//! - **Panic-Safe Operations**: All operations handle errors gracefully without panics
//! - **Recovery Mechanisms**: Automatic recovery from common failure scenarios
//! - **Error Context**: Rich error information for debugging and monitoring
//! - **Circuit Breakers**: Prevent cascading failures
//! - **Retry Logic**: Configurable retry mechanisms for transient failures
//!
//! ## Usage
//!
//! ```rust,no_run
//! use crate::utils::error_handling::{SafeOperations, RecoveryConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let safe_ops = SafeOperations::new(RecoveryConfig::default());
//!
//! // Safe operation with automatic recovery
//! let result = safe_ops.execute_safely(|| {
//!     // Your operation here
//!     Ok("success")
//! }).await?;
//!
//! // Operation with retry logic
//! let result = safe_ops.with_retry(|| {
//!     // Your potentially failing operation
//!     Ok("retry_success")
//! }, 3).await?;
//! # Ok(())
//! # }
//! ```

use crate::utils::error::{HiveError, HiveResult};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Configuration for error recovery mechanisms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// Maximum number of retry attempts for transient failures
    pub max_retries: u32,
    /// Base delay for exponential backoff (in milliseconds)
    pub base_delay_ms: u64,
    /// Maximum delay for exponential backoff (in milliseconds)
    pub max_delay_ms: u64,
    /// Whether to enable circuit breaker pattern
    pub enable_circuit_breaker: bool,
    /// Circuit breaker threshold (number of failures before opening)
    pub circuit_breaker_threshold: u32,
    /// Circuit breaker timeout (in milliseconds)
    pub circuit_breaker_timeout_ms: u64,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
            enable_circuit_breaker: true,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_ms: 30000,
        }
    }
}

/// Circuit breaker state for preventing cascading failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    /// Circuit is closed, allowing operations to proceed
    Closed,
    /// Circuit is open, blocking operations to prevent cascading failures
    Open,
    /// Circuit is half-open, allowing limited operations to test recovery
    HalfOpen,
}

/// Circuit breaker for preventing cascading failures
#[derive(Debug)]
pub struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: u32,
    last_failure_time: Option<chrono::DateTime<chrono::Utc>>,
    config: RecoveryConfig,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration
    pub fn new(config: RecoveryConfig) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            last_failure_time: None,
            config,
        }
    }

    /// Record a successful operation
    pub fn record_success(&mut self) {
        match self.state {
            CircuitBreakerState::Open => {
                // Check if we should transition to half-open
                if let Some(last_failure) = self.last_failure_time {
                    let now = chrono::Utc::now();
                    let duration_since_failure = now.signed_duration_since(last_failure);
                    if duration_since_failure.num_milliseconds()
                        >= self.config.circuit_breaker_timeout_ms as i64
                    {
                        self.state = CircuitBreakerState::HalfOpen;
                        self.failure_count = 0;
                    }
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Successful operation in half-open state, close the circuit
                self.state = CircuitBreakerState::Closed;
                self.failure_count = 0;
            }
            CircuitBreakerState::Closed => {
                // Reset failure count on success in closed state
                self.failure_count = 0;
            }
        }
    }

    /// Record a failed operation
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(chrono::Utc::now());

        if self.failure_count >= self.config.circuit_breaker_threshold {
            self.state = CircuitBreakerState::Open;
        }
    }

    /// Check if operations are allowed to proceed
    pub fn allow_operation(&self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::HalfOpen => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    let now = chrono::Utc::now();
                    let duration_since_failure = now.signed_duration_since(last_failure);
                    duration_since_failure.num_milliseconds()
                        >= self.config.circuit_breaker_timeout_ms as i64
                } else {
                    false
                }
            }
        }
    }

    /// Get the current circuit breaker state
    pub fn get_state(&self) -> &CircuitBreakerState {
        &self.state
    }
}

/// Safe operations wrapper with error handling and recovery
#[derive(Clone)]
pub struct SafeOperations {
    config: RecoveryConfig,
    circuit_breaker: Arc<RwLock<CircuitBreaker>>,
}

impl SafeOperations {
    /// Create a new safe operations wrapper
    pub fn new(config: RecoveryConfig) -> Self {
        let circuit_breaker = Arc::new(RwLock::new(CircuitBreaker::new(config.clone())));
        Self {
            config,
            circuit_breaker,
        }
    }

    /// Execute an operation safely with error handling
    pub async fn execute_safely<F, T>(&self, operation: F) -> HiveResult<T>
    where
        F: std::future::Future<Output = HiveResult<T>> + Send + 'static,
    {
        // Check circuit breaker state
        {
            let breaker = self.circuit_breaker.read().await;
            if !breaker.allow_operation() {
                return Err(HiveError::CircuitBreakerOpen {
                    reason:
                        "Circuit breaker is open, blocking operation to prevent cascading failures"
                            .to_string(),
                });
            }
        }

        // Execute the operation
        let result = operation.await;

        // Update circuit breaker state based on result
        {
            let mut breaker = self.circuit_breaker.write().await;
            match &result {
                Ok(_) => breaker.record_success(),
                Err(_) => breaker.record_failure(),
            }
        }

        result
    }

    /// Execute an operation with retry logic
    pub async fn with_retry<F, Fut, T>(&self, operation: F, max_retries: u32) -> HiveResult<T>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = HiveResult<T>> + Send + 'static,
        T: Clone + Send + Sync + 'static,
    {
        let mut last_error = None;
        let max_retries = max_retries.min(self.config.max_retries);

        for attempt in 0..=max_retries {
            // Check circuit breaker state
            {
                let breaker = self.circuit_breaker.read().await;
                if !breaker.allow_operation() {
                    return Err(HiveError::CircuitBreakerOpen {
                        reason: "Circuit breaker is open, blocking retry attempts".to_string(),
                    });
                }
            }

            // Execute the operation
            let result = operation().await;

            // Update circuit breaker state based on result
            {
                let mut breaker = self.circuit_breaker.write().await;
                match &result {
                    Ok(_) => {
                        breaker.record_success();
                        return result;
                    }
                    Err(e) => {
                        breaker.record_failure();
                        last_error = Some(e.clone());

                        // Don't retry on certain error types
                        match e {
                            HiveError::ValidationError { .. }
                            | HiveError::AuthenticationError { .. }
                            | HiveError::AuthorizationError { .. }
                            | HiveError::SecurityError { .. } => {
                                break; // Don't retry these errors
                            }
                            _ => {
                                // Continue to retry for transient errors
                            }
                        }
                    }
                }
            }

            // If this is not the last attempt, wait before retrying
            if attempt < max_retries {
                let delay_ms = self.calculate_backoff_delay(attempt);
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;

                if let Some(ref err) = last_error {
                    tracing::warn!(
                        "Operation failed (attempt {}/{}), retrying in {}ms: {}",
                        attempt + 1,
                        max_retries + 1,
                        delay_ms,
                        err
                    );
                }
            }
        }

        Err(last_error.unwrap_or(HiveError::OperationFailed {
            reason: "Operation failed after all retry attempts".to_string(),
        }))
    }

    /// Calculate exponential backoff delay
    fn calculate_backoff_delay(&self, attempt: u32) -> u64 {
        let delay = self.config.base_delay_ms * 2u64.pow(attempt);
        delay.min(self.config.max_delay_ms)
    }

    /// Get circuit breaker status
    pub async fn get_circuit_breaker_status(&self) -> CircuitBreakerState {
        let breaker = self.circuit_breaker.read().await;
        breaker.get_state().clone()
    }

    /// Reset circuit breaker state
    pub async fn reset_circuit_breaker(&self) {
        let mut breaker = self.circuit_breaker.write().await;
        breaker.state = CircuitBreakerState::Closed;
        breaker.failure_count = 0;
        breaker.last_failure_time = None;
    }
}

/// Panic-safe option handling utilities
pub mod safe_option {
    use super::*;

    /// Safely extract value from Option with proper error handling
    pub fn expect_or_error<T>(option: Option<T>, error_message: &str) -> HiveResult<T> {
        option.ok_or_else(|| HiveError::OperationFailed {
            reason: error_message.to_string(),
        })
    }

    /// Safely extract value from Option with custom error creation
    pub fn expect_or_else<T, F>(option: Option<T>, error_fn: F) -> HiveResult<T>
    where
        F: FnOnce() -> HiveError,
    {
        option.ok_or_else(error_fn)
    }

    /// Safely extract value from Option with default value
    pub fn unwrap_or_default<T>(option: Option<T>) -> T
    where
        T: Default,
    {
        option.unwrap_or_default()
    }

    /// Safely extract value from Option with provided default
    pub fn unwrap_or<T>(option: Option<T>, default: T) -> T {
        option.unwrap_or(default)
    }
}

/// Panic-safe result handling utilities
pub mod safe_result {
    use super::*;

    /// Safely extract value from Result with proper error handling
    pub fn expect_or_error<T>(result: HiveResult<T>, error_message: &str) -> HiveResult<T> {
        result.map_err(|_| HiveError::OperationFailed {
            reason: error_message.to_string(),
        })
    }

    /// Safely extract value from Result with error mapping
    pub fn map_error<T, F>(result: HiveResult<T>, error_mapper: F) -> HiveResult<T>
    where
        F: FnOnce(HiveError) -> HiveError,
    {
        result.map_err(error_mapper)
    }

    /// Safely extract value from Result with additional context
    pub fn with_context<T>(result: HiveResult<T>, context: &str) -> HiveResult<T> {
        result.map_err(|e| HiveError::OperationFailed {
            reason: format!("{}: {}", context, e),
        })
    }
}

/// JSON value handling utilities
pub mod safe_json {
    use super::*;
    use serde_json::Value;

    /// Safely extract string from JSON value
    pub fn get_string(value: &Value, key: &str) -> HiveResult<String> {
        value
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| HiveError::ValidationError {
                field: key.to_string(),
                reason: format!("Missing or invalid string field: {}", key),
            })
    }

    /// Safely extract number from JSON value
    pub fn get_number(value: &Value, key: &str) -> HiveResult<f64> {
        value
            .get(key)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| HiveError::ValidationError {
                field: key.to_string(),
                reason: format!("Missing or invalid number field: {}", key),
            })
    }

    /// Safely extract boolean from JSON value
    pub fn get_boolean(value: &Value, key: &str) -> HiveResult<bool> {
        value
            .get(key)
            .and_then(|v| v.as_bool())
            .ok_or_else(|| HiveError::ValidationError {
                field: key.to_string(),
                reason: format!("Missing or invalid boolean field: {}", key),
            })
    }

    /// Safely extract array from JSON value
    pub fn get_array<'a>(value: &'a Value, key: &str) -> HiveResult<Vec<&'a Value>> {
        value
            .get(key)
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().collect())
            .ok_or_else(|| HiveError::ValidationError {
                field: key.to_string(),
                reason: format!("Missing or invalid array field: {}", key),
            })
    }

    /// Safely extract object from JSON value
    pub fn get_object<'a>(
        value: &'a Value,
        key: &str,
    ) -> HiveResult<&'a serde_json::Map<String, Value>> {
        value
            .get(key)
            .and_then(|v| v.as_object())
            .ok_or_else(|| HiveError::ValidationError {
                field: key.to_string(),
                reason: format!("Missing or invalid object field: {}", key),
            })
    }

    /// Safely extract optional string from JSON value
    pub fn get_optional_string(value: &Value, key: &str) -> Option<String> {
        value
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Safely extract optional number from JSON value
    pub fn get_optional_number(value: &Value, key: &str) -> Option<f64> {
        value.get(key).and_then(|v| v.as_f64())
    }

    /// Safely extract optional boolean from JSON value
    pub fn get_optional_boolean(value: &Value, key: &str) -> Option<bool> {
        value.get(key).and_then(|v| v.as_bool())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_safe_operations_success() {
        let safe_ops = SafeOperations::new(RecoveryConfig::default());

        let result = safe_ops.execute_safely(async { Ok("success") }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap_or_else(|e| panic!("Safe operation should succeed: {e}")), "success");
    }

    #[tokio::test]
    async fn test_safe_operations_failure() {
        let safe_ops = SafeOperations::new(RecoveryConfig::default());

        let result = safe_ops
            .execute_safely::<_, ()>(async {
                Err(HiveError::OperationFailed {
                    reason: "Test failure".to_string(),
                })
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_retry_logic_success() {
        let safe_ops = SafeOperations::new(RecoveryConfig::default());
        let attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));

        let attempt_count_clone = attempt_count.clone();
        let result = safe_ops
            .with_retry(
                || {
                    let count =
                        attempt_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    async move {
                        if count < 2 {
                            Err(HiveError::OperationFailed {
                                reason: "Transient failure".to_string(),
                            })
                        } else {
                            Ok("success_after_retry")
                        }
                    }
                },
                3,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap_or_else(|e| panic!("Retry operation should succeed: {e}")), "success_after_retry");
        assert_eq!(attempt_count.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let config = RecoveryConfig {
            circuit_breaker_threshold: 2,
            circuit_breaker_timeout_ms: 100,
            ..Default::default()
        };
        let safe_ops = SafeOperations::new(config);

        // Fail enough times to open circuit breaker
        for _ in 0..3 {
            let _ = safe_ops
                .execute_safely::<_, ()>(async {
                    Err(HiveError::OperationFailed {
                        reason: "Failure".to_string(),
                    })
                })
                .await;
        }

        // Next operation should be blocked by circuit breaker
        let result = safe_ops
            .execute_safely(async { Ok("should_be_blocked") })
            .await;

        assert!(matches!(result, Err(HiveError::CircuitBreakerOpen { .. })));
    }

    #[test]
    fn test_safe_option_utilities() {
        // Test expect_or_error
        let some_value: Option<i32> = Some(42);
        assert_eq!(
            safe_option::expect_or_error(some_value, "error").unwrap_or_else(|e| panic!("expect_or_error should succeed with Some value: {e}")),
            42
        );

        let none_value: Option<i32> = None;
        assert!(safe_option::expect_or_error(none_value, "error").is_err());

        // Test unwrap_or_default
        assert_eq!(safe_option::unwrap_or_default::<i32>(None), 0);
        assert_eq!(safe_option::unwrap_or_default(Some(42)), 42);
    }

    #[test]
    fn test_safe_json_utilities() -> Result<(), Box<dyn std::error::Error>> {
        let json_value = serde_json::json!({
            "string_field": "test",
            "number_field": 42.0,
            "boolean_field": true,
            "array_field": [1, 2, 3],
            "object_field": {"nested": "value"}
        });

        // Test successful extractions
        let string_val = safe_json::get_string(&json_value, "string_field")?;
        assert_eq!(string_val, "test");

        let number_val = safe_json::get_number(&json_value, "number_field")?;
        assert_eq!(number_val, 42.0);

        let boolean_val = safe_json::get_boolean(&json_value, "boolean_field")?;
        assert_eq!(boolean_val, true);

        let array_val = safe_json::get_array(&json_value, "array_field")?;
        assert_eq!(array_val.len(), 3);

        let object_val = safe_json::get_object(&json_value, "object_field")?;
        assert_eq!(object_val.len(), 1);

        // Test error cases
        assert!(safe_json::get_string(&json_value, "missing_field").is_err());
        assert!(safe_json::get_number(&json_value, "string_field").is_err());

        Ok(())
    }
}
