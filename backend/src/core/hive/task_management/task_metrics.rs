//! Task Metrics Collection and Analytics
//!
//! Comprehensive metrics tracking for task performance, execution patterns,
//! and system analytics.

use super::task_types::*;
use crate::utils::error::HiveResult;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Task metrics collector and analytics engine
#[derive(Clone)]
pub struct TaskMetricsCollector {
    /// Individual task metrics
    task_metrics: Arc<RwLock<HashMap<Uuid, TaskMetrics>>>,
    /// Aggregated performance analytics
    performance_analytics: Arc<RwLock<TaskPerformanceAnalytics>>,
    /// Historical execution results
    execution_history: Arc<RwLock<Vec<TaskExecutionResult>>>,
    /// Recent performance window for trend analysis
    recent_performance: Arc<RwLock<Vec<TaskExecutionResult>>>,
}

impl TaskMetricsCollector {
    /// Create a new task metrics collector
    pub fn new() -> Self {
        Self {
            task_metrics: Arc::new(RwLock::new(HashMap::new())),
            performance_analytics: Arc::new(RwLock::new(TaskPerformanceAnalytics::default())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
            recent_performance: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record task creation
    pub async fn record_task_created(&self, task_id: Uuid) -> HiveResult<()> {
        let metrics = TaskMetrics {
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            assigned_agent: None,
            execution_attempts: 0,
            status: TaskStatus::Pending,
        };

        self.task_metrics.write().await.insert(task_id, metrics);
        self.update_analytics().await;
        Ok(())
    }

    /// Record task assignment to an agent
    pub async fn record_task_assigned(&self, task_id: Uuid, agent_id: Uuid) -> HiveResult<()> {
        if let Some(metrics) = self.task_metrics.write().await.get_mut(&task_id) {
            metrics.assigned_agent = Some(agent_id);
            metrics.status = TaskStatus::Assigned;
        }
        self.update_analytics().await;
        Ok(())
    }

    /// Record task execution start
    pub async fn record_task_started(&self, task_id: Uuid) -> HiveResult<()> {
        if let Some(metrics) = self.task_metrics.write().await.get_mut(&task_id) {
            metrics.started_at = Some(Utc::now());
            metrics.status = TaskStatus::Running;
            metrics.execution_attempts += 1;
        }
        self.update_analytics().await;
        Ok(())
    }

    /// Record task completion
    pub async fn record_task_completed(&self, result: TaskExecutionResult) -> HiveResult<()> {
        // Update task metrics
        if let Some(metrics) = self.task_metrics.write().await.get_mut(&result.task_id) {
            metrics.completed_at = Some(Utc::now());
            metrics.status = if result.success {
                TaskStatus::Completed
            } else {
                TaskStatus::Failed
            };
        }

        // Store execution result
        self.execution_history.write().await.push(result.clone());

        // Update recent performance window (keep last 100 results)
        {
            let mut recent = self.recent_performance.write().await;
            recent.push(result);
            if recent.len() > 100 {
                recent.remove(0);
            }
        }

        // Update analytics
        self.update_analytics().await;
        Ok(())
    }

    /// Get metrics for a specific task
    pub async fn get_task_metrics(&self, task_id: Uuid) -> Option<TaskMetrics> {
        self.task_metrics.read().await.get(&task_id).cloned()
    }

    /// Get all task metrics
    pub async fn get_all_task_metrics(&self) -> HashMap<Uuid, TaskMetrics> {
        self.task_metrics.read().await.clone()
    }

    /// Get performance analytics
    pub async fn get_performance_analytics(&self) -> TaskPerformanceAnalytics {
        self.performance_analytics.read().await.clone()
    }

    /// Get recent performance data
    pub async fn get_recent_performance(&self) -> serde_json::Value {
        let recent = self.recent_performance.read().await;

        if recent.is_empty() {
            return serde_json::json!({
                "recent_success_rate": 0.0,
                "recent_average_time_ms": 0.0,
                "sample_size": 0
            });
        }

        let successful = recent.iter().filter(|r| r.success).count();
        let total = recent.len();
        let success_rate = successful as f64 / total as f64;

        let total_time: u64 = recent.iter().map(|r| r.execution_time_ms).sum();
        let average_time = total_time as f64 / total as f64;

        serde_json::json!({
            "recent_success_rate": success_rate,
            "recent_average_time_ms": average_time,
            "sample_size": total
        })
    }

    /// Get task count by status
    pub async fn count_tasks_by_status(&self, status: TaskStatus) -> usize {
        self.task_metrics
            .read()
            .await
            .values()
            .filter(|metrics| metrics.status == status)
            .count()
    }

    /// Get comprehensive analytics
    pub async fn get_analytics(&self) -> serde_json::Value {
        let performance = self.get_performance_analytics().await;
        let recent = self.get_recent_performance().await;
        let task_metrics = self.task_metrics.read().await;

        // Calculate status distribution
        let mut status_counts = HashMap::new();
        for metrics in task_metrics.values() {
            *status_counts.entry(metrics.status.clone()).or_insert(0) += 1;
        }

        // Calculate agent performance
        let mut agent_performance = HashMap::new();
        let execution_history = self.execution_history.read().await;

        for result in execution_history.iter() {
            let entry = agent_performance.entry(result.agent_id).or_insert_with(|| {
                serde_json::json!({
                    "agent_id": result.agent_id,
                    "tasks_completed": 0,
                    "tasks_failed": 0,
                    "total_execution_time_ms": 0,
                    "success_rate": 0.0,
                    "average_execution_time_ms": 0.0
                })
            });

            if result.success {
                let current_completed = entry
                    .get("tasks_completed")
                    .and_then(|v| v.as_u64())
                    .unwrap_or_default();
                entry["tasks_completed"] = (current_completed + 1).into();
            } else {
                let current_failed = entry
                    .get("tasks_failed")
                    .and_then(|v| v.as_u64())
                    .unwrap_or_default();
                entry["tasks_failed"] = (current_failed + 1).into();
            }

            let current_total_time = entry
                .get("total_execution_time_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or_default();
            entry["total_execution_time_ms"] =
                (current_total_time + result.execution_time_ms).into();
        }

        // Calculate success rates and averages for agents
        for (_, agent_data) in agent_performance.iter_mut() {
            let completed = agent_data
                .get("tasks_completed")
                .and_then(|v| v.as_u64())
                .unwrap_or_default();
            let failed = agent_data
                .get("tasks_failed")
                .and_then(|v| v.as_u64())
                .unwrap_or_default();
            let total = completed + failed;

            if total > 0 {
                agent_data["success_rate"] = serde_json::json!(completed as f64 / total as f64);
                let total_time = agent_data
                    .get("total_execution_time_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or_default();
                agent_data["average_execution_time_ms"] =
                    serde_json::json!(total_time as f64 / total as f64);
            }
        }

        serde_json::json!({
            "performance_metrics": performance,
            "recent_performance": recent,
            "status_distribution": status_counts,
            "agent_performance": agent_performance.values().collect::<Vec<_>>(),
            "total_tasks_tracked": task_metrics.len(),
            "execution_history_size": execution_history.len()
        })
    }

    /// Update aggregated analytics
    async fn update_analytics(&self) {
        let task_metrics = self.task_metrics.read().await;
        let execution_history = self.execution_history.read().await;

        let total_tasks = execution_history.len() as u64;
        let successful_tasks = execution_history.iter().filter(|r| r.success).count() as u64;
        let failed_tasks = total_tasks - successful_tasks;

        let success_rate = if total_tasks > 0 {
            successful_tasks as f64 / total_tasks as f64
        } else {
            0.0
        };

        let average_execution_time_ms = if total_tasks > 0 {
            let total_time: u64 = execution_history.iter().map(|r| r.execution_time_ms).sum();
            total_time as f64 / total_tasks as f64
        } else {
            0.0
        };

        let throughput = if average_execution_time_ms > 0.0 {
            1000.0 / average_execution_time_ms // Tasks per second
        } else {
            0.0
        };

        let current_queue_size = task_metrics
            .values()
            .filter(|m| matches!(m.status, TaskStatus::Pending | TaskStatus::Assigned))
            .count();

        let analytics = TaskPerformanceAnalytics {
            total_tasks,
            successful_tasks,
            failed_tasks,
            average_execution_time_ms,
            success_rate,
            throughput,
            current_queue_size,
            peak_queue_size: current_queue_size, // Simplified - would need proper tracking
        };

        *self.performance_analytics.write().await = analytics;
    }

    /// Clear old metrics to prevent memory growth
    pub async fn cleanup_old_metrics(&self, retention_hours: u32) -> HiveResult<()> {
        let cutoff_time = Utc::now() - chrono::Duration::hours(retention_hours as i64);

        // Remove old task metrics
        {
            let mut metrics = self.task_metrics.write().await;
            metrics.retain(|_, task_metrics| task_metrics.created_at > cutoff_time);
        }

        // Remove old execution history
        {
            let mut history = self.execution_history.write().await;
            // Keep only recent results (this is simplified - in practice you'd check timestamps)
            let current_len = history.len();
            if current_len > 10000 {
                history.drain(0..current_len - 10000);
            }
        }

        tracing::info!("Cleaned up metrics older than {} hours", retention_hours);
        Ok(())
    }

    /// Get metrics summary
    pub async fn get_metrics_summary(&self) -> serde_json::Value {
        let analytics = self.get_performance_analytics().await;
        let task_count = self.task_metrics.read().await.len();
        let history_size = self.execution_history.read().await.len();

        serde_json::json!({
            "total_tasks": analytics.total_tasks,
            "success_rate": analytics.success_rate,
            "average_execution_time_ms": analytics.average_execution_time_ms,
            "throughput": analytics.throughput,
            "current_queue_size": analytics.current_queue_size,
            "tracked_tasks": task_count,
            "history_size": history_size
        })
    }
}

impl Default for TaskMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
