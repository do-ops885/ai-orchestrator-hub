//! # Coordinator Lifecycle Management
//!
//! This module handles the startup and shutdown operations for the HiveCoordinator.
//! It manages the lifecycle of background processes and coordination loops.

use super::core::HiveCoordinator;
use super::messages::CoordinationMessage;
use crate::utils::error::HiveResult;
use std::sync::Arc;

impl HiveCoordinator {
    /// Start the hive coordinator and all background processes.
    ///
    /// This method activates all subsystems and begins processing coordination messages.
    /// It starts background processes for work stealing, learning cycles, metrics collection,
    /// and resource monitoring. The coordinator becomes fully operational after this call.
    ///
    /// ## Process Lifecycle
    ///
    /// 1. Starts all background processes managed by `ProcessManager`
    /// 2. Launches coordination message processing loop
    /// 3. System becomes ready to accept agent and task operations
    ///
    /// ## Performance Impact
    ///
    /// Creates several background tasks with configurable intervals.
    /// Memory usage increases by ~50-100KB for background task stacks.
    ///
    /// ## Error Handling
    ///
    /// If any background process fails to start, the entire operation fails
    /// and the coordinator should be recreated.
    pub async fn start(&self) -> HiveResult<()> {
        // Start background processes
        self.process_manager
            .start_all_processes(
                &self.agent_manager,
                &self.task_distributor,
                &self.metrics_collector,
                &self.resource_manager,
            )
            .await?;

        // Start coordination message processing
        self.start_coordination_loop().await?;

        tracing::info!("HiveCoordinator {} started successfully", self.id);
        Ok(())
    }

    /// Gracefully shutdown the hive coordinator.
    ///
    /// Performs a clean shutdown of all subsystems in the correct order.
    /// Ensures all pending operations complete, resources are cleaned up,
    /// and background processes are terminated gracefully.
    ///
    /// ## Shutdown Sequence
    ///
    /// 1. Sends shutdown signal to all subsystems
    /// 2. Waits for pending tasks to complete or timeout
    /// 3. Stops background processes and monitoring
    /// 4. Cleans up resources and communication channels
    /// 5. Updates final metrics and logs shutdown
    pub async fn shutdown(&self) -> HiveResult<()> {
        // Send shutdown signal
        if let Err(e) = self.coordination_tx.send(CoordinationMessage::Shutdown) {
            tracing::warn!("Failed to send shutdown signal: {}", e);
        }

        // Wait for background processes to finish
        self.process_manager.stop_all_processes().await?;

        tracing::info!("HiveCoordinator shutdown complete");
        Ok(())
    }

    /// Start the coordination message processing loop.
    ///
    /// Launches a background task that continuously processes coordination
    /// messages from all subsystems. This enables real-time communication
    /// and coordination between different parts of the hive system.
    ///
    /// ## Message Processing
    ///
    /// Handles various message types:
    /// - Agent registration/removal notifications
    /// - Task completion events
    /// - Metrics updates
    /// - Resource alerts
    /// - Shutdown signals
    ///
    /// ## Performance
    ///
    /// Runs as a dedicated background task with minimal CPU overhead.
    /// Processes messages asynchronously to avoid blocking operations.
    ///
    /// ## Error Handling
    ///
    /// Individual message processing errors are logged but don't stop
    /// the coordination loop. Critical errors may trigger system alerts.
    pub(super) async fn start_coordination_loop(&self) -> HiveResult<()> {
        let mut rx = {
            let mut rx_guard = self.coordination_rx.write().await;
            rx_guard
                .take()
                .ok_or_else(|| crate::utils::error::HiveError::OperationFailed {
                    reason: "Coordination receiver already taken".to_string(),
                })?
        };

        let metrics_collector = self.metrics_collector.clone();
        let _agent_manager = Arc::clone(&self.agent_manager);
        let _task_distributor = Arc::clone(&self.task_distributor);

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                match message {
                    CoordinationMessage::AgentRegistered { agent_id } => {
                        tracing::debug!("Agent {} registered", agent_id);
                        metrics_collector
                            .record_agent_event("registered", agent_id)
                            .await;
                    }
                    CoordinationMessage::AgentRemoved { agent_id } => {
                        tracing::debug!("Agent {} removed", agent_id);
                        metrics_collector
                            .record_agent_event("removed", agent_id)
                            .await;
                    }
                    CoordinationMessage::TaskCompleted {
                        task_id,
                        agent_id,
                        success,
                    } => {
                        tracing::debug!(
                            "Task {} completed by agent {} (success: {})",
                            task_id,
                            agent_id,
                            success
                        );
                        metrics_collector
                            .record_task_completion(task_id, agent_id, success)
                            .await;
                    }
                    CoordinationMessage::MetricsUpdate { metrics } => {
                        metrics_collector.update_metrics(metrics).await;
                    }
                    CoordinationMessage::ResourceAlert { resource, usage } => {
                        tracing::warn!(
                            "Resource alert: {} usage at {:.1}%",
                            resource,
                            usage * 100.0
                        );
                        // Could trigger auto-scaling or other responses
                    }
                    CoordinationMessage::Shutdown => {
                        tracing::info!("Coordination loop shutting down");
                        break;
                    }
                }
            }
        });

        Ok(())
    }
}
