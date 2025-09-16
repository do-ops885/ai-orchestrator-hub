//! Production Monitoring System Example
//!
//! This example demonstrates how to set up and use the comprehensive
//! production monitoring system for the AI Orchestrator Hub.

use ai_orchestrator_hub::infrastructure::{
    monitoring::{ProductionMonitoringSystem, ProductionMonitoringConfig, EnhancedDashboard, PrometheusExporter},
    metrics::MetricsCollector,
    intelligent_alerting::IntelligentAlertConfig,
};
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting AI Orchestrator Hub Production Monitoring Example");

    // 1. Create production monitoring configuration
    let monitoring_config = ProductionMonitoringConfig {
        health_check_interval_seconds: 30,
        performance_collection_interval_seconds: 15,
        metrics_retention_hours: 168,
        alert_evaluation_interval_seconds: 60,
        dashboard_refresh_interval_seconds: 30,
        enable_prometheus_exporter: true,
        prometheus_port: 9090,
        enable_grafana_integration: true,
        grafana_url: Some("http://localhost:3000".to_string()),
        notification_channels: vec![
            super::NotificationChannelConfig {
                channel_type: "console".to_string(),
                endpoint: None,
                enabled: true,
                severity_filter: vec!["info".to_string(), "warning".to_string(), "critical".to_string()],
            },
        ],
        alerting_thresholds: Default::default(),
        business_metrics_config: Default::default(),
    };

    // 2. Create metrics collector
    let metrics_collector = Arc::new(MetricsCollector::new(10000));

    // 3. Create intelligent alerting configuration
    let alert_config = IntelligentAlertConfig {
        max_alert_history: 10000,
        adaptive_learning_enabled: true,
        predictive_alerting_enabled: true,
        alert_correlation_window_minutes: 5,
        suppression_window_minutes: 30,
        escalation_enabled: true,
    };

    // 4. Initialize production monitoring system
    let monitoring_system = Arc::new(ProductionMonitoringSystem::new(monitoring_config).await?);

    // 5. Create enhanced dashboard
    let dashboard = EnhancedDashboard::new().with_production_monitoring(Arc::clone(&monitoring_system));

    // 6. Create Prometheus exporter
    let prometheus_exporter = PrometheusExporter::new(
        Arc::clone(&monitoring_system),
        Arc::clone(&metrics_collector),
        9090,
    );

    // 7. Start all monitoring components
    println!("📊 Starting monitoring components...");

    // Start production monitoring system
    monitoring_system.start().await?;
    println!("✅ Production monitoring system started");

    // Start dashboard
    dashboard.start().await?;
    println!("✅ Enhanced dashboard started");

    // Start Prometheus exporter
    prometheus_exporter.start().await?;
    println!("✅ Prometheus exporter started on port 9090");

    // 8. Set up notification channels
    monitoring_system.add_notification_channel(super::NotificationChannelConfig {
        channel_type: "webhook".to_string(),
        endpoint: Some("https://hooks.slack.com/services/YOUR/WEBHOOK".to_string()),
        enabled: true,
        severity_filter: vec!["warning".to_string(), "critical".to_string()],
    }).await?;
    println!("✅ Notification channels configured");

    // 9. Create default dashboard
    dashboard.create_default_dashboard().await?;
    println!("✅ Default dashboard created");

    // 10. Add some custom metrics for demonstration
    let custom_metric = PrometheusExporter::create_metric(
        "ai_orchestrator_custom_metric".to_string(),
        "Custom demonstration metric".to_string(),
        super::PrometheusMetricType::Gauge,
        42.0,
        std::collections::HashMap::from([
            ("service".to_string(), "demo".to_string()),
            ("version".to_string(), "1.0".to_string()),
        ]),
    );

    prometheus_exporter.add_custom_metric(custom_metric).await?;
    println!("✅ Custom metrics added");

    // 11. Simulate some monitoring data
    println!("🔄 Simulating monitoring data...");

    // Update business metrics
    monitoring_system.update_business_metrics(super::BusinessMetrics {
        task_completion_rate: 96.5,
        agent_utilization_rate: 82.3,
        system_uptime_percentage: 99.8,
        customer_satisfaction_score: 4.6,
        total_tasks_processed: 15420,
        average_task_duration_ms: 245.0,
        peak_concurrent_users: 150,
        system_throughput_tasks_per_second: 12.5,
        timestamp: chrono::Utc::now(),
    }).await?;

    // 12. Test the monitoring system
    println!("🧪 Testing monitoring system...");
    monitoring_system.test_monitoring_system().await?;
    println!("✅ Monitoring system test completed");

    // 13. Generate monitoring report
    println!("📋 Generating monitoring report...");
    let report = monitoring_system.generate_monitoring_report().await?;
    println!("Monitoring Report:\n{}", serde_json::to_string_pretty(&report)?);

    // 14. Get dashboard data
    println!("📊 Getting dashboard data...");
    let dashboard_data = dashboard.get_all_widget_data().await?;
    println!("Dashboard has {} widgets with data", dashboard_data.len());

    // 15. Display current metrics
    let current_metrics = metrics_collector.get_current_metrics().await;
    println!("📈 Current System Metrics:");
    println!("  CPU Usage: {:.1}%", current_metrics.resource_usage.cpu_usage_percent);
    println!("  Memory Usage: {:.1}%", current_metrics.resource_usage.memory_usage_percent);
    println!("  Active Agents: {}", current_metrics.agent_metrics.active_agents);
    println!("  Tasks in Queue: {}", current_metrics.task_metrics.tasks_in_queue);
    println!("  Error Rate: {:.2}/min", current_metrics.error_metrics.error_rate_per_minute);

    // 16. Display alert statistics
    let alert_stats = monitoring_system.get_alert_statistics().await;
    println!("🚨 Alert Statistics:");
    println!("  Total Alerts: {}", alert_stats.total_alerts);
    println!("  Alerts (24h): {}", alert_stats.alerts_last_24h);
    println!("  Critical Alerts (24h): {}", alert_stats.critical_alerts_24h);

    // 17. Process alerts
    let alerts = monitoring_system.process_alerts().await?;
    if !alerts.is_empty() {
        println!("⚠️  Active Alerts:");
        for alert in alerts.iter().take(5) {
            println!("  - {}: {}", alert.base_alert.title, alert.base_alert.description);
        }
    }

    // 18. Get business metrics
    let business_metrics = monitoring_system.get_business_metrics().await;
    println!("💼 Business Metrics:");
    println!("  Task Completion Rate: {:.1}%", business_metrics.task_completion_rate);
    println!("  Agent Utilization: {:.1}%", business_metrics.agent_utilization_rate);
    println!("  System Uptime: {:.1}%", business_metrics.system_uptime_percentage);
    println!("  Total Tasks Processed: {}", business_metrics.total_tasks_processed);
    println!("  System Throughput: {:.1} tasks/sec", business_metrics.system_throughput_tasks_per_second);

    // 19. Export dashboard configuration
    let dashboard_config = dashboard.export_dashboard_config().await?;
    println!("📄 Dashboard configuration exported ({} bytes)", dashboard_config.len());

    // 20. Generate Prometheus metrics sample
    let prometheus_metrics = prometheus_exporter.generate_metrics().await?;
    println!("📊 Prometheus metrics sample (first 500 chars):");
    println!("{}", &prometheus_metrics[..prometheus_metrics.len().min(500)]);

    println!("\n🎉 Production monitoring example completed successfully!");
    println!("📋 Summary:");
    println!("  ✅ Production monitoring system initialized");
    println!("  ✅ Enhanced dashboard configured");
    println!("  ✅ Prometheus exporter running on port 9090");
    println!("  ✅ Notification channels set up");
    println!("  ✅ Business metrics tracking enabled");
    println!("  ✅ Alert system operational");
    println!("  ✅ Custom metrics added");
    println!("  ✅ Monitoring report generated");

    println!("\n🔗 Access Points:");
    println!("  📊 Prometheus Metrics: http://localhost:9090/metrics");
    println!("  📈 Grafana Dashboard: http://localhost:3000 (if configured)");
    println!("  📋 Health Check: http://localhost:9090/health");

    // Keep the system running for a while to demonstrate real-time monitoring
    println!("\n⏳ Monitoring system is running... Press Ctrl+C to stop");

    // In a real application, you would keep this running
    // For demo purposes, we'll just wait a bit
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // Cleanup
    monitoring_system.stop().await?;
    println!("🛑 Monitoring system stopped");

    Ok(())
}

// Helper modules for the example
mod tests {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use chrono::{DateTime, Utc};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NotificationChannelConfig {
        pub channel_type: String,
        pub endpoint: Option<String>,
        pub enabled: bool,
        pub severity_filter: Vec<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BusinessMetrics {
        pub task_completion_rate: f64,
        pub agent_utilization_rate: f64,
        pub system_uptime_percentage: f64,
        pub customer_satisfaction_score: f64,
        pub total_tasks_processed: u64,
        pub average_task_duration_ms: f64,
        pub peak_concurrent_users: u32,
        pub system_throughput_tasks_per_second: f64,
        pub timestamp: DateTime<Utc>,
    }

    #[derive(Debug, Clone)]
    pub enum PrometheusMetricType {
        Counter,
        Gauge,
        Histogram,
        Summary,
    }

    impl Default for super::ProductionAlertThresholds {
        fn default() -> Self {
            Self {
                agent_health_critical_threshold: 0.5,
                agent_health_warning_threshold: 0.7,
                system_cpu_critical: 90.0,
                system_cpu_warning: 75.0,
                system_memory_critical: 95.0,
                system_memory_warning: 80.0,
                task_failure_rate_critical: 25.0,
                task_failure_rate_warning: 10.0,
                response_time_critical_ms: 5000.0,
                response_time_warning_ms: 1000.0,
                error_rate_critical_per_minute: 10.0,
                error_rate_warning_per_minute: 2.0,
                agent_utilization_low_threshold: 30.0,
                throughput_drop_threshold_percent: 20.0,
            }
        }
    }

    impl Default for super::BusinessMetricsConfig {
        fn default() -> Self {
            Self {
                enable_business_metrics: true,
                task_completion_target_percentage: 95.0,
                agent_utilization_target_percentage: 80.0,
                system_uptime_target_percentage: 99.9,
                customer_satisfaction_target_score: 4.5,
            }
        }
    }

    // Type aliases for the example
    pub type ProductionAlertThresholds = crate::infrastructure::monitoring::production_monitoring::ProductionAlertThresholds;
    pub type BusinessMetricsConfig = crate::infrastructure::monitoring::production_monitoring::BusinessMetricsConfig;
}