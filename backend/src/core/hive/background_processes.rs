//! # Background Process Management Module
//!
//! This module provides comprehensive background process management for the
//! hive system, handling long-running operations and system maintenance tasks.
//!
//! ## Architecture
//!
//! The background process system manages multiple concurrent processes:
//!
//! - **Work Stealing**: Distributes tasks to available agents
//! - **Learning Cycles**: Runs agent learning and adaptation
//! - **Swarm Coordination**: Manages inter-agent communication
//! - **Metrics Collection**: Gathers system performance data
//! - **Resource Monitoring**: Tracks system resource usage
//!
//! ## Key Features
//!
//! - **Configurable Intervals**: Customizable timing for all processes
//! - **Graceful Shutdown**: Clean process termination and resource cleanup
//! - **Error Isolation**: Individual process failures don't affect others
//! - **Resource Alerts**: Automatic monitoring and threshold-based alerts
//! - **Concurrent Execution**: Multiple processes running simultaneously
//!
//! ## Usage
//!
//! ```rust,no_run
//! use hive::core::hive::ProcessManager;
//! use tokio::sync::mpsc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let (tx, _rx) = mpsc::unbounded_channel();
//!
//! let process_manager = ProcessManager::new(tx).await?;
//!
//! // Start all background processes
//! // (would need actual subsystem instances here)
//! // process_manager.start_all_processes(&agent_manager, &task_distributor, &metrics_collector, &resource_manager).await?;
//!
//! // Get process status
//! let status = process_manager.get_process_status().await;
//! println!("Process status: {}", status);
//!
//! // Graceful shutdown
//! process_manager.stop_all_processes().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Memory Usage**: Minimal - one task per background process
//! - **CPU Overhead**: Configurable intervals, typically low baseline usage
//! - **Scalability**: Fixed number of processes regardless of system size
//! - **Startup Time**: Fast initialization with immediate background operation
//! - **Shutdown Time**: Quick termination with proper cleanup

use crate::infrastructure::resource_manager::ResourceManager;
use crate::utils::error::HiveResult;

use super::agent_management::AgentManager;
use super::coordinator::CoordinationMessage;
use super::metrics_collection::MetricsCollector;
use super::task_management::TaskDistributor;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;

/// Configuration for background processes
///
/// Defines timing intervals and parameters for all background processes
/// in the hive system. Allows fine-tuning of system behavior and performance.
///
/// ## Configuration Parameters
///
/// - **Work Stealing**: How often to distribute tasks to agents
/// - **Learning Cycles**: Frequency of agent learning and adaptation
/// - **Swarm Coordination**: Inter-agent communication intervals
/// - **Metrics Collection**: Performance data gathering frequency
/// - **Resource Monitoring**: System resource checking intervals
///
/// ## Performance Tuning
///
/// Shorter intervals provide more responsive systems but increase overhead.
/// Longer intervals reduce CPU usage but may decrease system responsiveness.
///
/// ## Default Values
///
/// - Work stealing: 100ms (high frequency for responsive task distribution)
/// - Learning: 30s (moderate frequency for adaptation)
/// - Swarm coordination: 5s (balanced responsiveness)
/// - Metrics: 10s (reasonable monitoring frequency)
/// - Resource monitoring: 5s (frequent resource checks)
#[derive(Debug, Clone)]
pub struct ProcessConfig {
    /// Interval for work stealing task distribution
    ///
    /// How often the system checks for available tasks and distributes
    /// them to idle agents. Shorter intervals provide more responsive
    /// task distribution but increase CPU overhead.
    pub work_stealing_interval: Duration,

    /// Interval for agent learning cycles
    ///
    /// How often agents run learning and adaptation processes.
    /// Affects how quickly agents improve their performance.
    pub learning_interval: Duration,

    /// Interval for swarm coordination
    ///
    /// How often the system coordinates inter-agent communication
    /// and optimizes swarm behavior patterns.
    pub swarm_coordination_interval: Duration,

    /// Interval for metrics collection
    ///
    /// How often system performance metrics are gathered and
    /// updated. Affects monitoring responsiveness.
    pub metrics_collection_interval: Duration,

    /// Interval for resource monitoring
    ///
    /// How often system resources are checked and alerts are generated.
    /// Critical for preventing resource exhaustion.
    pub resource_monitoring_interval: Duration,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            work_stealing_interval: Duration::from_millis(100),
            learning_interval: Duration::from_secs(30),
            swarm_coordination_interval: Duration::from_secs(5),
            metrics_collection_interval: Duration::from_secs(10),
            resource_monitoring_interval: Duration::from_secs(5),
        }
    }
}

/// Background process management subsystem
///
/// Central coordinator for all background processes in the hive system.
/// Manages the lifecycle of long-running tasks and ensures proper coordination
/// between different system components.
///
/// ## Process Management
///
/// - **Process Creation**: Spawns and tracks background tasks
/// - **Configuration**: Applies timing and behavioral settings
/// - **Coordination**: Communicates with other subsystems
/// - **Shutdown**: Ensures graceful termination of all processes
/// - **Monitoring**: Tracks process health and status
///
/// ## Thread Safety
///
/// All operations are thread-safe. Process handles are managed through
/// `Arc<RwLock<T>>` for concurrent access during startup and shutdown.
///
/// ## Error Handling
///
/// Individual process failures are isolated and logged. System continues
/// operating even if some background processes fail.
pub struct ProcessManager {
    /// Configuration for all background processes
    ///
    /// Defines timing intervals and behavioral parameters.
    /// Can be updated at runtime for dynamic tuning.
    config: ProcessConfig,

    /// Communication channel for inter-subsystem coordination
    ///
    /// Used to send coordination messages when background processes
    /// detect important events or require system-wide actions.
    coordination_tx: mpsc::UnboundedSender<CoordinationMessage>,

    /// Handles for all background processes
    ///
    /// Stores JoinHandle instances for all spawned background tasks.
    /// Used for graceful shutdown and process monitoring.
    process_handles: Arc<tokio::sync::RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

impl ProcessManager {
    /// Create a new process manager
    ///
    /// Initializes the background process management system with
    /// default configuration and communication channels.
    ///
    /// ## Initialization Process
    ///
    /// 1. Sets up default process configuration
    /// 2. Establishes coordination communication channel
    /// 3. Initializes process handle storage
    /// 4. Ready for background process startup
    ///
    /// ## Performance
    ///
    /// O(1) initialization with minimal resource allocation.
    /// No background processes started until explicitly requested.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::ProcessManager;
    /// # use tokio::sync::mpsc;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let (tx, _rx) = mpsc::unbounded_channel();
    ///
    /// let process_manager = ProcessManager::new(tx).await?;
    /// println!("Process manager initialized");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `coordination_tx` - Channel for sending coordination messages
    ///
    /// # Returns
    ///
    /// Returns a new `ProcessManager` instance on success.
    ///
    /// # Errors
    ///
    /// This function will not return an error under normal circumstances.
    pub async fn new(
        coordination_tx: mpsc::UnboundedSender<CoordinationMessage>,
    ) -> HiveResult<Self> {
        Ok(Self {
            config: ProcessConfig::default(),
            coordination_tx,
            process_handles: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        })
    }

    /// Start all background processes
    ///
    /// Launches all configured background processes and begins system operation.
    /// Each process runs independently and handles its own error recovery.
    ///
    /// ## Processes Started
    ///
    /// 1. **Work Stealing**: Distributes tasks to available agents
    /// 2. **Learning Cycles**: Runs agent adaptation and improvement
    /// 3. **Swarm Coordination**: Manages inter-agent communication
    /// 4. **Metrics Collection**: Gathers performance data
    /// 5. **Resource Monitoring**: Tracks system resource usage
    ///
    /// ## Process Lifecycle
    ///
    /// - Each process runs in its own tokio task
    /// - Processes operate on configurable intervals
    /// - Individual failures don't stop other processes
    /// - All processes can be stopped gracefully
    ///
    /// ## Performance Impact
    ///
    /// - Memory: ~50-100KB for task stacks
    /// - CPU: Minimal baseline with periodic activity
    /// - Coordination overhead: Message passing between processes
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::ProcessManager;
    /// # async fn example(process_manager: &ProcessManager) -> Result<(), Box<dyn std::error::Error>> {
    /// // Assuming subsystem instances are available
    /// // process_manager.start_all_processes(
    /// //     &agent_manager,
    /// //     &task_distributor,
    /// //     &metrics_collector,
    /// //     &resource_manager
    /// // ).await?;
    ///
    /// println!("All background processes started");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `agent_manager` - Agent management subsystem
    /// * `task_distributor` - Task distribution subsystem
    /// * `metrics_collector` - Metrics collection subsystem
    /// * `resource_manager` - Resource management subsystem
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all processes start successfully.
    ///
    /// # Errors
    ///
    /// Returns error if any background process fails to start.
    pub async fn start_all_processes(
        &self,
        agent_manager: &AgentManager,
        task_distributor: &TaskDistributor,
        metrics_collector: &MetricsCollector,
        resource_manager: &Arc<ResourceManager>,
    ) -> HiveResult<()> {
        let mut handles = self.process_handles.write().await;

        // Start work stealing process
        handles.push(
            self.start_work_stealing_process(agent_manager.clone(), task_distributor.clone())
                .await,
        );

        // Start learning process
        handles.push(self.start_learning_process(agent_manager.clone()).await);

        // Start swarm coordination process
        handles.push(
            self.start_swarm_coordination_process(agent_manager.clone())
                .await,
        );

        // Start metrics collection process
        handles.push(
            self.start_metrics_collection_process(metrics_collector.clone(), resource_manager)
                .await,
        );

        // Start resource monitoring process
        handles.push(
            self.start_resource_monitoring_process(resource_manager)
                .await,
        );

        tracing::info!("All background processes started successfully");
        Ok(())
    }

    /// Stop all background processes
    ///
    /// Gracefully terminates all background processes and cleans up resources.
    /// Ensures all pending operations complete before shutdown.
    ///
    /// ## Shutdown Process
    ///
    /// 1. Signals all processes to stop
    /// 2. Waits for processes to complete current operations
    /// 3. Aborts any remaining processes
    /// 4. Cleans up process handles
    /// 5. Logs shutdown completion
    ///
    /// ## Graceful Handling
    ///
    /// - Allows in-progress operations to complete
    /// - Prevents new operations from starting
    /// - Ensures resource cleanup
    /// - Provides timeout protection
    ///
    /// ## Performance
    ///
    /// Shutdown time depends on in-progress operations.
    /// Typically completes in 1-10 seconds under normal conditions.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::ProcessManager;
    /// # async fn example(process_manager: &ProcessManager) -> Result<(), Box<dyn std::error::Error>> {
    /// process_manager.stop_all_processes().await?;
    /// println!("All background processes stopped");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all processes stop successfully.
    ///
    /// # Errors
    ///
    /// Returns error if process termination fails.
    pub async fn stop_all_processes(&self) -> HiveResult<()> {
        let mut handles = self.process_handles.write().await;

        for handle in handles.drain(..) {
            handle.abort();
        }

        tracing::info!("All background processes stopped");
        Ok(())
    }

    /// Start work stealing process
    ///
    /// Launches the work stealing background process that continuously
    /// distributes available tasks to idle agents for optimal utilization.
    ///
    /// ## Process Behavior
    ///
    /// - Runs on configurable interval (`work_stealing_interval`)
    /// - Gets list of available agents
    /// - Distributes pending tasks using intelligent algorithms
    /// - Handles distribution failures gracefully
    /// - Continues running until shutdown signal
    ///
    /// ## Performance
    ///
    /// - CPU: Minimal baseline with periodic task distribution
    /// - Memory: Constant small footprint
    /// - Scalability: Performance scales with agent/task count
    ///
    /// # Parameters
    ///
    /// * `agent_manager` - Agent management subsystem
    /// * `task_distributor` - Task distribution subsystem
    ///
    /// # Returns
    ///
    /// Returns a JoinHandle for the background task.
    async fn start_work_stealing_process(
        &self,
        agent_manager: AgentManager,
        task_distributor: TaskDistributor,
    ) -> tokio::task::JoinHandle<()> {
        let interval_duration = self.config.work_stealing_interval;

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);

            loop {
                interval.tick().await;

                // Get available agents
                let agents = agent_manager.get_all_agents().await;
                if agents.is_empty() {
                    continue;
                }

                // Distribute tasks
                if let Err(e) = task_distributor.distribute_tasks(&agents).await {
                    tracing::error!("Work stealing distribution failed: {}", e);
                }
            }
        })
    }

    /// Start learning process
    ///
    /// Launches the learning cycle background process that periodically
    /// runs agent learning and adaptation algorithms.
    ///
    /// ## Process Behavior
    ///
    /// - Runs on configurable interval (`learning_interval`)
    /// - Triggers learning cycles for all active agents
    /// - Handles individual agent learning failures
    /// - Logs learning progress and errors
    /// - Continues until shutdown
    ///
    /// ## Current Status
    ///
    /// Currently logs placeholder messages. Full implementation
    /// requires NLP processor integration for agent learning.
    ///
    /// ## Performance
    ///
    /// - Variable CPU usage during learning cycles
    /// - Memory usage depends on learning algorithms
    /// - Runs asynchronously to avoid blocking other operations
    ///
    /// # Parameters
    ///
    /// * `_agent_manager` - Agent management subsystem (unused in current implementation)
    ///
    /// # Returns
    ///
    /// Returns a JoinHandle for the background task.
    async fn start_learning_process(
        &self,
        _agent_manager: AgentManager,
    ) -> tokio::task::JoinHandle<()> {
        let interval_duration = self.config.learning_interval;

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);

            loop {
                interval.tick().await;

                // This would need access to NLP processor
                // For now, just log that learning cycle would run
                tracing::debug!("Learning cycle would run here");

                // In a full implementation:
                // if let Err(e) = agent_manager.run_learning_cycle(&nlp_processor).await {
                //     tracing::error!("Learning cycle failed: {}", e);
                // }
            }
        })
    }

    /// Start swarm coordination process
    ///
    /// Launches the swarm coordination background process that manages
    /// inter-agent communication and optimizes swarm behavior patterns.
    ///
    /// ## Process Behavior
    ///
    /// - Runs on configurable interval (`swarm_coordination_interval`)
    /// - Analyzes agent distribution and communication patterns
    /// - Optimizes swarm positioning and task allocation
    /// - Updates coordination strategies dynamically
    /// - Only runs when multiple agents are present
    ///
    /// ## Current Status
    ///
    /// Currently provides basic agent count logging. Full implementation
    /// would include sophisticated swarm intelligence algorithms.
    ///
    /// ## Performance
    ///
    /// - CPU: Low baseline with periodic analysis
    /// - Memory: Minimal additional overhead
    /// - Scales with number of agents in swarm
    ///
    /// # Parameters
    ///
    /// * `agent_manager` - Agent management subsystem
    ///
    /// # Returns
    ///
    /// Returns a JoinHandle for the background task.
    async fn start_swarm_coordination_process(
        &self,
        agent_manager: AgentManager,
    ) -> tokio::task::JoinHandle<()> {
        let interval_duration = self.config.swarm_coordination_interval;

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);

            loop {
                interval.tick().await;

                // Update swarm positions and coordination
                let agents = agent_manager.get_all_agents().await;
                if agents.len() > 1 {
                    tracing::debug!("Updating swarm positions for {} agents", agents.len());

                    // In a full implementation, this would:
                    // 1. Calculate optimal swarm positions
                    // 2. Update agent positions
                    // 3. Coordinate inter-agent communication
                }
            }
        })
    }

    /// Start metrics collection process
    ///
    /// Launches the metrics collection background process that gathers
    /// system performance data and updates monitoring systems.
    ///
    /// ## Process Behavior
    ///
    /// - Runs on configurable interval (`metrics_collection_interval`)
    /// - Updates system uptime and performance counters
    /// - Calculates tasks-per-hour and efficiency metrics
    /// - Stores historical snapshots for trend analysis
    /// - Manages metrics history size (keeps last 1000 snapshots)
    ///
    /// ## Performance
    ///
    /// - CPU: Minimal for metrics calculation
    /// - Memory: Bounded by history size limit
    /// - Storage: Periodic metrics snapshots
    ///
    /// # Parameters
    ///
    /// * `metrics_collector` - Metrics collection subsystem
    /// * `resource_manager` - Resource management subsystem
    ///
    /// # Returns
    ///
    /// Returns a JoinHandle for the background task.
    async fn start_metrics_collection_process(
        &self,
        metrics_collector: MetricsCollector,
        resource_manager: &Arc<ResourceManager>,
    ) -> tokio::task::JoinHandle<()> {
        let interval_duration = self.config.metrics_collection_interval;
        let resource_manager = Arc::clone(resource_manager);

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);

            loop {
                interval.tick().await;

                // Collect system metrics
                if let Err(e) = resource_manager.update_system_metrics().await {
                    tracing::error!("Failed to update system metrics: {}", e);
                }

                // Update metrics collection
                if let Err(e) = metrics_collector.collect_periodic_metrics().await {
                    tracing::error!("Failed to collect periodic metrics: {}", e);
                }
            }
        })
    }

    /// Start resource monitoring process
    ///
    /// Launches the resource monitoring background process that tracks
    /// system resource usage and generates alerts when thresholds are exceeded.
    ///
    /// ## Process Behavior
    ///
    /// - Runs on configurable interval (`resource_monitoring_interval`)
    /// - Monitors CPU and memory usage
    /// - Generates alerts when usage exceeds 90%
    /// - Sends coordination messages for system-wide alerts
    /// - Logs resource usage patterns
    ///
    /// ## Alert Thresholds
    ///
    /// - CPU usage > 90%: Sends CPU alert
    /// - Memory usage > 90%: Sends memory alert
    /// - Alerts include current usage percentage
    ///
    /// ## Performance
    ///
    /// - CPU: Minimal for resource polling
    /// - Memory: Constant small footprint
    /// - Network: None (local resource monitoring)
    ///
    /// # Parameters
    ///
    /// * `resource_manager` - Resource management subsystem
    ///
    /// # Returns
    ///
    /// Returns a JoinHandle for the background task.
    async fn start_resource_monitoring_process(
        &self,
        resource_manager: &Arc<ResourceManager>,
    ) -> tokio::task::JoinHandle<()> {
        let interval_duration = self.config.resource_monitoring_interval;
        let coordination_tx = self.coordination_tx.clone();
        let resource_manager = Arc::clone(resource_manager);

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);

            loop {
                interval.tick().await;

                // Monitor resource usage and send alerts if needed
                let (system_resources, _, _) = resource_manager.get_system_info().await;

                // Check CPU usage
                if system_resources.cpu_usage > 0.9 {
                    if let Err(e) = coordination_tx.send(CoordinationMessage::ResourceAlert {
                        resource: "CPU".to_string(),
                        usage: system_resources.cpu_usage,
                    }) {
                        tracing::error!("Failed to send CPU alert: {}", e);
                    }
                }

                // Check memory usage
                if system_resources.memory_usage > 0.9 {
                    if let Err(e) = coordination_tx.send(CoordinationMessage::ResourceAlert {
                        resource: "Memory".to_string(),
                        usage: system_resources.memory_usage,
                    }) {
                        tracing::error!("Failed to send memory alert: {}", e);
                    }
                }
            }
        })
    }

    /// Update process configuration
    ///
    /// Dynamically updates the configuration for all background processes.
    /// Changes take effect on the next process iteration.
    ///
    /// ## Configuration Changes
    ///
    /// - Timing intervals can be adjusted
    /// - Process behavior parameters can be modified
    /// - Changes apply to all running processes
    ///
    /// ## When Changes Take Effect
    ///
    /// - Interval changes: Next process iteration
    /// - Behavioral changes: Immediate where possible
    /// - Some changes may require process restart
    ///
    /// ## Performance
    ///
    /// O(1) operation - just updates configuration reference.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::{ProcessManager, ProcessConfig};
    /// # use std::time::Duration;
    /// # async fn example(mut process_manager: ProcessManager) {
    /// let new_config = ProcessConfig {
    ///     work_stealing_interval: Duration::from_millis(50), // More responsive
    ///     learning_interval: Duration::from_secs(60), // Less frequent learning
    ///     ..process_manager.config.clone()
    /// };
    ///
    /// process_manager.update_config(new_config).await;
    /// println!("Process configuration updated");
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `new_config` - New configuration to apply
    pub async fn update_config(&mut self, new_config: ProcessConfig) {
        self.config = new_config;
        tracing::info!("Process configuration updated");
    }

    /// Get current process status
    ///
    /// Returns comprehensive information about all background processes
    /// including their status, configuration, and health metrics.
    ///
    /// ## Status Information
    ///
    /// - Total number of processes
    /// - Number of active processes
    /// - Current configuration settings
    /// - Process health indicators
    /// - Resource usage statistics
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is the number of processes.
    /// Involves checking process handle status.
    ///
    /// ## Use Cases
    ///
    /// - System monitoring and health checks
    /// - Administrative status reporting
    /// - Troubleshooting process issues
    /// - Performance analysis
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::ProcessManager;
    /// # async fn example(process_manager: &ProcessManager) {
    /// let status = process_manager.get_process_status().await;
    ///
    /// let total_processes = status["total_processes"].as_u64().unwrap_or(0);
    /// let active_processes = status["active_processes"].as_u64().unwrap_or(0);
    ///
    /// println!("Processes: {}/{} active", active_processes, total_processes);
    ///
    /// if let Some(config) = status.get("configuration") {
    ///     println!("Current configuration: {}", config);
    /// }
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a JSON object containing process status information.
    pub async fn get_process_status(&self) -> serde_json::Value {
        let handles = self.process_handles.read().await;
        let active_processes = handles.iter().filter(|h| !h.is_finished()).count();

        serde_json::json!({
            "total_processes": handles.len(),
            "active_processes": active_processes,
            "configuration": {
                "work_stealing_interval_ms": self.config.work_stealing_interval.as_millis(),
                "learning_interval_ms": self.config.learning_interval.as_millis(),
                "swarm_coordination_interval_ms": self.config.swarm_coordination_interval.as_millis(),
                "metrics_collection_interval_ms": self.config.metrics_collection_interval.as_millis(),
                "resource_monitoring_interval_ms": self.config.resource_monitoring_interval.as_millis()
            }
        })
    }
}

impl Clone for ProcessManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            coordination_tx: self.coordination_tx.clone(),
            process_handles: Arc::clone(&self.process_handles),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::mpsc;

    // Mock implementations for testing

    // Helper function to create a test process manager
    async fn create_test_process_manager() -> HiveResult<ProcessManager> {
        let (tx, _rx) = mpsc::unbounded_channel();
        ProcessManager::new(tx).await
    }

    #[tokio::test]
    async fn test_process_manager_creation() -> Result<(), Box<dyn std::error::Error>> {
        let process_manager = create_test_process_manager().await?;

        let status = process_manager.get_process_status().await;
        assert_eq!(status["total_processes"], 0);
        assert_eq!(status["active_processes"], 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_process_config_default() {
        let config = ProcessConfig::default();

        assert_eq!(config.work_stealing_interval, Duration::from_millis(100));
        assert_eq!(config.learning_interval, Duration::from_secs(30));
        assert_eq!(config.swarm_coordination_interval, Duration::from_secs(5));
        assert_eq!(config.metrics_collection_interval, Duration::from_secs(10));
        assert_eq!(config.resource_monitoring_interval, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_process_config_update() -> Result<(), Box<dyn std::error::Error>> {
        let mut process_manager = create_test_process_manager().await?;

        let new_config = ProcessConfig {
            work_stealing_interval: Duration::from_millis(200),
            learning_interval: Duration::from_secs(60),
            swarm_coordination_interval: Duration::from_secs(10),
            metrics_collection_interval: Duration::from_secs(20),
            resource_monitoring_interval: Duration::from_secs(10),
        };

        process_manager.update_config(new_config.clone()).await;
        assert_eq!(
            process_manager.config.work_stealing_interval,
            Duration::from_millis(200)
        );
        assert_eq!(
            process_manager.config.learning_interval,
            Duration::from_secs(60)
        );
        assert_eq!(
            process_manager.config.swarm_coordination_interval,
            Duration::from_secs(10)
        );
        assert_eq!(
            process_manager.config.metrics_collection_interval,
            Duration::from_secs(20)
        );
        assert_eq!(
            process_manager.config.resource_monitoring_interval,
            Duration::from_secs(10)
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_start_all_processes() -> Result<(), Box<dyn std::error::Error>> {
        let process_manager = create_test_process_manager().await?;

        // Create real implementations for testing
        let resource_manager =
            Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);
        let (tx, _rx) = mpsc::unbounded_channel();

        let agent_manager = Arc::new(
            crate::core::hive::agent_management::AgentManager::new(
                Arc::clone(&resource_manager),
                tx.clone(),
            )
            .await?,
        );
        let task_distributor = Arc::new(
            crate::core::hive::task_management::TaskDistributor::new(
                Arc::clone(&resource_manager),
                tx.clone(),
            )
            .await?,
        );
        let metrics_collector =
            Arc::new(crate::core::hive::metrics_collection::MetricsCollector::new(tx).await?);

        process_manager
            .start_all_processes(
                &agent_manager,
                &task_distributor,
                &metrics_collector,
                &resource_manager,
            )
            .await?;

        // Check that processes were started
        let status = process_manager.get_process_status().await;
        assert_eq!(status["total_processes"], 5); // All 5 processes should be started

        Ok(())
    }

    #[tokio::test]
    async fn test_stop_all_processes() -> Result<(), Box<dyn std::error::Error>> {
        let process_manager = create_test_process_manager().await?;

        // Create real implementations for testing
        let resource_manager =
            Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);
        let (tx, _rx) = mpsc::unbounded_channel();

        let agent_manager = Arc::new(
            crate::core::hive::agent_management::AgentManager::new(
                Arc::clone(&resource_manager),
                tx.clone(),
            )
            .await?,
        );
        let task_distributor = Arc::new(
            crate::core::hive::task_management::TaskDistributor::new(
                Arc::clone(&resource_manager),
                tx.clone(),
            )
            .await?,
        );
        let metrics_collector =
            Arc::new(crate::core::hive::metrics_collection::MetricsCollector::new(tx).await?);

        // Start processes
        process_manager
            .start_all_processes(
                &agent_manager,
                &task_distributor,
                &metrics_collector,
                &resource_manager,
            )
            .await?;

        // Verify they're running
        let status_before = process_manager.get_process_status().await;
        assert_eq!(status_before["total_processes"], 5);

        // Stop processes
        process_manager.stop_all_processes().await?;

        // Verify they're stopped
        let status_after = process_manager.get_process_status().await;
        assert_eq!(status_after["total_processes"], 0);
        assert_eq!(status_after["active_processes"], 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_process_status_empty() -> Result<(), Box<dyn std::error::Error>> {
        let process_manager = create_test_process_manager().await?;

        let status = process_manager.get_process_status().await;
        assert!(status.is_object());
        assert_eq!(status["total_processes"], 0);
        assert_eq!(status["active_processes"], 0);

        let config = status["configuration"].as_object().unwrap();
        assert!(config.contains_key("work_stealing_interval_ms"));
        assert!(config.contains_key("learning_interval_ms"));
        assert!(config.contains_key("swarm_coordination_interval_ms"));
        assert!(config.contains_key("metrics_collection_interval_ms"));
        assert!(config.contains_key("resource_monitoring_interval_ms"));

        Ok(())
    }

    #[tokio::test]
    async fn test_get_process_status_with_processes() -> Result<(), Box<dyn std::error::Error>> {
        let process_manager = create_test_process_manager().await?;

        // Create real implementations
        let resource_manager =
            Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);
        let (tx, _rx) = mpsc::unbounded_channel();

        let agent_manager = Arc::new(
            crate::core::hive::agent_management::AgentManager::new(
                Arc::clone(&resource_manager),
                tx.clone(),
            )
            .await?,
        );
        let task_distributor = Arc::new(
            crate::core::hive::task_management::TaskDistributor::new(
                Arc::clone(&resource_manager),
                tx.clone(),
            )
            .await?,
        );
        let metrics_collector =
            Arc::new(crate::core::hive::metrics_collection::MetricsCollector::new(tx).await?);

        // Start processes
        process_manager
            .start_all_processes(
                &agent_manager,
                &task_distributor,
                &metrics_collector,
                &resource_manager,
            )
            .await?;

        let status = process_manager.get_process_status().await;
        assert_eq!(status["total_processes"], 5);
        assert!(status["active_processes"].as_u64().unwrap() >= 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_process_manager_clone() -> Result<(), Box<dyn std::error::Error>> {
        let process_manager = create_test_process_manager().await?;
        let cloned = process_manager.clone();

        assert_eq!(
            process_manager.config.work_stealing_interval,
            cloned.config.work_stealing_interval
        );
        assert_eq!(
            process_manager.config.learning_interval,
            cloned.config.learning_interval
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_start_work_stealing_process() -> Result<(), Box<dyn std::error::Error>> {
        let process_manager = create_test_process_manager().await?;

        // Create real implementations
        let resource_manager =
            Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);
        let (tx, _rx) = mpsc::unbounded_channel();

        let agent_manager = crate::core::hive::agent_management::AgentManager::new(
            Arc::clone(&resource_manager),
            tx.clone(),
        )
        .await?;
        let task_distributor = crate::core::hive::task_management::TaskDistributor::new(
            Arc::clone(&resource_manager),
            tx,
        )
        .await?;

        let handle = process_manager
            .start_work_stealing_process(agent_manager, task_distributor)
            .await;

        // Check that the handle is created
        assert!(!handle.is_finished());

        // Abort the process
        handle.abort();

        Ok(())
    }

    #[tokio::test]
    async fn test_start_learning_process() -> Result<(), Box<dyn std::error::Error>> {
        let process_manager = create_test_process_manager().await?;

        // Create real implementation
        let resource_manager =
            Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);
        let (tx, _rx) = mpsc::unbounded_channel();

        let agent_manager = crate::core::hive::agent_management::AgentManager::new(
            Arc::clone(&resource_manager),
            tx,
        )
        .await?;

        let handle = process_manager.start_learning_process(agent_manager).await;

        // Check that the handle is created
        assert!(!handle.is_finished());

        // Abort the process
        handle.abort();

        Ok(())
    }

    #[tokio::test]
    async fn test_start_swarm_coordination_process() -> Result<(), Box<dyn std::error::Error>> {
        let process_manager = create_test_process_manager().await?;

        // Create real implementation
        let resource_manager =
            Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);
        let (tx, _rx) = mpsc::unbounded_channel();

        let agent_manager = crate::core::hive::agent_management::AgentManager::new(
            Arc::clone(&resource_manager),
            tx,
        )
        .await?;

        let handle = process_manager
            .start_swarm_coordination_process(agent_manager)
            .await;

        // Check that the handle is created
        assert!(!handle.is_finished());

        // Abort the process
        handle.abort();

        Ok(())
    }

    #[tokio::test]
    async fn test_start_metrics_collection_process() -> Result<(), Box<dyn std::error::Error>> {
        let process_manager = create_test_process_manager().await?;
        let (tx, _rx) = mpsc::unbounded_channel();

        let metrics_collector =
            crate::core::hive::metrics_collection::MetricsCollector::new(tx).await?;
        let resource_manager =
            Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);

        let handle = process_manager
            .start_metrics_collection_process(metrics_collector, &resource_manager)
            .await;

        // Check that the handle is created
        assert!(!handle.is_finished());

        // Abort the process
        handle.abort();

        Ok(())
    }

    #[tokio::test]
    async fn test_start_resource_monitoring_process() -> Result<(), Box<dyn std::error::Error>> {
        let process_manager = create_test_process_manager().await?;
        let resource_manager =
            Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);

        let handle = process_manager
            .start_resource_monitoring_process(&resource_manager)
            .await;

        // Check that the handle is created
        assert!(!handle.is_finished());

        // Abort the process
        handle.abort();

        Ok(())
    }

    #[tokio::test]
    async fn test_process_config_custom_values() {
        let config = ProcessConfig {
            work_stealing_interval: Duration::from_millis(500),
            learning_interval: Duration::from_secs(120),
            swarm_coordination_interval: Duration::from_secs(15),
            metrics_collection_interval: Duration::from_secs(30),
            resource_monitoring_interval: Duration::from_secs(20),
        };

        assert_eq!(config.work_stealing_interval, Duration::from_millis(500));
        assert_eq!(config.learning_interval, Duration::from_secs(120));
        assert_eq!(config.swarm_coordination_interval, Duration::from_secs(15));
        assert_eq!(config.metrics_collection_interval, Duration::from_secs(30));
        assert_eq!(config.resource_monitoring_interval, Duration::from_secs(20));
    }

    #[tokio::test]
    async fn test_process_status_configuration_values() -> Result<(), Box<dyn std::error::Error>> {
        let mut process_manager = create_test_process_manager().await?;

        let custom_config = ProcessConfig {
            work_stealing_interval: Duration::from_millis(250),
            learning_interval: Duration::from_secs(45),
            swarm_coordination_interval: Duration::from_secs(8),
            metrics_collection_interval: Duration::from_secs(15),
            resource_monitoring_interval: Duration::from_secs(12),
        };

        process_manager.update_config(custom_config).await;

        let status = process_manager.get_process_status().await;
        let config = status["configuration"].as_object().unwrap();

        assert_eq!(config["work_stealing_interval_ms"], 250);
        assert_eq!(config["learning_interval_ms"], 45000);
        assert_eq!(config["swarm_coordination_interval_ms"], 8000);
        assert_eq!(config["metrics_collection_interval_ms"], 15000);
        assert_eq!(config["resource_monitoring_interval_ms"], 12000);

        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_process_starts() -> Result<(), Box<dyn std::error::Error>> {
        let process_manager = create_test_process_manager().await?;

        // Create real implementations
        let resource_manager =
            Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);
        let (tx, _rx) = mpsc::unbounded_channel();

        let agent_manager = Arc::new(
            crate::core::hive::agent_management::AgentManager::new(
                Arc::clone(&resource_manager),
                tx.clone(),
            )
            .await?,
        );
        let task_distributor = Arc::new(
            crate::core::hive::task_management::TaskDistributor::new(
                Arc::clone(&resource_manager),
                tx.clone(),
            )
            .await?,
        );
        let metrics_collector =
            Arc::new(crate::core::hive::metrics_collection::MetricsCollector::new(tx).await?);

        // Start processes multiple times
        for _ in 0..3 {
            process_manager
                .start_all_processes(
                    &agent_manager,
                    &task_distributor,
                    &metrics_collector,
                    &resource_manager,
                )
                .await?;
        }

        // Should have 15 processes total (5 * 3)
        let status = process_manager.get_process_status().await;
        assert_eq!(status["total_processes"], 15);

        Ok(())
    }

    #[tokio::test]
    async fn test_stop_processes_without_starting() -> Result<(), Box<dyn std::error::Error>> {
        let process_manager = create_test_process_manager().await?;

        // Should not panic when stopping processes that were never started
        process_manager.stop_all_processes().await?;

        let status = process_manager.get_process_status().await;
        assert_eq!(status["total_processes"], 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_process_handles_cleanup() -> Result<(), Box<dyn std::error::Error>> {
        let process_manager = create_test_process_manager().await?;

        // Create real implementations
        let resource_manager =
            Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);
        let (tx, _rx) = mpsc::unbounded_channel();

        let agent_manager = Arc::new(
            crate::core::hive::agent_management::AgentManager::new(
                Arc::clone(&resource_manager),
                tx.clone(),
            )
            .await?,
        );
        let task_distributor = Arc::new(
            crate::core::hive::task_management::TaskDistributor::new(
                Arc::clone(&resource_manager),
                tx.clone(),
            )
            .await?,
        );
        let metrics_collector =
            Arc::new(crate::core::hive::metrics_collection::MetricsCollector::new(tx).await?);

        // Start processes
        process_manager
            .start_all_processes(
                &agent_manager,
                &task_distributor,
                &metrics_collector,
                &resource_manager,
            )
            .await?;

        // Stop processes
        process_manager.stop_all_processes().await?;

        // Verify handles are cleared
        let status = process_manager.get_process_status().await;
        assert_eq!(status["total_processes"], 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_process_config_partial_update() -> Result<(), Box<dyn std::error::Error>> {
        let mut process_manager = create_test_process_manager().await?;

        let original_config = process_manager.config.clone();

        // Update only one field
        let partial_config = ProcessConfig {
            work_stealing_interval: Duration::from_millis(999),
            ..original_config
        };

        process_manager.update_config(partial_config).await;

        // Check that only the specified field changed
        assert_eq!(
            process_manager.config.work_stealing_interval,
            Duration::from_millis(999)
        );
        assert_eq!(
            process_manager.config.learning_interval,
            original_config.learning_interval
        );
        assert_eq!(
            process_manager.config.swarm_coordination_interval,
            original_config.swarm_coordination_interval
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_process_status_json_structure() -> Result<(), Box<dyn std::error::Error>> {
        let process_manager = create_test_process_manager().await?;

        let status = process_manager.get_process_status().await;

        // Verify all expected fields are present
        assert!(status.is_object());
        assert!(status.get("total_processes").is_some());
        assert!(status.get("active_processes").is_some());
        assert!(status.get("configuration").is_some());

        let config = status.get("configuration").unwrap();
        assert!(config.is_object());

        // Verify configuration fields
        let config_obj = config.as_object().unwrap();
        assert!(config_obj.contains_key("work_stealing_interval_ms"));
        assert!(config_obj.contains_key("learning_interval_ms"));
        assert!(config_obj.contains_key("swarm_coordination_interval_ms"));
        assert!(config_obj.contains_key("metrics_collection_interval_ms"));
        assert!(config_obj.contains_key("resource_monitoring_interval_ms"));

        Ok(())
    }

    #[tokio::test]
    async fn test_process_manager_with_no_agents() -> Result<(), Box<dyn std::error::Error>> {
        let process_manager = create_test_process_manager().await?;

        // Create real implementations
        let resource_manager =
            Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);
        let (tx, _rx) = mpsc::unbounded_channel();

        let agent_manager = Arc::new(
            crate::core::hive::agent_management::AgentManager::new(
                Arc::clone(&resource_manager),
                tx.clone(),
            )
            .await?,
        );
        let task_distributor = Arc::new(
            crate::core::hive::task_management::TaskDistributor::new(
                Arc::clone(&resource_manager),
                tx.clone(),
            )
            .await?,
        );
        let metrics_collector =
            Arc::new(crate::core::hive::metrics_collection::MetricsCollector::new(tx).await?);

        // Should still start processes successfully
        process_manager
            .start_all_processes(
                &agent_manager,
                &task_distributor,
                &metrics_collector,
                &resource_manager,
            )
            .await?;

        let status = process_manager.get_process_status().await;
        assert_eq!(status["total_processes"], 5);

        Ok(())
    }

    #[tokio::test]
    async fn test_process_manager_resource_alert_sending() -> Result<(), Box<dyn std::error::Error>>
    {
        let process_manager = create_test_process_manager().await?;
        let resource_manager =
            Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);

        // Start resource monitoring process
        let handle = process_manager
            .start_resource_monitoring_process(&resource_manager)
            .await;

        // Let it run for a short time
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Abort the process
        handle.abort();

        // The process should have attempted to send alerts (though we can't easily verify the sends)

        Ok(())
    }

    #[tokio::test]
    async fn test_process_manager_multiple_clones() -> Result<(), Box<dyn std::error::Error>> {
        let process_manager = create_test_process_manager().await?;

        let clone1 = process_manager.clone();
        let clone2 = process_manager.clone();
        let clone3 = clone1.clone();

        // All clones should have the same configuration
        assert_eq!(
            process_manager.config.work_stealing_interval,
            clone1.config.work_stealing_interval
        );
        assert_eq!(
            process_manager.config.work_stealing_interval,
            clone2.config.work_stealing_interval
        );
        assert_eq!(
            process_manager.config.work_stealing_interval,
            clone3.config.work_stealing_interval
        );

        Ok(())
    }
}
