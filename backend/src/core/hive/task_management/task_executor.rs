//! Task Execution Engine
//!
//! Handles the actual execution of tasks with verification, monitoring,
//! and error recovery capabilities.

use super::task_types::*;
use crate::agents::agent::{Agent, AgentState};
use crate::tasks::task::Task;
use crate::utils::error::{HiveError, HiveResult};
use crate::AgentState;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Task execution engine with verification and monitoring
#[derive(Clone)]
pub struct TaskExecutor {
    /// Configuration for task execution
    config: TaskDistributionConfig,
    /// Currently executing tasks
    active_executions: Arc<RwLock<HashMap<Uuid, TaskExecution>>>,
    /// Execution history for analytics
    execution_history: Arc<RwLock<Vec<TaskExecutionResult>>>,
}

/// Information about a currently executing task
#[derive(Debug, Clone)]
struct TaskExecution {
    task_id: Uuid,
    agent_id: Uuid,
    started_at: Instant,
    status: TaskStatus,
}

impl TaskExecutor {
    /// Create a new task executor
    pub fn new(config: TaskDistributionConfig) -> Self {
        Self {
            config,
            active_executions: Arc::new(RwLock::new(HashMap::new())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Execute a task with comprehensive verification
    pub async fn execute_task_with_verification(
        &self,
        task: Task,
        agent: &Agent,
    ) -> HiveResult<TaskExecutionResult> {
        let task_id = task.id;
        let agent_id = agent.id;
        let start_time = Instant::now();

        // Record execution start
        self.record_execution_start(task_id, agent_id).await;

        // Verify agent capabilities match task requirements
        self.verify_agent_capabilities(&task, agent).await?;

        // Execute the task with timeout
        let execution_result = self.execute_with_timeout(task, agent).await;

        // Calculate execution time
        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // Create execution result
        let result = match execution_result {
            Ok(result_data) => TaskExecutionResult {
                task_id,
                agent_id,
                success: true,
                execution_time_ms,
                result: Some(result_data),
                error_message: None,
            },
            Err(error) => TaskExecutionResult {
                task_id,
                agent_id,
                success: false,
                execution_time_ms,
                result: None,
                error_message: Some(error.to_string()),
            },
        };

        // Record execution completion
        self.record_execution_completion(task_id, &result).await;

        // Store in execution history
        self.store_execution_result(result.clone()).await;

        Ok(result)
    }

    /// Verify that an agent has the required capabilities for a task
    async fn verify_agent_capabilities(&self, task: &Task, agent: &Agent) -> HiveResult<()> {
        // Check if agent has required capabilities
        for required_cap in &task.required_capabilities {
            let has_capability = agent.capabilities.iter().any(|cap| {
                cap.name == required_cap.name && cap.proficiency >= required_cap.minimum_proficiency
            });

            if !has_capability {
                return Err(HiveError::ValidationError {
                    field: "agent_capabilities".to_string(),
                    reason: format!(
                        "Agent {} lacks required capability: {} (min proficiency: {})",
                        agent.id, required_cap.name, required_cap.minimum_proficiency
                    ),
                });
            }
        }

        // Check agent availability
        if agent.state != AgentState::Idle {
            return Err(HiveError::ValidationError {
                field: "agent_state".to_string(),
                reason: format!(
                    "Agent {} is not available (current state: {})",
                    agent.id, agent.state
                ),
            });
        }

        Ok(())
    }

    /// Execute task with timeout protection
    async fn execute_with_timeout(
        &self,
        task: Task,
        agent: &Agent,
    ) -> HiveResult<serde_json::Value> {
        let timeout_duration = std::time::Duration::from_millis(self.config.execution_timeout_ms);

        // Create timeout future
        let execution_future = self.execute_task_internal(task, agent);
        let timeout_future = tokio::time::sleep(timeout_duration);

        // Race between execution and timeout
        tokio::select! {
            result = execution_future => result,
            _ = timeout_future => {
                Err(HiveError::TimeoutError {
                    operation: "task_execution".to_string(),
                    duration_ms: self.config.execution_timeout_ms,
                })
            }
        }
    }

    /// Internal task execution logic
    async fn execute_task_internal(
        &self,
        task: Task,
        agent: &Agent,
    ) -> HiveResult<serde_json::Value> {
        // This is a placeholder for actual task execution
        // In a real implementation, this would:
        // 1. Send task to agent
        // 2. Monitor execution progress
        // 3. Handle agent communication
        // 4. Return execution results

        tracing::info!("Executing task {} with agent {}", task.id, agent.id);

        // Simulate task execution based on task type
        match task.task_type.as_str() {
            "computation" => {
                // Simulate computational task
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                Ok(serde_json::json!({
                    "result": "computation_complete",
                    "value": 42,
                    "agent_id": agent.id,
                    "task_id": task.id
                }))
            }
            "io" => {
                // Simulate I/O task
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                Ok(serde_json::json!({
                    "result": "io_complete",
                    "bytes_processed": 1024,
                    "agent_id": agent.id,
                    "task_id": task.id
                }))
            }
            "network" => {
                // Simulate network task
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                Ok(serde_json::json!({
                    "result": "network_complete",
                    "requests_processed": 10,
                    "agent_id": agent.id,
                    "task_id": task.id
                }))
            }
            _ => {
                // Default task execution
                tokio::time::sleep(std::time::Duration::from_millis(75)).await;
                Ok(serde_json::json!({
                    "result": "task_complete",
                    "task_type": task.task_type,
                    "agent_id": agent.id,
                    "task_id": task.id
                }))
            }
        }
    }

    /// Record the start of task execution
    async fn record_execution_start(&self, task_id: Uuid, agent_id: Uuid) {
        let execution = TaskExecution {
            task_id,
            agent_id,
            started_at: Instant::now(),
            status: TaskStatus::Running,
        };

        self.active_executions
            .write()
            .await
            .insert(task_id, execution);
        tracing::debug!(
            "Started execution of task {} on agent {}",
            task_id,
            agent_id
        );
    }

    /// Record the completion of task execution
    async fn record_execution_completion(&self, task_id: Uuid, result: &TaskExecutionResult) {
        self.active_executions.write().await.remove(&task_id);

        let status = if result.success {
            "completed"
        } else {
            "failed"
        };
        tracing::info!(
            "Task {} execution {} in {}ms",
            task_id,
            status,
            result.execution_time_ms
        );
    }

    /// Store execution result in history
    async fn store_execution_result(&self, result: TaskExecutionResult) {
        let mut history = self.execution_history.write().await;
        history.push(result);

        // Keep only the last 1000 execution results to prevent memory growth
        if history.len() > 1000 {
            history.remove(0);
        }
    }

    /// Get currently executing tasks
    pub async fn get_active_executions(&self) -> Vec<Uuid> {
        self.active_executions
            .read()
            .await
            .keys()
            .cloned()
            .collect()
    }

    /// Get execution history
    pub async fn get_execution_history(&self, limit: Option<usize>) -> Vec<TaskExecutionResult> {
        let history = self.execution_history.read().await;
        let limit = limit.unwrap_or(history.len());
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Get execution statistics
    pub async fn get_execution_stats(&self) -> TaskPerformanceAnalytics {
        let history = self.execution_history.read().await;
        let active_count = self.active_executions.read().await.len();

        if history.is_empty() {
            return TaskPerformanceAnalytics::default();
        }

        let total_tasks = history.len() as u64;
        let successful_tasks = history.iter().filter(|r| r.success).count() as u64;
        let failed_tasks = total_tasks - successful_tasks;

        let total_execution_time: u64 = history.iter().map(|r| r.execution_time_ms).sum();
        let average_execution_time_ms = if total_tasks > 0 {
            total_execution_time as f64 / total_tasks as f64
        } else {
            0.0
        };

        let success_rate = if total_tasks > 0 {
            successful_tasks as f64 / total_tasks as f64
        } else {
            0.0
        };

        // Calculate throughput (tasks per second) based on recent history
        let recent_window = std::cmp::min(100, history.len());
        let throughput = if recent_window > 0 && average_execution_time_ms > 0.0 {
            1000.0 / average_execution_time_ms // Convert ms to seconds
        } else {
            0.0
        };

        TaskPerformanceAnalytics {
            total_tasks,
            successful_tasks,
            failed_tasks,
            average_execution_time_ms,
            success_rate,
            throughput,
            current_queue_size: active_count,
            peak_queue_size: active_count, // Simplified - would need proper tracking
        }
    }

    /// Check if executor is healthy
    pub async fn is_healthy(&self) -> bool {
        let stats = self.get_execution_stats().await;

        // Consider healthy if:
        // - Success rate is above 80%
        // - Not too many active executions
        stats.success_rate >= 0.8 && stats.current_queue_size < self.config.max_concurrent_tasks
    }

    /// Get executor status
    pub async fn get_status(&self) -> serde_json::Value {
        let stats = self.get_execution_stats().await;
        let active_executions = self.get_active_executions().await;
        let is_healthy = self.is_healthy().await;

        serde_json::json!({
            "healthy": is_healthy,
            "active_executions": active_executions.len(),
            "max_concurrent": self.config.max_concurrent_tasks,
            "total_executed": stats.total_tasks,
            "success_rate": stats.success_rate,
            "average_execution_time_ms": stats.average_execution_time_ms,
            "throughput": stats.throughput,
            "execution_timeout_ms": self.config.execution_timeout_ms
        })
    }
}
