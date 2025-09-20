//! Comprehensive monitoring and metrics system
//!
//! Provides real-time monitoring, metrics collection, alerting,
//! and performance tracking for the AI Orchestrator Hub.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::{interval, timeout};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::agents::{Agent, AgentState};
use crate::communication::CommunicationManager;
use crate::core::hive::coordinator::core::HiveCoordinator;
use crate::tasks::task::TaskStatus;

/// Core metrics types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_io: NetworkMetrics,
    pub agent_metrics: AgentMetrics,
    pub swarm_metrics: SwarmMetrics,
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connections_active: u32,
    pub connections_total: u64,
    pub latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub total_agents: usize,
    pub active_agents: usize,
    pub idle_agents: usize,
    pub failed_agents: usize,
    pub average_response_time: f64,
    pub agent_health_scores: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmMetrics {
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub pending_tasks: usize,
    pub average_task_duration: f64,
    pub task_success_rate: f64,
    pub load_distribution: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub throughput: f64, // tasks per second
    pub latency_p50: f64,
    pub latency_p95: f64,
    pub latency_p99: f64,
    pub error_rate: f64,
    pub resource_utilization: f64,
}

/// Alert types and configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: AlertSeverity,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub acknowledged: bool,
    pub resolved: bool,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub condition: String, // Expression to evaluate
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub cooldown_minutes: u32,
    pub last_triggered: Option<DateTime<Utc>>,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub collection_interval_seconds: u64,
    pub retention_days: u32,
    pub enable_prometheus: bool,
    pub prometheus_port: u16,
    pub alert_rules: Vec<AlertRule>,
    pub notification_channels: Vec<String>,
}

/// Main monitoring system
pub struct MonitoringSystem {
    config: MonitoringConfig,
    metrics_history: Arc<RwLock<Vec<SystemMetrics>>>,
    alerts: Arc<RwLock<Vec<Alert>>>,
    swarm_coordinator: Arc<SwarmCoordinator>,
    communication_manager: Arc<CommunicationManager>,
    start_time: Instant,
}

impl MonitoringSystem {
    pub fn new(
        config: MonitoringConfig,
        swarm_coordinator: Arc<SwarmCoordinator>,
        communication_manager: Arc<CommunicationManager>,
    ) -> Self {
        Self {
            config,
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            swarm_coordinator,
            communication_manager,
            start_time: Instant::now(),
        }
    }

    /// Start the monitoring system
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let metrics_history = self.metrics_history.clone();
        let alerts = self.alerts.clone();
        let config = self.config.clone();
        let swarm_coordinator = self.swarm_coordinator.clone();
        let communication_manager = self.communication_manager.clone();

        // Start metrics collection
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.collection_interval_seconds));
            loop {
                interval.tick().await;

                if let Ok(metrics) = Self::collect_metrics(
                    &swarm_coordinator,
                    &communication_manager,
                ).await {
                    let mut history = metrics_history.write().await;
                    history.push(metrics);

                    // Keep only recent metrics
                    let max_entries = (config.retention_days * 24 * 60 * 60 / config.collection_interval_seconds) as usize;
                    if history.len() > max_entries {
                        history.remove(0);
                    }
                }

                // Check alert rules
                if let Err(e) = Self::check_alerts(&config.alert_rules, &alerts, &swarm_coordinator).await {
                    eprintln!("Error checking alerts: {}", e);
                }
            }
        });

        // Start Prometheus exporter if enabled
        if self.config.enable_prometheus {
            self.start_prometheus_exporter().await?;
        }

        Ok(())
    }

    /// Collect comprehensive system metrics
    async fn collect_metrics(
        swarm_coordinator: &SwarmCoordinator,
        communication_manager: &CommunicationManager,
    ) -> Result<SystemMetrics, Box<dyn std::error::Error + Send + Sync>> {
        let timestamp = Utc::now();

        // System resource metrics
        let (cpu_usage, memory_usage, disk_usage) = Self::collect_system_resources().await?;

        // Network metrics
        let network_io = Self::collect_network_metrics(communication_manager).await?;

        // Agent metrics
        let agent_metrics = Self::collect_agent_metrics(swarm_coordinator).await?;

        // Swarm metrics
        let swarm_metrics = Self::collect_swarm_metrics(swarm_coordinator).await?;

        // Performance metrics
        let performance_metrics = Self::calculate_performance_metrics(&swarm_metrics).await?;

        Ok(SystemMetrics {
            timestamp,
            cpu_usage,
            memory_usage,
            disk_usage,
            network_io,
            agent_metrics,
            swarm_metrics,
            performance_metrics,
        })
    }

    /// Collect system resource usage
    async fn collect_system_resources() -> Result<(f64, f64, f64), Box<dyn std::error::Error + Send + Sync>> {
        // CPU usage (simplified - in production use a proper system monitoring library)
        let cpu_usage = 45.5; // Placeholder

        // Memory usage
        let memory_usage = 67.8; // Placeholder

        // Disk usage
        let disk_usage = 23.4; // Placeholder

        Ok((cpu_usage, memory_usage, disk_usage))
    }

    /// Collect network I/O metrics
    async fn collect_network_metrics(
        communication_manager: &CommunicationManager,
    ) -> Result<NetworkMetrics, Box<dyn std::error::Error + Send + Sync>> {
        // Get connection statistics from communication manager
        let connections_active = communication_manager.active_connections().await;
        let connections_total = communication_manager.total_connections().await;

        // Calculate network throughput (simplified)
        let bytes_sent = 1024 * 1024; // 1MB placeholder
        let bytes_received = 2 * 1024 * 1024; // 2MB placeholder
        let latency_ms = 15.5; // 15.5ms placeholder

        Ok(NetworkMetrics {
            bytes_sent,
            bytes_received,
            connections_active,
            connections_total,
            latency_ms,
        })
    }

    /// Collect agent health and performance metrics
    async fn collect_agent_metrics(
        swarm_coordinator: &SwarmCoordinator,
    ) -> Result<AgentMetrics, Box<dyn std::error::Error + Send + Sync>> {
        let agents = swarm_coordinator.list_agents().await;

        let total_agents = agents.len();
        let mut active_agents = 0;
        let mut idle_agents = 0;
        let mut failed_agents = 0;
        let mut response_times = Vec::new();
        let mut health_scores = HashMap::new();

        for agent in agents {
            let agent_read = agent.read().await;

            match agent_read.status {
                AgentStatus::Active => active_agents += 1,
                AgentStatus::Idle => idle_agents += 1,
                AgentStatus::Failed => failed_agents += 1,
                _ => {}
            }

            // Calculate health score based on various factors
            let health_score = Self::calculate_agent_health_score(&agent_read).await;
            health_scores.insert(agent_read.id.to_string(), health_score);

            // Collect response time if available
            if let Some(last_response_time) = agent_read.last_response_time {
                response_times.push(last_response_time as f64);
            }
        }

        let average_response_time = if response_times.is_empty() {
            0.0
        } else {
            response_times.iter().sum::<f64>() / response_times.len() as f64
        };

        Ok(AgentMetrics {
            total_agents,
            active_agents,
            idle_agents,
            failed_agents,
            average_response_time,
            agent_health_scores: health_scores,
        })
    }

    /// Calculate agent health score (0.0 to 1.0)
    async fn calculate_agent_health_score(agent: &crate::agents::Agent) -> f64 {
        let mut score = 1.0;

        // Reduce score based on failures
        if agent.consecutive_failures > 0 {
            score -= (agent.consecutive_failures as f64) * 0.1;
        }

        // Reduce score if agent hasn't responded recently
        if let Some(last_seen) = agent.last_seen {
            let minutes_since_last_seen = (Utc::now() - last_seen).num_minutes();
            if minutes_since_last_seen > 5 {
                score -= 0.2;
            }
        }

        // Reduce score based on error rate
        let total_operations = agent.successful_operations + agent.failed_operations;
        if total_operations > 0 {
            let error_rate = agent.failed_operations as f64 / total_operations as f64;
            score -= error_rate * 0.5;
        }

        score.max(0.0).min(1.0)
    }

    /// Collect swarm performance metrics
    async fn collect_swarm_metrics(
        swarm_coordinator: &HiveCoordinator,
    ) -> Result<SwarmMetrics, Box<dyn std::error::Error + Send + Sync>> {
        let tasks = swarm_coordinator.list_tasks().await;
        let agents = swarm_coordinator.list_agents().await;

        let total_tasks = tasks.len() as u64;
        let completed_tasks = tasks.iter().filter(|t| t.status == TaskStatus::Completed).count() as u64;
        let failed_tasks = tasks.iter().filter(|t| t.status == TaskStatus::Failed).count() as u64;
        let pending_tasks = tasks.iter().filter(|t| t.status == TaskStatus::Pending).count();

        // Calculate average task duration
        let completed_task_durations: Vec<f64> = tasks
            .iter()
            .filter(|t| t.status == crate::swarm::TaskStatus::Completed)
            .filter_map(|t| {
                if let (Some(started), Some(completed)) = (t.started_at, t.completed_at) {
                    Some((completed - started).num_milliseconds() as f64)
                } else {
                    None
                }
            })
            .collect();

        let average_task_duration = if completed_task_durations.is_empty() {
            0.0
        } else {
            completed_task_durations.iter().sum::<f64>() / completed_task_durations.len() as f64
        };

        let task_success_rate = if total_tasks > 0 {
            completed_tasks as f64 / total_tasks as f64
        } else {
            1.0
        };

        // Calculate load distribution
        let mut load_distribution = HashMap::new();
        for agent in &agents {
            let agent_read = agent.read().await;
            load_distribution.insert(
                agent_read.id.to_string(),
                agent_read.active_tasks.len(),
            );
        }

        Ok(SwarmMetrics {
            total_tasks,
            completed_tasks,
            failed_tasks,
            pending_tasks,
            average_task_duration,
            task_success_rate,
            load_distribution,
        })
    }

    /// Calculate performance metrics
    async fn calculate_performance_metrics(
        swarm_metrics: &SwarmMetrics,
    ) -> Result<PerformanceMetrics, Box<dyn std::error::Error + Send + Sync>> {
        // Calculate throughput (tasks per second over last hour)
        let throughput = swarm_metrics.completed_tasks as f64 / 3600.0;

        // Calculate latency percentiles (simplified)
        let latency_p50 = swarm_metrics.average_task_duration;
        let latency_p95 = swarm_metrics.average_task_duration * 1.5;
        let latency_p99 = swarm_metrics.average_task_duration * 2.0;

        // Calculate error rate
        let total_tasks = swarm_metrics.total_tasks as f64;
        let error_rate = if total_tasks > 0.0 {
            swarm_metrics.failed_tasks as f64 / total_tasks
        } else {
            0.0
        };

        // Calculate resource utilization (simplified)
        let resource_utilization = 0.65; // Placeholder

        Ok(PerformanceMetrics {
            throughput,
            latency_p50,
            latency_p95,
            latency_p99,
            error_rate,
            resource_utilization,
        })
    }

    /// Check alert rules and generate alerts
    async fn check_alerts(
        alert_rules: &[AlertRule],
        alerts: &Arc<RwLock<Vec<Alert>>>,
        swarm_coordinator: &SwarmCoordinator,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for rule in alert_rules {
            if !rule.enabled {
                continue;
            }

            // Check cooldown
            if let Some(last_triggered) = rule.last_triggered {
                let minutes_since_triggered = (Utc::now() - last_triggered).num_minutes();
                if minutes_since_triggered < rule.cooldown_minutes as i64 {
                    continue;
                }
            }

            // Evaluate condition (simplified - in production use a proper expression evaluator)
            if Self::evaluate_alert_condition(&rule.condition, swarm_coordinator).await {
                let alert = Alert {
                    id: uuid::Uuid::new_v4().to_string(),
                    title: format!("Alert: {}", rule.name),
                    description: format!("Alert condition met: {}", rule.condition),
                    severity: rule.severity.clone(),
                    source: "monitoring_system".to_string(),
                    timestamp: Utc::now(),
                    acknowledged: false,
                    resolved: false,
                    metadata: HashMap::new(),
                };

                let mut alerts_write = alerts.write().await;
                alerts_write.push(alert);
            }
        }

        Ok(())
    }

    /// Evaluate alert condition (simplified implementation)
    async fn evaluate_alert_condition(
        condition: &str,
        swarm_coordinator: &SwarmCoordinator,
    ) -> bool {
        match condition {
            "high_error_rate" => {
                let metrics = Self::collect_swarm_metrics(swarm_coordinator).await.unwrap_or_default();
                metrics.task_success_rate < 0.95
            }
            "agent_failures" => {
                let metrics = Self::collect_agent_metrics(swarm_coordinator).await.unwrap_or_default();
                metrics.failed_agents > 0
            }
            "high_latency" => {
                let metrics = Self::collect_swarm_metrics(swarm_coordinator).await.unwrap_or_default();
                metrics.average_task_duration > 5000.0 // 5 seconds
            }
            _ => false,
        }
    }

    /// Start Prometheus metrics exporter
    async fn start_prometheus_exporter(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, this would set up a Prometheus HTTP server
        // For now, just log that it would start
        println!("Starting Prometheus exporter on port {}", self.config.prometheus_port);
        Ok(())
    }

    /// Get current metrics
    pub async fn get_current_metrics(&self) -> Option<SystemMetrics> {
        let history = self.metrics_history.read().await;
        history.last().cloned()
    }

    /// Get metrics history
    pub async fn get_metrics_history(&self, hours: u32) -> Vec<SystemMetrics> {
        let history = self.metrics_history.read().await;
        let cutoff = Utc::now() - chrono::Duration::hours(hours as i64);

        history
            .iter()
            .filter(|m| m.timestamp > cutoff)
            .cloned()
            .collect()
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts
            .iter()
            .filter(|a| !a.resolved)
            .cloned()
            .collect()
    }

    /// Acknowledge alert
    pub async fn acknowledge_alert(&self, alert_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            Ok(())
        } else {
            Err("Alert not found".into())
        }
    }

    /// Get system uptime
    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Generate health report
    pub async fn generate_health_report(&self) -> HashMap<String, serde_json::Value> {
        let mut report = HashMap::new();

        if let Some(metrics) = self.get_current_metrics().await {
            report.insert("status".to_string(), serde_json::json!("healthy"));
            report.insert("cpu_usage".to_string(), serde_json::json!(metrics.cpu_usage));
            report.insert("memory_usage".to_string(), serde_json::json!(metrics.memory_usage));
            report.insert("active_agents".to_string(), serde_json::json!(metrics.agent_metrics.active_agents));
            report.insert("task_success_rate".to_string(), serde_json::json!(metrics.swarm_metrics.task_success_rate));
        } else {
            report.insert("status".to_string(), serde_json::json!("unknown"));
        }

        report.insert("uptime_seconds".to_string(), serde_json::json!(self.get_uptime().as_secs()));
        report
    }
}

/// Alert manager for handling notifications
pub struct AlertManager {
    alerts: Arc<RwLock<Vec<Alert>>>,
    notification_channels: Vec<Box<dyn NotificationChannel + Send + Sync>>,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            alerts: Arc::new(RwLock::new(Vec::new())),
            notification_channels: Vec::new(),
        }
    }

    pub fn add_channel(&mut self, channel: Box<dyn NotificationChannel + Send + Sync>) {
        self.notification_channels.push(channel);
    }

    pub async fn process_alert(&self, alert: &Alert) {
        let mut alerts = self.alerts.write().await;
        alerts.push(alert.clone());

        // Send notifications
        for channel in &self.notification_channels {
            if let Err(e) = channel.send_notification(alert).await {
                eprintln!("Failed to send notification: {}", e);
            }
        }
    }

    pub async fn get_alerts(&self, include_resolved: bool) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        if include_resolved {
            alerts.clone()
        } else {
            alerts.iter().filter(|a| !a.resolved).cloned().collect()
        }
    }
}

/// Notification channel trait
#[async_trait::async_trait]
pub trait NotificationChannel {
    async fn send_notification(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Slack notification channel
pub struct SlackChannel {
    webhook_url: String,
}

impl SlackChannel {
    pub fn new(webhook_url: String) -> Self {
        Self { webhook_url }
    }
}

#[async_trait::async_trait]
impl NotificationChannel for SlackChannel {
    async fn send_notification(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::json!({
            "text": format!("🚨 {}: {}", alert.title, alert.description),
            "attachments": [{
                "color": match alert.severity {
                    AlertSeverity::Low => "good",
                    AlertSeverity::Medium => "warning",
                    AlertSeverity::High => "danger",
                    AlertSeverity::Critical => "danger",
                },
                "fields": [
                    {
                        "title": "Severity",
                        "value": format!("{:?}", alert.severity),
                        "short": true
                    },
                    {
                        "title": "Source",
                        "value": alert.source,
                        "short": true
                    }
                ]
            }]
        });

        let client = reqwest::Client::new();
        let response = client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Slack notification failed: {}", response.status()).into())
        }
    }
}

/// Email notification channel
pub struct EmailChannel {
    smtp_server: String,
    smtp_port: u16,
    username: String,
    password: String,
    from_address: String,
    to_addresses: Vec<String>,
}

impl EmailChannel {
    pub fn new(
        smtp_server: String,
        smtp_port: u16,
        username: String,
        password: String,
        from_address: String,
        to_addresses: Vec<String>,
    ) -> Self {
        Self {
            smtp_server,
            smtp_port,
            username,
            password,
            from_address,
            to_addresses,
        }
    }
}

#[async_trait::async_trait]
impl NotificationChannel for EmailChannel {
    async fn send_notification(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, this would use an SMTP library like lettre
        println!("Sending email notification for alert: {}", alert.title);
        Ok(())
    }
}