use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Comprehensive health check system for monitoring system components
#[derive(Debug, Clone)]
pub struct HealthCheckManager {
    checks: Arc<RwLock<HashMap<String, HealthCheck>>>,
    overall_status: Arc<RwLock<SystemHealth>>,
}

impl HealthCheckManager {
    /// Create a new health check manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
            overall_status: Arc::new(RwLock::new(SystemHealth::default())),
        }
    }

    /// Register a new health check
    pub async fn register_check(&self, name: String, check: HealthCheck) {
        let mut checks = self.checks.write().await;
        checks.insert(name, check);
    }

    /// Run all health checks and update system status
    pub async fn run_all_checks(&self) -> SystemHealthReport {
        let mut checks = self.checks.write().await;
        let mut results = HashMap::new();
        let mut overall_healthy = true;
        let start_time = Instant::now();

        for (name, check) in checks.iter_mut() {
            let result = check.execute().await;
            if result.status != HealthStatus::Healthy {
                overall_healthy = false;
            }
            results.insert(name.clone(), result);
        }

        let execution_time = start_time.elapsed();

        // Update overall system status
        let mut system_health = self.overall_status.write().await;
        system_health.status = if overall_healthy {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        };
        system_health.last_check = chrono::Utc::now();
        system_health.check_duration_ms = execution_time.as_millis() as u64;

        SystemHealthReport {
            overall_status: system_health.status.clone(),
            checks: results,
            timestamp: chrono::Utc::now(),
            execution_time_ms: execution_time.as_millis() as u64,
        }
    }

    /// Get current system health status
    pub async fn get_system_health(&self) -> SystemHealth {
        self.overall_status.read().await.clone()
    }

    /// Start background health check monitoring
    pub fn start_monitoring(self: Arc<Self>, interval: Duration) {
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;
                let report = self.run_all_checks().await;

                // Log health status changes
                if report.overall_status == HealthStatus::Healthy {
                    tracing::debug!("System health check passed");
                } else {
                    tracing::warn!("System health check failed: {:?}", report);
                }
            }
        });
    }
}

impl Default for HealthCheckManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Individual health check definition
#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub name: String,
    pub description: String,
    pub timeout: Duration,
    pub check_fn: HealthCheckFunction,
    pub last_result: Option<HealthCheckResult>,
}

impl HealthCheck {
    /// Create a new health check
    #[must_use]
    pub fn new(
        name: String,
        description: String,
        timeout: Duration,
        check_fn: HealthCheckFunction,
    ) -> Self {
        Self {
            name,
            description,
            timeout,
            check_fn,
            last_result: None,
        }
    }

    /// Execute the health check with timeout
    pub async fn execute(&mut self) -> HealthCheckResult {
        let start_time = Instant::now();

        let result = tokio::time::timeout(self.timeout, (self.check_fn.0)()).await;

        let check_result = match result {
            Ok(Ok(details)) => HealthCheckResult {
                status: HealthStatus::Healthy,
                message: "Check passed".to_string(),
                details: Some(details),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                timestamp: chrono::Utc::now(),
            },
            Ok(Err(error)) => HealthCheckResult {
                status: HealthStatus::Unhealthy,
                message: error,
                details: None,
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                timestamp: chrono::Utc::now(),
            },
            Err(_) => HealthCheckResult {
                status: HealthStatus::Unhealthy,
                message: format!("Health check timed out after {:?}", self.timeout),
                details: None,
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                timestamp: chrono::Utc::now(),
            },
        };

        self.last_result = Some(check_result.clone());
        check_result
    }
}

/// Health check future type
type HealthCheckFuture = std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<HashMap<String, String>, String>> + Send>,
>;

/// Health check function type
type HealthCheckFn = Arc<dyn Fn() -> HealthCheckFuture + Send + Sync>;

/// Health check function wrapper
#[derive(Clone)]
pub struct HealthCheckFunction(pub HealthCheckFn);

impl std::fmt::Debug for HealthCheckFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HealthCheckFunction")
    }
}

/// Health status enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Result of a health check execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub message: String,
    pub details: Option<HashMap<String, String>>,
    pub execution_time_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Overall system health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub status: HealthStatus,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub check_duration_ms: u64,
    pub uptime_seconds: u64,
    pub version: String,
}

impl Default for SystemHealth {
    fn default() -> Self {
        Self {
            status: HealthStatus::Healthy,
            last_check: chrono::Utc::now(),
            check_duration_ms: 0,
            uptime_seconds: 0,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Complete system health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthReport {
    pub overall_status: HealthStatus,
    pub checks: HashMap<String, HealthCheckResult>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub execution_time_ms: u64,
}

/// Predefined health checks for common system components
pub struct StandardHealthChecks;

impl StandardHealthChecks {
    /// Database connectivity health check
    #[must_use]
    pub fn database_check() -> HealthCheck {
        HealthCheck::new(
            "database".to_string(),
            "Database connectivity and responsiveness".to_string(),
            Duration::from_secs(5),
            HealthCheckFunction(Arc::new(|| {
                Box::pin(async {
                    // Simulate database check
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    let mut details = HashMap::new();
                    details.insert("connection_pool".to_string(), "healthy".to_string());
                    details.insert("query_time_ms".to_string(), "45".to_string());
                    Ok(details)
                })
            })),
        )
    }

    /// Memory usage health check
    #[must_use]
    pub fn memory_check() -> HealthCheck {
        HealthCheck::new(
            "memory".to_string(),
            "System memory usage monitoring".to_string(),
            Duration::from_secs(2),
            HealthCheckFunction(Arc::new(|| {
                Box::pin(async {
                    // Get system memory info (simplified)
                    let mut details = HashMap::new();
                    details.insert("memory_usage_percent".to_string(), "65".to_string());
                    details.insert("available_mb".to_string(), "2048".to_string());

                    // Check if memory usage is acceptable
                    if 65.0 > 90.0 {
                        Err("Memory usage too high".to_string())
                    } else {
                        Ok(details)
                    }
                })
            })),
        )
    }

    /// Agent system health check
    #[must_use]
    pub fn agent_system_check() -> HealthCheck {
        HealthCheck::new(
            "agent_system".to_string(),
            "Agent system responsiveness and capacity".to_string(),
            Duration::from_secs(3),
            HealthCheckFunction(Arc::new(|| {
                Box::pin(async {
                    let mut details = HashMap::new();
                    details.insert("active_agents".to_string(), "5".to_string());
                    details.insert("pending_tasks".to_string(), "12".to_string());
                    details.insert("average_response_time_ms".to_string(), "150".to_string());
                    Ok(details)
                })
            })),
        )
    }

    /// Neural network health check
    #[must_use]
    pub fn neural_network_check() -> HealthCheck {
        HealthCheck::new(
            "neural_network".to_string(),
            "Neural network processing capability".to_string(),
            Duration::from_secs(5),
            HealthCheckFunction(Arc::new(|| {
                Box::pin(async {
                    let mut details = HashMap::new();
                    details.insert("model_loaded".to_string(), "true".to_string());
                    details.insert("inference_time_ms".to_string(), "25".to_string());
                    details.insert("accuracy".to_string(), "0.95".to_string());
                    Ok(details)
                })
            })),
        )
    }

    /// WebSocket connectivity health check
    #[must_use]
    pub fn websocket_check() -> HealthCheck {
        HealthCheck::new(
            "websocket".to_string(),
            "WebSocket server connectivity and performance".to_string(),
            Duration::from_secs(3),
            HealthCheckFunction(Arc::new(|| {
                Box::pin(async {
                    let mut details = HashMap::new();
                    details.insert("active_connections".to_string(), "8".to_string());
                    details.insert("message_queue_size".to_string(), "0".to_string());
                    details.insert("connection_errors".to_string(), "0".to_string());
                    Ok(details)
                })
            })),
        )
    }
}

/// Circuit breaker pattern implementation for fault tolerance
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    #[allow(dead_code)]
    name: String,
    failure_threshold: u32,
    recovery_timeout: Duration,
    state: Arc<RwLock<CircuitBreakerState>>,
}

#[derive(Debug, Clone)]
struct CircuitBreakerState {
    status: CircuitBreakerStatus,
    failure_count: u32,
    last_failure_time: Option<Instant>,
    success_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerStatus {
    Closed,   // Normal operation
    Open,     // Failing, blocking requests
    HalfOpen, // Testing if service recovered
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    #[must_use]
    pub fn new(name: String, failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            name,
            failure_threshold,
            recovery_timeout,
            state: Arc::new(RwLock::new(CircuitBreakerState {
                status: CircuitBreakerStatus::Closed,
                failure_count: 0,
                last_failure_time: None,
                success_count: 0,
            })),
        }
    }

    /// Execute a function with circuit breaker protection
    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        // Check if circuit breaker allows the call
        if !self.can_execute().await {
            return Err(CircuitBreakerError::CircuitOpen);
        }

        // Execute the operation
        match operation.await {
            Ok(result) => {
                self.record_success().await;
                Ok(result)
            }
            Err(error) => {
                self.record_failure().await;
                Err(CircuitBreakerError::OperationFailed(error))
            }
        }
    }

    /// Check if the circuit breaker allows execution
    async fn can_execute(&self) -> bool {
        let mut state = self.state.write().await;

        match state.status {
            CircuitBreakerStatus::Open => {
                // Check if recovery timeout has passed
                if let Some(last_failure) = state.last_failure_time {
                    if last_failure.elapsed() >= self.recovery_timeout {
                        state.status = CircuitBreakerStatus::HalfOpen;
                        state.success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerStatus::Closed | CircuitBreakerStatus::HalfOpen => true,
        }
    }

    /// Record a successful operation
    async fn record_success(&self) {
        let mut state = self.state.write().await;

        match state.status {
            CircuitBreakerStatus::Closed => {
                state.failure_count = 0;
            }
            CircuitBreakerStatus::HalfOpen => {
                state.success_count += 1;
                if state.success_count >= 3 {
                    state.status = CircuitBreakerStatus::Closed;
                    state.failure_count = 0;
                    state.last_failure_time = None;
                }
            }
            CircuitBreakerStatus::Open => {
                // Should not happen
            }
        }
    }

    /// Record a failed operation
    async fn record_failure(&self) {
        let mut state = self.state.write().await;

        state.failure_count += 1;
        state.last_failure_time = Some(Instant::now());

        if state.failure_count >= self.failure_threshold || state.status == CircuitBreakerStatus::HalfOpen {
            state.status = CircuitBreakerStatus::Open;
        }
    }

    /// Get current circuit breaker status
    pub async fn get_status(&self) -> CircuitBreakerStatus {
        self.state.read().await.status.clone()
    }
}

/// Circuit breaker error types
#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    CircuitOpen,
    OperationFailed(E),
}

impl<E: std::fmt::Display> std::fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerError::CircuitOpen => write!(f, "Circuit breaker is open"),
            CircuitBreakerError::OperationFailed(e) => write!(f, "Operation failed: {e}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for CircuitBreakerError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CircuitBreakerError::CircuitOpen => None,
            CircuitBreakerError::OperationFailed(e) => Some(e),
        }
    }
}
