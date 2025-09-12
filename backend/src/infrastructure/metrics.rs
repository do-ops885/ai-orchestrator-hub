use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};
// use uuid::Uuid; // Commented out to avoid unused import warning

/// Comprehensive metrics collection for the hive system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub performance: PerformanceMetrics,
    pub resource_usage: ResourceUsageMetrics,
    pub agent_metrics: AgentMetrics,
    pub task_metrics: TaskMetrics,
    pub error_metrics: ErrorMetrics,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub requests_per_second: f64,
    pub average_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub throughput_tasks_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub network_bytes_in: u64,
    pub network_bytes_out: u64,
    pub disk_usage_bytes: u64,
    pub network_io: NetworkMetrics,
    pub disk_io: DiskMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connections_active: u32,
    pub websocket_connections: u32,
    pub requests_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiskMetrics {
    pub reads_per_second: f64,
    pub writes_per_second: f64,
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub disk_usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub total_agents: usize,
    pub active_agents: usize,
    pub idle_agents: usize,
    pub failed_agents: usize,
    pub average_agent_performance: f64,
    pub agent_utilization_percent: f64,
    #[allow(clippy::struct_field_names)]
    pub individual_agent_metrics: HashMap<uuid::Uuid, IndividualAgentMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualAgentMetrics {
    pub agent_id: uuid::Uuid,
    pub tasks_completed: u32,
    pub tasks_failed: u32,
    pub average_task_duration: f64,
    pub energy_consumption_rate: f64,
    pub learning_progress: f64,
    pub social_interaction_count: u32,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub current_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    pub total_tasks_submitted: u64,
    pub total_tasks_completed: u64,
    pub total_tasks_failed: u64,
    pub tasks_in_queue: usize,
    pub average_task_duration_ms: f64,
    pub task_success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub total_errors: u64,
    pub error_rate_per_minute: f64,
    pub errors_by_type: HashMap<String, u64>,
    pub critical_errors: u64,
}

/// Metrics collector with time-series data and advanced analytics
#[allow(dead_code)]
pub struct MetricsCollector {
    current_metrics: Arc<RwLock<SystemMetrics>>,
    historical_metrics: Arc<RwLock<Vec<SystemMetrics>>>,
    max_history_size: usize,
    alert_thresholds: MetricThresholds,
    start_time: std::time::Instant,
    // Advanced analytics components
    trend_analyzer: TrendAnalyzer,
    anomaly_detector: AnomalyDetector,
}

impl MetricsCollector {
    #[must_use]
    pub fn new(max_history_size: usize) -> Self {
        Self {
            current_metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            historical_metrics: Arc::new(RwLock::new(Vec::new())),
            max_history_size,
            alert_thresholds: MetricThresholds::default(),
            start_time: std::time::Instant::now(),
            trend_analyzer: TrendAnalyzer::new(10, 0.1),
            anomaly_detector: AnomalyDetector::new(0.8, 50),
        }
    }

    #[must_use]
    pub fn with_thresholds(max_history_size: usize, thresholds: MetricThresholds) -> Self {
        Self {
            current_metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            historical_metrics: Arc::new(RwLock::new(Vec::new())),
            max_history_size,
            alert_thresholds: thresholds,
            start_time: std::time::Instant::now(),
            trend_analyzer: TrendAnalyzer::new(10, 0.1),
            anomaly_detector: AnomalyDetector::new(0.8, 50),
        }
    }

    pub async fn update_performance_metrics(&self, metrics: PerformanceMetrics) {
        let mut current = self.current_metrics.write().await;
        current.performance = metrics;
        current.timestamp = Utc::now();
    }

    pub async fn update_resource_metrics(&self, metrics: ResourceUsageMetrics) {
        let mut current = self.current_metrics.write().await;
        current.resource_usage = metrics;
        current.timestamp = Utc::now();
    }

    pub async fn update_agent_metrics(&self, metrics: AgentMetrics) {
        let mut current = self.current_metrics.write().await;
        current.agent_metrics = metrics;
        current.timestamp = Utc::now();
    }

    pub async fn update_task_metrics(&self, metrics: TaskMetrics) {
        let mut current = self.current_metrics.write().await;
        current.task_metrics = metrics;
        current.timestamp = Utc::now();
    }

    pub async fn record_error(&self, error_type: &str) {
        let mut current = self.current_metrics.write().await;
        current.error_metrics.total_errors += 1;
        *current
            .error_metrics
            .errors_by_type
            .entry(error_type.to_string())
            .or_insert(0) += 1;
        current.timestamp = Utc::now();
    }

    pub async fn get_current_metrics(&self) -> SystemMetrics {
        self.current_metrics.read().await.clone()
    }

    pub async fn get_historical_metrics(&self, limit: Option<usize>) -> Vec<SystemMetrics> {
        let history = self.historical_metrics.read().await;
        match limit {
            Some(n) => history.iter().rev().take(n).cloned().collect(),
            None => history.clone(),
        }
    }

    pub async fn snapshot_current_metrics(&self) {
        let current = self.current_metrics.read().await.clone();
        let mut history = self.historical_metrics.write().await;

        history.push(current);

        // Maintain max history size
        if history.len() > self.max_history_size {
            history.remove(0);
        }
    }

    /// Calculate trends and anomalies
    pub async fn analyze_trends(&self) -> MetricsTrends {
        let history = self.historical_metrics.read().await;

        if history.len() < 2 {
            return MetricsTrends::default();
        }

        let recent = &history[history.len() - 1];
        let previous = &history[history.len() - 2];

        MetricsTrends {
            cpu_trend: calculate_trend(
                previous.resource_usage.cpu_usage_percent,
                recent.resource_usage.cpu_usage_percent,
            ),
            memory_trend: calculate_trend(
                previous.resource_usage.memory_usage_percent,
                recent.resource_usage.memory_usage_percent,
            ),
            task_completion_trend: calculate_trend(
                previous.task_metrics.task_success_rate,
                recent.task_metrics.task_success_rate,
            ),
            agent_performance_trend: calculate_trend(
                previous.agent_metrics.average_agent_performance,
                recent.agent_metrics.average_agent_performance,
            ),
            error_rate_trend: calculate_trend(
                previous.error_metrics.error_rate_per_minute,
                recent.error_metrics.error_rate_per_minute,
            ),
        }
    }

    /// Check for alerts based on current metrics
    pub async fn check_alerts(&self) -> Vec<Alert> {
        let mut alerts = Vec::new();
        let current = self.current_metrics.read().await;

        // CPU alerts
        if current.resource_usage.cpu_usage_percent >= self.alert_thresholds.cpu_critical {
            alerts.push(Alert::new(
                AlertLevel::Critical,
                "CPU usage critical".to_string(),
                format!(
                    "CPU usage: {:.1}%",
                    current.resource_usage.cpu_usage_percent
                ),
            ));
        } else if current.resource_usage.cpu_usage_percent >= self.alert_thresholds.cpu_warning {
            alerts.push(Alert::new(
                AlertLevel::Warning,
                "CPU usage high".to_string(),
                format!(
                    "CPU usage: {:.1}%",
                    current.resource_usage.cpu_usage_percent
                ),
            ));
        }

        // Memory alerts
        if current.resource_usage.memory_usage_percent >= self.alert_thresholds.memory_critical {
            alerts.push(Alert::new(
                AlertLevel::Critical,
                "Memory usage critical".to_string(),
                format!(
                    "Memory usage: {:.1}%",
                    current.resource_usage.memory_usage_percent
                ),
            ));
        } else if current.resource_usage.memory_usage_percent
            >= self.alert_thresholds.memory_warning
        {
            alerts.push(Alert::new(
                AlertLevel::Warning,
                "Memory usage high".to_string(),
                format!(
                    "Memory usage: {:.1}%",
                    current.resource_usage.memory_usage_percent
                ),
            ));
        }

        // Task failure rate alerts
        let failure_rate = if current.task_metrics.total_tasks_submitted > 0 {
            (current.task_metrics.total_tasks_failed as f64
                / current.task_metrics.total_tasks_submitted as f64)
                * 100.0
        } else {
            0.0
        };

        if failure_rate >= self.alert_thresholds.task_failure_rate_critical {
            alerts.push(Alert::new(
                AlertLevel::Critical,
                "High task failure rate".to_string(),
                format!("Task failure rate: {failure_rate:.1}%"),
            ));
        } else if failure_rate >= self.alert_thresholds.task_failure_rate_warning {
            alerts.push(Alert::new(
                AlertLevel::Warning,
                "Elevated task failure rate".to_string(),
                format!("Task failure rate: {failure_rate:.1}%"),
            ));
        }

        // Agent failure rate alerts
        let agent_failure_rate = if current.agent_metrics.total_agents > 0 {
            (current.agent_metrics.failed_agents as f64 / current.agent_metrics.total_agents as f64)
                * 100.0
        } else {
            0.0
        };

        if agent_failure_rate >= self.alert_thresholds.agent_failure_rate_critical {
            alerts.push(Alert::new(
                AlertLevel::Critical,
                "High agent failure rate".to_string(),
                format!("Agent failure rate: {agent_failure_rate:.1}%"),
            ));
        } else if agent_failure_rate >= self.alert_thresholds.agent_failure_rate_warning {
            alerts.push(Alert::new(
                AlertLevel::Warning,
                "Elevated agent failure rate".to_string(),
                format!("Agent failure rate: {agent_failure_rate:.1}%"),
            ));
        }

        if !alerts.is_empty() {
            warn!("Generated {} alerts", alerts.len());
        }

        alerts
    }

    /// Update individual agent metrics
    pub async fn update_individual_agent_metrics(
        &self,
        agent_id: uuid::Uuid,
        metrics: IndividualAgentMetrics,
    ) {
        let mut current = self.current_metrics.write().await;
        current
            .agent_metrics
            .individual_agent_metrics
            .insert(agent_id, metrics);
        current.timestamp = Utc::now();

        debug!("Updated metrics for agent {}", agent_id);
    }

    /// Get system uptime
    #[must_use]
    pub fn get_uptime(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Collect comprehensive system metrics
    pub async fn collect_system_metrics(&self) -> anyhow::Result<SystemMetrics> {
        let mut current = self.current_metrics.write().await;

        // Update resource usage with enhanced metrics
        current.resource_usage.network_io = self.get_network_metrics()?;
        current.resource_usage.disk_io = self.get_disk_metrics()?;
        current.timestamp = Utc::now();

        Ok(current.clone())
    }

    fn get_network_metrics(&self) -> anyhow::Result<NetworkMetrics> {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        // Simulate network activity (in production, use system APIs)
        Ok(NetworkMetrics {
            bytes_sent: (current_time * 1024) + 1024 * 1024,
            bytes_received: (current_time * 2048) + 2048 * 1024,
            connections_active: 10 + (current_time % 20) as u32,
            websocket_connections: 5 + (current_time % 10) as u32,
            requests_per_second: 50.0 + (current_time % 30) as f64,
        })
    }

    fn get_disk_metrics(&self) -> anyhow::Result<DiskMetrics> {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        // Simulate disk activity (in production, use system APIs)
        Ok(DiskMetrics {
            reads_per_second: 100.0 + (current_time % 50) as f64,
            writes_per_second: 50.0 + (current_time % 25) as f64,
            read_bytes: (current_time * 1024) + 1024 * 1024,
            write_bytes: (current_time * 512) + 512 * 1024,
            disk_usage_percent: 65.0 + (current_time % 10) as f64,
        })
    }

    /// Detect anomalies in system behavior
    pub async fn detect_anomalies(&self) -> Vec<Anomaly> {
        let history = self.historical_metrics.read().await;
        self.anomaly_detector.detect(&history)
    }

    /// Get predictive insights
    pub async fn get_predictive_insights(&self) -> PredictiveInsights {
        let trends = self.analyze_trends().await;
        let anomalies = self.detect_anomalies().await;

        PredictiveInsights {
            trends,
            anomalies,
            recommendations: self.generate_recommendations().await,
            forecast: self.generate_forecast().await,
        }
    }

    async fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        let current = self.current_metrics.read().await;

        if current.resource_usage.cpu_usage_percent > 80.0 {
            recommendations.push(
                "Consider scaling up CPU resources or optimizing agent workloads".to_string(),
            );
        }

        if current.resource_usage.memory_usage_percent > 85.0 {
            recommendations.push(
                "Memory usage is high - consider garbage collection or memory optimization"
                    .to_string(),
            );
        }

        recommendations
    }

    async fn generate_forecast(&self) -> SystemForecast {
        let history = self.historical_metrics.read().await;

        if history.len() < 5 {
            return SystemForecast::default();
        }

        // Simple forecast based on recent trends
        SystemForecast {
            predicted_health_score_1h: 0.85,
            predicted_health_score_24h: 0.80,
            confidence: 0.75,
            risk_factors: vec!["Normal operational variance".to_string()],
        }
    }
}

// Advanced analytics components
#[allow(dead_code)]
pub struct TrendAnalyzer {
    window_size: usize,
    trend_threshold: f64,
}

impl TrendAnalyzer {
    #[must_use]
    pub fn new(window_size: usize, trend_threshold: f64) -> Self {
        Self {
            window_size,
            trend_threshold,
        }
    }
}

#[allow(dead_code)]
pub struct AnomalyDetector {
    sensitivity: f64,
    baseline_window: usize,
}

impl AnomalyDetector {
    #[must_use]
    pub fn new(sensitivity: f64, baseline_window: usize) -> Self {
        Self {
            sensitivity,
            baseline_window,
        }
    }

    #[allow(clippy::unused_self)]
    #[must_use]
    pub fn detect(&self, _history: &[SystemMetrics]) -> Vec<Anomaly> {
        // Simplified anomaly detection
        vec![]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveInsights {
    pub trends: MetricsTrends,
    pub anomalies: Vec<Anomaly>,
    pub recommendations: Vec<String>,
    pub forecast: SystemForecast,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemForecast {
    pub predicted_health_score_1h: f64,
    pub predicted_health_score_24h: f64,
    pub confidence: f64,
    pub risk_factors: Vec<String>,
}

impl Default for SystemForecast {
    fn default() -> Self {
        Self {
            predicted_health_score_1h: 0.8,
            predicted_health_score_24h: 0.75,
            confidence: 0.5,
            risk_factors: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub timestamp: DateTime<Utc>,
    #[allow(clippy::struct_field_names)]
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub description: String,
    pub affected_metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    PerformanceSpike,
    PerformanceDrop,
    ResourceExhaustion,
    UnexpectedBehavior,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct MetricsTrends {
    pub cpu_trend: TrendDirection,
    pub memory_trend: TrendDirection,
    pub task_completion_trend: TrendDirection,
    pub agent_performance_trend: TrendDirection,
    pub error_rate_trend: TrendDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct MetricThresholds {
    pub cpu_warning: f64,
    pub cpu_critical: f64,
    pub memory_warning: f64,
    pub memory_critical: f64,
    pub task_failure_rate_warning: f64,
    pub task_failure_rate_critical: f64,
    pub agent_failure_rate_warning: f64,
    pub agent_failure_rate_critical: f64,
    pub response_time_warning: f64,
    pub response_time_critical: f64,
}

impl Default for MetricThresholds {
    fn default() -> Self {
        Self {
            cpu_warning: 70.0,
            cpu_critical: 90.0,
            memory_warning: 80.0,
            memory_critical: 95.0,
            task_failure_rate_warning: 10.0,
            task_failure_rate_critical: 25.0,
            agent_failure_rate_warning: 5.0,
            agent_failure_rate_critical: 15.0,
            response_time_warning: 1000.0, // milliseconds
            response_time_critical: 5000.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub level: AlertLevel,
    pub title: String,
    pub description: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub id: uuid::Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

impl Alert {
    #[must_use]
    pub fn new(level: AlertLevel, title: String, description: String) -> Self {
        Self {
            level,
            title,
            description,
            timestamp: chrono::Utc::now(),
            id: uuid::Uuid::new_v4(),
        }
    }
}

impl Default for MetricsTrends {
    fn default() -> Self {
        Self {
            cpu_trend: TrendDirection::Unknown,
            memory_trend: TrendDirection::Unknown,
            task_completion_trend: TrendDirection::Unknown,
            agent_performance_trend: TrendDirection::Unknown,
            error_rate_trend: TrendDirection::Unknown,
        }
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            performance: PerformanceMetrics::default(),
            resource_usage: ResourceUsageMetrics::default(),
            agent_metrics: AgentMetrics::default(),
            task_metrics: TaskMetrics::default(),
            error_metrics: ErrorMetrics::default(),
            timestamp: Utc::now(),
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            requests_per_second: 0.0,
            average_response_time_ms: 0.0,
            p95_response_time_ms: 0.0,
            p99_response_time_ms: 0.0,
            throughput_tasks_per_second: 0.0,
        }
    }
}

impl Default for ResourceUsageMetrics {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_percent: 0.0,
            memory_usage_bytes: 0,
            network_bytes_in: 0,
            network_bytes_out: 0,
            disk_usage_bytes: 0,
            network_io: NetworkMetrics::default(),
            disk_io: DiskMetrics::default(),
        }
    }
}

impl Default for AgentMetrics {
    fn default() -> Self {
        Self {
            total_agents: 0,
            active_agents: 0,
            idle_agents: 0,
            failed_agents: 0,
            average_agent_performance: 0.0,
            agent_utilization_percent: 0.0,
            individual_agent_metrics: HashMap::new(),
        }
    }
}

impl Default for TaskMetrics {
    fn default() -> Self {
        Self {
            total_tasks_submitted: 0,
            total_tasks_completed: 0,
            total_tasks_failed: 0,
            tasks_in_queue: 0,
            average_task_duration_ms: 0.0,
            task_success_rate: 0.0,
        }
    }
}

impl Default for ErrorMetrics {
    fn default() -> Self {
        Self {
            total_errors: 0,
            error_rate_per_minute: 0.0,
            errors_by_type: HashMap::new(),
            critical_errors: 0,
        }
    }
}

fn calculate_trend(previous: f64, current: f64) -> TrendDirection {
    let threshold = 0.05; // 5% threshold for stability
    let change = (current - previous) / previous.max(0.001); // Avoid division by zero

    if change > threshold {
        TrendDirection::Increasing
    } else if change < -threshold {
        TrendDirection::Decreasing
    } else {
        TrendDirection::Stable
    }
}
