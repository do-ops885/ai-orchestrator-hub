use crate::utils::error::HiveError;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    failure_threshold: u64,
    recovery_timeout: Duration,
    failure_count: AtomicU64,
    last_failure_time: RwLock<Option<Instant>>,
    state: RwLock<CircuitState>,
    is_executing: AtomicBool,
}

impl CircuitBreaker {
    #[must_use]
    pub fn new(failure_threshold: u64, recovery_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            recovery_timeout,
            failure_count: AtomicU64::new(0),
            last_failure_time: RwLock::new(None),
            state: RwLock::new(CircuitState::Closed),
            is_executing: AtomicBool::new(false),
        }
    }

    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, HiveError>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::fmt::Display,
    {
        // Check circuit state
        match *self.state.read().await {
            CircuitState::Open => {
                if self.should_attempt_reset().await {
                    self.transition_to_half_open().await;
                } else {
                    return Err(HiveError::CircuitBreakerOpen {
                        reason: "Circuit breaker is open, operation rejected".to_string(),
                    });
                }
            }
            CircuitState::HalfOpen => {
                if self.is_executing.load(Ordering::Acquire) {
                    return Err(HiveError::CircuitBreakerOpen {
                        reason: "Circuit breaker is half-open and already executing".to_string(),
                    });
                }
            }
            CircuitState::Closed => {}
        }

        self.is_executing.store(true, Ordering::Release);
        let result = operation();
        self.is_executing.store(false, Ordering::Release);

        match result {
            Ok(value) => {
                self.on_success().await;
                Ok(value)
            }
            Err(error) => {
                self.on_failure().await;
                Err(HiveError::OperationFailed {
                    reason: error.to_string(),
                })
            }
        }
    }

    async fn should_attempt_reset(&self) -> bool {
        if let Some(last_failure) = *self.last_failure_time.read().await {
            last_failure.elapsed() >= self.recovery_timeout
        } else {
            false
        }
    }

    async fn transition_to_half_open(&self) {
        *self.state.write().await = CircuitState::HalfOpen;
    }

    async fn on_success(&self) {
        self.failure_count.store(0, Ordering::Release);
        *self.state.write().await = CircuitState::Closed;
        *self.last_failure_time.write().await = None;
    }

    async fn on_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::AcqRel) + 1;
        *self.last_failure_time.write().await = Some(Instant::now());

        if failures >= self.failure_threshold {
            *self.state.write().await = CircuitState::Open;
        }
    }

    pub async fn get_state(&self) -> CircuitState {
        self.state.read().await.clone()
    }

    pub fn get_failure_count(&self) -> u64 {
        self.failure_count.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() -> Result<(), Box<dyn std::error::Error>> {
        let cb = CircuitBreaker::new(3, Duration::from_millis(100));

        let result = cb.execute(|| Ok::<i32, &str>(42)).await;
        assert!(result.is_ok());
        let value = result.map_err(|e| format!("Expected Ok, got Err: {:?}", e))?;
        assert_eq!(value, 42);
        Ok(())
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(100));

        // First failure
        let _ = cb.execute(|| Err::<i32, &str>("error")).await;
        assert_eq!(cb.get_failure_count(), 1);

        // Second failure - should open circuit
        let _ = cb.execute(|| Err::<i32, &str>("error")).await;
        assert_eq!(cb.get_failure_count(), 2);

        // Circuit should now be open
        let result = cb.execute(|| Ok::<i32, &str>(42)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let cb = CircuitBreaker::new(1, Duration::from_millis(50));

        // Cause failure to open circuit
        let _ = cb.execute(|| Err::<i32, &str>("error")).await;

        // Wait for recovery timeout (add some buffer)
        sleep(Duration::from_millis(100)).await;

        // Should transition to half-open and allow execution
        let result = cb.execute(|| Ok::<i32, &str>(42)).await;
        assert!(result.is_ok());

        // Should be closed again after success
        assert!(matches!(cb.get_state().await, CircuitState::Closed));
    }
}
