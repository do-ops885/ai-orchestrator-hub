//! # Agent Management Types
//!
//! This module defines the core data structures used in agent management
//! including registration results and performance metrics.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Result of agent registration
///
/// Contains the outcome of an agent registration attempt with
/// detailed information for success or failure cases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistrationResult {
    /// Unique identifier of the registered agent
    pub agent_id: Uuid,
    /// Whether the registration was successful
    pub success: bool,
    /// Human-readable message describing the result
    pub message: String,
}

/// Performance metrics for individual agents
///
/// Comprehensive performance tracking data for each agent including
/// task completion statistics, execution times, and efficiency metrics.
///
/// ## Metrics Tracked
///
/// - Task completion and failure counts
/// - Execution time statistics (total, average, last activity)
/// - Performance scoring based on success rate and speed
/// - Activity timestamps for monitoring
///
/// ## Performance Score Calculation
///
/// Performance score combines success rate with execution speed:
/// `score = success_rate * min(2.0, 1000.0 / average_execution_time_ms)`
///
/// Higher scores indicate better performance with a maximum speed bonus cap.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentMetrics {
    /// Number of tasks completed successfully by this agent
    pub tasks_completed: u64,
    /// Number of tasks that failed during execution
    pub tasks_failed: u64,
    /// Total execution time across all tasks in milliseconds
    pub total_execution_time_ms: u64,
    /// Average execution time per task in milliseconds
    pub average_execution_time_ms: f64,
    /// Timestamp of the last activity performed by this agent
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
    /// Overall performance score (0.0 to 2.0+)
    pub performance_score: f64,
}
