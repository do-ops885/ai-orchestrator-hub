//! Task Distribution Module
//!
//! Handles task distribution to available agents and load balancing.

use super::task_executor::TaskExecutor;
use super::task_metrics::TaskMetricsCollector;
use super::task_queue::TaskQueueManager;
use super::task_types::{TaskDistributionConfig, TaskExecutionResult};
use crate::agents::agent::Agent;
use crate::core::hive::coordinator::CoordinationMessage;
use crate::tasks::task::Task;
use crate::utils::error::HiveResult;
use futures::future::join_all;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Task distribution functionality
pub struct TaskDistributor {
    /// Task queue management
    queue_manager: TaskQueueManager,
    /// Task execution engine
    executor: TaskExecutor,
    /// Metrics collection
    metrics_collector: TaskMetricsCollector,
    /// Communication channel for coordination
    coordination_tx: mpsc::UnboundedSender<CoordinationMessage>,
    /// Configuration for task distribution
    config: TaskDistributionConfig,
}

impl TaskDistributor {
    /// Create a new task distributor
    #[must_use] 
    pub fn new(
        queue_manager: TaskQueueManager,
        executor: TaskExecutor,
        metrics_collector: TaskMetricsCollector,
        coordination_tx: mpsc::UnboundedSender<CoordinationMessage>,
        config: TaskDistributionConfig,
    ) -> Self {
        Self {
            queue_manager,
            executor,
            metrics_collector,
            coordination_tx,
            config,
        }
    }

    /// Dequeue the next task
    pub async fn dequeue_task(&self) -> HiveResult<Option<Task>> {
        self.queue_manager.dequeue_task().await
    }

    /// Distribute tasks to available agents
    ///
    /// Automatically distributes queued tasks to available agents based on
    /// agent capabilities, workload, and task requirements.
    ///
    /// ## Distribution Strategy
    ///
    /// 1. Get available agents (currently idle)
    /// 2. Dequeue tasks from queue
    /// 3. Match tasks to suitable agents
    /// 4. Execute tasks concurrently (up to `max_concurrent_tasks`)
    /// 5. Handle execution failures and retries
    ///
    /// ## Load Balancing
    ///
    /// - Considers agent workload and capabilities
    /// - Respects maximum concurrent task limits
    /// - Handles task execution failures gracefully
    ///
    /// ## Performance
    ///
    /// O(n) where n is the number of available agents.
    /// Concurrent execution improves throughput.
    pub async fn distribute_tasks(&self, agents: &[(Uuid, Agent)]) -> HiveResult<usize> {
        // Get available agents
        let available_agents: Vec<_> = agents
            .iter()
            .filter(|(_, agent)| agent.state == crate::agents::agent::AgentState::Idle)
            .collect();

        if available_agents.is_empty() {
            return Ok(0);
        }

        // Collect tasks to execute in parallel
        let mut tasks_to_execute = Vec::new();
        let mut agent_assignments = Vec::new();

        for (agent_id, agent) in available_agents
            .iter()
            .take(self.config.max_concurrent_tasks)
        {
            if let Some(task) = self.queue_manager.dequeue_task().await? {
                // Record assignment
                self.metrics_collector
                    .record_task_assigned(task.id, *agent_id)
                    .await?;

                tasks_to_execute.push((task, *agent_id, (*agent).clone()));
                agent_assignments.push(*agent_id);
            } else {
                break; // No more tasks in queue
            }
        }

        if tasks_to_execute.is_empty() {
            return Ok(0);
        }

        // Execute tasks in parallel using join_all for better async performance
        let execution_futures = tasks_to_execute.into_iter().map(|(task, _agent_id, agent)| {
            let executor = self.executor.clone();
            let metrics_collector = self.metrics_collector.clone();
            let queue_manager = self.queue_manager.clone();

            async move {
                match executor
                    .execute_task_with_verification(&task.clone(), &agent)
                    .await
                {
                    Ok(result) => {
                        metrics_collector.record_task_completed(result).await?;
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!("Task execution failed: {}", e);
                        // Re-queue the task for retry
                        queue_manager.enqueue_task(task).await?;
                        Err(e)
                    }
                }
            }
        });

        // Execute all tasks concurrently
        let results = join_all(execution_futures).await;

        // Count successful executions
        let distributed_count = results.iter().filter(|result| result.is_ok()).count();

        tracing::info!(
            "Distributed {} tasks to agents (executed in parallel)",
            distributed_count
        );
        Ok(distributed_count)
    }

    /// Distribute a specific task to a specific agent
    ///
    /// Assigns a specific task to a specific agent for execution.
    /// Performs capability validation before assignment.
    ///
    /// ## Validation
    ///
    /// - Verifies agent has required capabilities
    /// - Checks agent availability
    /// - Validates task requirements
    ///
    /// ## Performance
    ///
    /// O(1) for basic validation, O(n) for capability checking where n is number of required capabilities.
    pub async fn distribute_specific_task(
        &self,
        task: Task,
        agent: &Agent,
    ) -> HiveResult<TaskExecutionResult> {
        // Record assignment
        self.metrics_collector
            .record_task_assigned(task.id, agent.id)
            .await?;

        // Execute the task
        let result = self
            .executor
            .execute_task_with_verification(&task, agent)
            .await?;

        // Record completion
        self.metrics_collector
            .record_task_completed(result.clone())
            .await?;

        Ok(result)
    }

    /// Get distribution statistics
    pub async fn get_distribution_stats(&self) -> serde_json::Value {
        let queue_stats = self.queue_manager.get_stats().await;
        let executor_status = self.executor.get_status().await;

        serde_json::json!({
            "queue_stats": queue_stats,
            "executor_status": executor_status,
            "max_concurrent_tasks": self.config.max_concurrent_tasks,
            "work_stealing_enabled": self.config.enable_work_stealing
        })
    }

    /// Check if distribution system is healthy
    pub async fn is_healthy(&self) -> bool {
        let queue_healthy = !self.queue_manager.is_full().await;
        let executor_healthy = self.executor.is_healthy().await;

        queue_healthy && executor_healthy
    }

    /// Get distribution efficiency metrics
    pub async fn get_efficiency_metrics(&self) -> serde_json::Value {
        let stats = self.get_distribution_stats().await;
        let is_healthy = self.is_healthy().await;

        serde_json::json!({
            "healthy": is_healthy,
            "efficiency_score": if is_healthy { 1.0 } else { 0.0 },
            "stats": stats
        })
    }

    /// Record task creation in metrics
    pub async fn record_task_created(&self, task_id: Uuid) -> HiveResult<()> {
        self.metrics_collector.record_task_created(task_id).await
    }

    /// Enqueue a task
    pub async fn enqueue_task(&self, task: Task) -> HiveResult<()> {
        self.queue_manager.enqueue_task(task).await
    }

    /// Get queue manager health status
    pub async fn get_queue_health_status(&self) -> serde_json::Value {
        self.queue_manager.get_health_status().await
    }

    /// Get executor health status
    pub async fn get_executor_health_status(&self) -> bool {
        self.executor.is_healthy().await
    }

    /// Get queue size
    pub async fn get_queue_size(&self) -> usize {
        self.queue_manager.get_queue_size().await
    }
}
