//! Enhanced Error Recovery and Circuit Breaker Implementation
//!
//! This module provides comprehensive error recovery mechanisms and utilities
//! to replace unwrap() calls with proper error handling throughout the system.
//!
//! This module provides comprehensive error recovery mechanisms including
//! circuit breakers, retry logic, and graceful degradation strategies.

use crate::utils::error::{HiveError, HiveResult};
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, testing if service has recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open the circuit
    pub failure_threshold: u32,
    /// Success threshold to close the circuit from half-open
    pub success_threshold: u32,
    /// Timeout before transitioning from open to half-open
    pub timeout: Duration,
    /// Window size for tracking failures
    pub window_size: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            window_size: Duration::from_secs(300),
        }
    }
}

/// Circuit breaker implementation
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitBreakerState>>,
}

#[derive(Debug)]
struct CircuitBreakerState {
    current_state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    last_state_change: Instant,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitBreakerState {
                current_state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure_time: None,
                last_state_change: Instant::now(),
            })),
        }
    }

    /// Execute a function with circuit breaker protection
    pub async fn execute<F, T, E>(&self, operation: F) -> HiveResult<T>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        // Check if circuit is open
        if self.is_open().await {
            return Err(HiveError::CircuitBreakerOpen {
                reason: "Circuit breaker is open".to_string(),
            });
        }

        // Execute the operation
        match operation.await {
            Ok(result) => {
                self.record_success().await;
                Ok(result)
            }
            Err(e) => {
                self.record_failure().await;
                Err(HiveError::OperationFailed {
                    reason: format!("Operation failed: {}", e),
                })
            }
        }
    }

    /// Check if the circuit is open
    async fn is_open(&self) -> bool {
        let state = self.state.read().await;
        match state.current_state {
            CircuitState::Open => {
                // Check if timeout has elapsed to transition to half-open
                if state.last_state_change.elapsed() >= self.config.timeout {
                    drop(state);
                    self.transition_to_half_open().await;
                    false
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    /// Record a successful operation
    async fn record_success(&self) {
        let mut state = self.state.write().await;
        match state.current_state {
            CircuitState::HalfOpen => {
                state.success_count += 1;
                if state.success_count >= self.config.success_threshold {
                    state.current_state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                    state.last_state_change = Instant::now();
                    info!("Circuit breaker transitioned to CLOSED");
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success
                state.failure_count = 0;
            }
            _ => {}
        }
    }

    /// Record a failed operation
    async fn record_failure(&self) {
        let mut state = self.state.write().await;
        state.failure_count += 1;
        state.last_failure_time = Some(Instant::now());

        match state.current_state {
            CircuitState::Closed => {
                if state.failure_count >= self.config.failure_threshold {
                    state.current_state = CircuitState::Open;
                    state.last_state_change = Instant::now();
                    warn!("Circuit breaker transitioned to OPEN");
                }
            }
            CircuitState::HalfOpen => {
                state.current_state = CircuitState::Open;
                state.last_state_change = Instant::now();
                warn!("Circuit breaker transitioned back to OPEN");
            }
            _ => {}
        }
    }

    /// Transition to half-open state
    async fn transition_to_half_open(&self) {
        let mut state = self.state.write().await;
        state.current_state = CircuitState::HalfOpen;
        state.success_count = 0;
        state.last_state_change = Instant::now();
        info!("Circuit breaker transitioned to HALF_OPEN");
    }

    /// Get current circuit state
    pub async fn get_state(&self) -> CircuitState {
        self.state.read().await.current_state.clone()
    }
}

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// Retry mechanism with exponential backoff
pub struct RetryMechanism {
    config: RetryConfig,
}

impl RetryMechanism {
    /// Create a new retry mechanism
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute an operation with retry logic
    pub async fn execute<F, T, E>(&self, mut operation: F) -> HiveResult<T>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Display,
    {
        let mut attempt = 0;
        let mut delay = self.config.base_delay;

        loop {
            attempt += 1;

            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt >= self.config.max_attempts {
                        error!("Operation failed after {} attempts: {}", attempt, e);
                        return Err(HiveError::OperationFailed {
                            reason: format!("Operation failed after {} attempts: {}", attempt, e),
                        });
                    }

                    warn!(
                        "Operation failed on attempt {}: {}. Retrying in {:?}",
                        attempt, e, delay
                    );
                    tokio::time::sleep(delay).await;

                    // Calculate next delay with exponential backoff
                    delay = std::cmp::min(
                        Duration::from_millis(
                            (delay.as_millis() as f64 * self.config.backoff_multiplier) as u64,
                        ),
                        self.config.max_delay,
                    );
                }
            }
        }
    }
}

/// Error recovery coordinator that combines circuit breaker and retry mechanisms
pub struct ErrorRecoveryCoordinator {
    circuit_breaker: CircuitBreaker,
    retry_mechanism: RetryMechanism,
}

impl ErrorRecoveryCoordinator {
    /// Create a new error recovery coordinator
    pub fn new(circuit_config: CircuitBreakerConfig, retry_config: RetryConfig) -> Self {
        Self {
            circuit_breaker: CircuitBreaker::new(circuit_config),
            retry_mechanism: RetryMechanism::new(retry_config),
        }
    }

    /// Execute an operation with both circuit breaker and retry protection
    pub async fn execute_with_recovery<F, T, E>(&self, operation: F) -> HiveResult<T>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>
            + Clone,
        E: std::fmt::Display + Send + Sync + 'static,
    {
        self.circuit_breaker
            .execute(self.retry_mechanism.execute(operation))
            .await
    }

    /// Get circuit breaker state
    pub async fn get_circuit_state(&self) -> CircuitState {
        self.circuit_breaker.get_state().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_circuit_breaker_open_close() -> Result<(), Box<dyn std::error::Error>> {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_millis(100),
            window_size: Duration::from_secs(10),
        };
        let circuit_breaker = CircuitBreaker::new(config);

        // Initially closed
        assert_eq!(circuit_breaker.get_state().await, CircuitState::Closed);

        // Simulate failures to open circuit
        let _ = circuit_breaker
            .execute(async { Err::<(), &str>("failure") })
            .await;
        let _ = circuit_breaker
            .execute(async { Err::<(), &str>("failure") })
            .await;

        // Should be open now
        assert_eq!(circuit_breaker.get_state().await, CircuitState::Open);

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should transition to half-open and then closed on success
        let _ = circuit_breaker.execute(async { Ok::<(), &str>(()) }).await;
        let _ = circuit_breaker.execute(async { Ok::<(), &str>(()) }).await;

        assert_eq!(circuit_breaker.get_state().await, CircuitState::Closed);
        Ok(())
    }

    #[tokio::test]
    async fn test_retry_mechanism() -> Result<(), Box<dyn std::error::Error>> {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
        };
        let retry = RetryMechanism::new(config);

        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = Arc::clone(&attempt_count);

        let result = retry
            .execute(move || {
                let count = attempt_count_clone.fetch_add(1, Ordering::SeqCst);
                Box::pin(async move {
                    if count < 2 {
                        Err("failure")
                    } else {
                        Ok("success")
                    }
                })
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
        Ok(())
    }
}

/// Safe unwrap alternatives to replace unwrap() calls
pub trait SafeUnwrap<T> {
    /// Safe unwrap with custom error message
    fn safe_unwrap(self, operation: &str, component: &str) -> HiveResult<T>;

    /// Safe unwrap with default value
    fn unwrap_or_default_with_log(self, operation: &str, component: &str) -> T
    where
        T: Default;

    /// Safe unwrap with custom default
    fn unwrap_or_with_log(self, default: T, operation: &str, component: &str) -> T;
}

impl<T> SafeUnwrap<T> for Option<T> {
    fn safe_unwrap(self, operation: &str, component: &str) -> HiveResult<T> {
        self.ok_or_else(|| {
            error!("Option unwrap failed in {} during {}", component, operation);
            HiveError::OperationFailed {
                reason: format!("Expected Some value in {} during {}", component, operation),
            }
        })
    }

    fn unwrap_or_default_with_log(self, operation: &str, component: &str) -> T
    where
        T: Default,
    {
        match self {
            Some(value) => value,
            None => {
                warn!("Using default value in {} during {}", component, operation);
                T::default()
            }
        }
    }

    fn unwrap_or_with_log(self, default: T, operation: &str, component: &str) -> T {
        match self {
            Some(value) => value,
            None => {
                warn!("Using fallback value in {} during {}", component, operation);
                default
            }
        }
    }
}

impl<T, E> SafeUnwrap<T> for Result<T, E>
where
    E: fmt::Display,
{
    fn safe_unwrap(self, operation: &str, component: &str) -> HiveResult<T> {
        self.map_err(|e| {
            error!(
                "Result unwrap failed in {} during {}: {}",
                component, operation, e
            );
            HiveError::OperationFailed {
                reason: format!(
                    "Operation failed in {} during {}: {}",
                    component, operation, e
                ),
            }
        })
    }

    fn unwrap_or_default_with_log(self, operation: &str, component: &str) -> T
    where
        T: Default,
    {
        match self {
            Ok(value) => value,
            Err(error) => {
                warn!(
                    "Using default value in {} during {} due to error: {}",
                    component, operation, error
                );
                T::default()
            }
        }
    }

    fn unwrap_or_with_log(self, default: T, operation: &str, component: &str) -> T {
        match self {
            Ok(value) => value,
            Err(error) => {
                warn!(
                    "Using fallback value in {} during {} due to error: {}",
                    component, operation, error
                );
                default
            }
        }
    }
}

/// JSON value safe access utilities
pub trait JsonSafeAccess {
    fn safe_get(&self, key: &str) -> HiveResult<&serde_json::Value>;
    fn safe_get_str(&self, key: &str) -> HiveResult<&str>;
    fn safe_get_u64(&self, key: &str) -> HiveResult<u64>;
    fn safe_get_f64(&self, key: &str) -> HiveResult<f64>;
    fn safe_get_bool(&self, key: &str) -> HiveResult<bool>;
    fn safe_get_array(&self) -> HiveResult<&Vec<serde_json::Value>>;
    fn safe_get_object(&self) -> HiveResult<&serde_json::Map<String, serde_json::Value>>;
}

impl JsonSafeAccess for serde_json::Value {
    fn safe_get(&self, key: &str) -> HiveResult<&serde_json::Value> {
        self.get(key).ok_or_else(|| HiveError::ValidationError {
            field: key.to_string(),
            reason: "Key not found in JSON object".to_string(),
        })
    }

    fn safe_get_str(&self, key: &str) -> HiveResult<&str> {
        self.safe_get(key)?
            .as_str()
            .ok_or_else(|| HiveError::ValidationError {
                field: key.to_string(),
                reason: "Value is not a string".to_string(),
            })
    }

    fn safe_get_u64(&self, key: &str) -> HiveResult<u64> {
        self.safe_get(key)?
            .as_u64()
            .ok_or_else(|| HiveError::ValidationError {
                field: key.to_string(),
                reason: "Value is not a valid u64".to_string(),
            })
    }

    fn safe_get_f64(&self, key: &str) -> HiveResult<f64> {
        self.safe_get(key)?
            .as_f64()
            .ok_or_else(|| HiveError::ValidationError {
                field: key.to_string(),
                reason: "Value is not a valid f64".to_string(),
            })
    }

    fn safe_get_bool(&self, key: &str) -> HiveResult<bool> {
        self.safe_get(key)?
            .as_bool()
            .ok_or_else(|| HiveError::ValidationError {
                field: key.to_string(),
                reason: "Value is not a boolean".to_string(),
            })
    }

    fn safe_get_array(&self) -> HiveResult<&Vec<serde_json::Value>> {
        self.as_array().ok_or_else(|| HiveError::ValidationError {
            field: "root".to_string(),
            reason: "Value is not an array".to_string(),
        })
    }

    fn safe_get_object(&self) -> HiveResult<&serde_json::Map<String, serde_json::Value>> {
        self.as_object().ok_or_else(|| HiveError::ValidationError {
            field: "root".to_string(),
            reason: "Value is not an object".to_string(),
        })
    }
}

/// Macro for safe unwrap with context
#[macro_export]
macro_rules! safe_unwrap {
    ($expr:expr, $op:expr, $component:expr) => {{
        use $crate::utils::error_recovery::SafeUnwrap;
        $expr.safe_unwrap($op, $component)
    }};
}

/// Macro for safe unwrap with default and logging
#[macro_export]
macro_rules! safe_unwrap_or_default {
    ($expr:expr, $op:expr, $component:expr) => {{
        use $crate::utils::error_recovery::SafeUnwrap;
        $expr.unwrap_or_default_with_log($op, $component)
    }};
}

/// Macro for safe unwrap with custom default and logging
#[macro_export]
macro_rules! safe_unwrap_or {
    ($expr:expr, $default:expr, $op:expr, $component:expr) => {{
        use $crate::utils::error_recovery::SafeUnwrap;
        $expr.unwrap_or_with_log($default, $op, $component)
    }};
}
