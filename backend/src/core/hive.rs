//! # Hive Coordination System
//!
//! This module implements the central coordination system for the multiagent hive.
//! The `HiveCoordinator` manages agent lifecycles, task distribution, neural processing,
//! and real-time communication between system components.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use crate::agents::{
    Agent, AgentBehavior, AgentCapability, AgentType, SimpleVerificationResult,
    SimpleVerificationSystem, SkillEvolutionConfig, SkillEvolutionSystem,
};
use crate::core::fallback::{FallbackConfig, IntelligentFallback};
use crate::infrastructure::ResourceManager;
use crate::neural::{HybridNeuralProcessor, NLPProcessor};
use crate::tasks::WorkStealingQueue;
use crate::tasks::{Task, TaskQueue, TaskRequiredCapability};

// Enhanced swarm coordination types
use crate::core::auto_scaling::{AutoScalingConfig, AutoScalingSystem};
use crate::core::swarm_coordination::SwarmCoordinationMetrics;
use crate::core::swarm_intelligence::SwarmFormation;

/// Comprehensive metrics tracking swarm performance and behavior.
///
/// These metrics provide insights into the overall health and efficiency
/// of the multiagent system, enabling monitoring and optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmMetrics {
    /// Total number of agents in the hive
    pub total_agents: usize,
    /// Number of agents currently active (not idle or failed)
    pub active_agents: usize,
    /// Total number of successfully completed tasks
    pub completed_tasks: usize,
    /// Total number of failed tasks
    pub failed_tasks: usize,
    /// Average performance score across all agents (0.0 to 1.0)
    pub average_performance: f64,
    /// Measure of how well agents work together (0.0 to 1.0)
    pub swarm_cohesion: f64,
    /// Progress of the collective learning process (0.0 to 1.0)
    pub learning_progress: f64,
}

/// Current status and state of the entire hive system.
///
/// This structure provides a comprehensive snapshot of the hive's current
/// state, including metrics, positioning, and energy levels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveStatus {
    /// Unique identifier for this hive instance
    pub id: Uuid,
    /// Timestamp when the hive was created
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last status update
    pub last_update: DateTime<Utc>,
    /// Current performance and behavioral metrics
    pub metrics: SwarmMetrics,
    /// Geometric center of the swarm in 2D space
    pub swarm_center: (f64, f64),
    /// Total energy across all agents in the hive
    pub total_energy: f64,
}

/// Central coordinator for the multiagent hive system with enhanced swarm intelligence.
///
/// The `HiveCoordinator` is the core component that manages all aspects of the
/// multiagent system, including agent lifecycles, task distribution, neural
/// processing, swarm formations, and inter-agent communication.
///
/// # Architecture
///
/// The coordinator uses a hybrid approach with both legacy and modern queue systems:
/// - Legacy `TaskQueue` for backward compatibility
/// - High-performance `WorkStealingQueue` for optimal task distribution
/// - Thread-safe `DashMap` for concurrent agent access
/// - Enhanced swarm formations with neural coordination
/// - Async communication channels for real-time coordination
///
/// # Example
///
/// ```rust
/// use multiagent_hive::HiveCoordinator;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let hive = HiveCoordinator::new().await?;
///     // Use the hive coordinator...
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct HiveCoordinator {
    /// Unique identifier for this hive instance
    pub id: Uuid,
    /// Thread-safe map of all agents in the hive
    pub agents: Arc<DashMap<Uuid, Agent>>,
    /// Legacy task queue for backward compatibility
    pub task_queue: Arc<RwLock<TaskQueue>>,
    /// High-performance work-stealing queue system
    pub work_stealing_queue: Arc<WorkStealingQueue>,
    /// Natural language processing engine
    pub nlp_processor: Arc<NLPProcessor>,
    /// Hybrid neural processing system (basic + optional advanced)
    pub neural_processor: Arc<RwLock<HybridNeuralProcessor>>,
    /// Current swarm performance metrics
    pub metrics: Arc<RwLock<SwarmMetrics>>,
    /// Geometric center of the swarm
    pub swarm_center: Arc<RwLock<(f64, f64)>>,
    /// Channel for sending inter-agent communication messages
    pub communication_channel: mpsc::UnboundedSender<CommunicationMessage>,
    /// Receiver for inter-agent communication messages
    pub communication_receiver: Arc<RwLock<mpsc::UnboundedReceiver<CommunicationMessage>>>,
    /// Intelligent resource management system
    pub resource_manager: Arc<ResourceManager>,
    /// Simple verification system for lightweight task validation
    pub simple_verification: Arc<SimpleVerificationSystem>,
    /// Enhanced swarm formations with neural coordination
    pub formations: Arc<RwLock<HashMap<Uuid, SwarmFormation>>>,
    /// Swarm coordination metrics
    pub coordination_metrics: Arc<RwLock<SwarmCoordinationMetrics>>,
    /// Auto-scaling system for dynamic agent management
    pub auto_scaling: Arc<AutoScalingSystem>,
    /// Skill evolution system for agent learning
    pub skill_evolution: Arc<SkillEvolutionSystem>,
    /// Intelligent fallback system for agent selection
    pub fallback_system: Arc<RwLock<IntelligentFallback>>,
    /// Timestamp when this coordinator was created
    pub created_at: DateTime<Utc>,
}

/// Inter-agent communication message structure
#[derive(Debug, Clone)]
pub struct CommunicationMessage {
    /// ID of the agent sending the message
    pub from_agent: Uuid,
    /// ID of the target agent (None for broadcast)
    pub to_agent: Option<Uuid>,
    /// Type of message being sent
    pub message_type: MessageType,
    /// Message content
    pub content: String,
    /// When the message was created
    pub timestamp: DateTime<Utc>,
}

/// Types of messages that can be sent between agents
#[derive(Debug, Clone)]
pub enum MessageType {
    /// Request for task assignment or information
    TaskRequest,
    /// Response to a task request
    TaskResponse,
    /// Sharing learned patterns or insights
    LearningShare,
    /// Agent status updates
    StatusUpdate,
    /// Coordination and collaboration messages
    Coordination,
    /// Emergency or critical system messages
    Emergency,
}

impl HiveCoordinator {
    /// Creates a new hive coordinator with default configuration.
    ///
    /// Initializes all subsystems including neural processing, resource management,
    /// and communication channels. The coordinator starts with empty agent and task
    /// collections but is ready to accept new agents and tasks.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the initialized `HiveCoordinator` on success,
    /// or an error if any subsystem fails to initialize.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Neural processing initialization fails
    /// - Resource manager initialization fails
    /// - Communication channel setup fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use multiagent_hive::HiveCoordinator;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let hive = HiveCoordinator::new().await?;
    ///     println!("Hive coordinator initialized with ID: {}", hive.id);
    ///     Ok(())
    /// }
    /// ```
    pub async fn new() -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::unbounded_channel();

        // Phase 2: Initialize resource manager for intelligent optimization
        let resource_manager = Arc::new(ResourceManager::new().await?);
        tracing::info!("🚀 Phase 2: Resource manager initialized - CPU-native, GPU-optional");

        let nlp_processor = Arc::new(NLPProcessor::new().await?);
        let simple_verification =
            Arc::new(SimpleVerificationSystem::new(Arc::clone(&nlp_processor)));

        // Initialize auto-scaling system
        let auto_scaling_config = AutoScalingConfig::default();
        let auto_scaling = Arc::new(AutoScalingSystem::new(
            Arc::clone(&resource_manager),
            auto_scaling_config,
        ));

        // Initialize skill evolution system
        let skill_evolution_config = SkillEvolutionConfig::default();
        let skill_evolution = Arc::new(SkillEvolutionSystem::new(
            Arc::clone(&nlp_processor),
            skill_evolution_config,
        ));

        // Initialize intelligent fallback system
        let fallback_config = FallbackConfig::default();
        let fallback_system = Arc::new(RwLock::new(IntelligentFallback::new(fallback_config)));

        let coordinator = Self {
            id: Uuid::new_v4(),
            agents: Arc::new(DashMap::new()),
            task_queue: Arc::new(RwLock::new(TaskQueue::new())),
            work_stealing_queue: Arc::new(WorkStealingQueue::new()),
            nlp_processor,
            neural_processor: Arc::new(RwLock::new(HybridNeuralProcessor::new().await?)),
            metrics: Arc::new(RwLock::new(SwarmMetrics {
                total_agents: 0,
                active_agents: 0,
                completed_tasks: 0,
                failed_tasks: 0,
                average_performance: 0.0,
                swarm_cohesion: 0.0,
                learning_progress: 0.0,
            })),
            swarm_center: Arc::new(RwLock::new((0.0, 0.0))),
            communication_channel: tx,
            communication_receiver: Arc::new(RwLock::new(rx)),
            resource_manager,
            simple_verification,
            formations: Arc::new(RwLock::new(HashMap::new())),
            coordination_metrics: Arc::new(RwLock::new(SwarmCoordinationMetrics::default())),
            auto_scaling: Arc::clone(&auto_scaling),
            skill_evolution: Arc::clone(&skill_evolution),
            fallback_system: Arc::clone(&fallback_system),
            created_at: Utc::now(),
        };

        // Start background processes with Phase 2 optimizations
        // Start background processes in a separate task
        let coordinator_arc = Arc::new(RwLock::new(coordinator));
        let coordinator_clone1 = Arc::clone(&coordinator_arc);
        let coordinator_clone2 = Arc::clone(&coordinator_arc);
        let coordinator_clone3 = Arc::clone(&coordinator_arc);
        let auto_scaling_clone = Arc::clone(&auto_scaling);
        let skill_evolution_clone = Arc::clone(&skill_evolution);
        tokio::spawn(async move {
            Self::start_background_processes(coordinator_clone1).await;

            // Start auto-scaling system
            auto_scaling_clone
                .start_auto_scaling(coordinator_clone2)
                .await;

            // Start skill evolution system
            let agents_for_evolution = {
                let coord = coordinator_clone3.read().await;
                Arc::clone(&coord.agents)
            };
            skill_evolution_clone
                .start_skill_evolution(agents_for_evolution)
                .await;
        });

        // Return the coordinator from the Arc
        let coordinator = coordinator_arc.read().await.clone();
        Ok(coordinator)
    }

    /// Starts all background processes for the hive system.
    ///
    /// This includes work-stealing task distribution, learning processes,
    /// swarm coordination, and metrics collection.
    async fn start_background_processes(coordinator: Arc<RwLock<Self>>) {
        Self::start_work_stealing_process(Arc::clone(&coordinator)).await;
        Self::start_learning_process(Arc::clone(&coordinator)).await;
        Self::start_swarm_coordination_process(Arc::clone(&coordinator)).await;
        Self::start_metrics_collection_process(Arc::clone(&coordinator)).await;
    }

    /// Starts the work-stealing task distribution process.
    async fn start_work_stealing_process(coordinator: Arc<RwLock<Self>>) {
        let (agents, task_queue, _work_stealing_queue, resource_manager, fallback_system) = {
            let coord = coordinator.read().await;
            (
                Arc::clone(&coord.agents),
                Arc::clone(&coord.task_queue),
                Arc::clone(&coord.work_stealing_queue),
                Arc::clone(&coord.resource_manager),
                Arc::clone(&coord.fallback_system),
            )
        };

        // High-performance work-stealing task distribution
        let agents_ws = Arc::clone(&agents);
        let resource_manager_ws = Arc::clone(&resource_manager);
        let coordinator_ws = Arc::clone(&coordinator);
        tokio::spawn(async move {
            loop {
                // Get current resource profile for adaptive timing
                let profile = resource_manager_ws.get_current_profile().await;
                let mut interval = tokio::time::interval(
                    tokio::time::Duration::from_millis(profile.update_frequency / 2), // More frequent for work-stealing
                );
                interval.tick().await;

                if let Err(e) = {
                    let coord = coordinator_ws.read().await;
                    coord.work_stealing_distribution(&agents_ws).await
                } {
                    tracing::error!("Work-stealing distribution error: {}", e);
                }
            }
        });

        // Legacy task distribution (for backward compatibility)
        let resource_manager_legacy = Arc::clone(&resource_manager);
        let fallback_system_legacy = Arc::clone(&fallback_system);
        tokio::spawn(async move {
            loop {
                let profile = resource_manager_legacy.get_current_profile().await;
                let mut interval = tokio::time::interval(
                    tokio::time::Duration::from_millis(profile.update_frequency * 2), // Less frequent
                );
                interval.tick().await;

                if let Err(e) =
                    Self::distribute_tasks(&agents, &task_queue, &fallback_system_legacy).await
                {
                    tracing::error!("Legacy task distribution error: {}", e);
                }
            }
        });

        // Phase 2: Resource monitoring and auto-optimization
        let resource_manager_monitor = Arc::clone(&resource_manager);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Err(e) = resource_manager_monitor.update_system_metrics().await {
                    tracing::error!("Resource monitoring error: {}", e);
                } else {
                    let (resources, profile, _hardware_class) =
                        resource_manager_monitor.get_system_info().await;
                    tracing::debug!(
                        "📊 System: CPU {:.1}%, Memory {:.1}%, Profile: {} (max agents: {})",
                        resources.cpu_usage,
                        resources.memory_usage,
                        profile.profile_name,
                        profile.max_agents
                    );
                }
            }
        });
    }

    /// Starts the learning process for agents and neural networks.
    async fn start_learning_process(coordinator: Arc<RwLock<Self>>) {
        // Learning process
        let coordinator_learning = Arc::clone(&coordinator);
        let (agents_learning, nlp_learning, neural_learning) = {
            let coord = coordinator_learning.read().await;
            (
                Arc::clone(&coord.agents),
                Arc::clone(&coord.nlp_processor),
                Arc::clone(&coord.neural_processor),
            )
        };
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Err(e) = Self::learning_cycle(&agents_learning, &nlp_learning).await {
                    tracing::error!("Learning cycle error: {}", e);
                }

                // Additional neural learning cycle
                let _neural_proc = neural_learning.read().await;
                // Neural learning happens during agent interactions
                tracing::debug!("Neural learning cycle completed");
            }
        });
    }

    /// Starts the swarm coordination and formation optimization process.
    async fn start_swarm_coordination_process(coordinator: Arc<RwLock<Self>>) {
        // Swarm coordination process
        let coordinator_swarm = Arc::clone(&coordinator);
        let (agents_swarm, swarm_center_coord) = {
            let coord = coordinator_swarm.read().await;
            (Arc::clone(&coord.agents), Arc::clone(&coord.swarm_center))
        };
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
            loop {
                interval.tick().await;
                if let Err(e) =
                    Self::update_swarm_positions(&agents_swarm, &swarm_center_coord).await
                {
                    tracing::error!("Swarm coordination error: {}", e);
                }
            }
        });
    }

    /// Starts the metrics collection and system monitoring process.
    async fn start_metrics_collection_process(coordinator: Arc<RwLock<Self>>) {
        // Metrics update process
        let coordinator_metrics = Arc::clone(&coordinator);
        let (agents_metrics, task_queue_metrics, metrics_update) = {
            let coord = coordinator_metrics.read().await;
            (
                Arc::clone(&coord.agents),
                Arc::clone(&coord.task_queue),
                Arc::clone(&coord.metrics),
            )
        };
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(15));
            loop {
                interval.tick().await;
                if let Err(e) =
                    Self::update_metrics(&agents_metrics, &task_queue_metrics, &metrics_update)
                        .await
                {
                    tracing::error!("Metrics update error: {}", e);
                }
            }
        });
    }

    /// Creates a new agent with the specified configuration.
    ///
    /// # Arguments
    /// * `config` - JSON configuration containing agent parameters:
    ///   - `name`: Agent name (default: "Agent")
    ///   - `type`: Agent type ("coordinator", "learner", "specialist:domain", or "worker")
    ///   - `capabilities`: Array of capability objects with name and proficiency
    ///
    /// # Returns
    /// Returns the UUID of the created agent on success.
    ///
    /// # Errors
    /// Returns an error if agent creation fails or configuration is invalid.
    pub async fn create_agent(&self, config: serde_json::Value) -> anyhow::Result<Uuid> {
        let name = config
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Agent")
            .to_string();

        let agent_type = match config.get("type").and_then(|v| v.as_str()) {
            Some("coordinator") => AgentType::Coordinator,
            Some("learner") => AgentType::Learner,
            Some(specialist) if specialist.starts_with("specialist:") => AgentType::Specialist(
                specialist
                    .strip_prefix("specialist:")
                    .unwrap_or(specialist)
                    .to_string(),
            ),
            _ => AgentType::Worker,
        };

        let mut agent = Agent::new(name.clone(), agent_type.clone());

        // Add capabilities from config
        if let Some(capabilities) = config.get("capabilities").and_then(|v| v.as_array()) {
            for cap in capabilities {
                if let (Some(cap_name), Some(proficiency)) = (
                    cap.get("name").and_then(|v| v.as_str()),
                    cap.get("proficiency").and_then(serde_json::Value::as_f64),
                ) {
                    agent.add_capability(AgentCapability {
                        name: cap_name.to_string(),
                        proficiency: proficiency.clamp(0.0, 1.0),
                        learning_rate: cap
                            .get("learning_rate")
                            .and_then(serde_json::Value::as_f64)
                            .unwrap_or(0.1),
                    });
                }
            }
        }

        // Set initial position
        // Set initial position
        {
            let mut rng = rand::thread_rng();
            agent.position = (rng.gen_range(-100.0..100.0), rng.gen_range(-100.0..100.0));
        }

        let agent_id = agent.id;

        // Determine if this agent should use advanced neural capabilities
        let use_advanced = config
            .get("use_advanced_neural")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        // Create neural agent capabilities
        let specialization = match &agent_type {
            AgentType::Learner => "learning",
            AgentType::Coordinator => "coordination",
            AgentType::Specialist(spec) => spec,
            AgentType::Worker => "general",
        }
        .to_string();

        // Register with neural processor
        let mut neural_processor = self.neural_processor.write().await;
        if let Err(e) = neural_processor
            .create_neural_agent(agent_id, specialization, use_advanced)
            .await
        {
            tracing::warn!("Failed to create neural agent capabilities: {}", e);
        }

        // Register agent with work-stealing queue system
        if let Err(e) = self.work_stealing_queue.register_agent(agent_id).await {
            tracing::warn!(
                "Failed to register agent {} with work-stealing queue: {}",
                agent_id,
                e
            );
        }

        self.agents.insert(agent_id, agent);

        tracing::info!(
            "Created agent {} with ID {} (neural: {}, work-stealing: enabled)",
            name,
            agent_id,
            use_advanced
        );
        Ok(agent_id)
    }

    /// Creates a new task with the specified configuration.
    ///
    /// # Arguments
    /// * `config` - JSON configuration containing task parameters:
    ///   - `description`: Task description (default: "Generic task")
    ///   - `type`: Task type (default: "general")
    ///   - `priority`: Priority level (0=Low, 1=Medium, 2=High, 3=Critical)
    ///   - `required_capabilities`: Array of required capability objects
    ///
    /// # Returns
    /// Returns the UUID of the created task on success.
    ///
    /// # Errors
    /// Returns an error if task creation fails or configuration is invalid.
    pub async fn create_task(&self, config: serde_json::Value) -> anyhow::Result<Uuid> {
        let description = config
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("Generic task")
            .to_string();

        let task_type = config
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("general")
            .to_string();

        let priority = match config
            .get("priority")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(1)
        {
            0 => crate::tasks::TaskPriority::Low,
            2 => crate::tasks::TaskPriority::High,
            3 => crate::tasks::TaskPriority::Critical,
            _ => crate::tasks::TaskPriority::Medium,
        };

        let mut required_capabilities = None;
        if let Some(caps) = config
            .get("required_capabilities")
            .and_then(|v| v.as_array())
        {
            let mut req_caps = Vec::new();
            for cap in caps {
                if let (Some(name), Some(min_prof)) = (
                    cap.get("name").and_then(|v| v.as_str()),
                    cap.get("min_proficiency")
                        .and_then(serde_json::Value::as_f64),
                ) {
                    req_caps.push(TaskRequiredCapability {
                        name: name.to_string(),
                        minimum_proficiency: min_prof,
                    });
                }
            }
            if !req_caps.is_empty() {
                required_capabilities = Some(req_caps);
            }
        }

        let task = Task::new(
            description.clone(),
            description.clone(),
            task_type,
            priority,
            required_capabilities.unwrap_or_default(),
        );
        let task_id = task.id;

        // Submit to work-stealing queue for high-performance distribution
        if let Err(e) = self.work_stealing_queue.submit_task(task.clone()).await {
            tracing::warn!(
                "Failed to submit task to work-stealing queue: {}, falling back to legacy queue",
                e
            );
            // Fallback to legacy queue
            let mut queue = self.task_queue.write().await;
            queue.add_task(task);
        }

        tracing::info!(
            "Created task {} with ID {} (work-stealing: enabled)",
            description,
            task_id
        );
        Ok(task_id)
    }

    async fn distribute_tasks(
        agents: &DashMap<Uuid, Agent>,
        task_queue: &RwLock<TaskQueue>,
        fallback_system: &Arc<RwLock<IntelligentFallback>>,
    ) -> anyhow::Result<()> {
        let mut queue = task_queue.write().await;

        while let Some(task) = queue.get_next_task() {
            // Collect available agents
            let available_agents: Vec<Agent> = agents.iter().map(|r| r.value().clone()).collect();

            // Use intelligent fallback system to find best agent
            let mut fallback_sys = fallback_system.write().await;
            let fallback_decision = fallback_sys
                .find_agent_with_fallback(
                    &task,
                    &available_agents,
                    None, // No additional context for now
                )
                .await;

            drop(fallback_sys); // Release the lock

            if let Some(agent_id) = fallback_decision.final_assignment {
                if let Some(mut agent_ref) = agents.get_mut(&agent_id) {
                    let agent = agent_ref.value_mut();

                    // Log fallback decision if detailed logging is enabled
                    if fallback_decision.attempts.len() > 1 {
                        tracing::info!(
                            "Fallback used for task {}: tier={}, quality={:.2}, attempts={}",
                            task.id,
                            fallback_decision
                                .attempts
                                .last()
                                .map_or_else(|| "Unknown".to_string(), |r| format!("{:?}", r.tier)),
                            fallback_decision.quality_degradation,
                            fallback_decision.attempts.len()
                        );
                    }

                    // Execute task asynchronously
                    let task_clone = task.clone();
                    let agent_clone = agent.clone();
                    let agents_map = agents.clone();
                    let fallback_clone = Arc::clone(fallback_system);

                    tokio::spawn(async move {
                        let mut agent_exec = agent_clone;
                        match agent_exec.execute_task(task_clone).await {
                            Ok(result) => {
                                // Update agent in the map
                                if let Some(mut agent_ref) = agents_map.get_mut(&agent_id) {
                                    *agent_ref.value_mut() = agent_exec;
                                }

                                // Log successful completion with fallback context
                                let quality_score = result.quality_score.unwrap_or(0.0);
                                if quality_score < 0.8 {
                                    tracing::info!(
                                        "Task {} completed with reduced quality ({:.2}) due to fallback",
                                        result.task_id,
                                        quality_score
                                    );
                                } else {
                                    tracing::info!("Task completed successfully: {:?}", result);
                                }
                            }
                            Err(e) => {
                                tracing::error!("Task execution failed: {}", e);

                                // Could implement retry logic here with different fallback tier
                                let mut fallback_sys = fallback_clone.write().await;
                                fallback_sys.cleanup_completed_decisions();
                            }
                        }
                    });
                }
            } else {
                // No suitable agent found even with fallback, put task back in queue
                tracing::warn!(
                    "No suitable agent found for task {} even with fallback (attempts: {})",
                    task.id,
                    fallback_decision.attempts.len()
                );
                queue.add_task(task);
                break;
            }
        }

        Ok(())
    }

    /// High-performance work-stealing task distribution with intelligent fallback
    async fn work_stealing_distribution(
        &self,
        agents: &DashMap<Uuid, Agent>,
    ) -> anyhow::Result<()> {
        // Process tasks for each agent using work-stealing
        let agent_futures: Vec<_> = agents
            .iter()
            .map(|entry| {
                let agent_id = *entry.key();
                let agent = entry.value().clone();
                let queue = Arc::clone(&self.work_stealing_queue);

                async move {
                    // Skip if agent is busy or low energy
                    if !matches!(agent.state, crate::agents::AgentState::Idle)
                        || agent.energy < 10.0
                    {
                        return Ok(());
                    }

                    // Try to get a task (including work stealing)
                    if let Some(task) = queue.get_task_for_agent(agent_id).await {
                        // Mark agent as busy in the work-stealing system
                        if let Some(agent_queue) = queue.agent_queues.get(&agent_id) {
                            agent_queue.set_busy(true).await;
                        }

                        // Execute task asynchronously with fallback-aware execution
                        let agents_map2 = agents.clone();
                        let queue_clone = Arc::clone(&self.work_stealing_queue);
                        let fallback_clone = Arc::clone(&self.fallback_system);
                        tokio::spawn(async move {
                            let mut agent_exec = agent;
                            let start_time = std::time::Instant::now();

                            match agent_exec.execute_task(task.clone()).await {
                                Ok(result) => {
                                    // Update agent in the map
                                    if let Some(mut agent_ref) = agents_map2.get_mut(&agent_id) {
                                        *agent_ref.value_mut() = agent_exec;
                                    }

                                    // Mark task completion and agent as available
                                    if let Some(agent_queue) =
                                        queue_clone.agent_queues.get(&agent_id)
                                    {
                                        agent_queue.mark_task_completed().await;
                                        agent_queue.set_busy(false).await;
                                    }

                                    let duration = start_time.elapsed();

                                    // Log with quality context for fallback monitoring
                                    let quality_indicator =
                                        if let Some(quality) = result.quality_score {
                                            if quality < 0.7 {
                                                " (reduced quality)"
                                            } else {
                                                ""
                                            }
                                        } else {
                                            ""
                                        };

                                    tracing::debug!(
                                        "🚀 Work-stealing task completed in {:?}: {}{}",
                                        duration,
                                        result.task_id,
                                        quality_indicator
                                    );
                                }
                                Err(e) => {
                                    // Mark agent as available even on failure
                                    if let Some(agent_queue) =
                                        queue_clone.agent_queues.get(&agent_id)
                                    {
                                        agent_queue.set_busy(false).await;
                                    }

                                    // Log failure for fallback system analysis
                                    tracing::error!(
                                        "Work-stealing task execution failed for task {}: {}",
                                        task.id,
                                        e
                                    );

                                    // Update fallback system with failure information
                                    let mut fallback_sys = fallback_clone.write().await;
                                    fallback_sys.cleanup_completed_decisions();
                                }
                            }
                        });
                    }

                    Ok::<(), anyhow::Error>(())
                }
            })
            .collect();

        // Execute all agent task processing concurrently
        let results = futures::future::join_all(agent_futures).await;

        // Log any errors
        for result in results {
            if let Err(e) = result {
                tracing::error!("Work-stealing agent processing error: {}", e);
            }
        }

        // Update work-stealing metrics
        self.work_stealing_queue.update_metrics().await;

        Ok(())
    }

    async fn learning_cycle(
        agents: &DashMap<Uuid, Agent>,
        nlp_processor: &NLPProcessor,
    ) -> anyhow::Result<()> {
        for mut agent_ref in agents.iter_mut() {
            let agent = agent_ref.value_mut();
            if let Err(e) = agent.learn(nlp_processor).await {
                tracing::error!("Learning error for agent {}: {}", agent.id, e);
            }
        }
        Ok(())
    }

    async fn update_swarm_positions(
        agents: &DashMap<Uuid, Agent>,
        swarm_center: &RwLock<(f64, f64)>,
    ) -> anyhow::Result<()> {
        // Calculate swarm center
        let mut center_x = 0.0;
        let mut center_y = 0.0;
        let agent_count = agents.len();

        if agent_count == 0 {
            return Ok(());
        }

        for agent_ref in agents {
            let agent = agent_ref.value();
            center_x += agent.position.0;
            center_y += agent.position.1;
        }

        center_x /= agent_count as f64;
        center_y /= agent_count as f64;

        *swarm_center.write().await = (center_x, center_y);

        // Update agent positions
        let agents_vec: Vec<Agent> = agents.iter().map(|r| r.value().clone()).collect();

        for mut agent_ref in agents.iter_mut() {
            let agent = agent_ref.value_mut();
            if let Err(e) = agent
                .update_position((center_x, center_y), &agents_vec)
                .await
            {
                tracing::error!("Position update error for agent {}: {}", agent.id, e);
            }
        }

        Ok(())
    }

    async fn update_metrics(
        agents: &DashMap<Uuid, Agent>,
        task_queue: &RwLock<TaskQueue>,
        metrics: &RwLock<SwarmMetrics>,
    ) -> anyhow::Result<()> {
        let _queue = task_queue.read().await;
        let mut metrics_guard = metrics.write().await;

        metrics_guard.total_agents = agents.len();
        metrics_guard.active_agents = agents
            .iter()
            .filter(|a| matches!(a.value().state, crate::agents::AgentState::Working))
            .count();

        // Calculate average performance
        let total_performance: f64 = agents
            .iter()
            .map(|a| {
                a.value()
                    .capabilities
                    .iter()
                    .map(|c| c.proficiency)
                    .sum::<f64>()
                    / a.value().capabilities.len().max(1) as f64
            })
            .sum();

        metrics_guard.average_performance = if agents.is_empty() {
            0.0
        } else {
            total_performance / agents.len() as f64
        };

        // Calculate swarm cohesion (how close agents are to each other)
        let mut total_distance = 0.0;
        let mut distance_count = 0;

        for agent1 in agents {
            for agent2 in agents {
                if agent1.key() != agent2.key() {
                    let dist = ((agent1.value().position.0 - agent2.value().position.0).powi(2)
                        + (agent1.value().position.1 - agent2.value().position.1).powi(2))
                    .sqrt();
                    total_distance += dist;
                    distance_count += 1;
                }
            }
        }

        metrics_guard.swarm_cohesion = if distance_count > 0 {
            1.0 / (1.0 + total_distance / f64::from(distance_count) / 100.0) // Normalize
        } else {
            1.0
        };

        // Calculate learning progress
        let total_experiences: usize = agents
            .iter()
            .map(|a| a.value().memory.experiences.len())
            .sum();

        metrics_guard.learning_progress =
            (total_experiences as f64 / (agents.len().max(1) * 100) as f64).min(1.0);

        Ok(())
    }

    /// Retrieves information about all agents in the hive.
    ///
    /// # Returns
    /// Returns a JSON object containing:
    /// - `agents`: Array of agent objects with their current state, capabilities, and metrics
    /// - `total_count`: Total number of agents in the hive
    pub async fn get_agents_info(&self) -> serde_json::Value {
        let agents: Vec<serde_json::Value> = self
            .agents
            .iter()
            .map(|agent_ref| {
                let agent = agent_ref.value();
                serde_json::json!({
                    "id": agent.id,
                    "name": agent.name,
                    "type": agent.agent_type,
                    "state": agent.state,
                    "capabilities": agent.capabilities,
                    "position": agent.position,
                    "energy": agent.energy,
                    "experience_count": agent.memory.experiences.len(),
                    "social_connections": agent.memory.social_connections.len(),
                })
            })
            .collect();

        serde_json::json!({
            "agents": agents,
            "total_count": agents.len()
        })
    }

    /// Retrieves information about all tasks in the hive.
    ///
    /// # Returns
    /// Returns a JSON object containing:
    /// - `tasks`: Array of task objects with their status, priority, and requirements
    /// - `total_count`: Total number of tasks in the system
    pub async fn get_tasks_info(&self) -> serde_json::Value {
        let queue = self.task_queue.read().await;
        let ws_metrics = self.work_stealing_queue.get_metrics().await;

        serde_json::json!({
            "legacy_queue": {
                "pending_tasks": queue.get_pending_count(),
                "completed_tasks": queue.get_completed_count(),
                "failed_tasks": queue.get_failed_count(),
            },
            "work_stealing_queue": {
                "total_queue_depth": ws_metrics.total_queue_depth,
                "global_queue_depth": ws_metrics.global_queue_depth,
                "active_agents": ws_metrics.active_agents,
                "steal_efficiency": ws_metrics.system_metrics.load_balance_efficiency,
                "total_steals": ws_metrics.system_metrics.successful_steals,
                "agent_queues": ws_metrics.agent_metrics.len(),
            }
        })
    }

    /// Retrieves the current status and metrics of the hive system.
    ///
    /// # Returns
    /// Returns a comprehensive JSON object containing:
    /// - `hive_id`: Unique identifier of this hive instance
    /// - `metrics`: Performance metrics (agent counts, task completion rates, etc.)
    /// - `swarm_center`: Current center point of the agent swarm
    /// - `total_energy`: Aggregate energy level of all agents
    /// - Timestamps for creation and last update
    pub async fn get_status(&self) -> serde_json::Value {
        let metrics = self.metrics.read().await;
        let swarm_center = self.swarm_center.read().await;

        serde_json::json!({
            "hive_id": self.id,
            "created_at": self.created_at,
            "last_update": Utc::now(),
            "metrics": *metrics,
            "swarm_center": *swarm_center,
            "total_energy": self.agents.iter().map(|a| a.value().energy).sum::<f64>(),
        })
    }

    /// Retrieves current resource utilization and system health information.
    ///
    /// # Returns
    /// Returns a JSON object containing:
    /// - `cpu_usage`: Current CPU utilization percentage
    /// - `memory_usage`: Current memory utilization percentage
    /// - `active_connections`: Number of active WebSocket connections
    /// - `system_health`: Overall system health status
    pub async fn get_resource_info(&self) -> serde_json::Value {
        let (system_resources, resource_profile, hardware_class) =
            self.resource_manager.get_system_info().await;

        serde_json::json!({
            "system_resources": system_resources,
            "resource_profile": resource_profile,
            "hardware_class": format!("{:?}", hardware_class),
            "phase_2_status": "active",
            "optimization_enabled": true
        })
    }

    /// Execute a task with simple verification (lightweight alternative to pair programming)
    pub async fn execute_task_with_simple_verification(
        &self,
        task_id: Uuid,
        original_goal: Option<&str>,
    ) -> anyhow::Result<(crate::tasks::TaskResult, SimpleVerificationResult)> {
        // Find and execute the task
        let task = if let Some(task) = self.work_stealing_queue.get_task_by_id(task_id).await {
            task
        } else {
            let mut task_queue = self.task_queue.write().await;
            let task = task_queue
                .pending_tasks
                .iter()
                .find(|t| t.id == task_id)
                .ok_or_else(|| anyhow::anyhow!("Task {} not found", task_id))?
                .clone();

            // Remove from pending queue
            task_queue.pending_tasks.retain(|t| t.id != task_id);
            drop(task_queue);
            task
        };

        // Find best agent for the task
        let mut best_agent_id = None;
        let mut best_fitness = 0.0;

        for agent_ref in self.agents.iter() {
            let agent = agent_ref.value();
            if agent.can_perform_task(&task)
                && matches!(agent.state, crate::agents::AgentState::Idle)
            {
                let fitness = agent.calculate_task_fitness(&task);
                if fitness > best_fitness {
                    best_fitness = fitness;
                    best_agent_id = Some(agent.id);
                }
            }
        }

        let agent_id =
            best_agent_id.ok_or_else(|| anyhow::anyhow!("No suitable agent available for task"))?;

        // Execute task
        let mut agent = self
            .agents
            .get_mut(&agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent {} not found", agent_id))?
            .clone();

        let execution_result = agent.execute_task(task.clone()).await?;

        // Verify result using simple verification
        let verification_result = self
            .simple_verification
            .verify_task_result(&task, &execution_result, original_goal)
            .await?;

        // Update agent in the hive
        if let Some(mut agent_ref) = self.agents.get_mut(&agent_id) {
            *agent_ref.value_mut() = agent;
        }

        tracing::info!(
            "Completed task {} with simple verification. Status: {:?}, Score: {:.2}",
            task_id,
            verification_result.verification_status,
            verification_result.overall_score
        );

        Ok((execution_result, verification_result))
    }

    /// Get simple verification system metrics
    pub async fn get_simple_verification_stats(&self) -> serde_json::Value {
        let metrics = self.simple_verification.get_metrics().await;

        serde_json::json!({
            "total_verifications": metrics.total_verifications,
            "passed_verifications": metrics.passed_verifications,
            "failed_verifications": metrics.failed_verifications,
            "success_rate": if metrics.total_verifications > 0 {
                metrics.passed_verifications as f64 / metrics.total_verifications as f64
            } else {
                0.0
            },
            "average_verification_time_ms": metrics.average_verification_time_ms,
            "average_confidence_score": metrics.average_confidence_score,
            "tier_usage": metrics.tier_usage,
            "rule_effectiveness": metrics.rule_effectiveness
        })
    }

    /// Configure simple verification system
    pub async fn configure_simple_verification(
        &self,
        config: serde_json::Value,
    ) -> anyhow::Result<()> {
        // Configure confidence threshold
        if let Some(threshold) = config
            .get("confidence_threshold")
            .and_then(serde_json::Value::as_f64)
        {
            // Note: This would require making simple_verification mutable
            // For now, we'll log the configuration request
            tracing::info!(
                "Simple verification configuration requested: confidence_threshold = {}",
                threshold
            );
        }

        // Add task-specific rules
        if let Some(task_rules) = config.get("task_rules").and_then(|v| v.as_object()) {
            for (task_type, rules_config) in task_rules {
                tracing::info!(
                    "Task-specific rules configuration for '{}': {:?}",
                    task_type,
                    rules_config
                );
                // In a real implementation, you would parse and apply these rules
            }
        }

        // Set AI reviewer agent
        if let Some(reviewer_id) = config.get("ai_reviewer_agent").and_then(|v| v.as_str()) {
            if let Ok(agent_uuid) = Uuid::parse_str(reviewer_id) {
                if self.agents.contains_key(&agent_uuid) {
                    tracing::info!("AI reviewer agent set to: {}", agent_uuid);
                    // Note: This would require making simple_verification mutable
                }
            }
        }

        Ok(())
    }

    /// Get auto-scaling system statistics
    pub async fn get_auto_scaling_stats(&self) -> serde_json::Value {
        self.auto_scaling.get_scaling_stats().await
    }

    /// Get skill evolution system statistics
    pub async fn get_skill_evolution_stats(&self) -> serde_json::Value {
        self.skill_evolution.get_evolution_stats().await
    }

    /// Get comprehensive system analytics including new features
    pub async fn get_enhanced_analytics(&self) -> serde_json::Value {
        let basic_status = self.get_status().await;
        let auto_scaling_stats = self.get_auto_scaling_stats().await;
        let skill_evolution_stats = self.get_skill_evolution_stats().await;
        let resource_info = self.get_resource_info().await;
        let fallback_stats = self.get_fallback_stats().await;

        serde_json::json!({
            "hive_status": basic_status,
            "auto_scaling": auto_scaling_stats,
            "skill_evolution": skill_evolution_stats,
            "resource_management": resource_info,
            "fallback_system": fallback_stats,
            "enhanced_features": {
                "dynamic_scaling_enabled": true,
                "skill_learning_enabled": true,
                "neural_coordination_active": true,
                "swarm_formations_active": true,
                "intelligent_fallback_enabled": true
            }
        })
    }

    /// Get fallback system statistics
    pub async fn get_fallback_stats(&self) -> serde_json::Value {
        let fallback_sys = self.fallback_system.read().await;
        let stats = fallback_sys.get_stats();

        let tier_distribution: std::collections::HashMap<String, u64> = stats
            .tier_distribution
            .iter()
            .map(|(tier, count)| (format!("{tier:?}"), *count))
            .collect();

        let tier_success_rates: std::collections::HashMap<String, f64> = stats
            .tier_success_rates
            .iter()
            .map(|(tier, rate)| (format!("{tier:?}"), *rate))
            .collect();

        serde_json::json!({
            "total_attempts": stats.total_attempts,
            "successful_fallbacks": stats.successful_fallbacks,
            "failed_fallbacks": stats.failed_fallbacks,
            "success_rate": if stats.total_attempts > 0 {
                stats.successful_fallbacks as f64 / stats.total_attempts as f64
            } else {
                0.0
            },
            "average_quality_degradation": stats.average_quality_degradation,
            "average_decision_time_ms": stats.average_decision_time_ms,
            "tier_distribution": tier_distribution,
            "tier_success_rates": tier_success_rates
        })
    }

    /// Get recent fallback decisions
    pub async fn get_recent_fallback_decisions(&self, limit: usize) -> serde_json::Value {
        let fallback_sys = self.fallback_system.read().await;
        let decisions = fallback_sys.get_recent_decisions(limit);

        let decisions_json: Vec<serde_json::Value> = decisions
            .iter()
            .map(|decision| {
                let attempts: Vec<serde_json::Value> = decision
                    .attempts
                    .iter()
                    .map(|attempt| {
                        serde_json::json!({
                            "success": attempt.success,
                            "tier": format!("{:?}", attempt.tier),
                            "selected_agent": attempt.selected_agent,
                            "quality_score": attempt.quality_score,
                            "reason": attempt.reason,
                            "timestamp": attempt.timestamp,
                        })
                    })
                    .collect();

                serde_json::json!({
                    "id": decision.id,
                    "task_id": decision.task_id,
                    "successful": decision.successful,
                    "quality_degradation": decision.quality_degradation,
                    "total_duration_ms": decision.total_duration_ms,
                    "attempts": attempts,
                    "final_assignment": decision.final_assignment,
                    "started_at": decision.started_at,
                    "completed_at": decision.completed_at,
                })
            })
            .collect();

        serde_json::json!({
            "decisions": decisions_json,
            "total_count": decisions.len()
        })
    }

    /// Configure the fallback system
    pub async fn configure_fallback_system(&self, config: serde_json::Value) -> anyhow::Result<()> {
        let mut fallback_sys = self.fallback_system.write().await;

        let new_config = FallbackConfig {
            enabled: config
                .get("enabled")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true),
            max_fallback_attempts: config
                .get("max_fallback_attempts")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(3) as usize,
            min_quality_threshold: config
                .get("min_quality_threshold")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.6),
            emergency_quality_threshold: config
                .get("emergency_quality_threshold")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.3),
            enable_emergency_generalization: config
                .get("enable_emergency_generalization")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true),
            availability_check_window: config
                .get("availability_check_window")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(300),
            detailed_logging: config
                .get("detailed_logging")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true),
        };

        fallback_sys.update_config(new_config);

        tracing::info!("Fallback system configuration updated");
        Ok(())
    }

    /// Get fallback system configuration
    pub async fn get_fallback_config(&self) -> serde_json::Value {
        let fallback_sys = self.fallback_system.read().await;
        let config = fallback_sys.get_config();

        serde_json::json!({
            "enabled": config.enabled,
            "max_fallback_attempts": config.max_fallback_attempts,
            "min_quality_threshold": config.min_quality_threshold,
            "emergency_quality_threshold": config.emergency_quality_threshold,
            "enable_emergency_generalization": config.enable_emergency_generalization,
            "availability_check_window": config.availability_check_window,
            "detailed_logging": config.detailed_logging
        })
    }

    /// Manually trigger fallback system cleanup
    pub async fn cleanup_fallback_system(&self) -> anyhow::Result<()> {
        let mut fallback_sys = self.fallback_system.write().await;
        fallback_sys.cleanup_completed_decisions();

        tracing::info!("Fallback system cleanup completed");
        Ok(())
    }
}
