//! Prometheus Metrics Exporter
//!
//! Exports monitoring metrics in Prometheus format for external monitoring systems
//! and integration with Grafana dashboards.

use super::production_monitoring::ProductionMonitoringSystem;
use crate::infrastructure::metrics::{MetricsCollector, SystemMetrics};
use crate::utils::error::HiveResult;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;

/// Prometheus metrics exporter
pub struct PrometheusExporter {
    production_monitoring: Arc<ProductionMonitoringSystem>,
    metrics_collector: Arc<MetricsCollector>,
    custom_metrics: Arc<RwLock<HashMap<String, PrometheusMetric>>>,
    port: u16,
}

#[derive(Debug, Clone)]
pub struct PrometheusMetric {
    pub name: String,
    pub help: String,
    pub metric_type: PrometheusMetricType,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum PrometheusMetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

impl PrometheusExporter {
    /// Create a new Prometheus exporter
    pub fn new(
        production_monitoring: Arc<ProductionMonitoringSystem>,
        metrics_collector: Arc<MetricsCollector>,
        port: u16,
    ) -> Self {
        Self {
            production_monitoring,
            metrics_collector,
            custom_metrics: Arc::new(RwLock::new(HashMap::new())),
            port,
        }
    }

    /// Start the Prometheus metrics server
    pub async fn start(&self) -> HiveResult<()> {
        let port = self.port;
        let metrics_route = warp::path("metrics").map(|| {
            // For now, return a simple response
            // In a real implementation, this would generate actual metrics
            warp::reply::with_header(
                "# AI Orchestrator Hub Metrics\n# Metrics generation not yet implemented\n",
                "content-type",
                "text/plain; version=0.0.4; charset=utf-8",
            )
        });

        let health_route = warp::path("health").map(|| warp::reply::html("OK"));

        let routes = metrics_route.or(health_route);

        tracing::info!("Starting Prometheus metrics server on port {}", port);

        tokio::spawn(async move {
            warp::serve(routes).run(([0, 0, 0, 0], port)).await;
        });

        Ok(())
    }

    /// Generate Prometheus metrics output
    pub async fn generate_metrics(&self) -> HiveResult<String> {
        let mut output = String::new();

        // Add header comments
        output.push_str("# AI Orchestrator Hub Metrics\n");
        output.push_str(&format!("# Generated at: {}\n\n", Utc::now().to_rfc3339()));

        // System metrics
        let system_metrics = self.metrics_collector.get_current_metrics().await;
        output.push_str(&self.generate_system_metrics(&system_metrics));

        // Agent metrics
        output.push_str(&self.generate_agent_metrics(&system_metrics));

        // Task metrics
        output.push_str(&self.generate_task_metrics(&system_metrics));

        // Error metrics
        output.push_str(&self.generate_error_metrics(&system_metrics));

        // Business metrics
        output.push_str(&self.generate_business_metrics().await);

        // Alert metrics
        output.push_str(&self.generate_alert_metrics().await);

        // Custom metrics
        output.push_str(&self.generate_custom_metrics().await);

        Ok(output)
    }

    /// Generate system resource metrics
    fn generate_system_metrics(&self, metrics: &SystemMetrics) -> String {
        let mut output = String::new();

        // CPU usage
        output.push_str(&format!(
            "# HELP ai_orchestrator_cpu_usage_percent Current CPU usage percentage\n"
        ));
        output.push_str(&format!("# TYPE ai_orchestrator_cpu_usage_percent gauge\n"));
        output.push_str(&format!(
            "ai_orchestrator_cpu_usage_percent{} {}\n",
            "", metrics.resource_usage.cpu_usage_percent
        ));

        // Memory usage
        output.push_str(&format!(
            "# HELP ai_orchestrator_memory_usage_percent Current memory usage percentage\n"
        ));
        output.push_str(&format!(
            "# TYPE ai_orchestrator_memory_usage_percent gauge\n"
        ));
        output.push_str(&format!(
            "ai_orchestrator_memory_usage_percent{} {}\n",
            "", metrics.resource_usage.memory_usage_percent
        ));

        // Memory usage bytes
        output.push_str(&format!(
            "# HELP ai_orchestrator_memory_usage_bytes Current memory usage in bytes\n"
        ));
        output.push_str(&format!(
            "# TYPE ai_orchestrator_memory_usage_bytes gauge\n"
        ));
        output.push_str(&format!(
            "ai_orchestrator_memory_usage_bytes{} {}\n",
            "", metrics.resource_usage.memory_usage_bytes
        ));

        // Network I/O
        output.push_str(&format!(
            "# HELP ai_orchestrator_network_bytes_in Total network bytes received\n"
        ));
        output.push_str(&format!(
            "# TYPE ai_orchestrator_network_bytes_in counter\n"
        ));
        output.push_str(&format!(
            "ai_orchestrator_network_bytes_in{} {}\n",
            "", metrics.resource_usage.network_bytes_in
        ));

        output.push_str(&format!(
            "# HELP ai_orchestrator_network_bytes_out Total network bytes sent\n"
        ));
        output.push_str(&format!(
            "# TYPE ai_orchestrator_network_bytes_out counter\n"
        ));
        output.push_str(&format!(
            "ai_orchestrator_network_bytes_out{} {}\n",
            "", metrics.resource_usage.network_bytes_out
        ));

        // Disk I/O
        output.push_str(&format!(
            "# HELP ai_orchestrator_disk_reads_per_second Disk read operations per second\n"
        ));
        output.push_str(&format!(
            "# TYPE ai_orchestrator_disk_reads_per_second gauge\n"
        ));
        output.push_str(&format!(
            "ai_orchestrator_disk_reads_per_second{} {}\n",
            "", metrics.resource_usage.disk_io.reads_per_second
        ));

        output.push_str(&format!(
            "# HELP ai_orchestrator_disk_writes_per_second Disk write operations per second\n"
        ));
        output.push_str(&format!(
            "# TYPE ai_orchestrator_disk_writes_per_second gauge\n"
        ));
        output.push_str(&format!(
            "ai_orchestrator_disk_writes_per_second{} {}\n",
            "", metrics.resource_usage.disk_io.writes_per_second
        ));

        // Network connections
        output.push_str(&format!(
            "# HELP ai_orchestrator_websocket_connections Active WebSocket connections\n"
        ));
        output.push_str(&format!(
            "# TYPE ai_orchestrator_websocket_connections gauge\n"
        ));
        output.push_str(&format!(
            "ai_orchestrator_websocket_connections{} {}\n",
            "", metrics.resource_usage.network_io.websocket_connections
        ));

        output.push_str("\n");
        output
    }

    /// Generate agent metrics
    fn generate_agent_metrics(&self, metrics: &SystemMetrics) -> String {
        let mut output = String::new();

        // Agent counts
        output.push_str(&format!(
            "# HELP ai_orchestrator_total_agents Total number of registered agents\n"
        ));
        output.push_str(&format!("# TYPE ai_orchestrator_total_agents gauge\n"));
        output.push_str(&format!(
            "ai_orchestrator_total_agents{} {}\n",
            "", metrics.agent_metrics.total_agents
        ));

        output.push_str(&format!(
            "# HELP ai_orchestrator_active_agents Number of active agents\n"
        ));
        output.push_str(&format!("# TYPE ai_orchestrator_active_agents gauge\n"));
        output.push_str(&format!(
            "ai_orchestrator_active_agents{} {}\n",
            "", metrics.agent_metrics.active_agents
        ));

        output.push_str(&format!(
            "# HELP ai_orchestrator_idle_agents Number of idle agents\n"
        ));
        output.push_str(&format!("# TYPE ai_orchestrator_idle_agents gauge\n"));
        output.push_str(&format!(
            "ai_orchestrator_idle_agents{} {}\n",
            "", metrics.agent_metrics.idle_agents
        ));

        output.push_str(&format!(
            "# HELP ai_orchestrator_failed_agents Number of failed agents\n"
        ));
        output.push_str(&format!("# TYPE ai_orchestrator_failed_agents gauge\n"));
        output.push_str(&format!(
            "ai_orchestrator_failed_agents{} {}\n",
            "", metrics.agent_metrics.failed_agents
        ));

        // Agent performance
        output.push_str(&format!(
            "# HELP ai_orchestrator_agent_performance_score Average agent performance score\n"
        ));
        output.push_str(&format!(
            "# TYPE ai_orchestrator_agent_performance_score gauge\n"
        ));
        output.push_str(&format!(
            "ai_orchestrator_agent_performance_score{} {}\n",
            "", metrics.agent_metrics.average_agent_performance
        ));

        output.push_str(&format!(
            "# HELP ai_orchestrator_agent_utilization_percent Agent utilization percentage\n"
        ));
        output.push_str(&format!(
            "# TYPE ai_orchestrator_agent_utilization_percent gauge\n"
        ));
        output.push_str(&format!(
            "ai_orchestrator_agent_utilization_percent{} {}\n",
            "", metrics.agent_metrics.agent_utilization_percent
        ));

        // Individual agent metrics
        for (agent_id, agent_metrics) in &metrics.agent_metrics.individual_agent_metrics {
            let labels = format!(r#"{{agent_id="{}"}}"#, agent_id);

            output.push_str(&format!(
                "# HELP ai_orchestrator_agent_tasks_completed Tasks completed by agent\n"
            ));
            output.push_str(&format!(
                "# TYPE ai_orchestrator_agent_tasks_completed counter\n"
            ));
            output.push_str(&format!(
                "ai_orchestrator_agent_tasks_completed{} {}\n",
                labels, agent_metrics.tasks_completed
            ));

            output.push_str(&format!(
                "# HELP ai_orchestrator_agent_tasks_failed Tasks failed by agent\n"
            ));
            output.push_str(&format!(
                "# TYPE ai_orchestrator_agent_tasks_failed counter\n"
            ));
            output.push_str(&format!(
                "ai_orchestrator_agent_tasks_failed{} {}\n",
                labels, agent_metrics.tasks_failed
            ));

            output.push_str(&format!(
                "# HELP ai_orchestrator_agent_average_task_duration Average task duration for agent\n"
            ));
            output.push_str(&format!(
                "# TYPE ai_orchestrator_agent_average_task_duration gauge\n"
            ));
            output.push_str(&format!(
                "ai_orchestrator_agent_average_task_duration{} {}\n",
                labels, agent_metrics.average_task_duration
            ));

            output.push_str(&format!(
                "# HELP ai_orchestrator_agent_learning_progress Agent learning progress\n"
            ));
            output.push_str(&format!(
                "# TYPE ai_orchestrator_agent_learning_progress gauge\n"
            ));
            output.push_str(&format!(
                "ai_orchestrator_agent_learning_progress{} {}\n",
                labels, agent_metrics.learning_progress
            ));
        }

        output.push_str("\n");
        output
    }

    /// Generate task metrics
    fn generate_task_metrics(&self, metrics: &SystemMetrics) -> String {
        let mut output = String::new();

        // Task counters
        output.push_str(&format!(
            "# HELP ai_orchestrator_tasks_submitted Total tasks submitted\n"
        ));
        output.push_str(&format!("# TYPE ai_orchestrator_tasks_submitted counter\n"));
        output.push_str(&format!(
            "ai_orchestrator_tasks_submitted{} {}\n",
            "", metrics.task_metrics.total_tasks_submitted
        ));

        output.push_str(&format!(
            "# HELP ai_orchestrator_tasks_completed Total tasks completed\n"
        ));
        output.push_str(&format!("# TYPE ai_orchestrator_tasks_completed counter\n"));
        output.push_str(&format!(
            "ai_orchestrator_tasks_completed{} {}\n",
            "", metrics.task_metrics.total_tasks_completed
        ));

        output.push_str(&format!(
            "# HELP ai_orchestrator_tasks_failed Total tasks failed\n"
        ));
        output.push_str(&format!("# TYPE ai_orchestrator_tasks_failed counter\n"));
        output.push_str(&format!(
            "ai_orchestrator_tasks_failed{} {}\n",
            "", metrics.task_metrics.total_tasks_failed
        ));

        // Task queue and performance
        output.push_str(&format!(
            "# HELP ai_orchestrator_tasks_in_queue Current tasks in queue\n"
        ));
        output.push_str(&format!("# TYPE ai_orchestrator_tasks_in_queue gauge\n"));
        output.push_str(&format!(
            "ai_orchestrator_tasks_in_queue{} {}\n",
            "", metrics.task_metrics.tasks_in_queue
        ));

        output.push_str(&format!(
            "# HELP ai_orchestrator_average_task_duration_ms Average task duration in milliseconds\n"
        ));
        output.push_str(&format!(
            "# TYPE ai_orchestrator_average_task_duration_ms gauge\n"
        ));
        output.push_str(&format!(
            "ai_orchestrator_average_task_duration_ms{} {}\n",
            "", metrics.task_metrics.average_task_duration_ms
        ));

        output.push_str(&format!(
            "# HELP ai_orchestrator_task_success_rate Task success rate percentage\n"
        ));
        output.push_str(&format!("# TYPE ai_orchestrator_task_success_rate gauge\n"));
        output.push_str(&format!(
            "ai_orchestrator_task_success_rate{} {}\n",
            "",
            metrics.task_metrics.task_success_rate * 100.0
        ));

        output.push_str("\n");
        output
    }

    /// Generate error metrics
    fn generate_error_metrics(&self, metrics: &SystemMetrics) -> String {
        let mut output = String::new();

        // Error counters
        output.push_str(&format!(
            "# HELP ai_orchestrator_total_errors Total number of errors\n"
        ));
        output.push_str(&format!("# TYPE ai_orchestrator_total_errors counter\n"));
        output.push_str(&format!(
            "ai_orchestrator_total_errors{} {}\n",
            "", metrics.error_metrics.total_errors
        ));

        output.push_str(&format!(
            "# HELP ai_orchestrator_critical_errors Number of critical errors\n"
        ));
        output.push_str(&format!("# TYPE ai_orchestrator_critical_errors counter\n"));
        output.push_str(&format!(
            "ai_orchestrator_critical_errors{} {}\n",
            "", metrics.error_metrics.critical_errors
        ));

        // Error rate
        output.push_str(&format!(
            "# HELP ai_orchestrator_error_rate_per_minute Errors per minute\n"
        ));
        output.push_str(&format!(
            "# TYPE ai_orchestrator_error_rate_per_minute gauge\n"
        ));
        output.push_str(&format!(
            "ai_orchestrator_error_rate_per_minute{} {}\n",
            "", metrics.error_metrics.error_rate_per_minute
        ));

        // Errors by type
        for (error_type, count) in &metrics.error_metrics.errors_by_type {
            let labels = format!(r#"{{error_type="{}"}}"#, error_type);

            output.push_str(&format!(
                "# HELP ai_orchestrator_errors_by_type Errors by type\n"
            ));
            output.push_str(&format!("# TYPE ai_orchestrator_errors_by_type counter\n"));
            output.push_str(&format!(
                "ai_orchestrator_errors_by_type{} {}\n",
                labels, count
            ));
        }

        output.push_str("\n");
        output
    }

    /// Generate business metrics
    async fn generate_business_metrics(&self) -> String {
        let mut output = String::new();

        let business_metrics = self.production_monitoring.get_business_metrics().await;
        {
            // Task completion rate
            output.push_str(&format!(
                    "# HELP ai_orchestrator_task_completion_rate Business metric: task completion rate\n"
                ));
            output.push_str(&format!(
                "# TYPE ai_orchestrator_task_completion_rate gauge\n"
            ));
            output.push_str(&format!(
                "ai_orchestrator_task_completion_rate{} {}\n",
                "", business_metrics.task_completion_rate
            ));

            // Agent utilization
            output.push_str(&format!(
                    "# HELP ai_orchestrator_agent_utilization_rate Business metric: agent utilization rate\n"
                ));
            output.push_str(&format!(
                "# TYPE ai_orchestrator_agent_utilization_rate gauge\n"
            ));
            output.push_str(&format!(
                "ai_orchestrator_agent_utilization_rate{} {}\n",
                "", business_metrics.agent_utilization_rate
            ));

            // System uptime
            output.push_str(&format!(
                "# HELP ai_orchestrator_system_uptime_percentage System uptime percentage\n"
            ));
            output.push_str(&format!(
                "# TYPE ai_orchestrator_system_uptime_percentage gauge\n"
            ));
            output.push_str(&format!(
                "ai_orchestrator_system_uptime_percentage{} {}\n",
                "", business_metrics.system_uptime_percentage
            ));

            // Customer satisfaction
            output.push_str(&format!(
                "# HELP ai_orchestrator_customer_satisfaction_score Customer satisfaction score\n"
            ));
            output.push_str(&format!(
                "# TYPE ai_orchestrator_customer_satisfaction_score gauge\n"
            ));
            output.push_str(&format!(
                "ai_orchestrator_customer_satisfaction_score{} {}\n",
                "", business_metrics.customer_satisfaction_score
            ));

            // Total tasks processed
            output.push_str(&format!(
                "# HELP ai_orchestrator_total_tasks_processed Total tasks processed\n"
            ));
            output.push_str(&format!(
                "# TYPE ai_orchestrator_total_tasks_processed counter\n"
            ));
            output.push_str(&format!(
                "ai_orchestrator_total_tasks_processed{} {}\n",
                "", business_metrics.total_tasks_processed
            ));

            // System throughput
            output.push_str(&format!(
                    "# HELP ai_orchestrator_system_throughput_tasks_per_second System throughput in tasks per second\n"
                ));
            output.push_str(&format!(
                "# TYPE ai_orchestrator_system_throughput_tasks_per_second gauge\n"
            ));
            output.push_str(&format!(
                "ai_orchestrator_system_throughput_tasks_per_second{} {}\n",
                "", business_metrics.system_throughput_tasks_per_second
            ));
        }

        output.push_str("\n");
        output
    }

    /// Generate alert metrics
    async fn generate_alert_metrics(&self) -> String {
        let mut output = String::new();

        let alert_stats = self.production_monitoring.get_alert_statistics().await;
        {
            // Alert counts
            output.push_str(&format!(
                "# HELP ai_orchestrator_total_alerts Total number of alerts\n"
            ));
            output.push_str(&format!("# TYPE ai_orchestrator_total_alerts counter\n"));
            output.push_str(&format!(
                "ai_orchestrator_total_alerts{} {}\n",
                "", alert_stats.total_alerts
            ));

            output.push_str(&format!(
                "# HELP ai_orchestrator_alerts_last_24h Alerts in the last 24 hours\n"
            ));
            output.push_str(&format!("# TYPE ai_orchestrator_alerts_last_24h gauge\n"));
            output.push_str(&format!(
                "ai_orchestrator_alerts_last_24h{} {}\n",
                "", alert_stats.alerts_last_24h
            ));

            output.push_str(&format!(
                "# HELP ai_orchestrator_alerts_last_hour Alerts in the last hour\n"
            ));
            output.push_str(&format!("# TYPE ai_orchestrator_alerts_last_hour gauge\n"));
            output.push_str(&format!(
                "ai_orchestrator_alerts_last_hour{} {}\n",
                "", alert_stats.alerts_last_hour
            ));

            output.push_str(&format!(
                "# HELP ai_orchestrator_critical_alerts_24h Critical alerts in the last 24 hours\n"
            ));
            output.push_str(&format!(
                "# TYPE ai_orchestrator_critical_alerts_24h gauge\n"
            ));
            output.push_str(&format!(
                "ai_orchestrator_critical_alerts_24h{} {}\n",
                "", alert_stats.critical_alerts_24h
            ));
        }

        output.push_str("\n");
        output
    }

    /// Generate custom metrics
    async fn generate_custom_metrics(&self) -> String {
        let mut output = String::new();
        let custom_metrics = self.custom_metrics.read().await;

        for metric in custom_metrics.values() {
            // Generate labels string
            let labels = if metric.labels.is_empty() {
                String::new()
            } else {
                let label_pairs: Vec<String> = metric
                    .labels
                    .iter()
                    .map(|(k, v)| format!(r#"{}="{}""#, k, v))
                    .collect();
                format!("{{{}}}", label_pairs.join(","))
            };

            // Generate metric type comment
            let type_str = match metric.metric_type {
                PrometheusMetricType::Counter => "counter",
                PrometheusMetricType::Gauge => "gauge",
                PrometheusMetricType::Histogram => "histogram",
                PrometheusMetricType::Summary => "summary",
            };

            output.push_str(&format!("# HELP {} {}\n", metric.name, metric.help));
            output.push_str(&format!("# TYPE {} {}\n", metric.name, type_str));
            output.push_str(&format!("{}{} {}\n", metric.name, labels, metric.value));
        }

        if !custom_metrics.is_empty() {
            output.push_str("\n");
        }

        output
    }

    /// Add a custom metric
    pub async fn add_custom_metric(&self, metric: PrometheusMetric) -> HiveResult<()> {
        let mut custom_metrics = self.custom_metrics.write().await;
        custom_metrics.insert(metric.name.clone(), metric);
        Ok(())
    }

    /// Remove a custom metric
    pub async fn remove_custom_metric(&self, metric_name: &str) -> HiveResult<()> {
        let mut custom_metrics = self.custom_metrics.write().await;
        custom_metrics.remove(metric_name);
        Ok(())
    }

    /// Update a custom metric value
    pub async fn update_custom_metric(&self, metric_name: &str, value: f64) -> HiveResult<()> {
        let mut custom_metrics = self.custom_metrics.write().await;
        if let Some(metric) = custom_metrics.get_mut(metric_name) {
            metric.value = value;
            metric.timestamp = Utc::now();
            Ok(())
        } else {
            Err(crate::utils::error::HiveError::NotFound {
                resource: format!("Custom metric {}", metric_name),
            })
        }
    }

    /// Get all custom metrics
    pub async fn get_custom_metrics(&self) -> HashMap<String, PrometheusMetric> {
        self.custom_metrics.read().await.clone()
    }

    /// Create a standard custom metric
    pub fn create_metric(
        name: String,
        help: String,
        metric_type: PrometheusMetricType,
        value: f64,
        labels: HashMap<String, String>,
    ) -> PrometheusMetric {
        PrometheusMetric {
            name,
            help,
            metric_type,
            value,
            labels,
            timestamp: Utc::now(),
        }
    }
}

impl Clone for PrometheusExporter {
    fn clone(&self) -> Self {
        Self {
            production_monitoring: Arc::clone(&self.production_monitoring),
            metrics_collector: Arc::clone(&self.metrics_collector),
            custom_metrics: Arc::clone(&self.custom_metrics),
            port: self.port,
        }
    }
}
