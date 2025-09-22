//! # Task Management Types
//!
//! This module defines the core data structures used in task management
//! including execution results and task metrics.

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
/// Comprehensive performance tracking data for each task including
/// execution statistics, timing information, and status tracking.
///
/// ## Metrics Tracked
///
/// - Execution attempts and success/failure counts
/// - Timing statistics (creation, assignment, completion)
/// - Agent assignment history
/// - Performance scoring based on execution efficiency
/// - Status transitions and lifecycle events
///
/// ## Performance Score Calculation
///
/// Performance score combines execution speed with reliability:
/// `score = (1.0 / (1.0 + execution_time_ms / 1000.0)) * success_rate`
///
/// Higher scores indicate better performance with faster, more reliable execution.
#[derive(Debug, Clone, Default)]
pub struct TaskMetrics {
    /// Number of times this task has been attempted
    pub execution_attempts: u32,
    /// Number of times this task has failed
    pub failure_count: u32,
    /// Total time spent executing this task in milliseconds
    pub total_execution_time_ms: u64,
    /// Average execution time per attempt in milliseconds
    pub average_execution_time_ms: f64,
    /// Timestamp when the task was created
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Timestamp when the task was first assigned to an agent
    pub assigned_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Timestamp when the task was completed
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Current status of the task
    pub status: TaskStatus,
    /// Overall performance score (0.0 to 1.0)
    pub performance_score: f64,
}

/// Status of a task in its lifecycle
///
/// Represents the current state of a task as it moves through
/// the execution pipeline from creation to completion.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Default)]
pub enum TaskStatus {
    /// Task has been created but not yet assigned
    #[default]
    Pending,
    /// Task has been assigned to an agent but not yet started
    Assigned,
    /// Task is currently being executed by an agent
    Running,
    /// Task has completed successfully
    Completed,
    /// Task has failed and may be retried
    Failed,
    /// Task has been cancelled
    Cancelled,
}

