use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub total_agents: usize,
    pub active_agents: usize,
    pub idle_agents: usize,
    pub failed_agents: usize,
    pub average_agent_performance: f64,
    pub agent_utilization_percent: f64,
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

/// Metrics collector with time-series data
pub struct MetricsCollector {
    current_metrics: Arc<RwLock<SystemMetrics>>,
    historical_metrics: Arc<RwLock<Vec<SystemMetrics>>>,
    max_history_size: usize,
}

impl MetricsCollector {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            current_metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            historical_metrics: Arc::new(RwLock::new(Vec::new())),
            max_history_size,
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
        *current.error_metrics.errors_by_type.entry(error_type.to_string()).or_insert(0) += 1;
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
            cpu_trend: calculate_trend(previous.resource_usage.cpu_usage_percent, recent.resource_usage.cpu_usage_percent),
            memory_trend: calculate_trend(previous.resource_usage.memory_usage_percent, recent.resource_usage.memory_usage_percent),
            task_completion_trend: calculate_trend(previous.task_metrics.task_success_rate, recent.task_metrics.task_success_rate),
            agent_performance_trend: calculate_trend(previous.agent_metrics.average_agent_performance, recent.agent_metrics.average_agent_performance),
            error_rate_trend: calculate_trend(previous.error_metrics.error_rate_per_minute, recent.error_metrics.error_rate_per_minute),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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