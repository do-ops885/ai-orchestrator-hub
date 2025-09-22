//! # Metrics Export Module
//!
//! This module handles exporting metrics in different formats
//! including JSON and Prometheus formats.

use crate::utils::error::{HiveError, HiveResult};

use super::collector::MetricsCollector;
use super::types::HiveMetrics;

impl MetricsCollector {
    /// Export metrics in specified format
    ///
    /// Exports the current system metrics in the requested format for
    /// integration with external monitoring and alerting systems.
    ///
    /// ## Supported Formats
    ///
    /// - `"json"`: Pretty-printed JSON format for human readability
    /// - `"prometheus"`: Prometheus exposition format for monitoring systems
    ///
    /// ## JSON Format
    ///
    /// ```json
    /// {
    ///   "agent_metrics": {
    ///     "total_agents": 42,
    ///     "active_agents": 38
    ///   },
    ///   "task_metrics": {
    ///     "total_tasks": 1000,
    ///     "success_rate": 0.95
    ///   }
    /// }
    /// ```
    ///
    /// ## Prometheus Format
    ///
    /// ```
    /// # HELP hive_agents_total Total number of agents
    /// # TYPE hive_agents_total counter
    /// hive_agents_total 42
    /// ```
    ///
    /// ## Performance
    ///
    /// O(1) for JSON export, O(1) for Prometheus export.
    /// Minimal overhead for format conversion.
    ///
    /// ## Use Cases
    ///
    /// - Monitoring system integration
    /// - API endpoints for metrics
    /// - Alerting system data sources
    /// - Performance analysis tools
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::MetricsCollector;
    /// # async fn example(metrics_collector: &MetricsCollector) -> Result<(), Box<dyn std::error::Error>> {
    /// // Export in JSON format
    /// let json_metrics = metrics_collector.export_metrics("json").await?;
    /// println!("JSON Metrics: {}", json_metrics);
    ///
    /// // Export in Prometheus format
    /// let prometheus_metrics = metrics_collector.export_metrics("prometheus").await?;
    /// println!("Prometheus Metrics:\n{}", prometheus_metrics);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `format` - Export format ("json" or "prometheus")
    ///
    /// # Returns
    ///
    /// Returns the metrics data as a formatted string on success.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Format is not supported
    /// - Metrics serialization fails
    /// - Internal data access fails
    pub async fn export_metrics(&self, format: &str) -> HiveResult<String> {
        let metrics = self.metrics.read().await;

        match format {
            "json" => {
                serde_json::to_string_pretty(&*metrics).map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to serialize metrics to JSON: {e}"),
                })
            }
            "prometheus" => Ok(self.format_prometheus_metrics(&metrics).await),
            _ => Err(HiveError::ValidationError {
                field: "format".to_string(),
                reason: format!("Unsupported export format: {format}"),
            }),
        }
    }

    /// Format metrics in Prometheus format
    ///
    /// Converts `HiveMetrics` into Prometheus exposition format for
    /// integration with Prometheus monitoring systems.
    ///
    /// ## Prometheus Format
    ///
    /// ```
    /// # HELP hive_agents_total Total number of agents
    /// # TYPE hive_agents_total counter
    /// hive_agents_total 42
    /// # HELP hive_tasks_success_rate Task success rate
    /// # TYPE hive_tasks_success_rate gauge
    /// hive_tasks_success_rate 0.95
    /// ```
    ///
    /// ## Metric Types
    ///
    /// - **Counters**: Monotonically increasing values (agents, tasks)
    /// - **Gauges**: Values that can increase/decrease (rates, percentages)
    ///
    /// ## Performance
    ///
    /// O(1) time complexity - fixed number of string formatting operations.
    ///
    /// # Parameters
    ///
    /// * `metrics` - Metrics data to format
    ///
    /// # Returns
    ///
    /// Returns Prometheus-formatted metrics as a string.
    async fn format_prometheus_metrics(&self, metrics: &HiveMetrics) -> String {
        format!(
            "# HELP hive_agents_total Total number of agents\n\
             # TYPE hive_agents_total counter\n\
             hive_agents_total {}\n\
             # HELP hive_agents_active Number of active agents\n\
             # TYPE hive_agents_active gauge\n\
             hive_agents_active {}\n\
             # HELP hive_tasks_total Total number of tasks\n\
             # TYPE hive_tasks_total counter\n\
             hive_tasks_total {}\n\
             # HELP hive_tasks_success_rate Task success rate\n\
             # TYPE hive_tasks_success_rate gauge\n\
             hive_tasks_success_rate {}\n\
             # HELP hive_system_uptime_seconds System uptime in seconds\n\
             # TYPE hive_system_uptime_seconds counter\n\
             hive_system_uptime_seconds {}\n\
             # HELP hive_system_cpu_usage_percent CPU usage percentage\n\
             # TYPE hive_system_cpu_usage_percent gauge\n\
             hive_system_cpu_usage_percent {}\n",
            metrics.agent_metrics.total_agents,
            metrics.agent_metrics.active_agents,
            metrics.task_metrics.total_tasks,
            metrics.task_metrics.success_rate,
            metrics.system_metrics.uptime_seconds,
            metrics.system_metrics.cpu_usage_percent
        )
    }
}
