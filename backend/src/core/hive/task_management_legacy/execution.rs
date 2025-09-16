//! # Task Execution Module
//!
//! This module handles task execution logic, result processing,
//! and execution history management.

use crate::utils::error::HiveResult;

use super::distributor::TaskDistributor;
use super::types::{TaskExecutionResult, TaskStatus};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task;
use uuid::Uuid;

impl TaskDistributor {
    /// Get task execution history
    ///
    /// Returns the complete history of task executions for analysis
    /// and performance monitoring.
    ///
    /// ## History Contents
    ///
    /// - Task and agent IDs
    /// - Execution success/failure status
    /// - Execution timing information
    /// - Result data and error messages
    ///
    /// ## Performance
    ///
    /// O(1) time complexity - direct clone of history vector.
    /// Memory overhead from cloning execution history.
    ///
    /// ## Use Cases
    ///
    /// - Performance analysis and optimization
    /// - Debugging failed executions
    /// - System monitoring and alerting
    /// - Historical reporting
    pub async fn get_execution_history(&self) -> Vec<TaskExecutionResult> {
        self.execution_history.read().await.clone()
    }

    /// Get execution statistics
    ///
    /// Returns comprehensive statistics about task executions
    /// including success rates, timing, and performance metrics.
    ///
    /// ## Statistics Calculated
    ///
    /// - Total, successful, and failed executions
    /// - Success rate percentage
    /// - Average execution time
    /// - Execution time distribution
    /// - Agent performance breakdown
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is history size.
    /// Involves iterating through all execution records.
    pub async fn get_execution_statistics(&self) -> serde_json::Value {
        let history = self.execution_history.read().await;
        let total_executions = history.len();

        if total_executions == 0 {
            return serde_json::json!({
                "total_executions": 0,
                "successful_executions": 0,
                "failed_executions": 0,
                "success_rate": 0.0,
                "average_execution_time_ms": 0.0,
                "agent_performance": {}
            });
        }

        // Move CPU-intensive computation to blocking thread
        let history_clone: Vec<_> = history.iter().cloned().collect();
        let stats = task::spawn_blocking(move || {
            Self::calculate_execution_statistics_sync(&history_clone)
        })
        .await
        .unwrap_or_else(|_| {
            tracing::warn!(
                "Failed to calculate execution statistics asynchronously, using basic stats"
            );
            serde_json::json!({
                "total_executions": total_executions,
                "successful_executions": 0,
                "failed_executions": total_executions,
                "success_rate": 0.0,
                "average_execution_time_ms": 0.0,
                "agent_performance": {}
            })
        });

        stats
    }

    /// Synchronous calculation of execution statistics
    fn calculate_execution_statistics_sync(history: &[TaskExecutionResult]) -> serde_json::Value {
        let total_executions = history.len();
        let successful_executions = history.iter().filter(|r| r.success).count();
        let failed_executions = total_executions - successful_executions;
        let success_rate = successful_executions as f64 / total_executions as f64;

        let total_execution_time: u64 = history.iter().map(|r| r.execution_time_ms).sum();
        let average_execution_time = total_execution_time as f64 / total_executions as f64;

        // Calculate agent performance
        let mut agent_performance = HashMap::new();
        for result in history.iter() {
            let agent_id = result.agent_id.to_string();
            let entry = agent_performance
                .entry(agent_id)
                .or_insert_with(|| serde_json::json!({"total": 0, "successful": 0, "failed": 0}));

            if let Some(obj) = entry.as_object_mut() {
                if let Some(total) = obj.get_mut("total") {
                    if let Some(num) = total.as_u64() {
                        *total = serde_json::json!(num + 1);
                    }
                }

                if result.success {
                    if let Some(successful) = obj.get_mut("successful") {
                        if let Some(num) = successful.as_u64() {
                            *successful = serde_json::json!(num + 1);
                        }
                    }
                } else {
                    if let Some(failed) = obj.get_mut("failed") {
                        if let Some(num) = failed.as_u64() {
                            *failed = serde_json::json!(num + 1);
                        }
                    }
                }
            }
        }

        serde_json::json!({
            "total_executions": total_executions,
            "successful_executions": successful_executions,
            "failed_executions": failed_executions,
            "success_rate": success_rate,
            "average_execution_time_ms": average_execution_time,
            "agent_performance": agent_performance
        })
    }

    /// Get task execution result
    ///
    /// Retrieves the execution result for a specific task from history.
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is history size.
    /// Involves searching through execution history.
    ///
    /// # Parameters
    ///
    /// * `task_id` - Unique identifier of the task
    ///
    /// # Returns
    ///
    /// Returns the execution result if found, None otherwise.
    pub async fn get_task_execution_result(&self, task_id: Uuid) -> Option<TaskExecutionResult> {
        let history = self.execution_history.read().await;
        history.iter().find(|r| r.task_id == task_id).cloned()
    }

    /// Get agent execution history
    ///
    /// Returns all execution results for tasks handled by a specific agent.
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is history size.
    /// Involves filtering through execution history.
    ///
    /// # Parameters
    ///
    /// * `agent_id` - Unique identifier of the agent
    ///
    /// # Returns
    ///
    /// Returns a vector of execution results for the agent.
    pub async fn get_agent_execution_history(&self, agent_id: Uuid) -> Vec<TaskExecutionResult> {
        let history = self.execution_history.read().await;
        history
            .iter()
            .filter(|r| r.agent_id == agent_id)
            .cloned()
            .collect()
    }

    /// Clear execution history
    ///
    /// Removes all execution history records. Useful for testing
    /// or when starting fresh monitoring periods.
    ///
    /// ## Performance
    ///
    /// O(1) time complexity - simple vector clear operation.
    ///
    /// ## Warning
    ///
    /// This operation cannot be undone. Historical data will be lost.
    pub async fn clear_execution_history(&self) {
        let mut history = self.execution_history.write().await;
        history.clear();
    }

    /// Get task metrics
    ///
    /// Retrieves the current metrics for a specific task.
    ///
    /// ## Performance
    ///
    /// O(1) average case - HashMap lookup.
    ///
    /// # Parameters
    ///
    /// * `task_id` - Unique identifier of the task
    ///
    /// # Returns
    ///
    /// Returns the task metrics if found, None otherwise.
    pub async fn get_task_metrics(&self, task_id: Uuid) -> Option<super::types::TaskMetrics> {
        let task_metrics = self.task_metrics.read().await;
        task_metrics.get(&task_id).cloned()
    }

    /// Update task status
    ///
    /// Updates the status of a specific task.
    ///
    /// ## Performance
    ///
    /// O(1) average case - HashMap update.
    ///
    /// # Parameters
    ///
    /// * `task_id` - Unique identifier of the task
    /// * `status` - New status for the task
    pub async fn update_task_status(&self, task_id: Uuid, status: TaskStatus) {
        let mut task_metrics = self.task_metrics.write().await;
        if let Some(metrics) = task_metrics.get_mut(&task_id) {
            // Store the status for matching before moving it
            let status_clone = match &status {
                TaskStatus::Completed => {
                    metrics.completed_at = Some(chrono::Utc::now());
                    status
                }
                TaskStatus::Failed => {
                    metrics.failure_count += 1;
                    status
                }
                _ => status,
            };

            metrics.status = status_clone;
        }
    }

    /// Get tasks by status
    ///
    /// Returns all tasks that have a specific status.
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is the number of tasks.
    /// Involves iterating through all task metrics.
    ///
    /// # Parameters
    ///
    /// * `status` - Status to filter by
    ///
    /// # Returns
    ///
    /// Returns a vector of task IDs with the specified status.
    pub async fn get_tasks_by_status(&self, status: TaskStatus) -> Vec<Uuid> {
        let task_metrics = self.task_metrics.read().await;
        task_metrics
            .iter()
            .filter(|(_, metrics)| metrics.status == status)
            .map(|(task_id, _)| *task_id)
            .collect()
    }

    /// Get system performance summary
    ///
    /// Returns a high-level summary of system performance
    /// based on task execution patterns and metrics.
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is the number of tasks.
    /// Involves aggregating metrics across all tasks.
    pub async fn get_system_performance_summary(&self) -> serde_json::Value {
        let task_metrics = self.task_metrics.read().await;
        let total_tasks = task_metrics.len();

        if total_tasks == 0 {
            return serde_json::json!({
                "total_tasks": 0,
                "completed_tasks": 0,
                "failed_tasks": 0,
                "pending_tasks": 0,
                "average_execution_time_ms": 0.0
            });
        }

        let mut completed_tasks = 0;
        let mut failed_tasks = 0;
        let mut pending_tasks = 0;
        let mut total_execution_time = 0u64;
        let mut execution_count = 0;

        for metrics in task_metrics.values() {
            match metrics.status {
                TaskStatus::Completed => completed_tasks += 1,
                TaskStatus::Failed => failed_tasks += 1,
                TaskStatus::Pending => pending_tasks += 1,
                _ => {}
            }

            if metrics.total_execution_time_ms > 0 {
                total_execution_time += metrics.total_execution_time_ms;
                execution_count += 1;
            }
        }

        let average_execution_time = if execution_count > 0 {
            total_execution_time as f64 / execution_count as f64
        } else {
            0.0
        };

        serde_json::json!({
            "total_tasks": total_tasks,
            "completed_tasks": completed_tasks,
            "failed_tasks": failed_tasks,
            "pending_tasks": pending_tasks,
            "average_execution_time_ms": average_execution_time
        })
    }
}
