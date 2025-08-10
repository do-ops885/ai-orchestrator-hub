use crate::agents::agent::{Agent, AgentType};
use crate::tasks::task::{Task, TaskPriority};
use std::collections::HashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use tracing::{info, debug, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmFormation {
    pub formation_id: Uuid,
    pub agents: Vec<Uuid>,
    pub formation_type: FormationType,
    pub efficiency_score: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormationType {
    Chain,      // Sequential processing
    Star,       // Central coordinator with workers
    Mesh,       // Fully connected network
    Hierarchy,  // Tree-like structure
    Ring,       // Circular formation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationMetrics {
    pub task_completion_rate: f64,
    pub average_response_time: f64,
    pub resource_utilization: f64,
    pub communication_overhead: f64,
}

pub struct SwarmIntelligenceEngine {
    formations: HashMap<Uuid, SwarmFormation>,
    agent_performance_history: HashMap<Uuid, Vec<f64>>,
    formation_metrics: HashMap<Uuid, FormationMetrics>,
    optimization_config: SwarmOptimizationConfig,
}

#[derive(Debug, Clone)]
pub struct SwarmOptimizationConfig {
    pub max_agents_per_formation: usize,
    pub min_agents_per_formation: usize,
    pub performance_history_size: usize,
    pub efficiency_threshold: f64,
    pub rebalance_interval_minutes: u32,
}

impl Default for SwarmOptimizationConfig {
    fn default() -> Self {
        Self {
            max_agents_per_formation: 10,
            min_agents_per_formation: 2,
            performance_history_size: 100,
            efficiency_threshold: 0.7,
            rebalance_interval_minutes: 15,
        }
    }
}

impl SwarmIntelligenceEngine {
    pub fn new() -> Self {
        Self {
            formations: HashMap::new(),
            agent_performance_history: HashMap::new(),
            formation_metrics: HashMap::new(),
            optimization_config: SwarmOptimizationConfig::default(),
        }
    }

    pub fn with_config(config: SwarmOptimizationConfig) -> Self {
        Self {
            formations: HashMap::new(),
            agent_performance_history: HashMap::new(),
            formation_metrics: HashMap::new(),
            optimization_config: config,
        }
    }

    pub async fn optimize_formation(&mut self, 
        agents: &[Agent], 
        task: &Task
    ) -> anyhow::Result<SwarmFormation> {
        info!("Optimizing formation for task: {}", task.id);
        
        let formation_type = self.determine_optimal_formation(task, agents).await?;
        let selected_agents = self.select_optimal_agents(agents, task, &formation_type).await?;
        
        let formation = SwarmFormation {
            formation_id: Uuid::new_v4(),
            agents: selected_agents,
            formation_type,
            efficiency_score: 0.0,
            created_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
        };

        debug!("Created formation {} with {} agents", formation.formation_id, formation.agents.len());
        self.formations.insert(formation.formation_id, formation.clone());
        Ok(formation)
    }

    async fn determine_optimal_formation(&self, task: &Task, agents: &[Agent]) -> anyhow::Result<FormationType> {
        let agent_count = agents.len();
        
        // Consider task characteristics and available agents
        let formation_type = match (task.priority.clone(), agent_count) {
            (TaskPriority::Critical, _) if agent_count >= 3 => FormationType::Star, // Fast coordination
            (TaskPriority::High, n) if n >= 5 => FormationType::Hierarchy, // Structured approach
            (TaskPriority::Medium, n) if n >= 3 => FormationType::Chain, // Sequential processing
            (TaskPriority::Low, n) if n >= 4 => FormationType::Mesh, // Distributed processing
            (_, n) if n >= 3 => FormationType::Ring, // Circular for small groups
            _ => FormationType::Chain, // Default fallback
        };

        // Consider task complexity
        let complexity_factor = task.required_capabilities.len() as f64;
        let adjusted_formation = if complexity_factor > 5.0 {
            match formation_type {
                FormationType::Chain => FormationType::Hierarchy,
                FormationType::Ring => FormationType::Star,
                other => other,
            }
        } else {
            formation_type
        };

        debug!("Selected formation type: {:?} for task complexity: {}", adjusted_formation, complexity_factor);
        Ok(adjusted_formation)
    }

    async fn select_optimal_agents(&self, 
        agents: &[Agent], 
        task: &Task, 
        formation_type: &FormationType
    ) -> anyhow::Result<Vec<Uuid>> {
        let mut selected = Vec::new();
        
        // Filter agents by required capabilities
        let capable_agents: Vec<&Agent> = agents.iter()
            .filter(|agent| self.agent_can_handle_task(agent, task))
            .collect();

        if capable_agents.is_empty() {
            return Err(anyhow::anyhow!("No capable agents found for task"));
        }

        match formation_type {
            FormationType::Star => {
                selected.extend(self.select_star_formation(&capable_agents).await?);
            }
            FormationType::Chain => {
                selected.extend(self.select_chain_formation(&capable_agents, task).await?);
            }
            FormationType::Hierarchy => {
                selected.extend(self.select_hierarchy_formation(&capable_agents).await?);
            }
            FormationType::Mesh => {
                selected.extend(self.select_mesh_formation(&capable_agents).await?);
            }
            FormationType::Ring => {
                selected.extend(self.select_ring_formation(&capable_agents).await?);
            }
        }

        // Ensure we don't exceed limits
        let max_agents = self.optimization_config.max_agents_per_formation.min(capable_agents.len());
        selected.truncate(max_agents);

        // Ensure minimum agents
        if selected.len() < self.optimization_config.min_agents_per_formation {
            // Add more agents if available
            for agent in capable_agents.iter().take(self.optimization_config.min_agents_per_formation) {
                if !selected.contains(&agent.id) {
                    selected.push(agent.id);
                }
            }
        }

        Ok(selected)
    }

    async fn select_star_formation(&self, agents: &[&Agent]) -> anyhow::Result<Vec<Uuid>> {
        let mut selected = Vec::new();
        
        // Select one coordinator
        if let Some(coordinator) = agents.iter()
            .find(|a| matches!(a.agent_type, AgentType::Coordinator)) {
            selected.push(coordinator.id);
        } else if let Some(best_agent) = agents.iter()
            .max_by(|a, b| self.calculate_coordination_score(a).partial_cmp(&self.calculate_coordination_score(b)).unwrap_or(std::cmp::Ordering::Equal)) {
            selected.push(best_agent.id);
        }
        
        // Add workers
        let worker_ids: Vec<Uuid> = agents.iter()
            .filter(|a| !selected.contains(&a.id))
            .take(4)
            .map(|a| a.id)
            .collect();
        selected.extend(worker_ids);
        
        Ok(selected)
    }

    async fn select_chain_formation(&self, agents: &[&Agent], task: &Task) -> anyhow::Result<Vec<Uuid>> {
        let mut sorted_agents = agents.to_vec();
        sorted_agents.sort_by(|a, b| {
            let a_score = self.calculate_agent_score(a, task);
            let b_score = self.calculate_agent_score(b, task);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(sorted_agents.into_iter().take(4).map(|a| a.id).collect())
    }

    async fn select_hierarchy_formation(&self, agents: &[&Agent]) -> anyhow::Result<Vec<Uuid>> {
        let mut selected = Vec::new();
        
        // Select leader (coordinator or highest scoring agent)
        if let Some(leader) = agents.iter()
            .filter(|a| matches!(a.agent_type, AgentType::Coordinator))
            .max_by(|a, b| a.memory.experiences.len().cmp(&b.memory.experiences.len())) {
            selected.push(leader.id);
        } else if let Some(leader) = agents.iter()
            .max_by(|a, b| a.memory.experiences.len().cmp(&b.memory.experiences.len())) {
            selected.push(leader.id);
        }
        
        // Select sub-leaders
        let sub_leader_ids: Vec<Uuid> = agents.iter()
            .filter(|a| !selected.contains(&a.id))
            .filter(|a| matches!(a.agent_type, AgentType::Specialist(_)) || a.memory.experiences.len() > 50)
            .take(2)
            .map(|a| a.id)
            .collect();
        selected.extend(sub_leader_ids);
        
        // Add workers
        let worker_ids: Vec<Uuid> = agents.iter()
            .filter(|a| !selected.contains(&a.id))
            .take(3)
            .map(|a| a.id)
            .collect();
        selected.extend(worker_ids);
        
        Ok(selected)
    }

    async fn select_mesh_formation(&self, agents: &[&Agent]) -> anyhow::Result<Vec<Uuid>> {
        // Select diverse set of agents for mesh network
        let mut selected = Vec::new();
        let mut agent_types_used = std::collections::HashSet::new();
        
        // Prioritize diversity in agent types
        for agent in agents.iter() {
            let type_key = std::mem::discriminant(&agent.agent_type);
            if !agent_types_used.contains(&type_key) || selected.len() < 3 {
                selected.push(agent.id);
                agent_types_used.insert(type_key);
                
                if selected.len() >= 5 {
                    break;
                }
            }
        }
        
        Ok(selected)
    }

    async fn select_ring_formation(&self, agents: &[&Agent]) -> anyhow::Result<Vec<Uuid>> {
        // Select agents with balanced capabilities for ring formation
        let mut selected = Vec::new();
        
        // Sort by energy and experience for balanced ring
        let mut sorted_agents = agents.to_vec();
        sorted_agents.sort_by(|a, b| {
            let a_balance = a.energy + (a.memory.experiences.len() as f64 / 100.0);
            let b_balance = b.energy + (b.memory.experiences.len() as f64 / 100.0);
            b_balance.partial_cmp(&a_balance).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        for agent in sorted_agents.into_iter().take(4) {
            selected.push(agent.id);
        }
        
        Ok(selected)
    }

    fn agent_can_handle_task(&self, agent: &Agent, task: &Task) -> bool {
        // Check if agent has required capabilities
        task.required_capabilities.iter().all(|req_cap| {
            agent.capabilities.iter().any(|cap| {
                cap.name == req_cap.name && cap.proficiency >= req_cap.minimum_proficiency
            })
        }) && agent.energy > 0.1 // Minimum energy requirement
    }

    fn calculate_agent_score(&self, agent: &Agent, task: &Task) -> f64 {
        let mut score = 0.0;
        
        // Base score from energy and experience
        score += agent.energy * 0.3;
        score += (agent.memory.experiences.len() as f64 / 100.0).min(1.0) * 0.2;
        
        // Capability matching score
        let capability_score: f64 = task.required_capabilities.iter()
            .map(|req_cap| {
                agent.capabilities.iter()
                    .find(|cap| cap.name == req_cap.name)
                    .map(|cap| cap.proficiency)
                    .unwrap_or(0.0)
            })
            .sum();
        
        score += capability_score * 0.4;
        
        // Historical performance bonus
        if let Some(history) = self.agent_performance_history.get(&agent.id) {
            if !history.is_empty() {
                let avg_performance: f64 = history.iter().sum::<f64>() / history.len() as f64;
                score += avg_performance * 0.1;
            }
        }
        
        score
    }

    fn calculate_coordination_score(&self, agent: &Agent) -> f64 {
        let mut score = 0.0;
        
        // Coordination bonus for coordinators
        if matches!(agent.agent_type, AgentType::Coordinator) {
            score += 0.5;
        }
        
        // Experience and social connections
        score += (agent.memory.experiences.len() as f64 / 100.0).min(1.0) * 0.3;
        score += (agent.memory.social_connections.len() as f64 / 20.0).min(1.0) * 0.2;
        
        score
    }

    pub fn update_formation_performance(&mut self, 
        formation_id: Uuid, 
        performance_score: f64
    ) {
        if let Some(formation) = self.formations.get_mut(&formation_id) {
            formation.efficiency_score = performance_score;
            formation.last_updated = chrono::Utc::now();
            
            info!("Updated formation {} performance to {}", formation_id, performance_score);
        }
    }

    pub fn record_agent_performance(&mut self, agent_id: Uuid, performance: f64) {
        let history = self.agent_performance_history.entry(agent_id).or_insert_with(Vec::new);
        history.push(performance);
        
        // Keep only recent performance records
        if history.len() > self.optimization_config.performance_history_size {
            history.remove(0);
        }
        
        debug!("Recorded performance {} for agent {}", performance, agent_id);
    }

    pub fn get_formation_efficiency(&self, formation_id: Uuid) -> Option<f64> {
        self.formations.get(&formation_id).map(|f| f.efficiency_score)
    }

    pub fn get_active_formations(&self) -> Vec<&SwarmFormation> {
        self.formations.values().collect()
    }

    pub async fn rebalance_formations(&mut self, agents: &[Agent]) -> anyhow::Result<Vec<Uuid>> {
        let mut rebalanced_formations = Vec::new();
        
        // Find underperforming formations
        let underperforming: Vec<Uuid> = self.formations.iter()
            .filter(|(_, formation)| formation.efficiency_score < self.optimization_config.efficiency_threshold)
            .map(|(id, _)| *id)
            .collect();
        
        for formation_id in underperforming {
            if let Some(formation) = self.formations.get(&formation_id) {
                warn!("Rebalancing underperforming formation: {}", formation_id);
                
                // Try to improve the formation by replacing low-performing agents
                let current_agents: Vec<&Agent> = agents.iter()
                    .filter(|a| formation.agents.contains(&a.id))
                    .collect();
                
                // This is a simplified rebalancing - in practice, you'd want more sophisticated logic
                if current_agents.len() > self.optimization_config.min_agents_per_formation {
                    rebalanced_formations.push(formation_id);
                }
            }
        }
        
        Ok(rebalanced_formations)
    }
}

impl Default for SwarmIntelligenceEngine {
    fn default() -> Self {
        Self::new()
    }
}