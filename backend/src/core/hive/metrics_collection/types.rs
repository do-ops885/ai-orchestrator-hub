//! # Metrics Collection Types
//!
//! This module defines all the data structures used for metrics collection
//! and reporting in the hive system.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Comprehensive metrics for the hive system
///
/// Aggregates all system metrics into a single, serializable structure
/// for monitoring, analysis, and external system integration.
///
/// ## Metric Categories
///
/// - **Agent Metrics**: Agent counts, performance, and activity
/// - **Task Metrics**: Task execution statistics and success rates
/// - **System Metrics**: CPU, memory, uptime, and performance indicators
/// - **Resource Metrics**: Hardware resource utilization and availability
/// - **Timestamps**: When metrics were last updated
///
/// ## Serialization
///
/// Supports both JSON and Prometheus export formats for different
/// monitoring and alerting systems.
///
/// ## Performance
///
/// Designed for efficient serialization and minimal memory overhead.
/// All fields are optional or have sensible defaults.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HiveMetrics {
    /// Agent-related metrics
    ///
    /// Statistics about agent registration, activity, and performance
    /// including counts, success rates, and efficiency metrics.
    pub agent_metrics: AgentMetrics,

    /// Task-related metrics
    ///
    /// Statistics about task creation, execution, completion, and failure
    /// rates with timing and performance analysis.
    pub task_metrics: TaskMetrics,

    /// System performance metrics
    ///
    /// Overall system health indicators including uptime, CPU usage,
    /// memory consumption, and response times.
    pub system_metrics: SystemMetrics,

    /// Resource utilization metrics
    ///
    /// Hardware resource usage statistics including CPU cores,
    /// memory availability, disk usage, and network connections.
    pub resource_metrics: ResourceMetrics,

    /// Timestamp of last update
    ///
    /// When these metrics were last updated. Used for staleness detection
    /// and time-based analysis.
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Swarm metrics for persistence compatibility
///
/// Legacy metrics structure maintained for backward compatibility
/// with existing persistence and monitoring systems.
///
/// ## Metrics Included
///
/// - Total and active agent counts
/// - Task completion and failure statistics
/// - Average performance scores
/// - Swarm cohesion and learning progress
/// - System uptime tracking
///
/// ## Usage
///
/// Primarily used for data persistence and external system integration.
/// New code should prefer `HiveMetrics` for comprehensive analysis.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwarmMetrics {
    /// Total number of agents in the swarm
    pub total_agents: usize,
    /// Number of currently active agents
    pub active_agents: usize,
    /// Total tasks completed by the swarm
    pub completed_tasks: usize,
    /// Total tasks that failed execution
    pub failed_tasks: usize,
    /// Average performance score across all agents
    pub average_performance: f64,
    /// Measure of swarm coordination and communication effectiveness
    pub swarm_cohesion: f64,
    /// Progress in learning and adaptation (0.0 to 1.0)
    pub learning_progress: f64,
    /// System uptime in seconds
    pub uptime_seconds: u64,
}

/// Agent performance and activity metrics
///
/// Detailed statistics about agent behavior, performance, and lifecycle
/// events in the hive system.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentMetrics {
    /// Total number of agents ever registered
    pub total_agents: u64,
    /// Number of currently active agents
    pub active_agents: u64,
    /// Number of agents created in the current day
    pub agents_created_today: u64,
    /// Number of agents removed in the current day
    pub agents_removed_today: u64,
    /// Average performance score across all agents
    pub average_agent_performance: f64,
    /// ID of the highest-performing agent
    pub top_performer_id: Option<Uuid>,
}

/// Task execution and performance metrics
///
/// Statistics about task creation, execution, completion, and failure
/// patterns in the system.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskMetrics {
    /// Total number of tasks ever created
    pub total_tasks: u64,
    /// Number of tasks completed successfully
    pub completed_tasks: u64,
    /// Number of tasks that failed execution
    pub failed_tasks: u64,
    /// Number of tasks currently pending execution
    pub pending_tasks: u64,
    /// Average execution time in milliseconds
    pub average_execution_time_ms: f64,
    /// Rate of task completion per hour
    pub tasks_per_hour: f64,
    /// Overall task success rate (0.0 to 1.0)
    pub success_rate: f64,
}

/// System performance and health metrics
///
/// Overall system health indicators including resource usage,
/// uptime, and performance statistics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemMetrics {
    /// System uptime in seconds
    pub uptime_seconds: u64,
    /// Total memory usage in megabytes
    pub total_memory_usage_mb: f64,
    /// CPU usage as a percentage (0.0 to 100.0)
    pub cpu_usage_percent: f64,
    /// Network throughput in megabits per second
    pub network_throughput_mbps: f64,
    /// System error rate (0.0 to 1.0)
    pub error_rate: f64,
    /// Average response time in milliseconds
    pub response_time_ms: f64,
}

/// Hardware resource utilization metrics
///
/// Statistics about system hardware resource availability and usage
/// for capacity planning and performance optimization.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceMetrics {
    /// Number of available CPU cores
    pub available_cpu_cores: u32,
    /// Total system memory in megabytes
    pub memory_total_mb: f64,
    /// Available memory in megabytes
    pub memory_available_mb: f64,
    /// Disk usage as a percentage (0.0 to 100.0)
    pub disk_usage_percent: f64,
    /// Number of active network connections
    pub network_connections: u32,
}
