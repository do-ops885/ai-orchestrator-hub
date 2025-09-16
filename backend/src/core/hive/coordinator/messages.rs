//! # Coordination Messages
//!
//! This module defines all coordination message types and their handling logic.
//! Messages enable loose coupling between subsystems while maintaining real-time coordination.

use serde_json;
use uuid::Uuid;

/// Messages for internal coordination between subsystems
///
/// This enum defines all possible messages that can be passed between
/// the coordinator and its subsystems. Each message type corresponds
/// to a specific event or state change that requires coordination.
///
/// ## Message Flow
///
/// Messages are sent asynchronously through mpsc channels and processed
/// by background tasks. This allows for loose coupling between subsystems
/// while maintaining real-time coordination.
///
/// ## Performance
///
/// Messages are designed to be lightweight and copy-efficient.
/// Complex data is passed by reference where possible to minimize overhead.
#[derive(Debug, Clone)]
pub enum CoordinationMessage {
    /// Agent registration notification
    ///
    /// Sent when a new agent is successfully registered with the system.
    /// Triggers metrics updates and resource allocation adjustments.
    AgentRegistered { agent_id: Uuid },

    /// Agent removal notification
    ///
    /// Sent when an agent is removed from the system.
    /// Triggers cleanup operations and resource reallocation.
    AgentRemoved { agent_id: Uuid },

    /// Task completion notification
    ///
    /// Sent when a task execution completes, either successfully or with failure.
    /// Includes execution details for performance tracking and analytics.
    TaskCompleted {
        task_id: Uuid,
        agent_id: Uuid,
        success: bool,
    },

    /// System metrics update
    ///
    /// Periodic metrics update from monitoring subsystems.
    /// Contains current system state information for dashboard and alerting.
    MetricsUpdate { metrics: serde_json::Value },

    /// Resource threshold alert
    ///
    /// Sent when resource usage exceeds configured thresholds.
    /// May trigger auto-scaling or resource optimization actions.
    ResourceAlert { resource: String, usage: f64 },

    /// Shutdown signal
    ///
    /// System-wide shutdown command that gracefully stops all operations.
    /// Ensures clean shutdown of all subsystems and proper resource cleanup.
    Shutdown,
}
