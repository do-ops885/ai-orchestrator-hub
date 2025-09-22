//! Production Monitoring System
//!
//! Comprehensive production monitoring setup that integrates all monitoring components
//! for enterprise-grade observability and alerting.

use super::agent_monitor::AgentMonitor;
use super::diagnostics::Diagnostics;
use super::health_monitor::HealthMonitor;
use super::integration::Integration;
use super::performance_monitor::PerformanceMonitor;
use super::reporting::Reporting;
use super::types::{HealthSnapshot, MonitoringStatus, PerformanceStatusSummary};
use crate::infrastructure::intelligent_alerting::{
    IntelligentAlertConfig, IntelligentAlertingSystem,
};
use crate::infrastructure::metrics::MetricsCollector;
use crate::infrastructure::telemetry::{
    ConsoleTelemetrySubscriber, TelemetryCollector, WebhookTelemetrySubscriber,
};
use crate::utils::error::HiveResult;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Production monitoring configuration
#[derive(Debug, Clone)]
pub struct ProductionMonitoringConfig {
    pub health_check_interval_seconds: u64,
    pub performance_collection_interval_seconds: u64,
    pub metrics_retention_hours: u64,
    pub alert_evaluation_interval_seconds: u64,
    pub dashboard_refresh_interval_seconds: u64,
    pub enable_prometheus_exporter: bool,
    pub prometheus_port: u16,
    pub enable_grafana_integration: bool,
    pub grafana_url: Option<String>,
    pub notification_channels: Vec<NotificationChannelConfig>,
    pub alerting_thresholds: ProductionAlertThresholds,
    pub business_metrics_config: BusinessMetricsConfig,
}

#[derive(Debug, Clone)]
pub struct NotificationChannelConfig {
    pub channel_type: String,
    pub endpoint: Option<String>,
    pub enabled: bool,
    pub severity_filter: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProductionAlertThresholds {
    pub agent_health_critical_threshold: f64,
    pub agent_health_warning_threshold: f64,
    pub system_cpu_critical: f64,
    pub system_cpu_warning: f64,
    pub system_memory_critical: f64,
    pub system_memory_warning: f64,
    pub task_failure_rate_critical: f64,
    pub task_failure_rate_warning: f64,
    pub response_time_critical_ms: f64,
    pub response_time_warning_ms: f64,
    pub error_rate_critical_per_minute: f64,
    pub error_rate_warning_per_minute: f64,
    pub agent_utilization_low_threshold: f64,
    pub throughput_drop_threshold_percent: f64,
}

#[derive(Debug, Clone)]
pub struct BusinessMetricsConfig {
    pub enable_business_metrics: bool,
    pub task_completion_target_percentage: f64,
    pub agent_utilization_target_percentage: f64,
    pub system_uptime_target_percentage: f64,
    pub customer_satisfaction_target_score: f64,
}

/// Production monitoring system that orchestrates all monitoring components
pub struct ProductionMonitoringSystem {
    config: ProductionMonitoringConfig,
    health_monitor: Arc<HealthMonitor>,
    performance_monitor: Arc<PerformanceMonitor>,
    agent_monitor: Arc<AgentMonitor>,
    diagnostics: Arc<Diagnostics>,
    reporting: Arc<Reporting>,
    integration: Arc<Integration>,
    intelligent_alerting: Arc<IntelligentAlertingSystem>,
    metrics_collector: Arc<MetricsCollector>,
    telemetry_collector: Arc<TelemetryCollector>,
    business_metrics: Arc<RwLock<BusinessMetrics>>,
    monitoring_active: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone)]
pub struct BusinessMetrics {
    pub task_completion_rate: f64,
    pub agent_utilization_rate: f64,
    pub system_uptime_percentage: f64,
    pub customer_satisfaction_score: f64,
    pub total_tasks_processed: u64,
    pub average_task_duration_ms: f64,
    pub peak_concurrent_users: u32,
    pub system_throughput_tasks_per_second: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for ProductionMonitoringConfig {
    fn default() -> Self {
        Self {
            health_check_interval_seconds: 30,
            performance_collection_interval_seconds: 15,
            metrics_retention_hours: 168, // 7 days
            alert_evaluation_interval_seconds: 60,
            dashboard_refresh_interval_seconds: 30,
            enable_prometheus_exporter: true,
            prometheus_port: 9090,
            enable_grafana_integration: true,
            grafana_url: Some("http://localhost:3000".to_string()),
            notification_channels: vec![NotificationChannelConfig {
                channel_type: "console".to_string(),
                endpoint: None,
                enabled: true,
                severity_filter: vec![
                    "info".to_string(),
                    "warning".to_string(),
                    "critical".to_string(),
                ],
            }],
            alerting_thresholds: ProductionAlertThresholds::default(),
            business_metrics_config: BusinessMetricsConfig::default(),
        }
    }
}

impl Default for ProductionAlertThresholds {
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

impl Default for BusinessMetricsConfig {
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

impl ProductionMonitoringSystem {
    /// Create a new production monitoring system
    pub async fn new(config: ProductionMonitoringConfig) -> HiveResult<Self> {
        let metrics_collector = Arc::new(MetricsCollector::new(10000));

        // Initialize intelligent alerting system
        let alert_config = IntelligentAlertConfig {
            max_alert_history: 10000,
            adaptive_learning_enabled: true,
            predictive_alerting_enabled: true,
            alert_correlation_window_minutes: 5,
            suppression_window_minutes: 30,
            escalation_enabled: true,
        };

        let intelligent_alerting = Arc::new(IntelligentAlertingSystem::new(
            Arc::clone(&metrics_collector),
            alert_config,
        ));

        // Initialize telemetry collector
        let telemetry_collector = Arc::new(TelemetryCollector::new(50000));

        // Add console subscriber by default
        telemetry_collector
            .add_subscriber(Box::new(ConsoleTelemetrySubscriber))
            .await;

        // Add webhook subscribers based on configuration
        for channel in &config.notification_channels {
            if channel.enabled && channel.channel_type == "webhook" {
                if let Some(endpoint) = &channel.endpoint {
                    let webhook_subscriber = WebhookTelemetrySubscriber::new(endpoint.clone());
                    telemetry_collector
                        .add_subscriber(Box::new(webhook_subscriber))
                        .await;
                }
            }
        }

        let system = Self {
            config,
            health_monitor: Arc::new(HealthMonitor::new()),
            performance_monitor: Arc::new(PerformanceMonitor::new()),
            agent_monitor: Arc::new(
                AgentMonitor::new(crate::utils::config::MonitoringConfig {
                    monitoring_interval_secs: 30,
                    metrics_retention_days: 7,
                    alert_threshold: 0.8,
                    metrics_endpoint: "/metrics".to_string(),
                    health_endpoint: "/health".to_string(),
                    enable_agent_discovery: true,
                    enable_health_monitoring: true,
                    enable_performance_monitoring: true,
                    enable_behavior_analysis: true,
                    enable_dashboards: true,
                    enable_alerting: true,
                    enable_diagnostics: true,
                    enable_reporting: true,
                    enable_automation: true,
                    enable_external_integration: true,
                    diagnostics: crate::utils::config::DiagnosticsConfig {
                        component_health_scores: HashMap::new(),
                        component_issues: HashMap::new(),
                        component_recommendations: HashMap::new(),
                        network_components: vec![],
                        default_health_score: 1.0,
                        performance_bottlenecks: vec![],
                        optimization_opportunities: vec![],
                    },
                })
                .await?,
            ),
            diagnostics: Arc::new(Diagnostics::new()),
            reporting: Arc::new(Reporting::new()),
            integration: Arc::new(Integration::new()),
            intelligent_alerting,
            metrics_collector,
            telemetry_collector,
            business_metrics: Arc::new(RwLock::new(BusinessMetrics::default())),
            monitoring_active: Arc::new(RwLock::new(false)),
        };

        Ok(system)
    }

    /// Start the production monitoring system
    pub async fn start(&self) -> HiveResult<()> {
        info!("🚀 Starting production monitoring system...");

        *self.monitoring_active.write().await = true;

        // Start individual monitoring components
        self.health_monitor.start().await?;
        self.performance_monitor.start().await?;
        self.agent_monitor.start().await?;
        self.diagnostics.start().await?;
        self.reporting.start().await?;
        self.integration.start().await?;

        // Initialize default alert rules
        self.intelligent_alerting.initialize_default_rules().await;

        // Start background monitoring tasks
        self.start_background_tasks();

        // Start telemetry background tasks
        self.telemetry_collector.clone().start_background_tasks();

        info!("✅ Production monitoring system started successfully");
        Ok(())
    }

    /// Stop the production monitoring system
    pub async fn stop(&self) -> HiveResult<()> {
        info!("🛑 Stopping production monitoring system...");

        *self.monitoring_active.write().await = false;

        // Stop individual components
        self.health_monitor.stop().await?;
        self.performance_monitor.stop().await?;
        self.agent_monitor.stop().await?;
        self.diagnostics.stop().await?;
        self.reporting.stop().await?;
        self.integration.stop().await?;

        info!("✅ Production monitoring system stopped");
        Ok(())
    }

    /// Get comprehensive system health snapshot
    pub async fn get_system_health_snapshot(&self) -> HiveResult<HealthSnapshot> {
        self.health_monitor.get_health_snapshot().await
    }

    /// Get performance summary
    pub async fn get_performance_summary(&self) -> HiveResult<PerformanceStatusSummary> {
        self.performance_monitor.get_performance_summary().await
    }

    /// Get current business metrics
    pub async fn get_business_metrics(&self) -> BusinessMetrics {
        self.business_metrics.read().await.clone()
    }

    /// Update business metrics
    pub async fn update_business_metrics(&self, metrics: BusinessMetrics) -> HiveResult<()> {
        *self.business_metrics.write().await = metrics.clone();

        // Record telemetry event
        self.telemetry_collector
            .record_event(
                crate::infrastructure::telemetry::EventType::PerformanceMetric,
                "business_metrics".to_string(),
                serde_json::json!({
                    "task_completion_rate": metrics.task_completion_rate,
                    "agent_utilization_rate": metrics.agent_utilization_rate,
                    "system_uptime_percentage": metrics.system_uptime_percentage,
                    "total_tasks_processed": metrics.total_tasks_processed,
                    "system_throughput": metrics.system_throughput_tasks_per_second
                }),
                crate::infrastructure::telemetry::Severity::Info,
            )
            .await;

        Ok(())
    }

    /// Process intelligent alerts
    pub async fn process_alerts(
        &self,
    ) -> HiveResult<Vec<crate::infrastructure::intelligent_alerting::IntelligentAlert>> {
        self.intelligent_alerting
            .process_intelligent_alerts()
            .await
            .map_err(|e| crate::utils::error::HiveError::AlertingSystemError {
                alert_type: "intelligent".to_string(),
                reason: e.to_string(),
            })
    }

    /// Get monitoring status
    pub async fn get_monitoring_status(&self) -> MonitoringStatus {
        let alerts_count = self
            .intelligent_alerting
            .get_alert_statistics()
            .await
            .total_alerts as u32;
        let monitored_agents = self.agent_monitor.get_monitored_agents_count().await;

        MonitoringStatus {
            is_active: *self.monitoring_active.read().await,
            last_update: chrono::Utc::now(),
            monitored_agents,
            alerts_count,
        }
    }

    /// Generate comprehensive monitoring report
    pub async fn generate_monitoring_report(&self) -> HiveResult<serde_json::Value> {
        let health_snapshot = self.get_system_health_snapshot().await?;
        let performance_summary = self.get_performance_summary().await?;
        let business_metrics = self.get_business_metrics().await;
        let monitoring_status = self.get_monitoring_status().await;
        let telemetry_metrics = self.telemetry_collector.get_metrics().await;

        let report = serde_json::json!({
            "timestamp": chrono::Utc::now(),
            "system_health": {
                "overall_status": health_snapshot.overall_status,
                "agent_health_count": health_snapshot.agent_health.len(),
                "system_cpu_usage": health_snapshot.system_health.cpu_usage,
                "system_memory_usage": health_snapshot.system_health.memory_usage,
                "active_connections": health_snapshot.system_health.active_connections
            },
            "performance": {
                "overall_score": performance_summary.overall_score,
                "trend": performance_summary.trend,
                "bottlenecks": performance_summary.bottlenecks,
                "recommendations": performance_summary.recommendations
            },
            "business_metrics": {
                "task_completion_rate": business_metrics.task_completion_rate,
                "agent_utilization_rate": business_metrics.agent_utilization_rate,
                "system_uptime_percentage": business_metrics.system_uptime_percentage,
                "total_tasks_processed": business_metrics.total_tasks_processed,
                "system_throughput": business_metrics.system_throughput_tasks_per_second
            },
            "monitoring_status": {
                "is_active": monitoring_status.is_active,
                "monitored_agents": monitoring_status.monitored_agents,
                "alerts_count": monitoring_status.alerts_count
            },
            "telemetry": {
                "total_events": telemetry_metrics.total_events,
                "events_per_minute": telemetry_metrics.average_events_per_minute,
                "uptime_hours": telemetry_metrics.uptime_seconds as f64 / 3600.0
            }
        });

        Ok(report)
    }

    /// Start background monitoring tasks
    fn start_background_tasks(&self) {
        let system = Arc::new(self.clone());

        // Health check task
        {
            let system = Arc::clone(&system);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(
                    system.config.health_check_interval_seconds,
                ));

                while *system.monitoring_active.read().await {
                    interval.tick().await;

                    if let Err(e) = system.perform_health_checks().await {
                        error!("Health check failed: {}", e);
                    }
                }
            });
        }

        // Performance collection task
        {
            let system = Arc::clone(&system);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(
                    system.config.performance_collection_interval_seconds,
                ));

                while *system.monitoring_active.read().await {
                    interval.tick().await;

                    if let Err(e) = system.collect_performance_metrics().await {
                        error!("Performance collection failed: {}", e);
                    }
                }
            });
        }

        // Alert evaluation task
        {
            let system = Arc::clone(&system);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(
                    system.config.alert_evaluation_interval_seconds,
                ));

                while *system.monitoring_active.read().await {
                    interval.tick().await;

                    if let Err(e) = system.evaluate_alerts().await {
                        error!("Alert evaluation failed: {}", e);
                    }
                }
            });
        }

        // Business metrics update task
        if self.config.business_metrics_config.enable_business_metrics {
            let system = Arc::clone(&system);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes

                while *system.monitoring_active.read().await {
                    interval.tick().await;

                    if let Err(e) = system.update_business_metrics_task().await {
                        error!("Business metrics update failed: {}", e);
                    }
                }
            });
        }
    }

    /// Perform comprehensive health checks
    async fn perform_health_checks(&self) -> HiveResult<()> {
        // Update system health metrics
        self.health_monitor.update_system_metrics().await?;

        // Check agent health
        let agent_ids = self.agent_monitor.get_agent_ids().await;
        for agent_id in agent_ids {
            let health_status = self.health_monitor.check_agent_health(agent_id).await?;
            // Update agent health status if needed
            if let Some(mut agent_health) = self.health_monitor.get_agent_health(agent_id).await? {
                if agent_health.status != health_status {
                    agent_health.status = health_status;
                    self.health_monitor
                        .update_agent_health(agent_health)
                        .await?;
                }
            }
        }

        Ok(())
    }

    /// Collect performance metrics
    async fn collect_performance_metrics(&self) -> HiveResult<()> {
        // Collect system metrics
        let system_metrics = self.metrics_collector.collect_system_metrics().await?;

        // Update performance metrics
        self.metrics_collector
            .update_performance_metrics(system_metrics.performance)
            .await;
        self.metrics_collector
            .update_resource_metrics(system_metrics.resource_usage)
            .await;
        self.metrics_collector
            .update_agent_metrics(system_metrics.agent_metrics)
            .await;
        self.metrics_collector
            .update_task_metrics(system_metrics.task_metrics)
            .await;

        // Snapshot for historical data
        self.metrics_collector.snapshot_current_metrics().await;

        Ok(())
    }

    /// Evaluate alerts
    async fn evaluate_alerts(&self) -> HiveResult<()> {
        let alerts = self.process_alerts().await?;

        if !alerts.is_empty() {
            warn!("Generated {} intelligent alerts", alerts.len());

            // Record alert telemetry
            for alert in &alerts {
                self.telemetry_collector
                    .record_event(
                        crate::infrastructure::telemetry::EventType::SystemAlert,
                        "alert_system".to_string(),
                        serde_json::json!({
                            "alert_title": alert.base_alert.title,
                            "severity": format!("{:?}", alert.base_alert.level),
                            "description": alert.base_alert.description,
                            "predicted": alert.predicted,
                            "confidence": alert.confidence
                        }),
                        match alert.base_alert.level {
                            crate::infrastructure::metrics::AlertLevel::Critical => {
                                crate::infrastructure::telemetry::Severity::Critical
                            }
                            crate::infrastructure::metrics::AlertLevel::Warning => {
                                crate::infrastructure::telemetry::Severity::Warning
                            }
                            _ => crate::infrastructure::telemetry::Severity::Info,
                        },
                    )
                    .await;
            }
        }

        Ok(())
    }

    /// Update business metrics
    async fn update_business_metrics_task(&self) -> HiveResult<()> {
        let current_metrics = self.metrics_collector.get_current_metrics().await;

        // Calculate business metrics
        let task_completion_rate = if current_metrics.task_metrics.total_tasks_submitted > 0 {
            (current_metrics.task_metrics.total_tasks_completed as f64
                / current_metrics.task_metrics.total_tasks_submitted as f64)
                * 100.0
        } else {
            0.0
        };

        let agent_utilization_rate = if current_metrics.agent_metrics.total_agents > 0 {
            current_metrics.agent_metrics.agent_utilization_percent
        } else {
            0.0
        };

        // Calculate system uptime (simplified - would need actual uptime tracking)
        let system_uptime_percentage = 99.5; // Placeholder

        let business_metrics = BusinessMetrics {
            task_completion_rate,
            agent_utilization_rate,
            system_uptime_percentage,
            customer_satisfaction_score: 4.2, // Placeholder
            total_tasks_processed: current_metrics.task_metrics.total_tasks_completed,
            average_task_duration_ms: current_metrics.task_metrics.average_task_duration_ms,
            peak_concurrent_users: 100, // Placeholder
            system_throughput_tasks_per_second: current_metrics
                .performance
                .throughput_tasks_per_second,
            timestamp: chrono::Utc::now(),
        };

        self.update_business_metrics(business_metrics).await?;

        Ok(())
    }

    /// Add notification channel
    pub async fn add_notification_channel(
        &self,
        channel: NotificationChannelConfig,
    ) -> HiveResult<()> {
        match channel.channel_type.as_str() {
            "webhook" => {
                if let Some(endpoint) = &channel.endpoint {
                    let webhook_subscriber = WebhookTelemetrySubscriber::new(endpoint.clone());
                    self.telemetry_collector
                        .add_subscriber(Box::new(webhook_subscriber))
                        .await;
                }
            }
            "console" => {
                // Console is already added by default
            }
            _ => {
                warn!(
                    "Unsupported notification channel type: {}",
                    channel.channel_type
                );
            }
        }

        info!("Added notification channel: {}", channel.channel_type);
        Ok(())
    }

    /// Get alert statistics
    pub async fn get_alert_statistics(
        &self,
    ) -> crate::infrastructure::intelligent_alerting::AlertStatistics {
        self.intelligent_alerting.get_alert_statistics().await
    }

    /// Get metrics collector (for internal use)
    #[must_use]
    pub fn get_metrics_collector(&self) -> Arc<MetricsCollector> {
        Arc::clone(&self.metrics_collector)
    }

    /// Get telemetry collector (for internal use)
    #[must_use]
    pub fn get_telemetry_collector(&self) -> Arc<TelemetryCollector> {
        Arc::clone(&self.telemetry_collector)
    }

    /// Test the monitoring system
    pub async fn test_monitoring_system(&self) -> HiveResult<()> {
        info!("🧪 Testing production monitoring system...");

        // Test health monitoring
        let health_snapshot = self.get_system_health_snapshot().await?;
        info!(
            "✅ Health monitoring test passed - Overall status: {:?}",
            health_snapshot.overall_status
        );

        // Test performance monitoring
        let performance_summary = self.get_performance_summary().await?;
        info!(
            "✅ Performance monitoring test passed - Score: {:.1}",
            performance_summary.overall_score
        );

        // Test alert system
        let alerts = self.process_alerts().await?;
        info!(
            "✅ Alert system test passed - Generated {} alerts",
            alerts.len()
        );

        // Test business metrics
        let business_metrics = self.get_business_metrics().await;
        info!(
            "✅ Business metrics test passed - Task completion rate: {:.1}%",
            business_metrics.task_completion_rate
        );

        // Test monitoring report generation
        let report = self.generate_monitoring_report().await?;
        info!(
            "✅ Monitoring report generation test passed - Report size: {} bytes",
            report.to_string().len()
        );

        info!("🎉 All monitoring system tests passed!");
        Ok(())
    }
}

impl Default for BusinessMetrics {
    fn default() -> Self {
        Self {
            task_completion_rate: 0.0,
            agent_utilization_rate: 0.0,
            system_uptime_percentage: 100.0,
            customer_satisfaction_score: 0.0,
            total_tasks_processed: 0,
            average_task_duration_ms: 0.0,
            peak_concurrent_users: 0,
            system_throughput_tasks_per_second: 0.0,
            timestamp: chrono::Utc::now(),
        }
    }
}

impl Clone for ProductionMonitoringSystem {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            health_monitor: Arc::clone(&self.health_monitor),
            performance_monitor: Arc::clone(&self.performance_monitor),
            agent_monitor: Arc::clone(&self.agent_monitor),
            diagnostics: Arc::clone(&self.diagnostics),
            reporting: Arc::clone(&self.reporting),
            integration: Arc::clone(&self.integration),
            intelligent_alerting: Arc::clone(&self.intelligent_alerting),
            metrics_collector: Arc::clone(&self.metrics_collector),
            telemetry_collector: Arc::clone(&self.telemetry_collector),
            business_metrics: Arc::clone(&self.business_metrics),
            monitoring_active: Arc::clone(&self.monitoring_active),
        }
    }
}
