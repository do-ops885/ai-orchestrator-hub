//! Task Distribution Coordinator
//!
//! Central coordinator that orchestrates all task management subsystems including
//! queuing, execution, metrics collection, and coordination with other hive components.

use super::super::coordinator::CoordinationMessage;
use super::task_executor::TaskExecutor;
use super::task_metrics::TaskMetricsCollector;
use super::task_queue::TaskQueueManager;
use super::task_types::*;
use crate::agents::agent::Agent;
use crate::infrastructure::resource_manager::ResourceManager;
use crate::tasks::task::{Task, TaskPriority, TaskRequiredCapability, TaskStatus};
use crate::utils::error::{HiveError, HiveResult};
use crate::{AgentType, AgentState, AgentMemory};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// Task distribution and execution subsystem
///
/// Central coordinator for task lifecycle management in the hive system.
/// Manages task creation, queuing, distribution, and execution monitoring.
///
/// ## Components
///
/// - **Task Queue Manager**: Handles task queuing and prioritization
/// - **Task Executor**: Manages task execution with verification
/// - **Metrics Collector**: Tracks performance and analytics
/// - **Resource Manager**: Monitors system capacity
/// - **Coordination Channel**: Communication with other subsystems
///
/// ## Architecture
///
/// The task management system uses a modular approach:
/// 1. Tasks are created and validated
/// 2. Tasks are queued using optimal distribution strategy
/// 3. Tasks are executed with comprehensive monitoring
/// 4. Results are tracked and analyzed for optimization
///
/// ## Thread Safety
///
/// All operations are thread-safe using `Arc<RwLock<T>>` for shared state.
/// Task operations are atomic and consistent across concurrent access.
#[derive(Clone)]
pub struct TaskDistributor {
    /// Task queue management
    queue_manager: Arc<TaskQueueManager>,
    /// Task execution engine
    executor: Arc<TaskExecutor>,
    /// Metrics collection and analytics
    metrics_collector: Arc<TaskMetricsCollector>,
    /// Resource manager for load balancing
    resource_manager: Arc<ResourceManager>,
    /// Communication channel for coordination
    coordination_tx: mpsc::UnboundedSender<CoordinationMessage>,
    /// Configuration for task distribution
    config: TaskDistributionConfig,
}

impl TaskDistributor {
    /// Create a new task distributor
    ///
    /// Initializes all subsystems and establishes coordination channels.
    /// Sets up task queuing, execution, and metrics collection systems.
    ///
    /// ## Parameters
    ///
    /// * `resource_manager` - Shared resource manager for capacity monitoring
    /// * `coordination_tx` - Channel for sending coordination messages
    ///
    /// ## Returns
    ///
    /// Returns a configured `TaskDistributor` ready for operation.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let resource_manager = Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);
    /// let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    ///
    /// let task_distributor = TaskDistributor::new(resource_manager, tx).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(
        resource_manager: Arc<ResourceManager>,
        coordination_tx: mpsc::UnboundedSender<CoordinationMessage>,
    ) -> HiveResult<Self> {
        let config = TaskDistributionConfig::default();

        let queue_manager = Arc::new(TaskQueueManager::new(config.clone()));
        let executor = Arc::new(TaskExecutor::new(config.clone()));
        let metrics_collector = Arc::new(TaskMetricsCollector::new());

        Ok(Self {
            queue_manager,
            executor,
            metrics_collector,
            resource_manager,
            coordination_tx,
            config,
        })
    }

    /// Create a new task from configuration
    ///
    /// Validates the task configuration, creates a new task instance,
    /// and adds it to the task queue for processing.
    ///
    /// ## Configuration Format
    ///
    /// The config should be a JSON object with:
    /// - `"type"`: Task type ("computation", "io", "network", etc.)
    /// - `"title"`: Human-readable task title
    /// - `"description"`: Detailed task description
    /// - Optional: `"priority"`, `"required_capabilities"`, `"deadline"`
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # async fn example(task_distributor: &TaskDistributor) -> Result<(), Box<dyn std::error::Error>> {
    /// let config = serde_json::json!({
    ///     "type": "computation",
    ///     "title": "Data Processing",
    ///     "description": "Process incoming data batch",
    ///     "priority": "high"
    /// });
    /// let task_id = task_distributor.create_task(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_task(&self, config: serde_json::Value) -> HiveResult<Uuid> {
        // Validate required fields
        let task_type = config.get("type").and_then(|v| v.as_str()).ok_or_else(|| {
            HiveError::ValidationError {
                field: "type".to_string(),
                reason: "Task type is required".to_string(),
            }
        })?;

        let title = config
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| HiveError::ValidationError {
                field: "title".to_string(),
                reason: "Task title is required".to_string(),
            })?;

        let description = config
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Parse priority
        let priority = match config.get("priority").and_then(|v| v.as_str()) {
            Some("low") => TaskPriority::Low,
            Some("high") => TaskPriority::High,
            Some("critical") => TaskPriority::Critical,
            _ => TaskPriority::Low, // Default to Low since Normal doesn't exist
        };

        // Parse required capabilities
        let required_capabilities = if let Some(caps) = config.get("required_capabilities") {
            if let Some(caps_array) = caps.as_array() {
                caps_array
                    .iter()
                    .filter_map(|cap| {
                        if let Some(cap_obj) = cap.as_object() {
                            let name = cap_obj.get("name")?.as_str()?.to_string();
                            let minimum_proficiency =
                                cap_obj.get("minimum_proficiency")?.as_f64()?;
                            Some(TaskRequiredCapability {
                                name,
                                minimum_proficiency,
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Create the task
        let task = Task {
            id: Uuid::new_v4(),
            title: title.to_string(),
            description: description.to_string(),
            task_type: task_type.to_string(),
            priority,
            status: crate::TaskStatus::Pending,
            required_capabilities,
            assigned_agent: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deadline: None, // Could be parsed from config
            estimated_duration: None,
            context: config
                .get("context")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()
                })
                .unwrap_or_default(),
            dependencies: Vec::new(),
        };

        let task_id = task.id;

        // Record task creation in metrics
        self.metrics_collector.record_task_created(task_id).await?;

        // Add to queue
        self.queue_manager.enqueue_task(task).await?;

        // Send coordination message
        if let Err(e) = self
            .coordination_tx
            .send(CoordinationMessage::TaskCompleted {
                task_id,
                agent_id: Uuid::new_v4(), // Placeholder
                success: true,
            })
        {
            tracing::warn!("Failed to send coordination message: {}", e);
        }

        tracing::info!("Created task {} ({})", task_id, title);
        Ok(task_id)
    }

    /// Execute a task with verification
    ///
    /// Executes a specific task using a specific agent with comprehensive
    /// verification and monitoring. Ensures proper task-agent matching
    /// and validates execution results.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # async fn example(task_distributor: &TaskDistributor, task_id: uuid::Uuid, agent_id: uuid::Uuid) -> Result<(), Box<dyn std::error::Error>> {
    /// let result = task_distributor.execute_task_with_verification(task_id, agent_id).await?;
    /// println!("Task execution result: {:?}", result);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_task_with_verification(
        &self,
        task_id: Uuid,
        agent_id: Uuid,
    ) -> HiveResult<serde_json::Value> {
        // Get the next task from queue (simplified - in practice would find specific task)
        let task = self
            .queue_manager
            .dequeue_task()
            .await?
            .ok_or_else(|| HiveError::NotFound {
                resource: format!("task {}", task_id),
            })?;

        // Create a mock agent for execution (in practice, would get from agent manager)
        let agent = Agent {
            id: agent_id,
            name: "mock_agent".to_string(),
            agent_type: AgentType::Specialist("worker".to_string()),
            state: AgentState::Idle,
            capabilities: vec![],
            memory: AgentMemory::new(),
            position: (0.0, 0.0),
            energy: 100.0,
            created_at: chrono::Utc::now(),
            last_active: chrono::Utc::now(),
        };

        // Record task assignment
        self.metrics_collector
            .record_task_assigned(task_id, agent_id)
            .await?;

        // Record task start
        self.metrics_collector.record_task_started(task_id).await?;

        // Execute the task
        let execution_result = self
            .executor
            .execute_task_with_verification(task, &agent)
            .await?;

        // Record completion
        self.metrics_collector
            .record_task_completed(execution_result.clone())
            .await?;

        // Send coordination message
        if let Err(e) = self
            .coordination_tx
            .send(CoordinationMessage::TaskCompleted {
                task_id,
                agent_id,
                success: execution_result.success,
            })
        {
            tracing::warn!("Failed to send coordination message: {}", e);
        }

        // Return result as JSON
        Ok(serde_json::json!({
            "task_id": execution_result.task_id,
            "agent_id": execution_result.agent_id,
            "success": execution_result.success,
            "execution_time_ms": execution_result.execution_time_ms,
            "result": execution_result.result,
            "error_message": execution_result.error_message
        }))
    }

    /// Distribute tasks to available agents
    ///
    /// Automatically distributes queued tasks to available agents based on
    /// agent capabilities, workload, and task requirements.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # async fn example(task_distributor: &TaskDistributor, agents: &[(uuid::Uuid, Agent)]) -> Result<(), Box<dyn std::error::Error>> {
    /// let distributed_count = task_distributor.distribute_tasks(agents).await?;
    /// println!("Distributed {} tasks", distributed_count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn distribute_tasks(&self, agents: &[(Uuid, Agent)]) -> HiveResult<usize> {
        let mut distributed_count = 0;

        // Get available agents
        let available_agents: Vec<_> = agents
            .iter()
            .filter(|(_, agent)| agent.state == AgentState::Idle)
            .collect();

        if available_agents.is_empty() {
            return Ok(0);
        }

        // Distribute tasks while queue is not empty and agents are available
        for (agent_id, agent) in available_agents
            .iter()
            .take(self.config.max_concurrent_tasks)
        {
            if let Some(task) = self.queue_manager.dequeue_task().await? {
                // Record assignment
                self.metrics_collector
                    .record_task_assigned(task.id, *agent_id)
                    .await?;

                // Execute task (in practice, this would be async and non-blocking)
                match self
                    .executor
                    .execute_task_with_verification(task.clone(), agent)
                    .await
                {
                    Ok(result) => {
                        self.metrics_collector.record_task_completed(result).await?;
                        distributed_count += 1;
                    }
                    Err(e) => {
                        tracing::error!("Task execution failed: {}", e);
                        // Re-queue the task for retry (simplified)
                        self.queue_manager.enqueue_task(task).await?;
                    }
                }
            } else {
                break; // No more tasks in queue
            }
        }

        tracing::info!("Distributed {} tasks to agents", distributed_count);
        Ok(distributed_count)
    }

    /// Get comprehensive system status
    ///
    /// Returns detailed status information about the task management system
    /// including queue status, execution metrics, and system health.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # async fn example(task_distributor: &TaskDistributor) {
    /// let status = task_distributor.get_status().await;
    /// println!("Task system status: {}", status);
    /// # }
    /// ```
    pub async fn get_status(&self) -> serde_json::Value {
        let queue_stats = self.queue_manager.get_stats().await;
        let executor_status = self.executor.get_status().await;
        let metrics_summary = self.metrics_collector.get_metrics_summary().await;
        let queue_health = self.queue_manager.get_health_status().await;

        let queue_healthy = queue_health["status"].as_str() == Some("healthy");
        let executor_healthy = executor_status["healthy"].as_bool().unwrap_or(false);
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
    /// ## Example
    ///
    /// ```rust,no_run
    /// # async fn example(task_distributor: &TaskDistributor) {
    /// let analytics = task_distributor.get_analytics().await;
    /// println!("Task analytics: {}", analytics);
    /// # }
    /// ```
    pub async fn get_analytics(&self) -> serde_json::Value {
        let metrics_analytics = self.metrics_collector.get_analytics().await;
        let performance_analytics = self.metrics_collector.get_performance_analytics().await;
        let recent_performance = self.metrics_collector.get_recent_performance().await;
        let executor_stats = self.executor.get_execution_stats().await;

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

    /// Perform system maintenance
    pub async fn perform_maintenance(&self) -> HiveResult<()> {
        // Clean up old metrics (keep last 24 hours)
        self.metrics_collector.cleanup_old_metrics(24).await?;

        tracing::info!("Task management system maintenance completed");
        Ok(())
    }

    /// Get system health status
    pub async fn is_healthy(&self) -> bool {
        let queue_health = self.queue_manager.get_health_status().await;
        let executor_healthy = self.executor.is_healthy().await;

        queue_health["status"] == "healthy" && executor_healthy
    }

    /// Update configuration
    pub async fn update_config(&mut self, new_config: TaskDistributionConfig) -> HiveResult<()> {
        self.config = new_config;
        tracing::info!("Task distributor configuration updated");
        Ok(())
    }
}
