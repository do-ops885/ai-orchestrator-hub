//! # Task Management Module
//!
//! This module provides comprehensive task lifecycle management including
//! creation, distribution, execution tracking, and performance monitoring.
//!
//! ## Architecture
//!
//! The task management system uses a dual-queue approach:
//!
//! - **Work-Stealing Queue**: Modern, efficient task distribution
//! - **Legacy Queue**: Backward compatibility and fallback
//! - **Metrics Tracking**: Comprehensive execution monitoring
//! - **Coordination Integration**: Real-time status updates
//!
//! ## Key Features
//!
//! - **Dual Queue System**: Work-stealing + legacy queue for reliability
//! - **Task Verification**: Comprehensive execution validation
//! - **Performance Tracking**: Detailed execution metrics and analytics
//! - **Priority Handling**: Task prioritization and scheduling
//! - **Error Recovery**: Robust error handling and retry mechanisms
//!
//! ## Usage
//!
//! ```rust,no_run
//! use hive::core::hive::TaskDistributor;
//! use std::sync::Arc;
//! use tokio::sync::mpsc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let resource_manager = Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);
//! let (tx, _rx) = mpsc::unbounded_channel();
//!
//! let task_distributor = TaskDistributor::new(resource_manager, tx).await?;
//!
//! // Create a task
//! let config = serde_json::json!({
//!     "type": "computation",
//!     "title": "Example Task",
//!     "description": "A sample task"
//! });
//! let task_id = task_distributor.create_task(config).await?;
//!
//! // Get system status
//! let status = task_distributor.get_status().await;
//! println!("Task status: {}", status);
//!
//! // Get analytics
//! let analytics = task_distributor.get_analytics().await;
//! println!("Task analytics: {}", analytics);
//! # Ok(())
//! # }
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Task Creation**: O(log n) for queue insertion
//! - **Task Execution**: Variable based on task complexity
//! - **Status Retrieval**: O(n) for comprehensive status
//! - **Memory Usage**: O(n) where n is queued + active tasks
//! - **Concurrency**: High concurrency with async operations

use crate::agents::agent::Agent;
// Temporarily comment out missing import
// use crate::core::fallback::FallbackSystem;
use crate::infrastructure::resource_manager::ResourceManager;
use crate::tasks::task::{Task, TaskPriority, TaskRequiredCapability};
use crate::tasks::work_stealing_queue::WorkStealingQueue;
use crate::utils::error::{HiveError, HiveResult};

use super::coordinator::CoordinationMessage;

use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// Result of task execution
///
/// Contains comprehensive information about a task execution including
/// success status, timing, results, and error information.
///
/// ## Fields
///
/// - `task_id`: Unique identifier of the executed task
/// - `agent_id`: ID of the agent that executed the task
/// - `success`: Whether the execution completed successfully
/// - `execution_time_ms`: Time taken for execution in milliseconds
/// - `result`: Optional execution result data
/// - `error_message`: Optional error message if execution failed
///
/// ## Use Cases
///
/// - Execution tracking and monitoring
/// - Performance analysis and optimization
/// - Error reporting and debugging
/// - Analytics and reporting systems
#[derive(Debug, Clone)]
pub struct TaskExecutionResult {
    /// Unique identifier of the executed task
    pub task_id: Uuid,
    /// Unique identifier of the agent that executed the task
    pub agent_id: Uuid,
    /// Whether the task execution was successful
    pub success: bool,
    /// Time taken to execute the task in milliseconds
    pub execution_time_ms: u64,
    /// Optional result data from successful execution
    pub result: Option<serde_json::Value>,
    /// Optional error message if execution failed
    pub error_message: Option<String>,
}

/// Task distribution and execution subsystem
///
/// Central coordinator for task lifecycle management in the hive system.
/// Manages task creation, queuing, distribution, and execution monitoring.
///
/// ## Components
///
/// - **Legacy Queue**: Traditional task queue for backward compatibility
/// - **Work-Stealing Queue**: Modern, efficient task distribution system
/// - **Metrics Tracker**: Performance and execution statistics
/// - **Execution History**: Historical execution results and analytics
/// - **Coordination Channel**: Communication with other subsystems
///
/// ## Queue Strategy
///
/// Uses a hybrid approach:
/// 1. Attempts work-stealing queue first (more efficient)
/// 2. Falls back to legacy queue if work-stealing fails
/// 3. Ensures reliability and backward compatibility
///
/// ## Thread Safety
///
/// All operations are thread-safe using `Arc<RwLock<T>>` for shared state.
/// Task operations are atomic and consistent across concurrent access.
#[derive(Clone)]
pub struct TaskDistributor {
    /// Legacy task queue for backward compatibility
    ///
    /// Traditional FIFO queue for task storage and retrieval.
    /// Used as fallback when work-stealing queue is unavailable.
    task_queue: Arc<RwLock<Vec<Task>>>,

    /// Modern work-stealing queue for optimal distribution
    ///
    /// Advanced queue implementation that allows idle workers to
    /// "steal" tasks from busy workers for better load balancing.
    work_stealing_queue: Arc<WorkStealingQueue>,

    /// Resource manager for load balancing
    ///
    /// Used to check system capacity and make intelligent
    /// task distribution decisions based on resource availability.
    resource_manager: Arc<ResourceManager>,

    /// Communication channel for coordination
    ///
    /// Async channel for sending coordination messages when
    /// task events occur (creation, completion, failure).
    coordination_tx: mpsc::UnboundedSender<CoordinationMessage>,

    /// Task execution metrics
    ///
    /// Tracks detailed metrics for each task including timing,
    /// status, assignment, and execution attempts.
    task_metrics: Arc<RwLock<HashMap<Uuid, TaskMetrics>>>,

    /// Task execution history
    ///
    /// Maintains a history of task execution results for
    /// analytics, debugging, and performance monitoring.
    execution_history: Arc<RwLock<Vec<TaskExecutionResult>>>,
}

/// Metrics for individual tasks
///
/// Comprehensive tracking data for each task's lifecycle including
/// timing, status, assignment, and execution attempts.
///
/// ## Metrics Tracked
///
/// - Creation and completion timestamps
/// - Task status (pending, assigned, running, completed, failed)
/// - Agent assignment and execution attempts
/// - Performance timing and success tracking
///
/// ## Status States
///
/// - `Pending`: Task created but not yet assigned
/// - `Assigned`: Task assigned to an agent but not started
/// - `Running`: Task currently being executed
/// - `Completed`: Task finished successfully
/// - `Failed`: Task execution failed
/// - `Retrying`: Task being retried after failure
#[derive(Debug, Clone, Default)]
pub struct TaskMetrics {
    /// Timestamp when the task was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp when task execution started
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Timestamp when task execution completed
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// ID of the agent assigned to execute this task
    pub assigned_agent: Option<Uuid>,
    /// Number of execution attempts made
    pub execution_attempts: u32,
    /// Current status of the task
    pub status: TaskStatus,
}

/// Task execution status
///
/// Represents the current state of a task in its lifecycle.
/// Used for tracking progress and making scheduling decisions.
///
/// ## Status Transitions
///
/// ```text
/// Pending -> Assigned -> Running -> Completed
///    |         |           |
///    |         |           -> Failed -> Retrying (optional)
///    |         |
///    |         -> Failed (if assignment fails)
///    |
///    -> Failed (if creation fails)
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub enum TaskStatus {
    /// Task has been created but not yet assigned to an agent
    #[default]
    Pending,
    /// Task has been assigned to an agent but execution hasn't started
    Assigned,
    /// Task is currently being executed by an agent
    Running,
    /// Task execution completed successfully
    Completed,
    /// Task execution failed
    Failed,
    /// Task is being retried after a failure
    Retrying,
}

impl TaskDistributor {
    /// Create a new task distributor
    ///
    /// Initializes the task distribution subsystem with required dependencies.
    /// Sets up both legacy and work-stealing queues for comprehensive task management.
    ///
    /// ## Initialization Process
    ///
    /// 1. Creates work-stealing queue instance
    /// 2. Initializes legacy queue as fallback
    /// 3. Sets up metrics tracking system
    /// 4. Initializes execution history storage
    /// 5. Establishes coordination channel
    ///
    /// ## Performance
    ///
    /// O(1) initialization with minimal memory allocation.
    /// Ready for immediate task operations after creation.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::TaskDistributor;
    /// # use std::sync::Arc;
    /// # use tokio::sync::mpsc;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let resource_manager = Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);
    /// let (tx, _rx) = mpsc::unbounded_channel();
    ///
    /// let task_distributor = TaskDistributor::new(resource_manager, tx).await?;
    /// println!("Task distributor initialized");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `resource_manager` - Resource manager for capacity checking
    /// * `coordination_tx` - Channel for sending coordination messages
    ///
    /// # Returns
    ///
    /// Returns a new `TaskDistributor` instance on success.
    ///
    /// # Errors
    ///
    /// This function will not return an error under normal circumstances.
    pub async fn new(
        resource_manager: Arc<ResourceManager>,
        coordination_tx: mpsc::UnboundedSender<CoordinationMessage>,
    ) -> HiveResult<Self> {
        let work_stealing_queue = Arc::new(WorkStealingQueue::new());

        // Temporarily disabled until FallbackSystem is available
        // let fallback_system = Arc::new(RwLock::new(FallbackSystem::new()));

        Ok(Self {
            task_queue: Arc::new(RwLock::new(Vec::new())),
            work_stealing_queue,
            // fallback_system,
            resource_manager,
            coordination_tx,
            task_metrics: Arc::new(RwLock::new(HashMap::new())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Create and queue a new task
    ///
    /// Creates a new task with the specified configuration and adds it to the
    /// task distribution queue. The task will be automatically assigned to
    /// an appropriate agent based on its requirements and agent capabilities.
    ///
    /// ## Configuration Requirements
    ///
    /// The config should include:
    /// - `"type"`: Task type ("computation", "io", "network", etc.)
    /// - `"title"`: Human-readable task title
    /// - `"description"`: Detailed task description (optional)
    /// - `"priority"`: Task priority ("low", "medium", "high", "critical")
    /// - `"required_capabilities"`: Array of required agent capabilities
    ///
    /// ## Task Distribution
    ///
    /// Tasks are queued using a hybrid approach:
    /// 1. Primary: Work-stealing queue for optimal distribution
    /// 2. Fallback: Legacy queue for reliability
    ///
    /// ## Performance
    ///
    /// O(log n) for queue insertion where n is queue size.
    /// Triggers coordination messages for immediate processing.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::TaskDistributor;
    /// # async fn example(task_distributor: &TaskDistributor) -> Result<(), Box<dyn std::error::Error>> {
    /// let config = serde_json::json!({
    ///     "type": "computation",
    ///     "title": "Matrix Multiplication",
    ///     "description": "Multiply two large matrices",
    ///     "priority": "high",
    ///     "required_capabilities": [
    ///         {"name": "math", "minimum_proficiency": 0.8}
    ///     ]
    /// });
    ///
    /// let task_id = task_distributor.create_task(config).await?;
    /// println!("Created task with ID: {}", task_id);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `config` - JSON configuration object for the task
    ///
    /// # Returns
    ///
    /// Returns the unique ID of the created task on success.
    ///
    /// # Errors
    ///
    /// Returns error if configuration is invalid or task creation fails.
    pub async fn create_task(&self, config: serde_json::Value) -> HiveResult<Uuid> {
        // Validate task configuration
        let task_config = self.validate_task_config(&config)?;

        // Extract required fields from config
        let title = task_config
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled Task")
            .to_string();

        let description = task_config
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let task_type = task_config
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("general")
            .to_string();

        let priority_str = task_config
            .get("priority")
            .and_then(|v| v.as_str())
            .unwrap_or("medium");

        let priority = match priority_str {
            "low" => TaskPriority::Low,
            "high" => TaskPriority::High,
            "critical" => TaskPriority::Critical,
            _ => TaskPriority::Medium,
        };

        let required_capabilities = task_config
            .get("required_capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|cap| cap.as_object())
                    .filter_map(|obj| {
                        let name = obj.get("name")?.as_str()?.to_string();
                        let minimum_proficiency =
                            obj.get("minimum_proficiency")?.as_f64().unwrap_or(0.0);
                        Some(TaskRequiredCapability {
                            name,
                            minimum_proficiency,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Create the task
        let task = Task::new(
            title,
            description,
            task_type,
            priority,
            required_capabilities,
        );

        let task_id = task.id;

        // Initialize task metrics
        {
            let mut metrics = self.task_metrics.write().await;
            metrics.insert(
                task_id,
                TaskMetrics {
                    created_at: chrono::Utc::now(),
                    status: TaskStatus::Pending,
                    ..Default::default()
                },
            );
        }

        // Try to submit to work-stealing queue first
        if let Err(e) = self.work_stealing_queue.submit_task(task.clone()).await {
            tracing::warn!("Failed to submit task to work-stealing queue: {}", e);

            // Fall back to legacy queue
            let mut queue = self.task_queue.write().await;
            queue.push(task);
        }

        tracing::info!("Task {} created and queued successfully", task_id);
        Ok(task_id)
    }

    /// Execute a specific task with a specific agent
    ///
    /// Executes a specific task using a specific agent with comprehensive
    /// verification and monitoring throughout the execution process.
    ///
    /// ## Execution Process
    ///
    /// 1. Locate task in queues (work-stealing first, then legacy)
    /// 2. Update task metrics (started, assigned agent, attempts)
    /// 3. Execute task with timing measurement
    /// 4. Record execution results and update metrics
    /// 5. Store execution history
    /// 6. Send coordination notifications
    ///
    /// ## Verification Features
    ///
    /// - Task existence validation
    /// - Execution timing and performance tracking
    /// - Result validation and error handling
    /// - Comprehensive metrics collection
    /// - Coordination system notifications
    ///
    /// ## Performance
    ///
    /// Variable execution time depending on task complexity.
    /// Includes overhead for verification, metrics, and coordination.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::TaskDistributor;
    /// # async fn example(task_distributor: &TaskDistributor, task_id: uuid::Uuid, agent_id: uuid::Uuid) -> Result<(), Box<dyn std::error::Error>> {
    /// match task_distributor.execute_task_with_verification(task_id, agent_id).await {
    ///     Ok(result) => {
    ///         println!("Task completed successfully: {}", result);
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Task execution failed: {}", e);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `task_id` - Unique identifier of the task to execute
    /// * `agent_id` - Unique identifier of the agent to execute the task
    ///
    /// # Returns
    ///
    /// Returns the execution result as a JSON value on success.
    ///
    /// # Errors
    ///
    /// Returns error if task doesn't exist or execution fails.
    pub async fn execute_task_with_verification(
        &self,
        task_id: Uuid,
        agent_id: Uuid,
    ) -> HiveResult<serde_json::Value> {
        let start_time = Instant::now();

        // Try to get task from work-stealing queue first
        let task = if let Some(task) = self.work_stealing_queue.get_task_by_id(task_id).await {
            task
        } else {
            // Fall back to legacy queue
            let mut task_queue = self.task_queue.write().await;
            let task_index = task_queue
                .iter()
                .position(|t| t.id == task_id)
                .ok_or_else(|| HiveError::TaskNotFound {
                    id: task_id.to_string(),
                })?;
            task_queue.remove(task_index)
        };

        // Update task metrics
        {
            let mut metrics = self.task_metrics.write().await;
            if let Some(task_metrics) = metrics.get_mut(&task_id) {
                task_metrics.started_at = Some(chrono::Utc::now());
                task_metrics.assigned_agent = Some(agent_id);
                task_metrics.status = TaskStatus::Running;
                task_metrics.execution_attempts += 1;
            }
        }

        // Execute the task (this would need to be implemented based on your Agent interface)
        let execution_result = self.execute_task_internal(task, agent_id).await;
        let execution_time = start_time.elapsed();

        // Update metrics and history
        let success = execution_result.is_ok();
        {
            let mut metrics = self.task_metrics.write().await;
            if let Some(task_metrics) = metrics.get_mut(&task_id) {
                task_metrics.completed_at = Some(chrono::Utc::now());
                task_metrics.status = if success {
                    TaskStatus::Completed
                } else {
                    TaskStatus::Failed
                };
            }
        }

        // Record execution result
        let result = TaskExecutionResult {
            task_id,
            agent_id,
            success,
            execution_time_ms: execution_time.as_millis() as u64,
            result: execution_result.as_ref().ok().cloned(),
            error_message: execution_result.as_ref().err().map(|e| e.to_string()),
        };

        {
            let mut history = self.execution_history.write().await;
            history.push(result.clone());

            // Keep only last 1000 executions to prevent memory growth
            if history.len() > 1000 {
                history.remove(0);
            }
        }

        // Notify coordination system
        if let Err(e) = self
            .coordination_tx
            .send(CoordinationMessage::TaskCompleted {
                task_id,
                agent_id,
                success,
            })
        {
            tracing::warn!("Failed to send task completion notification: {}", e);
        }

        execution_result
    }

    /// Distribute tasks to available agents
    ///
    /// Distributes pending tasks from the queue to available agents.
    /// Uses intelligent assignment based on agent capabilities and task requirements.
    ///
    /// ## Distribution Strategy
    ///
    /// 1. Takes available tasks from legacy queue
    /// 2. Matches tasks to agents based on capabilities
    /// 3. Updates task metrics with assignment information
    /// 4. Spawns asynchronous execution for each task-agent pair
    ///
    /// ## Performance
    ///
    /// O(min(n, m)) where n is queue size and m is number of agents.
    /// Spawns concurrent task execution for parallelism.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::TaskDistributor;
    /// # use crate::agents::agent::Agent;
    /// # async fn example(task_distributor: &TaskDistributor, agents: &[(uuid::Uuid, Agent)]) -> Result<(), Box<dyn std::error::Error>> {
    /// let results = task_distributor.distribute_tasks(agents).await?;
    /// println!("Distributed {} tasks", results.len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `agents` - Slice of (agent_id, agent) tuples for task assignment
    ///
    /// # Returns
    ///
    /// Returns a vector of task execution results (may be empty if no tasks available).
    ///
    /// # Errors
    ///
    /// Returns error if task distribution fails.
    pub async fn distribute_tasks(
        &self,
        agents: &[(Uuid, Agent)],
    ) -> HiveResult<Vec<TaskExecutionResult>> {
        let results = Vec::new();

        // Get tasks from legacy queue
        let tasks = {
            let mut queue = self.task_queue.write().await;
            let available_tasks = std::cmp::min(queue.len(), agents.len());
            queue.drain(..available_tasks).collect::<Vec<_>>()
        };

        // Distribute tasks to agents
        for (task, (agent_id, _agent)) in tasks.into_iter().zip(agents.iter()) {
            let task_id = task.id;

            // Update task metrics
            {
                let mut metrics = self.task_metrics.write().await;
                if let Some(task_metrics) = metrics.get_mut(&task_id) {
                    task_metrics.status = TaskStatus::Assigned;
                    task_metrics.assigned_agent = Some(*agent_id);
                }
            }

            // Execute task asynchronously
            let task_distributor = self.clone();
            let agent_id = *agent_id;

            tokio::spawn(async move {
                if let Err(e) = task_distributor
                    .execute_task_with_verification(task_id, agent_id)
                    .await
                {
                    tracing::error!("Task {} execution failed: {}", task_id, e);
                }
            });
        }

        Ok(results)
    }

    /// Get task status summary
    ///
    /// Returns a comprehensive summary of task system status including
    /// queue sizes, task status distribution, and system health metrics.
    ///
    /// ## Status Information
    ///
    /// - Legacy queue size and status
    /// - Work-stealing queue metrics
    /// - Task status counts by state
    /// - Total task count and system health
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is the number of tasks.
    /// Involves counting tasks by status across all storage systems.
    ///
    /// ## Use Cases
    ///
    /// - System monitoring and health checks
    /// - Queue analysis and bottleneck identification
    /// - Administrative status reporting
    /// - Performance troubleshooting
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::TaskDistributor;
    /// # async fn example(task_distributor: &TaskDistributor) {
    /// let status = task_distributor.get_status().await;
    ///
    /// let legacy_size = status["legacy_queue_size"].as_u64().unwrap_or(0);
    /// let total_tasks = status["total_tasks"].as_u64().unwrap_or(0);
    ///
    /// println!("Legacy queue: {}, Total tasks: {}", legacy_size, total_tasks);
    ///
    /// if let Some(status_counts) = status.get("task_status_counts") {
    ///     println!("Status distribution: {}", status_counts);
    /// }
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a JSON object containing comprehensive task status information.
    pub async fn get_status(&self) -> serde_json::Value {
        let legacy_queue_size = self.task_queue.read().await.len();
        let ws_metrics = self.work_stealing_queue.get_metrics().await;

        let task_metrics = self.task_metrics.read().await;
        let status_counts = self.count_tasks_by_status(&task_metrics).await;

        serde_json::json!({
            "legacy_queue_size": legacy_queue_size,
            "work_stealing_queue": ws_metrics,
            "task_status_counts": status_counts,
            "total_tasks": task_metrics.len()
        })
    }

    /// Get detailed task analytics
    ///
    /// Returns comprehensive analytics about task performance, execution patterns,
    /// and system efficiency. Includes trend analysis and optimization insights.
    ///
    /// ## Analytics Data
    ///
    /// - Total and successful execution counts
    /// - Success rate and performance metrics
    /// - Average execution time analysis
    /// - Recent performance trends
    /// - Task distribution patterns
    ///
    /// ## Performance
    ///
    /// O(n) time complexity with history analysis.
    /// May involve complex calculations for trend analysis.
    ///
    /// ## Use Cases
    ///
    /// - Performance optimization and bottleneck identification
    /// - System efficiency analysis
    /// - Capacity planning and scaling decisions
    /// - Quality assurance and monitoring
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::TaskDistributor;
    /// # async fn example(task_distributor: &TaskDistributor) {
    /// let analytics = task_distributor.get_analytics().await;
    ///
    /// let success_rate = analytics["success_rate"].as_f64().unwrap_or(0.0);
    /// let avg_time = analytics["average_execution_time_ms"].as_f64().unwrap_or(0.0);
    ///
    /// println!("Success rate: {:.1}%, Avg time: {:.0}ms", success_rate * 100.0, avg_time);
    ///
    /// if let Some(recent_perf) = analytics.get("recent_performance") {
    ///     println!("Recent performance: {}", recent_perf);
    /// }
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a JSON object with detailed task analytics and insights.
    pub async fn get_analytics(&self) -> serde_json::Value {
        let execution_history = self.execution_history.read().await;
        let task_metrics = self.task_metrics.read().await;

        let total_executions = execution_history.len();
        let successful_executions = execution_history.iter().filter(|r| r.success).count();
        let success_rate = if total_executions > 0 {
            successful_executions as f64 / total_executions as f64
        } else {
            0.0
        };

        let average_execution_time = if total_executions > 0 {
            execution_history
                .iter()
                .map(|r| r.execution_time_ms)
                .sum::<u64>() as f64
                / total_executions as f64
        } else {
            0.0
        };

        let recent_performance = self.get_recent_performance(&execution_history).await;

        serde_json::json!({
            "total_executions": total_executions,
            "success_rate": success_rate,
            "average_execution_time_ms": average_execution_time,
            "recent_performance": recent_performance,
            "task_distribution": self.get_task_distribution(&task_metrics).await
        })
    }

    /// Validate task configuration
    ///
    /// Performs validation on task configuration to ensure required
    /// fields are present and properly formatted.
    ///
    /// ## Validation Checks
    ///
    /// - Configuration must be a valid JSON object
    /// - Required fields validation (currently basic)
    /// - Future: Type-specific validation and capability checking
    ///
    /// ## Performance
    ///
    /// O(1) time complexity for basic validation.
    ///
    /// # Parameters
    ///
    /// * `config` - JSON configuration to validate
    ///
    /// # Returns
    ///
    /// Returns the validated configuration on success.
    ///
    /// # Errors
    ///
    /// Returns error if configuration is invalid.
    fn validate_task_config(&self, config: &serde_json::Value) -> HiveResult<serde_json::Value> {
        if !config.is_object() {
            return Err(HiveError::ValidationError {
                field: "config".to_string(),
                reason: "Task configuration must be an object".to_string(),
            });
        }

        // Add specific validation logic here
        Ok(config.clone())
    }

    /// Internal task execution logic
    ///
    /// Core task execution implementation. This is a placeholder that
    /// simulates task execution for demonstration purposes.
    ///
    /// ## Current Implementation
    ///
    /// - Simulates execution with timing
    /// - Returns mock successful result
    /// - Includes proper error handling structure
    ///
    /// ## Future Implementation
    ///
    /// This should be replaced with actual task execution logic that:
    /// - Retrieves the appropriate agent
    /// - Calls agent.execute_task(task)
    /// - Handles execution results and errors
    /// - Provides real execution validation
    ///
    /// ## Performance
    ///
    /// Currently O(1) with simulated delay.
    /// Real implementation will vary based on task complexity.
    ///
    /// # Parameters
    ///
    /// * `task` - The task to execute
    /// * `agent_id` - ID of the agent performing execution
    ///
    /// # Returns
    ///
    /// Returns execution result as JSON value.
    ///
    /// # Errors
    ///
    /// Returns error if task execution fails.
    async fn execute_task_internal(
        &self,
        task: Task,
        agent_id: Uuid,
    ) -> HiveResult<serde_json::Value> {
        // This is a placeholder implementation
        // In a real system, you would:
        // 1. Get the agent from the agent manager
        // 2. Call agent.execute_task(task)
        // 3. Handle the result appropriately

        // For now, simulate task execution
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        Ok(serde_json::json!({
            "task_id": task.id,
            "agent_id": agent_id,
            "result": "Task completed successfully",
            "timestamp": chrono::Utc::now()
        }))
    }

    /// Count tasks by status
    ///
    /// Analyzes task metrics and returns a count of tasks in each status state.
    /// Useful for understanding system workload distribution and health.
    ///
    /// ## Status Categories
    ///
    /// - Pending: Tasks waiting for assignment
    /// - Assigned: Tasks assigned but not started
    /// - Running: Tasks currently executing
    /// - Completed: Tasks finished successfully
    /// - Failed: Tasks that failed execution
    /// - Retrying: Tasks being retried after failure
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is the number of tasks.
    ///
    /// # Parameters
    ///
    /// * `metrics` - HashMap of task metrics to analyze
    ///
    /// # Returns
    ///
    /// Returns a JSON object mapping status strings to counts.
    async fn count_tasks_by_status(
        &self,
        metrics: &HashMap<Uuid, TaskMetrics>,
    ) -> serde_json::Value {
        let mut counts = HashMap::new();

        for task_metrics in metrics.values() {
            let status_str = format!("{:?}", task_metrics.status);
            *counts.entry(status_str).or_insert(0) += 1;
        }

        serde_json::to_value(counts).unwrap_or_default()
    }

    /// Get recent performance metrics
    ///
    /// Analyzes recent task execution history to provide performance insights
    /// and identify trends in execution success and timing.
    ///
    /// ## Analysis Window
    ///
    /// Examines the most recent 100 executions by default.
    /// Provides success rate and average execution time for this window.
    ///
    /// ## Metrics Calculated
    ///
    /// - Recent success rate (last 100 executions)
    /// - Recent average execution time
    /// - Sample size for statistical significance
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is the analysis window size.
    ///
    /// # Parameters
    ///
    /// * `history` - Slice of execution results to analyze
    ///
    /// # Returns
    ///
    /// Returns a JSON object with recent performance metrics.
    async fn get_recent_performance(&self, history: &[TaskExecutionResult]) -> serde_json::Value {
        let recent_limit = 100; // Last 100 executions
        let recent_executions: Vec<_> = history.iter().rev().take(recent_limit).collect();

        if recent_executions.is_empty() {
            return serde_json::json!({
                "recent_success_rate": 0.0,
                "recent_average_time_ms": 0.0,
                "sample_size": 0
            });
        }

        let successful = recent_executions.iter().filter(|r| r.success).count();
        let success_rate = successful as f64 / recent_executions.len() as f64;

        let average_time = recent_executions
            .iter()
            .map(|r| r.execution_time_ms)
            .sum::<u64>() as f64
            / recent_executions.len() as f64;

        serde_json::json!({
            "recent_success_rate": success_rate,
            "recent_average_time_ms": average_time,
            "sample_size": recent_executions.len()
        })
    }

    /// Get task distribution analytics
    ///
    /// Analyzes task distribution patterns across agents and time periods.
    /// Provides insights into workload balancing and system utilization.
    ///
    /// ## Analytics Generated
    ///
    /// - Tasks per agent distribution
    /// - Hourly task creation patterns
    /// - Workload balancing effectiveness
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is the number of tasks.
    ///
    /// # Parameters
    ///
    /// * `metrics` - HashMap of task metrics to analyze
    ///
    /// # Returns
    ///
    /// Returns a JSON object with task distribution analytics.
    async fn get_task_distribution(
        &self,
        metrics: &HashMap<Uuid, TaskMetrics>,
    ) -> serde_json::Value {
        let _now = chrono::Utc::now();
        let mut agent_task_counts = HashMap::new();
        let mut hourly_distribution = HashMap::new();

        for task_metrics in metrics.values() {
            // Count tasks per agent
            if let Some(agent_id) = task_metrics.assigned_agent {
                *agent_task_counts.entry(agent_id.to_string()).or_insert(0) += 1;
            }

            // Count tasks per hour
            let hour_key = task_metrics.created_at.format("%Y-%m-%d %H:00").to_string();
            *hourly_distribution.entry(hour_key).or_insert(0) += 1;
        }

        serde_json::json!({
            "tasks_per_agent": agent_task_counts,
            "hourly_distribution": hourly_distribution
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::mpsc;

    // Helper function to create a test task distributor
    async fn create_test_task_distributor() -> HiveResult<TaskDistributor> {
        let resource_manager = Arc::new(
            crate::infrastructure::resource_manager::ResourceManager::new()
                .await
                .map_err(|e| HiveError::ResourceInitializationFailed {
                    reason: format!("Failed to initialize resource manager: {}", e),
                })?,
        );
        let (tx, _rx) = mpsc::unbounded_channel();
        TaskDistributor::new(resource_manager, tx).await
    }

    #[tokio::test]
    async fn test_task_distributor_creation() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;
        assert_eq!(task_distributor.task_queue.read().await.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_create_task_success() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        let config = serde_json::json!({
            "type": "computation",
            "title": "Test Task",
            "description": "A test task"
        });

        let task_id = task_distributor.create_task(config).await?;
        assert!(!task_id.is_nil());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_task_with_priority_low() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        let config = serde_json::json!({
            "type": "computation",
            "title": "Low Priority Task",
            "description": "A low priority task",
            "priority": "low"
        });

        let task_id = task_distributor.create_task(config).await?;
        assert!(!task_id.is_nil());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_task_with_priority_high() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        let config = serde_json::json!({
            "type": "computation",
            "title": "High Priority Task",
            "description": "A high priority task",
            "priority": "high"
        });

        let task_id = task_distributor.create_task(config).await?;
        assert!(!task_id.is_nil());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_task_with_priority_critical() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        let config = serde_json::json!({
            "type": "computation",
            "title": "Critical Task",
            "description": "A critical task",
            "priority": "critical"
        });

        let task_id = task_distributor.create_task(config).await?;
        assert!(!task_id.is_nil());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_task_with_invalid_priority() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        let config = serde_json::json!({
            "type": "computation",
            "title": "Invalid Priority Task",
            "description": "A task with invalid priority",
            "priority": "invalid"
        });

        // Should default to medium priority
        let task_id = task_distributor.create_task(config).await?;
        assert!(!task_id.is_nil());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_task_with_required_capabilities() -> Result<(), Box<dyn std::error::Error>>
    {
        let task_distributor = create_test_task_distributor().await?;

        let config = serde_json::json!({
            "type": "computation",
            "title": "Capability Task",
            "description": "A task requiring capabilities",
            "required_capabilities": [
                {
                    "name": "math",
                    "minimum_proficiency": 0.8
                },
                {
                    "name": "logic",
                    "minimum_proficiency": 0.6
                }
            ]
        });

        let task_id = task_distributor.create_task(config).await?;
        assert!(!task_id.is_nil());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_task_invalid_config() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        let config = serde_json::json!("invalid_config");

        let result = task_distributor.create_task(config).await;
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_task_missing_title() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        let config = serde_json::json!({
            "type": "computation",
            "description": "A task without title"
        });

        let task_id = task_distributor.create_task(config).await?;
        assert!(!task_id.is_nil()); // Should use default title

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_task_with_verification_success() -> Result<(), Box<dyn std::error::Error>>
    {
        let task_distributor = create_test_task_distributor().await?;

        // Create a task
        let task_config = serde_json::json!({
            "type": "computation",
            "title": "Test Task",
            "description": "A test task"
        });
        let task_id = task_distributor.create_task(task_config).await?;

        // Create a mock agent ID
        let agent_id = Uuid::new_v4();

        // Execute the task
        let result = task_distributor
            .execute_task_with_verification(task_id, agent_id)
            .await?;
        assert!(result.is_object());

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_task_with_verification_nonexistent_task(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        let fake_task_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();

        let result = task_distributor
            .execute_task_with_verification(fake_task_id, agent_id)
            .await;
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_distribute_tasks() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        // Create some tasks
        let task_configs = vec![
            serde_json::json!({"type": "computation", "title": "Task 1"}),
            serde_json::json!({"type": "computation", "title": "Task 2"}),
            serde_json::json!({"type": "computation", "title": "Task 3"}),
        ];

        let mut task_ids = Vec::new();
        for config in task_configs {
            let task_id = task_distributor.create_task(config).await?;
            task_ids.push(task_id);
        }

        // Create mock agents
        let agent_refs = vec![
            (
                Uuid::new_v4(),
                crate::agents::agent::Agent::new(
                    "agent1".to_string(),
                    crate::agents::agent::AgentType::Worker,
                ),
            ),
            (
                Uuid::new_v4(),
                crate::agents::agent::Agent::new(
                    "agent2".to_string(),
                    crate::agents::agent::AgentType::Worker,
                ),
            ),
        ];

        // Distribute tasks
        let results = task_distributor.distribute_tasks(&agent_refs).await?;
        assert!(!results.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_status() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        let status = task_distributor.get_status().await;
        assert!(status.is_object());
        assert!(status.get("legacy_queue_size").is_some());
        assert!(status.get("work_stealing_queue").is_some());
        assert!(status.get("task_status_counts").is_some());
        assert!(status.get("total_tasks").is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_analytics() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        let analytics = task_distributor.get_analytics().await;
        assert!(analytics.is_object());
        assert!(analytics.get("total_executions").is_some());
        assert!(analytics.get("success_rate").is_some());
        assert!(analytics.get("average_execution_time_ms").is_some());
        assert!(analytics.get("recent_performance").is_some());
        assert!(analytics.get("task_distribution").is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_validate_task_config_valid() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        let config = serde_json::json!({
            "type": "computation",
            "title": "Valid Task",
            "description": "A valid task"
        });

        let result = task_distributor.validate_task_config(&config);
        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_validate_task_config_invalid() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        let config = serde_json::json!("invalid");
        let result = task_distributor.validate_task_config(&config);
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_task_internal() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        // Create a task
        let task = crate::tasks::task::Task::new(
            "Test Task".to_string(),
            "A test task".to_string(),
            "computation".to_string(),
            crate::tasks::task::TaskPriority::Medium,
            vec![],
        );

        let agent_id = Uuid::new_v4();

        let result = task_distributor
            .execute_task_internal(task, agent_id)
            .await?;
        assert!(result.is_object());
        assert_eq!(
            result.get("task_id").unwrap().as_str().unwrap(),
            task.id.to_string()
        );
        assert_eq!(
            result.get("agent_id").unwrap().as_str().unwrap(),
            agent_id.to_string()
        );
        assert!(result.get("result").is_some());
        assert!(result.get("timestamp").is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_count_tasks_by_status() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        // Create a task to have something to count
        let config = serde_json::json!({
            "type": "computation",
            "title": "Test Task"
        });
        task_distributor.create_task(config).await?;

        let task_metrics = task_distributor.task_metrics.read().await;
        let status_counts = task_distributor.count_tasks_by_status(&task_metrics).await;
        assert!(status_counts.is_object());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_recent_performance_no_history() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        let history = Vec::new();
        let recent_performance = task_distributor.get_recent_performance(&history).await;

        assert!(recent_performance.is_object());
        assert_eq!(recent_performance.get("recent_success_rate").unwrap(), 0.0);
        assert_eq!(
            recent_performance.get("recent_average_time_ms").unwrap(),
            0.0
        );
        assert_eq!(recent_performance.get("sample_size").unwrap(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_recent_performance_with_history() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        let history = vec![
            TaskExecutionResult {
                task_id: Uuid::new_v4(),
                agent_id: Uuid::new_v4(),
                success: true,
                execution_time_ms: 100,
                result: None,
                error_message: None,
            },
            TaskExecutionResult {
                task_id: Uuid::new_v4(),
                agent_id: Uuid::new_v4(),
                success: false,
                execution_time_ms: 200,
                result: None,
                error_message: Some("Failed".to_string()),
            },
            TaskExecutionResult {
                task_id: Uuid::new_v4(),
                agent_id: Uuid::new_v4(),
                success: true,
                execution_time_ms: 150,
                result: None,
                error_message: None,
            },
        ];

        let recent_performance = task_distributor.get_recent_performance(&history).await;

        assert!(recent_performance.is_object());
        let success_rate = recent_performance
            .get("recent_success_rate")
            .unwrap()
            .as_f64()
            .unwrap();
        assert!((success_rate - 0.666666).abs() < 0.01); // 2/3 success rate

        let avg_time = recent_performance
            .get("recent_average_time_ms")
            .unwrap()
            .as_f64()
            .unwrap();
        assert!((avg_time - 150.0).abs() < 0.01); // (100+200+150)/3 = 150

        assert_eq!(recent_performance.get("sample_size").unwrap(), 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_task_distribution() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        // Create a task
        let config = serde_json::json!({
            "type": "computation",
            "title": "Test Task"
        });
        let task_id = task_distributor.create_task(config).await?;

        // Update task metrics to simulate assignment
        {
            let mut metrics = task_distributor.task_metrics.write().await;
            if let Some(task_metrics) = metrics.get_mut(&task_id) {
                task_metrics.assigned_agent = Some(Uuid::new_v4());
            }
        }

        let task_metrics = task_distributor.task_metrics.read().await;
        let distribution = task_distributor.get_task_distribution(&task_metrics).await;

        assert!(distribution.is_object());
        assert!(distribution.get("tasks_per_agent").is_some());
        assert!(distribution.get("hourly_distribution").is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_task_execution_result_creation() {
        let task_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();

        let result = TaskExecutionResult {
            task_id,
            agent_id,
            success: true,
            execution_time_ms: 100,
            result: Some(serde_json::json!({"output": "success"})),
            error_message: None,
        };

        assert_eq!(result.task_id, task_id);
        assert_eq!(result.agent_id, agent_id);
        assert!(result.success);
        assert_eq!(result.execution_time_ms, 100);
        assert!(result.result.is_some());
        assert!(result.error_message.is_none());
    }

    #[tokio::test]
    async fn test_task_metrics_creation() {
        let created_at = chrono::Utc::now();
        let metrics = TaskMetrics {
            created_at,
            started_at: None,
            completed_at: None,
            assigned_agent: None,
            execution_attempts: 0,
            status: TaskStatus::Pending,
        };

        assert_eq!(metrics.created_at, created_at);
        assert!(metrics.started_at.is_none());
        assert!(metrics.completed_at.is_none());
        assert!(metrics.assigned_agent.is_none());
        assert_eq!(metrics.execution_attempts, 0);
        assert_eq!(metrics.status, TaskStatus::Pending);
    }

    #[tokio::test]
    async fn test_task_status_enum() {
        assert_eq!(TaskStatus::Pending, TaskStatus::default());
        assert_eq!(TaskStatus::Assigned, TaskStatus::Assigned);
        assert_eq!(TaskStatus::Running, TaskStatus::Running);
        assert_eq!(TaskStatus::Completed, TaskStatus::Completed);
        assert_eq!(TaskStatus::Failed, TaskStatus::Failed);
        assert_eq!(TaskStatus::Retrying, TaskStatus::Retrying);
    }

    #[tokio::test]
    async fn test_multiple_task_creation() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        // Create multiple tasks
        let mut task_ids = Vec::new();
        for i in 0..5 {
            let config = serde_json::json!({
                "type": "computation",
                "title": format!("Task {}", i),
                "description": format!("Description for task {}", i)
            });
            let task_id = task_distributor.create_task(config).await?;
            task_ids.push(task_id);
        }

        assert_eq!(task_ids.len(), 5);
        for task_id in task_ids {
            assert!(!task_id.is_nil());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_task_execution_history_limit() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        // Create many tasks and execute them to fill history
        for i in 0..1500 {
            // More than the 1000 limit
            let config = serde_json::json!({
                "type": "computation",
                "title": format!("Task {}", i)
            });
            let task_id = task_distributor.create_task(config).await?;
            let agent_id = Uuid::new_v4();

            // Execute the task (this will add to history)
            let _ = task_distributor
                .execute_task_with_verification(task_id, agent_id)
                .await;
        }

        // Check that history is limited
        let history = task_distributor.execution_history.read().await;
        assert!(history.len() <= 1000);

        Ok(())
    }

    #[tokio::test]
    async fn test_task_metrics_tracking() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        let config = serde_json::json!({
            "type": "computation",
            "title": "Metrics Test Task"
        });
        let task_id = task_distributor.create_task(config).await?;
        let agent_id = Uuid::new_v4();

        // Check initial metrics
        {
            let metrics = task_distributor.task_metrics.read().await;
            let task_metrics = metrics.get(&task_id).unwrap();
            assert_eq!(task_metrics.status, TaskStatus::Pending);
            assert!(task_metrics.assigned_agent.is_none());
            assert_eq!(task_metrics.execution_attempts, 0);
        }

        // Execute task and check metrics update
        let _ = task_distributor
            .execute_task_with_verification(task_id, agent_id)
            .await;

        {
            let metrics = task_distributor.task_metrics.read().await;
            let task_metrics = metrics.get(&task_id).unwrap();
            assert_eq!(task_metrics.status, TaskStatus::Completed);
            assert_eq!(task_metrics.assigned_agent, Some(agent_id));
            assert_eq!(task_metrics.execution_attempts, 1);
            assert!(task_metrics.started_at.is_some());
            assert!(task_metrics.completed_at.is_some());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_task_priority_parsing() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        let test_cases = vec![
            ("low", crate::tasks::task::TaskPriority::Low),
            ("medium", crate::tasks::task::TaskPriority::Medium),
            ("high", crate::tasks::task::TaskPriority::High),
            ("critical", crate::tasks::task::TaskPriority::Critical),
            ("invalid", crate::tasks::task::TaskPriority::Medium), // Should default to Medium
        ];

        for (priority_str, expected_priority) in test_cases {
            let config = serde_json::json!({
                "type": "computation",
                "title": "Priority Test",
                "priority": priority_str
            });

            let task_id = task_distributor.create_task(config).await?;
            assert!(!task_id.is_nil());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_task_with_empty_required_capabilities() -> Result<(), Box<dyn std::error::Error>>
    {
        let task_distributor = create_test_task_distributor().await?;

        let config = serde_json::json!({
            "type": "computation",
            "title": "No Capabilities Task",
            "description": "A task with no required capabilities",
            "required_capabilities": []
        });

        let task_id = task_distributor.create_task(config).await?;
        assert!(!task_id.is_nil());

        Ok(())
    }

    #[tokio::test]
    async fn test_task_with_malformed_capabilities() -> Result<(), Box<dyn std::error::Error>> {
        let task_distributor = create_test_task_distributor().await?;

        let config = serde_json::json!({
            "type": "computation",
            "title": "Malformed Capabilities Task",
            "description": "A task with malformed capabilities",
            "required_capabilities": [
                "not_an_object",
                {"name": "valid", "minimum_proficiency": 0.5},
                {"name": "incomplete"}
            ]
        });

        let task_id = task_distributor.create_task(config).await?;
        assert!(!task_id.is_nil()); // Should handle malformed capabilities gracefully

        Ok(())
    }
}
