//! Core Agent Monitor
//!
//! Central monitoring coordinator that orchestrates all monitoring subsystems

use super::types::{AgentInfo, MonitoringStatus, HealthSnapshot, PerformanceStatusSummary, BehaviorStatusSummary, ReportType, ExportFormat, HealthStatus};
use super::{
    AgentDiscovery, Automation, BehaviorAnalyzer, Dashboard, Diagnostics, HealthMonitor,
    Integration, PerformanceMonitor, Reporting,
};
use crate::infrastructure::intelligent_alerting::IntelligentAlertingSystem;
use crate::infrastructure::metrics::MetricsCollector;
use crate::infrastructure::telemetry::TelemetryCollector;
use crate::utils::config::MonitoringConfig;
use crate::utils::error::{HiveError, HiveResult};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AgentMonitor {
    pub config: Arc<MonitoringConfig>,
    pub metrics_collector: Arc<MetricsCollector>,
    pub telemetry_collector: Arc<TelemetryCollector>,
    pub alerting_system: Arc<IntelligentAlertingSystem>,
    pub agent_discovery: Arc<AgentDiscovery>,
    pub health_monitor: Arc<HealthMonitor>,
    pub performance_monitor: Arc<PerformanceMonitor>,
    pub behavior_analyzer: Arc<BehaviorAnalyzer>,
    pub dashboard: Arc<Dashboard>,
    pub diagnostics: Arc<Diagnostics>,
    pub reporting: Arc<Reporting>,
    pub automation: Arc<Automation>,
    pub integration: Arc<Integration>,
}

impl AgentMonitor {
    /// Create a new agent monitor with all subsystems
    pub async fn new(config: MonitoringConfig) -> HiveResult<Self> {
        let config = Arc::new(config);

        // Initialize core systems
        let metrics_collector = Arc::new(MetricsCollector::new(1000));
        let telemetry_collector = Arc::new(TelemetryCollector::new(1000));

        // Create alerting config
        let alert_config =
            crate::infrastructure::intelligent_alerting::IntelligentAlertConfig::default();
        let alerting_system = Arc::new(IntelligentAlertingSystem::new(
            Arc::clone(&metrics_collector),
            alert_config,
        ));

        // Initialize monitoring subsystems
        let agent_discovery = Arc::new(AgentDiscovery::new());
        let health_monitor = Arc::new(HealthMonitor::new());
        let performance_monitor = Arc::new(PerformanceMonitor::new());
        let behavior_analyzer = Arc::new(BehaviorAnalyzer::new());
        let dashboard = Arc::new(Dashboard::new());
        let diagnostics = Arc::new(Diagnostics::new());
        let reporting = Arc::new(Reporting::new());
        let automation = Arc::new(Automation::new());
        let integration = Arc::new(Integration::new());

        Ok(Self {
            config,
            metrics_collector,
            telemetry_collector,
            alerting_system,
            agent_discovery,
            health_monitor,
            performance_monitor,
            behavior_analyzer,
            dashboard,
            diagnostics,
            reporting,
            automation,
            integration,
        })
    }

    /// Start all monitoring subsystems
    pub async fn start(&self) -> HiveResult<()> {
        tracing::info!("Starting agent monitoring systems");

        // Start subsystems in order
        self.health_monitor.start().await?;
        self.performance_monitor.start().await?;
        self.behavior_analyzer.start().await?;
        self.automation.start().await?;
        self.integration.start().await?;

        tracing::info!("All monitoring systems started successfully");
        Ok(())
    }

    /// Stop all monitoring subsystems
    pub async fn stop(&self) -> HiveResult<()> {
        tracing::info!("Stopping agent monitoring systems");

        // Stop subsystems in reverse order
        self.integration.stop().await?;
        self.automation.stop().await?;
        self.behavior_analyzer.stop().await?;
        self.performance_monitor.stop().await?;
        self.health_monitor.stop().await?;

        tracing::info!("All monitoring systems stopped successfully");
        Ok(())
    }

    /// Register a new agent for monitoring
    pub async fn register_agent(&self, agent_info: AgentInfo) -> HiveResult<()> {
        self.agent_discovery
            .register_agent(agent_info.clone())
            .await?;
        self.health_monitor.add_agent(agent_info.id).await?;
        self.performance_monitor.add_agent(agent_info.id).await?;
        self.behavior_analyzer.add_agent(agent_info.id).await?;

        tracing::info!("Agent {} registered for monitoring", agent_info.id);
        Ok(())
    }

    /// Unregister an agent from monitoring
    pub async fn unregister_agent(&self, agent_id: Uuid) -> HiveResult<()> {
        self.agent_discovery.unregister_agent(agent_id).await?;
        self.health_monitor.remove_agent(agent_id).await?;
        self.performance_monitor.remove_agent(agent_id).await?;
        self.behavior_analyzer.remove_agent(agent_id).await?;

        tracing::info!("Agent {} unregistered from monitoring", agent_id);
        Ok(())
    }

    /// Get comprehensive monitoring status
    pub async fn get_status(&self) -> HiveResult<MonitoringStatus> {
        let agents = self.agent_discovery.get_all_agents().await?;
        let _health_snapshot = self.health_monitor.get_health_snapshot().await?;
        // Placeholder for active alerts - would need to implement this method
        let alerts: Vec<String> = Vec::new(); // self.alerting_system.get_active_alerts().await?;

        Ok(MonitoringStatus {
            is_active: true,
            last_update: chrono::Utc::now(),
            monitored_agents: agents.len() as u32,
            alerts_count: alerts.len() as u32,
        })
    }

    /// Get the count of monitored agents
    pub async fn get_monitored_agents_count(&self) -> u32 {
        self.agent_discovery
            .get_all_agents()
            .await
            .map(|agents| agents.len() as u32)
            .unwrap_or(0)
    }

    /// Get agent IDs for all monitored agents
    pub async fn get_agent_ids(&self) -> Vec<Uuid> {
        self.agent_discovery
            .get_all_agents()
            .await
            .map(|agents| agents.into_iter().map(|a| a.id).collect())
            .unwrap_or_default()
    }

    /// Get health snapshot for all monitored agents
    pub async fn get_health_snapshot(&self) -> HiveResult<HealthSnapshot> {
        self.health_monitor.get_health_snapshot().await
    }

    /// Get performance summary for all monitored agents
    pub async fn get_performance_summary(&self) -> HiveResult<PerformanceStatusSummary> {
        self.performance_monitor.get_performance_summary().await
    }

    /// Get behavior analysis summary
    pub async fn get_behavior_summary(&self) -> HiveResult<BehaviorStatusSummary> {
        self.behavior_analyzer.get_behavior_summary().await
    }

    /// Generate comprehensive monitoring report
    pub async fn generate_report(&self, report_type: ReportType) -> HiveResult<String> {
        self.reporting.generate_report(report_type).await
    }

    /// Export monitoring data in specified format
    pub async fn export_data(&self, format: ExportFormat) -> HiveResult<String> {
        match format {
            ExportFormat::Json => self.export_json().await,
            ExportFormat::Csv => self.export_csv().await,
            ExportFormat::Xml => self.export_xml().await,
            ExportFormat::Prometheus => self.export_prometheus().await,
        }
    }

    async fn export_json(&self) -> HiveResult<String> {
        let status = self.get_status().await?;
        let health = self.get_health_snapshot().await?;
        let performance = self.get_performance_summary().await?;
        let behavior = self.get_behavior_summary().await?;

        let export_data = serde_json::json!({
            "status": status,
            "health": health,
            "performance": performance,
            "behavior": behavior,
            "timestamp": chrono::Utc::now()
        });

        serde_json::to_string_pretty(&export_data).map_err(|e| HiveError::OperationFailed {
            reason: format!("Failed to serialize monitoring data: {e}"),
        })
    }

    async fn export_csv(&self) -> HiveResult<String> {
        // Simplified CSV export - would be expanded based on requirements
        let health = self.get_health_snapshot().await?;
        let mut csv = String::from("agent_id,status,cpu_usage,memory_usage,last_heartbeat\n");

        for agent_health in health.agent_health {
            csv.push_str(&format!(
                "{},{:?},{},{},{}\n",
                agent_health.agent_id,
                agent_health.status,
                agent_health.resource_usage.cpu_usage,
                agent_health.resource_usage.memory_usage,
                agent_health.last_heartbeat
            ));
        }

        Ok(csv)
    }

    async fn export_xml(&self) -> HiveResult<String> {
        // Simplified XML export
        let status = self.get_status().await?;
        Ok(format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<monitoring>
    <status>
        <active>{}</active>
        <monitored_agents>{}</monitored_agents>
        <alerts_count>{}</alerts_count>
        <last_update>{}</last_update>
    </status>
</monitoring>"#,
            status.is_active, status.monitored_agents, status.alerts_count, status.last_update
        ))
    }

    async fn export_prometheus(&self) -> HiveResult<String> {
        let status = self.get_status().await?;
        let health = self.get_health_snapshot().await?;

        let mut metrics = String::new();

        // System metrics
        metrics.push_str(&format!(
            "# HELP hive_monitored_agents Number of monitored agents\n# TYPE hive_monitored_agents gauge\nhive_monitored_agents {}\n",
            status.monitored_agents
        ));

        metrics.push_str(&format!(
            "# HELP hive_active_alerts Number of active alerts\n# TYPE hive_active_alerts gauge\nhive_active_alerts {}\n",
            status.alerts_count
        ));

        // Agent health metrics
        for agent_health in health.agent_health {
            let status_value = match agent_health.status {
                HealthStatus::Healthy => 1,
                HealthStatus::Warning => 2,
                HealthStatus::Critical => 3,
                HealthStatus::Unknown => 0,
            };

            metrics.push_str(&format!(
                "hive_agent_health{{agent_id=\"{}\"}} {}\n",
                agent_health.agent_id, status_value
            ));

            metrics.push_str(&format!(
                "hive_agent_cpu_usage{{agent_id=\"{}\"}} {}\n",
                agent_health.agent_id, agent_health.resource_usage.cpu_usage
            ));

            metrics.push_str(&format!(
                "hive_agent_memory_usage{{agent_id=\"{}\"}} {}\n",
                agent_health.agent_id, agent_health.resource_usage.memory_usage
            ));
        }

        Ok(metrics)
    }
}
