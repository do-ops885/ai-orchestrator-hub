//! # Core Coordinator Implementation
//!
//! This module contains the main `HiveCoordinator` struct and its core operations.
//! It focuses on the essential coordinator functionality without lifecycle or status concerns.

use crate::agents::agent::Agent;
use crate::core::hive::agent_management::AgentManager;
use crate::core::hive::background_processes::ProcessManager;
use crate::core::hive::metrics_collection::{
    HiveMetrics, MetricsCollector as HiveMetricsCollector,
};
use crate::core::hive::task_management::TaskDistributor;
use crate::infrastructure::resource_manager::ResourceManager;
use crate::neural::core::HybridNeuralProcessor;
use crate::neural::nlp::NLPProcessor;
use crate::utils::error::{HiveError, HiveResult};

use super::messages::CoordinationMessage;

use serde_json;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// Central coordinator for the multiagent hive system with enhanced modularity.
///
/// The `HiveCoordinator` serves as the main entry point and orchestrates all hive operations.
/// It maintains references to specialized subsystem managers and coordinates their interactions
/// through async message passing for optimal performance and maintainability.
///
/// ## Subsystem Delegation
///
/// - `AgentManager`: Agent lifecycle, registration, and performance monitoring
/// - `TaskDistributor`: Task queuing, distribution, and execution tracking
/// - `ProcessManager`: Background processes and system maintenance
/// - `MetricsCollector`: Comprehensive metrics gathering and reporting
/// - `ResourceManager`: System resource monitoring and capacity planning
///
/// ## Thread Safety
///
/// All operations are async and thread-safe. The coordinator uses `Arc<RwLock<T>>` for
/// shared state and `mpsc` channels for inter-subsystem communication.
///
/// ## Error Handling
///
/// Operations return `HiveResult<T>` with detailed error information for debugging
/// and recovery. Common errors include resource exhaustion, validation failures,
/// and subsystem communication issues.
///
/// ## Performance
///
/// - Memory usage scales with active agents and tasks
/// - CPU overhead minimized through efficient async operations
/// - Configurable background process intervals for resource control
#[derive(Clone)]
pub struct HiveCoordinator {
    /// Unique identifier for this hive instance
    ///
    /// Used for tracking and logging purposes across the system.
    pub id: Uuid,

    /// Agent management subsystem
    ///
    /// Handles all agent-related operations including registration,
    /// lifecycle management, and performance monitoring.
    pub(super) agent_manager: Arc<AgentManager>,

    /// Task distribution subsystem
    ///
    /// Manages task queuing, distribution to available agents,
    /// and execution tracking with work-stealing algorithms.
    pub(super) task_distributor: Arc<TaskDistributor>,

    /// Background process management
    ///
    /// Coordinates long-running processes for system maintenance,
    /// learning cycles, and periodic resource monitoring.
    pub(super) process_manager: Arc<ProcessManager>,

    /// Metrics collection subsystem
    ///
    /// Gathers comprehensive metrics from all subsystems for
    /// monitoring, analytics, and performance optimization.
    pub(super) metrics_collector: HiveMetricsCollector,

    /// Resource management
    ///
    /// Monitors system resources and provides capacity planning
    /// information to other subsystems.
    pub(super) resource_manager: Arc<ResourceManager>,

    /// Neural processing engine
    ///
    /// Advanced neural network processing for intelligent decision making
    /// and adaptive behavior in the hive system.
    pub(super) neural_processor: Arc<RwLock<HybridNeuralProcessor>>,

    /// Natural language processing
    ///
    /// Handles text analysis, command interpretation, and
    /// natural language interfaces for the system.
    pub(super) nlp_processor: Arc<NLPProcessor>,

    /// Communication channel for inter-subsystem coordination
    ///
    /// Async message passing channel for coordinating operations
    /// between different subsystems without tight coupling.
    pub(super) coordination_tx: mpsc::UnboundedSender<CoordinationMessage>,

    /// Receiver for coordination messages
    ///
    /// Wrapped in `RwLock` for safe concurrent access during
    /// initialization and shutdown operations.
    pub(super) coordination_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<CoordinationMessage>>>>,
}

impl HiveCoordinator {
    /// Creates a new hive coordinator with modular architecture.
    ///
    /// This method initializes all subsystems in the correct order and establishes
    /// communication channels between them for coordinated operation.
    ///
    /// ## Performance
    ///
    /// Initialization time depends on system resources and subsystem complexity.
    /// Typically completes in 100-500ms on modern hardware.
    ///
    /// ## Errors
    ///
    /// Returns error if any subsystem fails to initialize.
    pub async fn new() -> HiveResult<Self> {
        let id = Uuid::new_v4();
        let (coordination_tx, coordination_rx) = mpsc::unbounded_channel();

        // Initialize core systems
        let resource_manager = Arc::new(ResourceManager::new().await.map_err(|e| {
            HiveError::ResourceInitializationFailed {
                reason: format!("Failed to initialize resource manager: {e}"),
            }
        })?);

        let nlp_processor =
            Arc::new(crate::neural::nlp::NLPProcessor::new().await.map_err(|e| {
                HiveError::NeuralProcessingError {
                    reason: format!("Failed to initialize NLP processor: {e}"),
                }
            })?);

        let neural_processor = Arc::new(RwLock::new(
            crate::neural::core::HybridNeuralProcessor::new()
                .await
                .map_err(|e| HiveError::NeuralProcessingError {
                    reason: format!("Failed to initialize neural processor: {e}"),
                })?,
        ));

        // Initialize modular subsystems
        let agent_manager = Arc::new(
            AgentManager::new(Arc::clone(&resource_manager), coordination_tx.clone()).await?,
        );

        let task_distributor =
            TaskDistributor::new(Arc::clone(&resource_manager), coordination_tx.clone()).await?;

        let process_manager = ProcessManager::new(coordination_tx.clone()).await?;

        let metrics_collector = HiveMetricsCollector::new(coordination_tx.clone()).await?;

        Ok(Self {
            id,
            agent_manager,
            task_distributor: Arc::new(task_distributor),
            process_manager: Arc::new(process_manager),
            metrics_collector,
            resource_manager,
            neural_processor,
            nlp_processor,
            coordination_tx,
            coordination_rx: Arc::new(RwLock::new(Some(coordination_rx))),
        })
    }

    // Agent Operations

    /// Create a new agent with the given configuration.
    pub async fn create_agent(&self, config: serde_json::Value) -> HiveResult<Uuid> {
        self.agent_manager.create_agent(config).await
    }

    /// Remove an agent from the system.
    pub async fn remove_agent(&self, agent_id: Uuid) -> HiveResult<()> {
        self.agent_manager.remove_agent(agent_id).await
    }

    /// Get an agent by ID.
    pub async fn get_agent(&self, agent_id: Uuid) -> Option<Agent> {
        self.agent_manager.get_agent(agent_id).await
    }

    /// Get all active agents.
    pub async fn get_all_agents(&self) -> Vec<(Uuid, Agent)> {
        self.agent_manager.get_all_agents().await
    }

    /// Get the current number of agents in the system.
    #[must_use] 
    pub fn get_agent_count(&self) -> usize {
        self.agent_manager.get_agent_count()
    }

    /// Update an agent in the system.
    pub async fn update_agent(&self, agent_id: Uuid, agent: Agent) {
        self.agent_manager.update_agent(agent_id, agent).await;
    }

    // Task Operations

    /// Create a new task with the given configuration.
    pub async fn create_task(&self, config: serde_json::Value) -> HiveResult<Uuid> {
        self.task_distributor.create_task(config).await
    }

    /// Get the current number of tasks in the queue.
    pub async fn get_task_count(&self) -> usize {
        self.task_distributor.get_task_count().await
    }

    /// Execute a task with verification.
    pub async fn execute_task_with_verification(
        &self,
        task_id: Uuid,
        agent_id: Uuid,
    ) -> HiveResult<serde_json::Value> {
        self.task_distributor
            .execute_task_with_verification(task_id, agent_id)
            .await
    }

    // System Operations

    /// Get current system metrics.
    pub async fn get_metrics(&self) -> HiveMetrics {
        self.metrics_collector.get_current_metrics().await
    }

    /// Get reference to NLP processor.
    #[must_use] 
    pub fn get_nlp_processor(&self) -> &Arc<crate::neural::nlp::NLPProcessor> {
        &self.nlp_processor
    }

    /// Get reference to neural processor.
    #[must_use] 
    pub fn get_neural_processor(&self) -> &Arc<RwLock<crate::neural::core::HybridNeuralProcessor>> {
        &self.neural_processor
    }
}

impl Drop for HiveCoordinator {
    fn drop(&mut self) {
        tracing::info!("HiveCoordinator {} is being dropped", self.id);
    }
}
