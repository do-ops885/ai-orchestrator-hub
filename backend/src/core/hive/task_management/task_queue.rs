//! Task Queue Management
//!
//! Manages task queuing, prioritization, and distribution using both
//! legacy and work-stealing queue implementations.

use super::task_types::*;
use crate::tasks::task::Task;
use crate::tasks::work_stealing_queue::WorkStealingQueue;
use crate::utils::error::{HiveError, HiveResult};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Task queue manager with dual queue strategy
#[derive(Clone)]
pub struct TaskQueueManager {
    /// Legacy FIFO queue for backward compatibility
    legacy_queue: Arc<RwLock<VecDeque<Task>>>,
    /// Modern work-stealing queue for optimal distribution
    work_stealing_queue: Arc<WorkStealingQueue>,
    /// Configuration for queue behavior
    config: TaskDistributionConfig,
    /// Queue statistics
    stats: Arc<RwLock<TaskQueueStats>>,
}

impl TaskQueueManager {
    /// Create a new task queue manager
    pub fn new(config: TaskDistributionConfig) -> Self {
        Self {
            legacy_queue: Arc::new(RwLock::new(VecDeque::new())),
            work_stealing_queue: Arc::new(WorkStealingQueue::new()),
            config,
            stats: Arc::new(RwLock::new(TaskQueueStats::default())),
        }
    }

    /// Add a task to the queue
    pub async fn enqueue_task(&self, task: Task) -> HiveResult<()> {
        // Check queue capacity
        let current_size = self.get_queue_size().await;
        if current_size >= self.config.max_queue_size {
            return Err(HiveError::ResourceExhausted {
                resource: "task_queue".to_string(),
            });
        }

        // Try work-stealing queue first if enabled
        if self.config.enable_work_stealing {
            if let Err(e) = self.work_stealing_queue.submit_task(task.clone()).await {
                tracing::warn!("Work-stealing queue failed, falling back to legacy: {}", e);
                // Fall back to legacy queue
                self.legacy_queue.write().await.push_back(task);
            }
        } else {
            // Use legacy queue directly
            self.legacy_queue.write().await.push_back(task);
        }

        // Update statistics
        self.update_stats().await;
        Ok(())
    }

    /// Get the next task from the queue
    pub async fn dequeue_task(&self) -> HiveResult<Option<Task>> {
        // Try work-stealing queue first if enabled
        if self.config.enable_work_stealing {
            if let Some(task) = self.work_stealing_queue.pop_global().await {
                self.update_stats().await;
                return Ok(Some(task));
            }
        }

        // Fall back to legacy queue
        let task = self.legacy_queue.write().await.pop_front();
        if task.is_some() {
            self.update_stats().await;
        }
        Ok(task)
    }

    /// Get the current queue size
    pub async fn get_queue_size(&self) -> usize {
        let legacy_size = self.legacy_queue.read().await.len();
        let work_stealing_size = if self.config.enable_work_stealing {
            self.work_stealing_queue.len().await
        } else {
            0
        };
        legacy_size + work_stealing_size
    }

    /// Get queue statistics
    pub async fn get_stats(&self) -> TaskQueueStats {
        self.stats.read().await.clone()
    }

    /// Check if the queue is empty
    pub async fn is_empty(&self) -> bool {
        self.get_queue_size().await == 0
    }

    /// Check if the queue is full
    pub async fn is_full(&self) -> bool {
        self.get_queue_size().await >= self.config.max_queue_size
    }

    /// Clear all tasks from the queue
    pub async fn clear(&self) -> HiveResult<()> {
        self.legacy_queue.write().await.clear();
        if self.config.enable_work_stealing {
            if let Err(e) = self.work_stealing_queue.clear().await {
                return Err(HiveError::OperationFailed {
                    reason: format!("Failed to clear work-stealing queue: {}", e),
                });
            }
        }
        self.update_stats().await;
        Ok(())
    }

    /// Get tasks by status (for monitoring)
    pub async fn get_tasks_by_status(&self, status: TaskStatus) -> Vec<Task> {
        // This is a simplified implementation
        // In a real system, you'd need to track task statuses
        let legacy_tasks = self.legacy_queue.read().await.clone();
        legacy_tasks.into_iter().collect()
    }

    /// Update queue statistics
    async fn update_stats(&self) {
        let current_size = self.get_queue_size().await;
        let mut stats = self.stats.write().await;

        // Update current queue size
        stats.pending_tasks = current_size;
        stats.total_capacity = self.config.max_queue_size;
        stats.utilization_percentage = if self.config.max_queue_size > 0 {
            (current_size as f64 / self.config.max_queue_size as f64) * 100.0
        } else {
            0.0
        };

        // Update peak queue size
        if current_size > stats.peak_queue_size {
            stats.peak_queue_size = current_size;
        }
    }

    /// Get queue health status
    pub async fn get_health_status(&self) -> serde_json::Value {
        let stats = self.get_stats().await;
        let current_size = self.get_queue_size().await;

        let health_status = if stats.utilization_percentage > 90.0 {
            "critical"
        } else if stats.utilization_percentage > 70.0 {
            "warning"
        } else {
            "healthy"
        };

        serde_json::json!({
            "status": health_status,
            "current_size": current_size,
            "capacity": self.config.max_queue_size,
            "utilization_percentage": stats.utilization_percentage,
            "work_stealing_enabled": self.config.enable_work_stealing,
            "peak_size": stats.peak_queue_size
        })
    }

    /// Resize the queue capacity
    pub async fn resize_capacity(&mut self, new_capacity: usize) -> HiveResult<()> {
        if new_capacity == 0 {
            return Err(HiveError::ValidationError {
                field: "capacity".to_string(),
                reason: "Queue capacity must be greater than 0".to_string(),
            });
        }

        self.config.max_queue_size = new_capacity;
        self.update_stats().await;
        tracing::info!("Queue capacity resized to {}", new_capacity);
        Ok(())
    }
}
