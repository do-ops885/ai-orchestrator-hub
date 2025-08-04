use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::nlp::NLPProcessor;
use crate::task::{Task, TaskResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    Worker,
    Coordinator,
    Specialist(String),
    Learner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentState {
    Idle,
    Working,
    Learning,
    Communicating,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    pub name: String,
    pub proficiency: f64, // 0.0 to 1.0
    pub learning_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMemory {
    pub experiences: Vec<Experience>,
    pub learned_patterns: HashMap<String, f64>,
    pub social_connections: HashMap<Uuid, f64>, // agent_id -> trust_score
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub timestamp: DateTime<Utc>,
    pub task_type: String,
    pub success: bool,
    pub context: String,
    pub learned_insight: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub agent_type: AgentType,
    pub state: AgentState,
    pub capabilities: Vec<AgentCapability>,
    pub memory: AgentMemory,
    pub position: (f64, f64), // For swarm positioning
    pub energy: f64,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

impl Agent {
    pub fn new(name: String, agent_type: AgentType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            agent_type,
            state: AgentState::Idle,
            capabilities: Vec::new(),
            memory: AgentMemory {
                experiences: Vec::new(),
                learned_patterns: HashMap::new(),
                social_connections: HashMap::new(),
            },
            position: (0.0, 0.0),
            energy: 100.0,
            created_at: Utc::now(),
            last_active: Utc::now(),
        }
    }

    pub fn add_capability(&mut self, capability: AgentCapability) {
        self.capabilities.push(capability);
    }

    pub fn get_capability_score(&self, capability_name: &str) -> f64 {
        self.capabilities
            .iter()
            .find(|c| c.name == capability_name)
            .map(|c| c.proficiency)
            .unwrap_or(0.0)
    }

    pub fn learn_from_experience(&mut self, experience: Experience) {
        // Update capability proficiency based on experience
        if let Some(capability) = self.capabilities
            .iter_mut()
            .find(|c| c.name == experience.task_type)
        {
            let adjustment = if experience.success {
                capability.learning_rate * 0.1
            } else {
                -capability.learning_rate * 0.05
            };
            capability.proficiency = (capability.proficiency + adjustment).clamp(0.0, 1.0);
        }

        // Store experience in memory
        self.memory.experiences.push(experience);
        
        // Limit memory size
        if self.memory.experiences.len() > 1000 {
            self.memory.experiences.remove(0);
        }
    }

    pub fn update_social_connection(&mut self, agent_id: Uuid, interaction_success: bool) {
        let current_trust = self.memory.social_connections.get(&agent_id).unwrap_or(&0.5);
        let adjustment = if interaction_success { 0.1 } else { -0.1 };
        let new_trust = (current_trust + adjustment).clamp(0.0, 1.0);
        self.memory.social_connections.insert(agent_id, new_trust);
    }

    pub fn can_perform_task(&self, task: &Task) -> bool {
        if !task.required_capabilities.is_empty() {
            let required = &task.required_capabilities;
            required.iter().all(|req_cap| {
                self.get_capability_score(&req_cap.name) >= req_cap.minimum_proficiency
            })
        } else {
            true
        }
    }

    pub fn calculate_task_fitness(&self, task: &Task) -> f64 {
        let mut fitness = 0.0;
        let mut total_weight = 0.0;

        if !task.required_capabilities.is_empty() {
            let required_caps = &task.required_capabilities;
            for req_cap in required_caps {
                let agent_proficiency = self.get_capability_score(&req_cap.name);
                fitness += agent_proficiency * 1.0;
                total_weight += 1.0;
            }
        }

        if total_weight > 0.0 {
            fitness / total_weight
        } else {
            0.5 // Default fitness if no specific requirements
        }
    }
}

#[async_trait]
pub trait AgentBehavior {
    async fn execute_task(&mut self, task: Task) -> anyhow::Result<TaskResult>;
    async fn communicate(&mut self, message: &str, target_agent: Option<Uuid>) -> anyhow::Result<String>;
    async fn learn(&mut self, nlp_processor: &NLPProcessor) -> anyhow::Result<()>;
    async fn update_position(&mut self, swarm_center: (f64, f64), neighbors: &[Agent]) -> anyhow::Result<()>;
}

#[async_trait]
impl AgentBehavior for Agent {
    async fn execute_task(&mut self, task: Task) -> anyhow::Result<TaskResult> {
        self.state = AgentState::Working;
        self.last_active = Utc::now();

        // Simulate task execution with some randomness
        let fitness = self.calculate_task_fitness(&task);
        let success_probability = fitness * 0.8 + 0.2; // 20% base success rate
        
        let success = rand::random::<f64>() < success_probability;
        
        // Create experience
        let experience = Experience {
            timestamp: Utc::now(),
            task_type: task.task_type.clone(),
            success,
            context: task.description.clone(),
            learned_insight: if success {
                Some(format!("Successfully completed {} task", task.task_type))
            } else {
                Some(format!("Failed {} task - need improvement", task.task_type))
            },
        };

        self.learn_from_experience(experience);
        self.state = AgentState::Idle;

        Ok(TaskResult {
            task_id: task.id,
            agent_id: self.id,
            success,
            output: if success {
                format!("Task completed successfully by agent {}", self.name)
            } else {
                format!("Task failed - agent {} needs more training", self.name)
            },
            error_message: if success { None } else { Some("Task execution failed".to_string()) },
            execution_time: (rand::random::<u64>() % 10000 + 1000),
            completed_at: Utc::now(),
            quality_score: Some(if success { 0.8 } else { 0.2 }),
            learned_insights: Vec::new(),
        })
    }

    async fn communicate(&mut self, message: &str, target_agent: Option<Uuid>) -> anyhow::Result<String> {
        self.state = AgentState::Communicating;
        
        // Simulate communication processing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let response = match target_agent {
            Some(target) => format!("Agent {} responding to {}: Acknowledged - {}", self.name, target, message),
            None => format!("Agent {} broadcasting: {}", self.name, message),
        };

        self.state = AgentState::Idle;
        Ok(response)
    }

    async fn learn(&mut self, nlp_processor: &NLPProcessor) -> anyhow::Result<()> {
        self.state = AgentState::Learning;
        
        // Analyze recent experiences for patterns
        let recent_experiences: Vec<_> = self.memory.experiences
            .iter()
            .rev()
            .take(10)
            .collect();

        for experience in recent_experiences {
            if let Some(insight) = &experience.learned_insight {
                let tokens: Vec<String> = insight.split_whitespace().map(|s| s.to_string()).collect();
                let sentiment = nlp_processor.analyze_sentiment(&tokens);
                let pattern_key = format!("{}_{}", experience.task_type, if experience.success { "success" } else { "failure" });
                
                let current_pattern_strength = self.memory.learned_patterns
                    .get(&pattern_key)
                    .unwrap_or(&0.0);
                
                let new_strength = current_pattern_strength + sentiment * 0.1;
                self.memory.learned_patterns.insert(pattern_key, new_strength.clamp(-1.0, 1.0));
            }
        }

        self.state = AgentState::Idle;
        Ok(())
    }

    async fn update_position(&mut self, swarm_center: (f64, f64), neighbors: &[Agent]) -> anyhow::Result<()> {
        // Implement boids-like flocking behavior
        let mut separation = (0.0, 0.0);
        let mut alignment = (0.0, 0.0);
        let mut cohesion = (0.0, 0.0);
        
        let mut neighbor_count = 0;
        
        for neighbor in neighbors {
            if neighbor.id == self.id {
                continue;
            }
            
            let distance = ((neighbor.position.0 - self.position.0).powi(2) + 
                           (neighbor.position.1 - self.position.1).powi(2)).sqrt();
            
            if distance < 50.0 { // Interaction radius
                neighbor_count += 1;
                
                // Separation: steer away from nearby neighbors
                if distance < 20.0 {
                    separation.0 += self.position.0 - neighbor.position.0;
                    separation.1 += self.position.1 - neighbor.position.1;
                }
                
                // Alignment: steer towards average heading of neighbors
                alignment.0 += neighbor.position.0;
                alignment.1 += neighbor.position.1;
                
                // Cohesion: steer towards average position of neighbors
                cohesion.0 += neighbor.position.0;
                cohesion.1 += neighbor.position.1;
            }
        }
        
        if neighbor_count > 0 {
            // Normalize forces
            alignment.0 /= neighbor_count as f64;
            alignment.1 /= neighbor_count as f64;
            cohesion.0 /= neighbor_count as f64;
            cohesion.1 /= neighbor_count as f64;
        }
        
        // Apply forces with weights
        let new_x = self.position.0 + 
                   separation.0 * 0.1 + 
                   alignment.0 * 0.05 + 
                   cohesion.0 * 0.05 +
                   (swarm_center.0 - self.position.0) * 0.01;
                   
        let new_y = self.position.1 + 
                   separation.1 * 0.1 + 
                   alignment.1 * 0.05 + 
                   cohesion.1 * 0.05 +
                   (swarm_center.1 - self.position.1) * 0.01;
        
        self.position = (new_x, new_y);
        Ok(())
    }
}