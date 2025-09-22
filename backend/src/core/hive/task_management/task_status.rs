//! Task Status and Analytics Module
//!
//! Provides comprehensive status reporting and analytics for task management.

use super::task_executor::TaskExecutor;
use super::task_metrics::TaskMetricsCollector;
use super::task_queue::TaskQueueManager;
use super::task_types::TaskDistributionConfig;
use crate::utils::error::HiveResult;

/// Task status and analytics functionality
pub struct TaskStatusReporter {
    /// Task queue manager
    queue_manager: TaskQueueManager,
    /// Task executor
    executor: TaskExecutor,
    /// Metrics collector
    metrics_collector: TaskMetricsCollector,
    /// Configuration
    config: TaskDistributionConfig,
}

impl TaskStatusReporter {
    /// Create a new status reporter
    #[must_use] 
    pub fn new(
        queue_manager: TaskQueueManager,
        executor: TaskExecutor,
        metrics_collector: TaskMetricsCollector,
        config: TaskDistributionConfig,
    ) -> Self {
        Self {
            queue_manager,
            executor,
            metrics_collector,
            config,
        }
    }

    /// Get comprehensive system status
    ///
    /// Returns detailed status information about the task management system
    /// including queue status, execution metrics, and system health.
    ///
    /// ## Status Information
    ///
    /// - Queue sizes (legacy and work-stealing)
    /// - Task status distribution
    /// - Execution statistics and timing
    /// - Performance metrics and trends
    /// - System health indicators
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is the number of tasks.
    /// Involves aggregating metrics across all components.
    pub async fn get_status(&self) -> serde_json::Value {
        // Execute all status queries in parallel for better performance
        let (queue_stats, executor_status, metrics_summary, queue_health) = tokio::join!(
            self.queue_manager.get_stats(),
            self.executor.get_status(),
            self.metrics_collector.get_metrics_summary(),
            self.queue_manager.get_health_status()
        );

        let queue_healthy = queue_health.get("status").and_then(|v| v.as_str()) == Some("healthy");
        let executor_healthy = executor_status
            .get("healthy")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or_default();
        let system_healthy = queue_healthy && executor_healthy;

        serde_json::json!({
            "queue": {
                "stats": queue_stats,
                "health": queue_health,
                "legacy_queue_size": queue_stats.pending_tasks // For backward compatibility
            },
            "executor": executor_status,
            "metrics": metrics_summary,
            "system": {
                "healthy": system_healthy,
                "max_concurrent_tasks": self.config.max_concurrent_tasks,
                "max_queue_size": self.config.max_queue_size,
                "work_stealing_enabled": self.config.enable_work_stealing
            },
            "timestamp": chrono::Utc::now()
        })
    }

    /// Get detailed analytics
    ///
    /// Returns comprehensive analytics including performance metrics,
    /// agent performance, and trend analysis.
    ///
    /// ## Analytics Data
    ///
    /// - Task execution patterns and timing
    /// - Success/failure rates and trends
    /// - Agent performance breakdown
    /// - Queue efficiency metrics
    /// - System throughput analysis
    ///
    /// ## Performance
    ///
    /// O(n) time complexity with sorting operations for rankings.
    /// May involve complex calculations for trend analysis.
    pub async fn get_analytics(&self) -> serde_json::Value {
        // Execute all analytics queries in parallel for better performance
        let (metrics_analytics, performance_analytics, recent_performance, executor_stats) = tokio::join!(
            self.metrics_collector.get_analytics(),
            self.metrics_collector.get_performance_analytics(),
            self.metrics_collector.get_recent_performance(),
            self.executor.get_execution_stats()
        );

        serde_json::json!({
            "performance_metrics": performance_analytics,
            "recent_performance": recent_performance,
            "detailed_metrics": metrics_analytics,
            "executor_stats": executor_stats,
            "system_analytics": {
                "queue_utilization": self.queue_manager.get_stats().await.utilization_percentage,
                "executor_health": self.executor.is_healthy().await,
                "total_tasks_processed": performance_analytics.total_tasks,
                "overall_success_rate": performance_analytics.success_rate
            },
            "timestamp": chrono::Utc::now()
        })
    }

    /// Get task count summary
    pub async fn get_task_count(&self) -> usize {
        self.queue_manager.get_queue_size().await
    }

    /// Check queue health
    pub async fn check_queue_health(&self) -> HiveResult<serde_json::Value> {
        let queue_stats = self.queue_manager.get_stats().await;
        Ok(serde_json::json!({
            "queue_size": queue_stats.pending_tasks,
            "healthy": !self.queue_manager.is_full().await
        }))
    }

    /// Get system health status
    pub async fn get_system_health(&self) -> serde_json::Value {
        let status = self.get_status().await;
        let is_healthy = status
            .get("system")
            .and_then(|s| s.get("healthy"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or_default();

        serde_json::json!({
            "healthy": is_healthy,
            "status": status,
            "timestamp": chrono::Utc::now()
        })
    }
}
