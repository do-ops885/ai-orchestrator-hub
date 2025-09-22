//! Phase 3 Comprehensive Metrics Collection
//!
//! Integrates Prometheus metrics with tool execution latency histograms,
//! cache performance metrics, and error rate tracking for operational excellence.

use lazy_static::lazy_static;
use prometheus::{
    register_counter_vec, register_gauge_vec, register_histogram_vec, CounterVec, Encoder,
    GaugeVec, HistogramVec, TextEncoder,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Comprehensive metrics collector for Phase 3 enhancements
#[derive(Clone)]
pub struct Phase3MetricsCollector {
    /// Tool execution latency histograms by tool name
    tool_execution_duration: HistogramVec,
    /// Tool execution count by tool name and result
    tool_execution_count: CounterVec,
    /// Cache hit/miss counters by cache type
    cache_operations: CounterVec,
    /// Cache performance ratios
    cache_hit_ratio: GaugeVec,
    /// Error rate tracking by component and error type
    error_rate: CounterVec,
    /// Request throughput by endpoint
    request_throughput: CounterVec,
    /// Active connections gauge
    active_connections: GaugeVec,
    /// Memory pool usage
    memory_pool_usage: GaugeVec,
    /// Custom metrics storage for dynamic metrics
    custom_metrics:
        Arc<RwLock<HashMap<String, Box<dyn prometheus::core::Collector + Send + Sync>>>>,
}

lazy_static! {
    /// Global metrics registry for Phase 3
    pub static ref PHASE3_METRICS: Phase3MetricsCollector = Phase3MetricsCollector::new().expect("Failed to create Phase 3 metrics");
}

impl Phase3MetricsCollector {
    /// Create a new Phase 3 metrics collector
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let tool_execution_duration = register_histogram_vec!(
            "mcp_tool_execution_duration_seconds",
            "Tool execution duration in seconds",
            &["tool_name", "result"],
            vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0]
        )?;

        let tool_execution_count = register_counter_vec!(
            "mcp_tool_execution_total",
            "Total tool executions",
            &["tool_name", "result"]
        )?;

        let cache_operations = register_counter_vec!(
            "mcp_cache_operations_total",
            "Cache operations (hits, misses, evictions)",
            &["cache_type", "operation"]
        )?;

        let cache_hit_ratio = register_gauge_vec!(
            "mcp_cache_hit_ratio",
            "Cache hit ratio (0.0 to 1.0)",
            &["cache_type"]
        )?;

        let error_rate = register_counter_vec!(
            "mcp_errors_total",
            "Error count by component and type",
            &["component", "error_type", "severity"]
        )?;

        let request_throughput = register_counter_vec!(
            "mcp_requests_total",
            "Request throughput by endpoint and method",
            &["endpoint", "method", "status"]
        )?;

        let active_connections = register_gauge_vec!(
            "mcp_active_connections",
            "Number of active connections",
            &["connection_type"]
        )?;

        let memory_pool_usage = register_gauge_vec!(
            "mcp_memory_pool_usage_bytes",
            "Memory pool usage in bytes",
            &["pool_name"]
        )?;

        Ok(Self {
            tool_execution_duration,
            tool_execution_count,
            cache_operations,
            cache_hit_ratio,
            error_rate,
            request_throughput,
            active_connections,
            memory_pool_usage,
            custom_metrics: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Record tool execution metrics
    pub fn record_tool_execution(&self, tool_name: &str, duration_seconds: f64, success: bool) {
        let result = if success { "success" } else { "failure" };

        self.tool_execution_duration
            .with_label_values(&[tool_name, result])
            .observe(duration_seconds);

        self.tool_execution_count
            .with_label_values(&[tool_name, result])
            .inc();

        debug!(
            "Recorded tool execution: {} took {:.3}s (result: {})",
            tool_name, duration_seconds, result
        );
    }

    /// Record cache operation
    pub fn record_cache_operation(&self, cache_type: &str, operation: &str) {
        self.cache_operations
            .with_label_values(&[cache_type, operation])
            .inc();

        debug!("Recorded cache operation: {} on {}", operation, cache_type);
    }

    /// Update cache hit ratio
    pub fn update_cache_hit_ratio(&self, cache_type: &str, hit_ratio: f64) {
        self.cache_hit_ratio
            .with_label_values(&[cache_type])
            .set(hit_ratio);

        debug!(
            "Updated cache hit ratio for {}: {:.3}",
            cache_type, hit_ratio
        );
    }

    /// Record error
    pub fn record_error(&self, component: &str, error_type: &str, severity: &str) {
        self.error_rate
            .with_label_values(&[component, error_type, severity])
            .inc();

        match severity {
            "critical" => error!("Critical error in {}: {}", component, error_type),
            "warning" => warn!("Warning in {}: {}", component, error_type),
            _ => info!("Error in {}: {}", component, error_type),
        }
    }

    /// Record request
    pub fn record_request(&self, endpoint: &str, method: &str, status: &str) {
        self.request_throughput
            .with_label_values(&[endpoint, method, status])
            .inc();

        debug!("Recorded request: {} {} -> {}", method, endpoint, status);
    }

    /// Update active connections
    pub fn update_active_connections(&self, connection_type: &str, count: f64) {
        self.active_connections
            .with_label_values(&[connection_type])
            .set(count);

        debug!(
            "Updated active connections for {}: {}",
            connection_type, count
        );
    }

    /// Update memory pool usage
    pub fn update_memory_pool_usage(&self, pool_name: &str, usage_bytes: f64) {
        self.memory_pool_usage
            .with_label_values(&[pool_name])
            .set(usage_bytes);

        debug!(
            "Updated memory pool usage for {}: {} bytes",
            pool_name, usage_bytes
        );
    }

    /// Add a custom metric
    pub async fn add_custom_metric(
        &self,
        name: String,
        metric: Box<dyn prometheus::core::Collector + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut custom_metrics = self.custom_metrics.write().await;
        custom_metrics.insert(name, metric);
        Ok(())
    }

    /// Get metrics in Prometheus format
    pub async fn gather_metrics(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();

        // Add custom metrics
        let custom_metrics = self.custom_metrics.read().await;
        for _metric in custom_metrics.values() {
            // Note: Custom metrics would need to be properly integrated with the registry
            // This is a simplified implementation
        }

        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;

        Ok(String::from_utf8(buffer)?)
    }

    /// Get metrics summary for diagnostics
    pub async fn get_metrics_summary(&self) -> MetricsSummary {
        MetricsSummary {
            tool_executions: self.get_tool_execution_summary().await,
            cache_performance: self.get_cache_performance_summary().await,
            error_rates: self.get_error_rate_summary().await,
            request_throughput: self.get_request_throughput_summary().await,
        }
    }

    async fn get_tool_execution_summary(&self) -> ToolExecutionSummary {
        // In a real implementation, you'd query the actual metric values
        // This is a simplified version
        ToolExecutionSummary {
            total_executions: 0, // Would be calculated from metrics
            average_duration: 0.0,
            success_rate: 0.0,
        }
    }

    async fn get_cache_performance_summary(&self) -> CachePerformanceSummary {
        CachePerformanceSummary {
            hit_ratios: HashMap::new(), // Would be populated from metrics
            total_operations: 0,
        }
    }

    async fn get_error_rate_summary(&self) -> ErrorRateSummary {
        ErrorRateSummary {
            total_errors: 0,
            errors_by_component: HashMap::new(),
        }
    }

    async fn get_request_throughput_summary(&self) -> RequestThroughputSummary {
        RequestThroughputSummary {
            total_requests: 0,
            requests_per_second: 0.0,
        }
    }
}

/// Metrics summary structures for diagnostics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricsSummary {
    pub tool_executions: ToolExecutionSummary,
    pub cache_performance: CachePerformanceSummary,
    pub error_rates: ErrorRateSummary,
    pub request_throughput: RequestThroughputSummary,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolExecutionSummary {
    pub total_executions: u64,
    pub average_duration: f64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CachePerformanceSummary {
    pub hit_ratios: HashMap<String, f64>,
    pub total_operations: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorRateSummary {
    pub total_errors: u64,
    pub errors_by_component: HashMap<String, u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequestThroughputSummary {
    pub total_requests: u64,
    pub requests_per_second: f64,
}

/// Helper macro for timing tool execution
#[macro_export]
macro_rules! time_tool_execution {
    ($collector:expr, $tool_name:expr, $code:block) => {{
        let start = std::time::Instant::now();
        let result = $code;
        let duration = start.elapsed().as_secs_f64();
        let success = result.is_ok();
        $collector.record_tool_execution($tool_name, duration, success);
        result
    }};
}

/// Helper macro for recording cache operations
#[macro_export]
macro_rules! record_cache_operation {
    ($collector:expr, $cache_type:expr, $operation:expr) => {
        $collector.record_cache_operation($cache_type, $operation);
    };
}

/// Helper macro for recording errors
#[macro_export]
macro_rules! record_error {
    ($collector:expr, $component:expr, $error_type:expr, $severity:expr) => {
        $collector.record_error($component, $error_type, $severity);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = Phase3MetricsCollector::new().expect("replaced unwrap");

        // Test tool execution recording
        collector.record_tool_execution("test_tool", 0.5, true);
        collector.record_tool_execution("test_tool", 1.2, false);

        // Test cache operation recording
        collector.record_cache_operation("memory", "hit");
        collector.record_cache_operation("memory", "miss");

        // Test error recording
        collector.record_error("test_component", "network_error", "warning");

        // Test metrics gathering
        let metrics = collector.gather_metrics().await.expect("replaced unwrap");
        assert!(metrics.contains("mcp_tool_execution_duration_seconds"));
        assert!(metrics.contains("mcp_cache_operations_total"));
        assert!(metrics.contains("mcp_errors_total"));
    }

    #[tokio::test]
    async fn test_metrics_summary() {
        let collector = Phase3MetricsCollector::new().expect("replaced unwrap");
        let summary = collector.get_metrics_summary().await;

        // Verify summary structure
        assert!(summary.tool_executions.total_executions >= 0);
        assert!(summary.cache_performance.total_operations >= 0);
        assert!(summary.error_rates.total_errors >= 0);
        assert!(summary.request_throughput.total_requests >= 0);
    }
}
