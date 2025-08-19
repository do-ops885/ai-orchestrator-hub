use crate::infrastructure::metrics::{
    MetricsCollector, Anomaly, AnomalySeverity, PredictiveInsights,
};
use crate::infrastructure::metrics::{Alert, AlertLevel};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Intelligent alerting system with adaptive thresholds and predictive capabilities
pub struct IntelligentAlertingSystem {
    alert_rules: Arc<RwLock<Vec<AlertRule>>>,
    alert_history: Arc<RwLock<Vec<Alert>>>,
    adaptive_thresholds: Arc<RwLock<AdaptiveThresholds>>,
    notification_channels: Arc<RwLock<Vec<NotificationChannel>>>,
    alert_suppression: Arc<RwLock<AlertSuppression>>,
    metrics_collector: Arc<MetricsCollector>,
    config: IntelligentAlertConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentAlertConfig {
    pub max_alert_history: usize,
    pub adaptive_learning_enabled: bool,
    pub predictive_alerting_enabled: bool,
    pub alert_correlation_window_minutes: u32,
    pub suppression_window_minutes: u32,
    pub escalation_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub metric_name: String,
    pub condition: AlertCondition,
    pub threshold: f64,
    pub severity: AlertLevel,
    pub enabled: bool,
    pub adaptive: bool,
    pub created_at: DateTime<Utc>,
    pub last_triggered: Option<DateTime<Utc>>,
    pub trigger_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equals,
    NotEquals,
    RateOfChange,
    Anomaly,
    Predictive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveThresholds {
    pub thresholds: HashMap<String, AdaptiveThreshold>,
    pub learning_rate: f64,
    pub adaptation_window_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveThreshold {
    pub metric_name: String,
    pub current_threshold: f64,
    pub baseline_threshold: f64,
    pub confidence: f64,
    pub last_updated: DateTime<Utc>,
    pub adaptation_history: Vec<ThresholdAdjustment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdAdjustment {
    pub timestamp: DateTime<Utc>,
    pub old_threshold: f64,
    pub new_threshold: f64,
    pub reason: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub id: Uuid,
    pub name: String,
    pub channel_type: ChannelType,
    pub config: ChannelConfig,
    pub enabled: bool,
    pub severity_filter: Vec<AlertLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelType {
    Email,
    Webhook,
    Slack,
    Discord,
    Console,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub endpoint: Option<String>,
    pub headers: HashMap<String, String>,
    pub template: Option<String>,
    pub rate_limit_per_hour: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSuppression {
    pub suppressed_alerts: HashMap<String, DateTime<Utc>>,
    pub correlation_groups: HashMap<String, Vec<Uuid>>,
    pub escalation_chains: HashMap<Uuid, EscalationChain>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationChain {
    pub alert_id: Uuid,
    pub levels: Vec<EscalationLevel>,
    pub current_level: usize,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationLevel {
    pub delay_minutes: u32,
    pub channels: Vec<Uuid>,
    pub severity_upgrade: Option<AlertLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentAlert {
    pub base_alert: Alert,
    pub correlation_id: Option<Uuid>,
    pub predicted: bool,
    pub confidence: f64,
    pub related_anomalies: Vec<Uuid>,
    pub suggested_actions: Vec<String>,
    pub auto_resolution_possible: bool,
}

impl IntelligentAlertingSystem {
    pub fn new(
        metrics_collector: Arc<MetricsCollector>,
        config: IntelligentAlertConfig,
    ) -> Self {
        Self {
            alert_rules: Arc::new(RwLock::new(Vec::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            adaptive_thresholds: Arc::new(RwLock::new(AdaptiveThresholds::new())),
            notification_channels: Arc::new(RwLock::new(Vec::new())),
            alert_suppression: Arc::new(RwLock::new(AlertSuppression::new())),
            metrics_collector,
            config,
        }
    }

    /// Initialize with default alert rules
    pub async fn initialize_default_rules(&self) {
        let mut rules = self.alert_rules.write().await;

        // CPU usage alert
        rules.push(AlertRule {
            id: Uuid::new_v4(),
            name: "High CPU Usage".to_string(),
            description: "CPU usage exceeds threshold".to_string(),
            metric_name: "cpu_usage".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 80.0,
            severity: AlertLevel::Warning,
            enabled: true,
            adaptive: true,
            created_at: Utc::now(),
            last_triggered: None,
            trigger_count: 0,
        });

        // Memory usage alert
        rules.push(AlertRule {
            id: Uuid::new_v4(),
            name: "High Memory Usage".to_string(),
            description: "Memory usage exceeds threshold".to_string(),
            metric_name: "memory_usage".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 85.0,
            severity: AlertLevel::Warning,
            enabled: true,
            adaptive: true,
            created_at: Utc::now(),
            last_triggered: None,
            trigger_count: 0,
        });

        // System health alert
        rules.push(AlertRule {
            id: Uuid::new_v4(),
            name: "System Health Degradation".to_string(),
            description: "Overall system health score is low".to_string(),
            metric_name: "system_health_score".to_string(),
            condition: AlertCondition::LessThan,
            threshold: 0.7,
            severity: AlertLevel::Critical,
            enabled: true,
            adaptive: true,
            created_at: Utc::now(),
            last_triggered: None,
            trigger_count: 0,
        });

        // Anomaly detection alert
        rules.push(AlertRule {
            id: Uuid::new_v4(),
            name: "Performance Anomaly Detected".to_string(),
            description: "Unusual system behavior detected".to_string(),
            metric_name: "anomaly_detection".to_string(),
            condition: AlertCondition::Anomaly,
            threshold: 0.8, // Confidence threshold
            severity: AlertLevel::Warning,
            enabled: true,
            adaptive: false,
            created_at: Utc::now(),
            last_triggered: None,
            trigger_count: 0,
        });

        info!("Initialized {} default alert rules", rules.len());
    }

    /// Add a notification channel
    pub async fn add_notification_channel(&self, channel: NotificationChannel) {
        let mut channels = self.notification_channels.write().await;
        channels.push(channel);
        info!("Added notification channel");
    }

    /// Process alerts with intelligent analysis
    pub async fn process_intelligent_alerts(&self) -> anyhow::Result<Vec<IntelligentAlert>> {
        let mut intelligent_alerts = Vec::new();

        // Get current metrics and insights
        let insights = self.metrics_collector.get_predictive_insights().await;
        let current_metrics = self.metrics_collector.collect_system_metrics().await?;

        // Process traditional threshold-based alerts
        let threshold_alerts = self.check_threshold_alerts(&current_metrics).await?;

        // Process anomaly-based alerts
        let anomaly_alerts = self.check_anomaly_alerts(&insights.anomalies).await?;

        // Process predictive alerts
        let predictive_alerts = if self.config.predictive_alerting_enabled {
            self.check_predictive_alerts(&insights).await?
        } else {
            Vec::new()
        };

        // Combine and correlate alerts
        intelligent_alerts.extend(threshold_alerts);
        intelligent_alerts.extend(anomaly_alerts);
        intelligent_alerts.extend(predictive_alerts);

        // Apply correlation and suppression
        let correlated_alerts = self.correlate_alerts(intelligent_alerts).await?;
        let final_alerts = self.apply_suppression(correlated_alerts).await?;

        // Update adaptive thresholds if enabled
        if self.config.adaptive_learning_enabled {
            self.update_adaptive_thresholds(&current_metrics, &final_alerts)
                .await?;
        }

        // Send notifications
        for alert in &final_alerts {
            self.send_notifications(alert).await?;
        }

        // Store in history
        self.store_alert_history(&final_alerts).await?;

        Ok(final_alerts)
    }

    async fn check_threshold_alerts(
        &self,
        metrics: &crate::infrastructure::metrics::SystemMetrics,
    ) -> anyhow::Result<Vec<IntelligentAlert>> {
        let mut alerts = Vec::new();
        let rules = self.alert_rules.read().await;
        let adaptive_thresholds = self.adaptive_thresholds.read().await;

        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }

            let metric_value = self.get_metric_value(metrics, &rule.metric_name);
            let threshold = if rule.adaptive {
                adaptive_thresholds
                    .thresholds
                    .get(&rule.metric_name)
                    .map(|t| t.current_threshold)
                    .unwrap_or(rule.threshold)
            } else {
                rule.threshold
            };

            let triggered = match rule.condition {
                AlertCondition::GreaterThan => metric_value > threshold,
                AlertCondition::LessThan => metric_value < threshold,
                AlertCondition::Equals => (metric_value - threshold).abs() < 0.001,
                AlertCondition::NotEquals => (metric_value - threshold).abs() >= 0.001,
                _ => false,
            };

            if triggered {
                let base_alert = Alert::new(
                    rule.severity.clone(),
                    rule.name.clone(),
                    format!(
                        "{}: {} (threshold: {})",
                        rule.description, metric_value, threshold
                    ),
                );

                let intelligent_alert = IntelligentAlert {
                    base_alert,
                    correlation_id: None,
                    predicted: false,
                    confidence: 1.0,
                    related_anomalies: Vec::new(),
                    suggested_actions: self.generate_suggested_actions(
                        &rule.metric_name,
                        metric_value,
                        threshold,
                    ),
                    auto_resolution_possible: self.can_auto_resolve(&rule.metric_name),
                };

                alerts.push(intelligent_alert);
            }
        }

        Ok(alerts)
    }

    async fn check_anomaly_alerts(
        &self,
        anomalies: &[Anomaly],
    ) -> anyhow::Result<Vec<IntelligentAlert>> {
        let mut alerts = Vec::new();

        for anomaly in anomalies {
            let severity = match anomaly.severity {
                AnomalySeverity::Low => AlertLevel::Info,
                AnomalySeverity::Medium => AlertLevel::Warning,
                AnomalySeverity::High | AnomalySeverity::Critical => AlertLevel::Critical,
            };

            let base_alert = Alert::new(
                severity,
                "Anomaly Detected".to_string(),
                anomaly.description.clone(),
            );

            let intelligent_alert = IntelligentAlert {
                base_alert,
                correlation_id: None,
                predicted: false,
                confidence: 0.8,
                related_anomalies: vec![Uuid::new_v4()], // Would use actual anomaly IDs
                suggested_actions: self.generate_anomaly_actions(anomaly),
                auto_resolution_possible: false,
            };

            alerts.push(intelligent_alert);
        }

        Ok(alerts)
    }

    async fn check_predictive_alerts(
        &self,
        insights: &PredictiveInsights,
    ) -> anyhow::Result<Vec<IntelligentAlert>> {
        let mut alerts = Vec::new();

        // Check if predicted health score indicates future issues
        if insights.forecast.predicted_health_score_1h < 0.6 {
            let base_alert = Alert::new(
                AlertLevel::Warning,
                "Predicted Performance Degradation".to_string(),
                format!(
                    "System health predicted to drop to {:.2} within 1 hour",
                    insights.forecast.predicted_health_score_1h
                ),
            );

            let intelligent_alert = IntelligentAlert {
                base_alert,
                correlation_id: None,
                predicted: true,
                confidence: insights.forecast.confidence,
                related_anomalies: Vec::new(),
                suggested_actions: insights.recommendations.clone(),
                auto_resolution_possible: true,
            };

            alerts.push(intelligent_alert);
        }

        Ok(alerts)
    }

    async fn correlate_alerts(
        &self,
        alerts: Vec<IntelligentAlert>,
    ) -> anyhow::Result<Vec<IntelligentAlert>> {
        // Simple correlation based on timing and metric relationships
        let mut correlated_alerts = alerts;
        let correlation_window =
            Duration::minutes(self.config.alert_correlation_window_minutes as i64);

        // Group alerts that occurred within the correlation window
        let now = Utc::now();
        for alert in &mut correlated_alerts {
            if (now - alert.base_alert.timestamp) <= correlation_window {
                // Assign correlation ID to related alerts
                alert.correlation_id = Some(Uuid::new_v4());
            }
        }

        Ok(correlated_alerts)
    }

    async fn apply_suppression(
        &self,
        alerts: Vec<IntelligentAlert>,
    ) -> anyhow::Result<Vec<IntelligentAlert>> {
        let mut suppression = self.alert_suppression.write().await;
        let mut final_alerts = Vec::new();
        let suppression_window = Duration::minutes(self.config.suppression_window_minutes as i64);
        let now = Utc::now();

        for alert in alerts {
            let alert_key = format!(
                "{}_{}",
                alert.base_alert.title,
                match alert.base_alert.level {
                    AlertLevel::Info => 0,
                    AlertLevel::Warning => 1,
                    AlertLevel::Critical => 2,
                }
            );

            // Check if this alert type is currently suppressed
            if let Some(last_sent) = suppression.suppressed_alerts.get(&alert_key) {
                if (now - *last_sent) < suppression_window {
                    debug!("Alert suppressed: {}", alert.base_alert.title);
                    continue;
                }
            }

            // Add to final alerts and update suppression
            suppression.suppressed_alerts.insert(alert_key, now);
            final_alerts.push(alert);
        }

        Ok(final_alerts)
    }

    async fn update_adaptive_thresholds(
        &self,
        _metrics: &crate::infrastructure::metrics::SystemMetrics,
        alerts: &[IntelligentAlert],
    ) -> anyhow::Result<()> {
        let mut adaptive_thresholds = self.adaptive_thresholds.write().await;

        // Update thresholds based on alert frequency and system behavior
        for alert in alerts {
            if let Some(metric_name) = self.extract_metric_name(&alert.base_alert.title) {
                if let Some(threshold) = adaptive_thresholds.thresholds.get_mut(&metric_name) {
                    // Adjust threshold based on alert frequency
                    let adjustment_factor = if alert.base_alert.level == AlertLevel::Critical {
                        0.95 // Make threshold more sensitive for critical alerts
                    } else {
                        0.98 // Slight adjustment for other alerts
                    };

                    let new_threshold = threshold.current_threshold * adjustment_factor;

                    threshold.adaptation_history.push(ThresholdAdjustment {
                        timestamp: Utc::now(),
                        old_threshold: threshold.current_threshold,
                        new_threshold,
                        reason: "Alert frequency adjustment".to_string(),
                        confidence: 0.7,
                    });

                    threshold.current_threshold = new_threshold;
                    threshold.last_updated = Utc::now();

                    debug!(
                        "Adjusted threshold for {}: {} -> {}",
                        metric_name, threshold.current_threshold, new_threshold
                    );
                }
            }
        }

        Ok(())
    }

    async fn send_notifications(&self, alert: &IntelligentAlert) -> anyhow::Result<()> {
        let channels = self.notification_channels.read().await;

        for channel in channels.iter() {
            if !channel.enabled {
                continue;
            }

            // Check if channel accepts this alert severity
            if !channel.severity_filter.is_empty()
                && !channel.severity_filter.contains(&alert.base_alert.level)
            {
                continue;
            }

            match channel.channel_type {
                ChannelType::Console => match alert.base_alert.level {
                    AlertLevel::Critical => {
                        error!("🚨 CRITICAL ALERT: {}", alert.base_alert.description)
                    }
                    AlertLevel::Warning => warn!("⚠️  WARNING: {}", alert.base_alert.description),
                    AlertLevel::Info => info!("ℹ️  INFO: {}", alert.base_alert.description),
                },
                ChannelType::Webhook => {
                    if let Some(endpoint) = &channel.config.endpoint {
                        self.send_webhook_notification(endpoint, alert, &channel.config)
                            .await?;
                    }
                }
                _ => {
                    debug!(
                        "Notification channel type {:?} not yet implemented",
                        channel.channel_type
                    );
                }
            }
        }

        Ok(())
    }

    async fn send_webhook_notification(
        &self,
        endpoint: &str,
        alert: &IntelligentAlert,
        _config: &ChannelConfig,
    ) -> anyhow::Result<()> {
        let payload = serde_json::json!({
            "alert": {
                "title": alert.base_alert.title,
                "description": alert.base_alert.description,
                "level": alert.base_alert.level,
                "timestamp": alert.base_alert.timestamp,
                "predicted": alert.predicted,
                "confidence": alert.confidence,
                "suggested_actions": alert.suggested_actions
            }
        });

        // In a real implementation, you would use an HTTP client like reqwest
        info!("Would send webhook to {}: {}", endpoint, payload);
        Ok(())
    }

    async fn store_alert_history(&self, alerts: &[IntelligentAlert]) -> anyhow::Result<()> {
        let mut history = self.alert_history.write().await;

        for alert in alerts {
            history.push(alert.base_alert.clone());
        }

        // Maintain max history size
        if history.len() > self.config.max_alert_history {
            let excess = history.len() - self.config.max_alert_history;
            history.drain(0..excess);
        }

        Ok(())
    }

    // Helper methods

    fn get_metric_value(
        &self,
        metrics: &crate::infrastructure::metrics::SystemMetrics,
        metric_name: &str,
    ) -> f64 {
        match metric_name {
            "cpu_usage" => metrics.resource_usage.cpu_usage_percent,
            "memory_usage" => metrics.resource_usage.memory_usage_percent,
            "system_health_score" => 0.8, // Placeholder health score
            "agent_count" => metrics.agent_metrics.total_agents as f64,
            "active_tasks" => metrics.task_metrics.tasks_in_queue as f64,
            _ => 0.0,
        }
    }

    fn generate_suggested_actions(
        &self,
        metric_name: &str,
        _value: f64,
        _threshold: f64,
    ) -> Vec<String> {
        match metric_name {
            "cpu_usage" => vec![
                "Consider scaling up CPU resources".to_string(),
                "Optimize agent workloads".to_string(),
                "Check for runaway processes".to_string(),
            ],
            "memory_usage" => vec![
                "Increase available memory".to_string(),
                "Trigger garbage collection".to_string(),
                "Optimize memory usage patterns".to_string(),
            ],
            "system_health_score" => vec![
                "Investigate system components".to_string(),
                "Check resource utilization".to_string(),
                "Review recent changes".to_string(),
            ],
            _ => vec!["Monitor the situation".to_string()],
        }
    }

    fn generate_anomaly_actions(&self, anomaly: &Anomaly) -> Vec<String> {
        match anomaly.anomaly_type {
            crate::infrastructure::metrics::AnomalyType::PerformanceSpike => vec![
                "Investigate sudden performance increase".to_string(),
                "Check for unusual workload patterns".to_string(),
            ],
            crate::infrastructure::metrics::AnomalyType::PerformanceDrop => vec![
                "Investigate performance degradation".to_string(),
                "Check system resources".to_string(),
                "Review recent configuration changes".to_string(),
            ],
            _ => vec!["Investigate anomalous behavior".to_string()],
        }
    }

    fn can_auto_resolve(&self, metric_name: &str) -> bool {
        match metric_name {
            "memory_usage" => true, // Can trigger garbage collection
            "agent_count" => true,  // Can restart failed agents
            _ => false,
        }
    }

    fn extract_metric_name(&self, alert_title: &str) -> Option<String> {
        if alert_title.contains("CPU") {
            Some("cpu_usage".to_string())
        } else if alert_title.contains("Memory") {
            Some("memory_usage".to_string())
        } else if alert_title.contains("Health") {
            Some("system_health_score".to_string())
        } else {
            None
        }
    }

    /// Get alert statistics
    pub async fn get_alert_statistics(&self) -> AlertStatistics {
        let history = self.alert_history.read().await;
        let now = Utc::now();
        let last_24h = now - Duration::hours(24);
        let last_hour = now - Duration::hours(1);

        let alerts_24h = history.iter().filter(|a| a.timestamp >= last_24h).count();

        let alerts_1h = history.iter().filter(|a| a.timestamp >= last_hour).count();

        let critical_alerts_24h = history
            .iter()
            .filter(|a| a.timestamp >= last_24h && a.level == AlertLevel::Critical)
            .count();

        AlertStatistics {
            total_alerts: history.len(),
            alerts_last_24h: alerts_24h,
            alerts_last_hour: alerts_1h,
            critical_alerts_24h,
            most_frequent_alert: self.get_most_frequent_alert(&history),
        }
    }

    fn get_most_frequent_alert(&self, history: &[Alert]) -> Option<String> {
        let mut alert_counts: HashMap<String, usize> = HashMap::new();

        for alert in history {
            *alert_counts.entry(alert.title.clone()).or_insert(0) += 1;
        }

        alert_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(title, _)| title)
    }
}

impl AdaptiveThresholds {
    pub fn new() -> Self {
        Self {
            thresholds: HashMap::new(),
            learning_rate: 0.1,
            adaptation_window_hours: 24,
        }
    }
}

impl AlertSuppression {
    pub fn new() -> Self {
        Self {
            suppressed_alerts: HashMap::new(),
            correlation_groups: HashMap::new(),
            escalation_chains: HashMap::new(),
        }
    }
}

impl Default for IntelligentAlertConfig {
    fn default() -> Self {
        Self {
            max_alert_history: 10000,
            adaptive_learning_enabled: true,
            predictive_alerting_enabled: true,
            alert_correlation_window_minutes: 5,
            suppression_window_minutes: 30,
            escalation_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStatistics {
    pub total_alerts: usize,
    pub alerts_last_24h: usize,
    pub alerts_last_hour: usize,
    pub critical_alerts_24h: usize,
    pub most_frequent_alert: Option<String>,
}
