use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use dashmap::DashMap;
use rand::Rng;

use crate::agent::{Agent, AgentBehavior, AgentType, AgentCapability};
use crate::task::{Task, TaskQueue, TaskRequiredCapability};
use crate::nlp::NLPProcessor;
use crate::neural::HybridNeuralProcessor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmMetrics {
    pub total_agents: usize,
    pub active_agents: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub average_performance: f64,
    pub swarm_cohesion: f64,
    pub learning_progress: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveStatus {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub metrics: SwarmMetrics,
    pub swarm_center: (f64, f64),
    pub total_energy: f64,
}

pub struct HiveCoordinator {
    pub id: Uuid,
    pub agents: Arc<DashMap<Uuid, Agent>>,
    pub task_queue: Arc<RwLock<TaskQueue>>,
    pub nlp_processor: Arc<NLPProcessor>,
    pub neural_processor: Arc<RwLock<HybridNeuralProcessor>>,
    pub metrics: Arc<RwLock<SwarmMetrics>>,
    pub swarm_center: Arc<RwLock<(f64, f64)>>,
    pub communication_channel: mpsc::UnboundedSender<CommunicationMessage>,
    pub communication_receiver: Arc<RwLock<mpsc::UnboundedReceiver<CommunicationMessage>>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CommunicationMessage {
    pub from_agent: Uuid,
    pub to_agent: Option<Uuid>, // None for broadcast
    pub message_type: MessageType,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum MessageType {
    TaskRequest,
    TaskResponse,
    LearningShare,
    StatusUpdate,
    Coordination,
    Emergency,
}

impl HiveCoordinator {
    pub async fn new() -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        let coordinator = Self {
            id: Uuid::new_v4(),
            agents: Arc::new(DashMap::new()),
            task_queue: Arc::new(RwLock::new(TaskQueue::new())),
            nlp_processor: Arc::new(NLPProcessor::new().await?),
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
            created_at: Utc::now(),
        };

        // Start background processes
        coordinator.start_background_processes().await;
        
        Ok(coordinator)
    }

    async fn start_background_processes(&self) {
        let agents = Arc::clone(&self.agents);
        let task_queue = Arc::clone(&self.task_queue);
        let _nlp_processor = Arc::clone(&self.nlp_processor);
        let _metrics = Arc::clone(&self.metrics);
        let _swarm_center = Arc::clone(&self.swarm_center);

        // Task distribution process
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
            loop {
                interval.tick().await;
                if let Err(e) = Self::distribute_tasks(&agents, &task_queue).await {
                    tracing::error!("Task distribution error: {}", e);
                }
            }
        });

        // Learning process
        let agents_learning = Arc::clone(&self.agents);
        let nlp_learning = Arc::clone(&self.nlp_processor);
        let neural_learning = Arc::clone(&self.neural_processor);
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

        // Swarm coordination process
        let agents_swarm = Arc::clone(&self.agents);
        let swarm_center_coord = Arc::clone(&self.swarm_center);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
            loop {
                interval.tick().await;
                if let Err(e) = Self::update_swarm_positions(&agents_swarm, &swarm_center_coord).await {
                    tracing::error!("Swarm coordination error: {}", e);
                }
            }
        });

        // Metrics update process
        let agents_metrics = Arc::clone(&self.agents);
        let task_queue_metrics = Arc::clone(&self.task_queue);
        let metrics_update = Arc::clone(&self.metrics);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(15));
            loop {
                interval.tick().await;
                if let Err(e) = Self::update_metrics(&agents_metrics, &task_queue_metrics, &metrics_update).await {
                    tracing::error!("Metrics update error: {}", e);
                }
            }
        });
    }

    pub async fn create_agent(&self, config: serde_json::Value) -> anyhow::Result<Uuid> {
        let name = config.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Agent")
            .to_string();

        let agent_type = match config.get("type").and_then(|v| v.as_str()) {
            Some("coordinator") => AgentType::Coordinator,
            Some("learner") => AgentType::Learner,
            Some(specialist) if specialist.starts_with("specialist:") => {
                AgentType::Specialist(specialist.strip_prefix("specialist:").unwrap().to_string())
            }
            _ => AgentType::Worker,
        };

        let mut agent = Agent::new(name.clone(), agent_type.clone());

        // Add capabilities from config
        if let Some(capabilities) = config.get("capabilities").and_then(|v| v.as_array()) {
            for cap in capabilities {
                if let (Some(cap_name), Some(proficiency)) = (
                    cap.get("name").and_then(|v| v.as_str()),
                    cap.get("proficiency").and_then(|v| v.as_f64())
                ) {
                    agent.add_capability(AgentCapability {
                        name: cap_name.to_string(),
                        proficiency: proficiency.clamp(0.0, 1.0),
                        learning_rate: cap.get("learning_rate").and_then(|v| v.as_f64()).unwrap_or(0.1),
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
        let use_advanced = config.get("use_advanced_neural")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        // Create neural agent capabilities
        let specialization = match &agent_type {
            AgentType::Learner => "learning",
            AgentType::Coordinator => "coordination", 
            AgentType::Specialist(spec) => spec,
            _ => "general",
        }.to_string();
        
        // Register with neural processor
        let mut neural_processor = self.neural_processor.write().await;
        if let Err(e) = neural_processor.create_neural_agent(agent_id, specialization, use_advanced).await {
            tracing::warn!("Failed to create neural agent capabilities: {}", e);
        }
        
        self.agents.insert(agent_id, agent);

        tracing::info!("Created agent {} with ID {} (neural: {})", name, agent_id, use_advanced);
        Ok(agent_id)
    }

    pub async fn create_task(&self, config: serde_json::Value) -> anyhow::Result<Uuid> {
        let description = config.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("Generic task")
            .to_string();

        let task_type = config.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("general")
            .to_string();

        let priority = match config.get("priority").and_then(|v| v.as_u64()).unwrap_or(1) {
            0 => crate::task::TaskPriority::Low,
            1 => crate::task::TaskPriority::Medium,
            2 => crate::task::TaskPriority::High,
            3 => crate::task::TaskPriority::Critical,
            _ => crate::task::TaskPriority::Medium,
        };

        let mut required_capabilities = None;
        if let Some(caps) = config.get("required_capabilities").and_then(|v| v.as_array()) {
            let mut req_caps = Vec::new();
            for cap in caps {
                if let (Some(name), Some(min_prof)) = (
                    cap.get("name").and_then(|v| v.as_str()),
                    cap.get("min_proficiency").and_then(|v| v.as_f64())
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

        let task = Task::new(description.clone(), description.clone(), task_type, priority, required_capabilities.unwrap_or_default());
        let task_id = task.id;

        let mut queue = self.task_queue.write().await;
        queue.add_task(task);

        tracing::info!("Created task {} with ID {}", description, task_id);
        Ok(task_id)
    }

    async fn distribute_tasks(
        agents: &DashMap<Uuid, Agent>,
        task_queue: &RwLock<TaskQueue>,
    ) -> anyhow::Result<()> {
        let mut queue = task_queue.write().await;
        
        while let Some(task) = queue.get_next_task() {
            // Find the best agent for this task
            let mut best_agent_id = None;
            let mut best_fitness = 0.0;

            for agent_ref in agents.iter() {
                let agent = agent_ref.value();
                if agent.can_perform_task(&task) {
                    let fitness = agent.calculate_task_fitness(&task);
                    if fitness > best_fitness {
                        best_fitness = fitness;
                        best_agent_id = Some(agent.id);
                    }
                }
            }

            if let Some(agent_id) = best_agent_id {
                if let Some(mut agent_ref) = agents.get_mut(&agent_id) {
                    let agent = agent_ref.value_mut();
                    
                    // Execute task asynchronously
                    let task_clone = task.clone();
                    let agent_clone = agent.clone();
                    let agents_clone = agents.clone();
                    
                    tokio::spawn(async move {
                        let mut agent_exec = agent_clone;
                        match agent_exec.execute_task(task_clone).await {
                            Ok(result) => {
                                // Update agent in the map
                                if let Some(mut agent_ref) = agents_clone.get_mut(&agent_id) {
                                    *agent_ref.value_mut() = agent_exec;
                                }
                                tracing::info!("Task completed: {:?}", result);
                            }
                            Err(e) => {
                                tracing::error!("Task execution failed: {}", e);
                            }
                        }
                    });
                }
            } else {
                // No suitable agent found, put task back in queue
                queue.add_task(task);
                break;
            }
        }

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

        for agent_ref in agents.iter() {
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
            if let Err(e) = agent.update_position((center_x, center_y), &agents_vec).await {
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
        metrics_guard.active_agents = agents.iter()
            .filter(|a| matches!(a.value().state, crate::agent::AgentState::Working))
            .count();

        // Calculate average performance
        let total_performance: f64 = agents.iter()
            .map(|a| {
                a.value().capabilities.iter()
                    .map(|c| c.proficiency)
                    .sum::<f64>() / a.value().capabilities.len().max(1) as f64
            })
            .sum();

        metrics_guard.average_performance = if agents.len() > 0 {
            total_performance / agents.len() as f64
        } else {
            0.0
        };

        // Calculate swarm cohesion (how close agents are to each other)
        let mut total_distance = 0.0;
        let mut distance_count = 0;

        for agent1 in agents.iter() {
            for agent2 in agents.iter() {
                if agent1.key() != agent2.key() {
                    let dist = ((agent1.value().position.0 - agent2.value().position.0).powi(2) +
                               (agent1.value().position.1 - agent2.value().position.1).powi(2)).sqrt();
                    total_distance += dist;
                    distance_count += 1;
                }
            }
        }

        metrics_guard.swarm_cohesion = if distance_count > 0 {
            1.0 / (1.0 + total_distance / distance_count as f64 / 100.0) // Normalize
        } else {
            1.0
        };

        // Calculate learning progress
        let total_experiences: usize = agents.iter()
            .map(|a| a.value().memory.experiences.len())
            .sum();

        metrics_guard.learning_progress = (total_experiences as f64 / (agents.len().max(1) * 100) as f64).min(1.0);

        Ok(())
    }

    pub async fn get_agents_info(&self) -> serde_json::Value {
        let agents: Vec<serde_json::Value> = self.agents.iter()
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

    pub async fn get_tasks_info(&self) -> serde_json::Value {
        let queue = self.task_queue.read().await;
        serde_json::json!({
            "pending_tasks": queue.get_pending_count(),
            "completed_tasks": queue.get_completed_count(),
            "failed_tasks": queue.get_failed_count(),
        })
    }

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
}