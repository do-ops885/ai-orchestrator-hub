//! Task Management Types and Structures
//!
//! Common types, enums, and data structures used throughout the task management system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskMetrics {
    /// Timestamp when the task was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when task execution started
    pub started_at: Option<DateTime<Utc>>,
    /// Timestamp when task execution completed
    pub completed_at: Option<DateTime<Utc>>,
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
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// Task distribution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDistributionConfig {
    /// Maximum number of tasks to process concurrently
    pub max_concurrent_tasks: usize,
    /// Maximum number of retry attempts for failed tasks
    pub max_retry_attempts: u32,
    /// Timeout for task execution in milliseconds
    pub execution_timeout_ms: u64,
    /// Enable work-stealing queue optimization
    pub enable_work_stealing: bool,
    /// Maximum queue size before applying backpressure
    pub max_queue_size: usize,
}

impl Default for TaskDistributionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 100,
            max_retry_attempts: 3,
            execution_timeout_ms: 300_000, // 5 minutes
            enable_work_stealing: true,
            max_queue_size: 10_000,
        }
    }
}

/// Task assignment criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignmentCriteria {
    /// Required agent capabilities
    pub required_capabilities: Vec<String>,
    /// Minimum agent performance score
    pub min_performance_score: f64,
    /// Preferred agent types
    pub preferred_agent_types: Vec<String>,
    /// Maximum agent workload
    pub max_agent_workload: u32,
}

impl Default for TaskAssignmentCriteria {
    fn default() -> Self {
        Self {
            required_capabilities: Vec::new(),
            min_performance_score: 0.0,
            preferred_agent_types: Vec::new(),
            max_agent_workload: 10,
        }
    }
}

/// Task performance analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPerformanceAnalytics {
    /// Total number of tasks processed
    pub total_tasks: u64,
    /// Number of successful task executions
    pub successful_tasks: u64,
    /// Number of failed task executions
    pub failed_tasks: u64,
    /// Average execution time in milliseconds
    pub average_execution_time_ms: f64,
    /// Success rate as a percentage (0.0 to 1.0)
    pub success_rate: f64,
    /// Tasks processed per second
    pub throughput: f64,
    /// Current queue size
    pub current_queue_size: usize,
    /// Peak queue size in the current period
    pub peak_queue_size: usize,
}

impl Default for TaskPerformanceAnalytics {
    fn default() -> Self {
        Self {
            total_tasks: 0,
            successful_tasks: 0,
            failed_tasks: 0,
            average_execution_time_ms: 0.0,
            success_rate: 0.0,
            throughput: 0.0,
            current_queue_size: 0,
            peak_queue_size: 0,
        }
    }
}

/// Task queue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskQueueStats {
    /// Number of pending tasks
    pub pending_tasks: usize,
    /// Number of assigned tasks
    pub assigned_tasks: usize,
    /// Number of running tasks
    pub running_tasks: usize,
    /// Number of completed tasks
    pub completed_tasks: usize,
    /// Number of failed tasks
    pub failed_tasks: usize,
    /// Number of retrying tasks
    pub retrying_tasks: usize,
    /// Total queue capacity
    pub total_capacity: usize,
    /// Queue utilization percentage
    pub utilization_percentage: f64,
    /// Peak queue size observed
    pub peak_queue_size: usize,
}

impl Default for TaskQueueStats {
    fn default() -> Self {
        Self {
            pending_tasks: 0,
            assigned_tasks: 0,
            running_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            retrying_tasks: 0,
            total_capacity: 0,
            utilization_percentage: 0.0,
            peak_queue_size: 0,
        }
    }
}
