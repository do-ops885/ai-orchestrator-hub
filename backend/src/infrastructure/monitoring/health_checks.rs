//! Phase 3: Enhanced Health Checks
//!
//! Implements multi-level health checks, automatic dependency monitoring,
//! and health check endpoint with diagnostics for operational excellence.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Health status levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    /// Service is healthy and fully operational
    Healthy,
    /// Service is degraded but still operational
    Degraded,
    /// Service is unhealthy and may not function properly
    Unhealthy,
    /// Service is down and not operational
    Down,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Component name
    pub component: String,
    /// Health status
    pub status: HealthStatus,
    /// Check timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Additional details
    pub details: HashMap<String, serde_json::Value>,
    /// Error message if unhealthy
    pub error_message: Option<String>,
}

/// Comprehensive health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Overall system health status
    pub overall_status: HealthStatus,
    /// Timestamp of the report
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Individual component health checks
    pub components: HashMap<String, HealthCheckResult>,
    /// System uptime in seconds
    pub uptime_seconds: u64,
    /// System version
    pub version: String,
    /// Environment information
    pub environment: HashMap<String, String>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,
    /// Health check interval in seconds
    pub check_interval_secs: u64,
    /// Timeout for individual health checks in seconds
    pub check_timeout_secs: u64,
    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,
    /// Components to check
    pub components: Vec<String>,
    /// Custom health check endpoints
    pub custom_endpoints: HashMap<String, String>,
}

/// Health checker with multi-level checks
#[derive(Clone)]
pub struct HealthChecker {
    config: Arc<RwLock<HealthCheckConfig>>,
    system_start_time: std::time::Instant,
    component_checks: Arc<RwLock<HashMap<String, Box<dyn HealthCheck + Send + Sync>>>>,
    check_history: Arc<RwLock<HashMap<String, Vec<HealthCheckResult>>>>,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(config: HealthCheckConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            system_start_time: std::time::Instant::now(),
            component_checks: Arc::new(RwLock::new(HashMap::new())),
            check_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a health check for a component
    pub async fn register_health_check(
        &self,
        component: String,
        check: Box<dyn HealthCheck + Send + Sync>,
    ) {
        self.component_checks
            .write()
            .await
            .insert(component.clone(), check);
        info!("Registered health check for component: {}", component);
    }

    /// Perform comprehensive health check
    pub async fn check_health(
        &self,
    ) -> Result<HealthReport, Box<dyn std::error::Error + Send + Sync>> {
        let config = self.config.read().await;
        let mut component_results = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;

        // Check all registered components
        let checks = self.component_checks.read().await;
        for (component_name, check) in checks.iter() {
            let start_time = std::time::Instant::now();
            let result = match tokio::time::timeout(
                std::time::Duration::from_secs(config.check_timeout_secs),
                check.check_health(),
            )
            .await
            {
                Ok(Ok(result)) => {
                    let response_time = start_time.elapsed().as_millis() as u64;
                    HealthCheckResult {
                        component: component_name.clone(),
                        status: result.status,
                        timestamp: chrono::Utc::now(),
                        response_time_ms: response_time,
                        details: result.details,
                        error_message: None,
                    }
                }
                Ok(Err(e)) => {
                    let response_time = start_time.elapsed().as_millis() as u64;
                    HealthCheckResult {
                        component: component_name.clone(),
                        status: HealthStatus::Unhealthy,
                        timestamp: chrono::Utc::now(),
                        response_time_ms: response_time,
                        details: HashMap::new(),
                        error_message: Some(e.to_string()),
                    }
                }
                Err(_) => HealthCheckResult {
                    component: component_name.clone(),
                    status: HealthStatus::Unhealthy,
                    timestamp: chrono::Utc::now(),
                    response_time_ms: config.check_timeout_secs * 1000,
                    details: HashMap::new(),
                    error_message: Some("Health check timeout".to_string()),
                },
            };

            // Update overall status based on component status
            overall_status = Self::combine_status(overall_status, result.status.clone());

            // Store result in history
            self.store_check_result(component_name.clone(), result.clone())
                .await;

            component_results.insert(component_name.clone(), result);
        }

        // Check failure threshold
        overall_status = self
            .apply_failure_threshold(&component_results, overall_status)
            .await;

        let report = HealthReport {
            overall_status,
            timestamp: chrono::Utc::now(),
            components: component_results,
            uptime_seconds: self.system_start_time.elapsed().as_secs(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: self.get_environment_info(),
        };

        debug!("Health check completed: {:?}", report.overall_status);
        Ok(report)
    }

    /// Get detailed health report with diagnostics
    pub async fn get_detailed_report(
        &self,
    ) -> Result<DetailedHealthReport, Box<dyn std::error::Error + Send + Sync>> {
        let basic_report = self.check_health().await?;
        let history = self.get_check_history().await;

        Ok(DetailedHealthReport {
            basic_report,
            check_history: history,
            recommendations: self.generate_recommendations().await,
            metrics: self.get_health_metrics().await,
        })
    }

    /// Get health check history
    pub async fn get_check_history(&self) -> HashMap<String, Vec<HealthCheckResult>> {
        self.check_history.read().await.clone()
    }

    /// Update health check configuration
    pub async fn update_config(&self, new_config: HealthCheckConfig) {
        *self.config.write().await = new_config;
        info!("Health check configuration updated");
    }

    /// Get current configuration
    pub async fn get_config(&self) -> HealthCheckConfig {
        self.config.read().await.clone()
    }

    /// Combine two health statuses (takes the worst)
    fn combine_status(status1: HealthStatus, status2: HealthStatus) -> HealthStatus {
        match (status1, status2) {
            (HealthStatus::Down, _) | (_, HealthStatus::Down) => HealthStatus::Down,
            (HealthStatus::Unhealthy, _) | (_, HealthStatus::Unhealthy) => HealthStatus::Unhealthy,
            (HealthStatus::Degraded, _) | (_, HealthStatus::Degraded) => HealthStatus::Degraded,
            (HealthStatus::Healthy, HealthStatus::Healthy) => HealthStatus::Healthy,
        }
    }

    /// Apply failure threshold logic
    async fn apply_failure_threshold(
        &self,
        component_results: &HashMap<String, HealthCheckResult>,
        current_status: HealthStatus,
    ) -> HealthStatus {
        let config = self.config.read().await;
        let history = self.check_history.read().await;

        for (component_name, current_result) in component_results {
            if current_result.status == HealthStatus::Unhealthy {
                if let Some(component_history) = history.get(component_name) {
                    let recent_failures = component_history
                        .iter()
                        .rev()
                        .take(config.failure_threshold as usize)
                        .filter(|r| r.status == HealthStatus::Unhealthy)
                        .count();

                    if recent_failures >= config.failure_threshold as usize {
                        warn!(
                            "Component {} has {} consecutive failures, exceeding threshold of {}",
                            component_name, recent_failures, config.failure_threshold
                        );
                        return HealthStatus::Down;
                    }
                }
            }
        }

        current_status
    }

    /// Store check result in history
    async fn store_check_result(&self, component: String, result: HealthCheckResult) {
        let mut history = self.check_history.write().await;
        let component_history = history.entry(component).or_insert_with(Vec::new);

        // Keep only last 100 results per component
        if component_history.len() >= 100 {
            component_history.remove(0);
        }
        component_history.push(result);
    }

    /// Generate health recommendations
    async fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        let history = self.check_history.read().await;

        for (component, results) in history.iter() {
            if results.len() < 5 {
                continue; // Need more data
            }

            let recent_results = &results[results.len().saturating_sub(10)..];
            let failure_rate = recent_results
                .iter()
                .filter(|r| r.status != HealthStatus::Healthy)
                .count() as f64
                / recent_results.len() as f64;

            if failure_rate > 0.5 {
                recommendations.push(format!(
                    "Component '{}' has high failure rate ({:.1}%). Consider investigating.",
                    component,
                    failure_rate * 100.0
                ));
            }

            let avg_response_time = recent_results
                .iter()
                .map(|r| r.response_time_ms as f64)
                .sum::<f64>()
                / recent_results.len() as f64;

            if avg_response_time > 5000.0 {
                recommendations.push(format!(
                    "Component '{}' has slow response time ({:.0}ms avg). Consider optimization.",
                    component, avg_response_time
                ));
            }
        }

        recommendations
    }

    /// Get health metrics
    async fn get_health_metrics(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        let history = self.check_history.read().await;

        for (component, results) in history.iter() {
            if results.is_empty() {
                continue;
            }

            let healthy_count = results
                .iter()
                .filter(|r| r.status == HealthStatus::Healthy)
                .count();
            let availability = healthy_count as f64 / results.len() as f64;
            metrics.insert(format!("{}_availability", component), availability);

            let avg_response_time = results.iter().map(|r| r.response_time_ms).sum::<u64>() as f64
                / results.len() as f64;
            metrics.insert(
                format!("{}_avg_response_time_ms", component),
                avg_response_time,
            );
        }

        metrics
    }

    /// Get environment information
    fn get_environment_info(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();

        env.insert(
            "rust_version".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );
        env.insert("os".to_string(), std::env::consts::OS.to_string());
        env.insert("arch".to_string(), std::env::consts::ARCH.to_string());

        if let Ok(num_cpus) = std::thread::available_parallelism() {
            env.insert("cpu_cores".to_string(), num_cpus.to_string());
        }

        if let Ok(memory) = sys_info::mem_info() {
            env.insert(
                "total_memory_mb".to_string(),
                (memory.total / 1024).to_string(),
            );
        }

        env
    }
}

/// Detailed health report with diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedHealthReport {
    /// Basic health report
    pub basic_report: HealthReport,
    /// Check history for all components
    pub check_history: HashMap<String, Vec<HealthCheckResult>>,
    /// Health recommendations
    pub recommendations: Vec<String>,
    /// Health metrics
    pub metrics: HashMap<String, f64>,
}

/// Health check trait
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform health check
    async fn check_health(
        &self,
    ) -> Result<HealthCheckResponse, Box<dyn std::error::Error + Send + Sync>>;
}

/// Health check response
#[derive(Debug, Clone)]
pub struct HealthCheckResponse {
    /// Health status
    pub status: HealthStatus,
    /// Additional details
    pub details: HashMap<String, serde_json::Value>,
}

impl Default for HealthCheckResponse {
    fn default() -> Self {
        Self {
            status: HealthStatus::Healthy,
            details: HashMap::new(),
        }
    }
}

/// Built-in health checks
pub mod checks {
    use super::*;

    /// Database health check
    pub struct DatabaseHealthCheck {
        connection_string: String,
    }

    impl DatabaseHealthCheck {
        pub fn new(connection_string: String) -> Self {
            Self { connection_string }
        }
    }

    #[async_trait::async_trait]
    impl HealthCheck for DatabaseHealthCheck {
        async fn check_health(
            &self,
        ) -> Result<HealthCheckResponse, Box<dyn std::error::Error + Send + Sync>> {
            // Simplified database check - in real implementation, test actual connection
            let mut details = HashMap::new();
            details.insert(
                "connection_string".to_string(),
                serde_json::json!(self.connection_string),
            );

            // Simulate database check
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;

            Ok(HealthCheckResponse {
                status: HealthStatus::Healthy,
                details,
            })
        }
    }

    /// External service health check
    pub struct ExternalServiceHealthCheck {
        service_name: String,
        endpoint: String,
        timeout: std::time::Duration,
    }

    impl ExternalServiceHealthCheck {
        pub fn new(service_name: String, endpoint: String, timeout_secs: u64) -> Self {
            Self {
                service_name,
                endpoint,
                timeout: std::time::Duration::from_secs(timeout_secs),
            }
        }
    }

    #[async_trait::async_trait]
    impl HealthCheck for ExternalServiceHealthCheck {
        async fn check_health(
            &self,
        ) -> Result<HealthCheckResponse, Box<dyn std::error::Error + Send + Sync>> {
            let start_time = std::time::Instant::now();

            let client = reqwest::Client::new();
            let response = client
                .get(&self.endpoint)
                .timeout(self.timeout)
                .send()
                .await;

            let response_time = start_time.elapsed();

            let mut details = HashMap::new();
            details.insert("endpoint".to_string(), serde_json::json!(self.endpoint));
            details.insert(
                "response_time_ms".to_string(),
                serde_json::json!(response_time.as_millis()),
            );

            match response {
                Ok(resp) if resp.status().is_success() => {
                    details.insert(
                        "status_code".to_string(),
                        serde_json::json!(resp.status().as_u16()),
                    );
                    Ok(HealthCheckResponse {
                        status: HealthStatus::Healthy,
                        details,
                    })
                }
                Ok(resp) => {
                    details.insert(
                        "status_code".to_string(),
                        serde_json::json!(resp.status().as_u16()),
                    );
                    Ok(HealthCheckResponse {
                        status: HealthStatus::Degraded,
                        details,
                    })
                }
                Err(e) => {
                    details.insert("error".to_string(), serde_json::json!(e.to_string()));
                    Ok(HealthCheckResponse {
                        status: HealthStatus::Unhealthy,
                        details,
                    })
                }
            }
        }
    }

    /// Memory health check
    pub struct MemoryHealthCheck {
        warning_threshold_mb: u64,
        critical_threshold_mb: u64,
    }

    impl MemoryHealthCheck {
        pub fn new(warning_threshold_mb: u64, critical_threshold_mb: u64) -> Self {
            Self {
                warning_threshold_mb,
                critical_threshold_mb,
            }
        }
    }

    #[async_trait::async_trait]
    impl HealthCheck for MemoryHealthCheck {
        async fn check_health(
            &self,
        ) -> Result<HealthCheckResponse, Box<dyn std::error::Error + Send + Sync>> {
            let mut details = HashMap::new();

            if let Ok(mem_info) = sys_info::mem_info() {
                let used_mb = (mem_info.total - mem_info.free) / 1024;
                let available_mb = mem_info.free / 1024;

                details.insert(
                    "total_mb".to_string(),
                    serde_json::json!(mem_info.total / 1024),
                );
                details.insert("used_mb".to_string(), serde_json::json!(used_mb));
                details.insert("available_mb".to_string(), serde_json::json!(available_mb));

                let status = if available_mb < self.critical_threshold_mb {
                    HealthStatus::Unhealthy
                } else if available_mb < self.warning_threshold_mb {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Healthy
                };

                Ok(HealthCheckResponse { status, details })
            } else {
                details.insert(
                    "error".to_string(),
                    serde_json::json!("Failed to get memory info"),
                );
                Ok(HealthCheckResponse {
                    status: HealthStatus::Unhealthy,
                    details,
                })
            }
        }
    }

    /// CPU health check
    pub struct CpuHealthCheck {
        warning_threshold: f64,
        critical_threshold: f64,
    }

    impl CpuHealthCheck {
        pub fn new(warning_threshold: f64, critical_threshold: f64) -> Self {
            Self {
                warning_threshold,
                critical_threshold,
            }
        }
    }

    #[async_trait::async_trait]
    impl HealthCheck for CpuHealthCheck {
        async fn check_health(
            &self,
        ) -> Result<HealthCheckResponse, Box<dyn std::error::Error + Send + Sync>> {
            let mut details = HashMap::new();

            // Simplified CPU check - in real implementation, use system monitoring
            let cpu_usage = 45.0; // Mock value
            details.insert(
                "cpu_usage_percent".to_string(),
                serde_json::json!(cpu_usage),
            );

            let status = if cpu_usage > self.critical_threshold {
                HealthStatus::Unhealthy
            } else if cpu_usage > self.warning_threshold {
                HealthStatus::Degraded
            } else {
                HealthStatus::Healthy
            };

            Ok(HealthCheckResponse { status, details })
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_secs: 30,
            check_timeout_secs: 10,
            failure_threshold: 3,
            components: vec![
                "database".to_string(),
                "memory".to_string(),
                "cpu".to_string(),
            ],
            custom_endpoints: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_checker() {
        let config = HealthCheckConfig::default();
        let checker = HealthChecker::new(config);

        // Register a simple health check
        struct TestHealthCheck;
        #[async_trait::async_trait]
        impl HealthCheck for TestHealthCheck {
            async fn check_health(
                &self,
            ) -> Result<HealthCheckResponse, Box<dyn std::error::Error + Send + Sync>> {
                Ok(HealthCheckResponse::default())
            }
        }

        checker
            .register_health_check("test".to_string(), Box::new(TestHealthCheck))
            .await;

        let report = checker.check_health().await.expect("replaced unwrap");
        assert_eq!(report.overall_status, HealthStatus::Healthy);
        assert!(report.components.contains_key("test"));
    }

    #[test]
    fn test_status_combination() {
        assert_eq!(
            HealthChecker::combine_status(HealthStatus::Healthy, HealthStatus::Healthy),
            HealthStatus::Healthy
        );
        assert_eq!(
            HealthChecker::combine_status(HealthStatus::Healthy, HealthStatus::Degraded),
            HealthStatus::Degraded
        );
        assert_eq!(
            HealthChecker::combine_status(HealthStatus::Degraded, HealthStatus::Unhealthy),
            HealthStatus::Unhealthy
        );
        assert_eq!(
            HealthChecker::combine_status(HealthStatus::Healthy, HealthStatus::Down),
            HealthStatus::Down
        );
    }
}
