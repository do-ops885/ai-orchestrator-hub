//! # Agent Template
//!
//! This template provides a starting point for creating new agents in the AI Orchestrator Hub.
//! It implements the basic structure and patterns required for reliable agent development.
//!
//! ## Usage
//!
//! 1. Copy this template to `backend/src/agents/your_agent_name.rs`
//! 2. Replace `YourAgent` with your specific agent name
//! 3. Implement the required methods and add your specific logic
//! 4. Add the agent to `backend/src/agents/mod.rs`
//! 5. Update the agent creation logic in the main application

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::agents::{
    Agent, AgentBehavior, AgentCapability, AgentState, AgentType, CommunicationComplexity,
    Experience,
};
use crate::neural::NLPProcessor;
use crate::tasks::{Task, TaskResult};
use crate::utils::error::HiveResult;

/// Configuration for YourAgent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YourAgentConfig {
    /// Maximum concurrent tasks this agent can handle
    pub max_concurrent_tasks: usize,
    /// Specialized domain for this agent (if any)
    pub specialization: Option<String>,
    /// Learning rate for capability evolution
    pub learning_rate: f64,
    /// Custom configuration parameters
    pub custom_settings: HashMap<String, serde_json::Value>,
}

impl Default for YourAgentConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 3,
            specialization: None,
            learning_rate: 0.1,
            custom_settings: HashMap::new(),
        }
    }
}

/// Your custom agent implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YourAgent {
    /// Base agent structure
    pub agent: Agent,
    /// Agent-specific configuration
    pub config: YourAgentConfig,
    /// Agent-specific state
    pub custom_state: YourAgentState,
    /// Performance tracking
    pub performance_metrics: YourAgentMetrics,
}

/// Agent-specific state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YourAgentState {
    /// Current task being processed
    pub current_task: Option<Uuid>,
    /// Processing queue
    pub task_queue: Vec<Uuid>,
    /// Custom state fields
    pub custom_data: HashMap<String, serde_json::Value>,
}

/// Performance metrics for your agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YourAgentMetrics {
    /// Tasks completed successfully
    pub tasks_completed: u64,
    /// Tasks failed
    pub tasks_failed: u64,
    /// Average processing time
    pub avg_processing_time_ms: f64,
    /// Custom performance indicators
    pub custom_metrics: HashMap<String, f64>,
}

impl YourAgent {
    /// Create a new instance of YourAgent
    #[must_use]
    pub fn new(name: String, config: YourAgentConfig) -> Self {
        let agent_type = if config.specialization.is_some() {
            AgentType::Specialist(config.specialization.clone().unwrap())
        } else {
            AgentType::Worker
        };

        let mut agent = Agent::new(name, agent_type);

        // Add default capabilities
        agent.add_capability(AgentCapability {
            name: "task_processing".to_string(),
            proficiency: 0.8,
            learning_rate: config.learning_rate,
        });

        // Add specialized capabilities based on configuration
        if let Some(specialization) = &config.specialization {
            agent.add_capability(AgentCapability {
                name: specialization.clone(),
                proficiency: 0.7,
                learning_rate: config.learning_rate,
            });
        }

        Self {
            agent,
            config,
            custom_state: YourAgentState {
                current_task: None,
                task_queue: Vec::new(),
                custom_data: HashMap::new(),
            },
            performance_metrics: YourAgentMetrics {
                tasks_completed: 0,
                tasks_failed: 0,
                avg_processing_time_ms: 0.0,
                custom_metrics: HashMap::new(),
            },
        }
    }

    /// Validate agent configuration
    pub fn validate_config(&self) -> HiveResult<()> {
        if self.config.max_concurrent_tasks == 0 {
            return Err(crate::utils::error::HiveError::ValidationError(
                "max_concurrent_tasks must be greater than 0".to_string(),
            ));
        }

        if !(0.0..=1.0).contains(&self.config.learning_rate) {
            return Err(crate::utils::error::HiveError::ValidationError(
                "learning_rate must be between 0.0 and 1.0".to_string(),
            ));
        }

        Ok(())
    }

    /// Process a task with your custom logic
    async fn process_task(&mut self, task: &Task) -> HiveResult<TaskResult> {
        let start_time = std::time::Instant::now();

        // Update state to show we're working on this task
        self.custom_state.current_task = Some(task.id);

        // Simulate processing time (replace with your actual logic)
        let processing_time = rand::random::<u64>() % 5000 + 500;
        tokio::time::sleep(tokio::time::Duration::from_millis(processing_time)).await;

        // Your custom task processing logic goes here
        let success = self.perform_task_logic(task).await?;
        let execution_time = start_time.elapsed().as_millis() as u64;

        // Update performance metrics
        self.update_performance_metrics(success, execution_time);

        // Create experience for learning
        let experience = Experience {
            timestamp: Utc::now(),
            task_type: task.task_type.clone(),
            success,
            context: task.description.clone(),
            learned_insight: Some(self.generate_insight(success, task)),
        };

        self.agent.learn_from_experience(experience);

        // Clear current task
        self.custom_state.current_task = None;

        Ok(TaskResult {
            task_id: task.id,
            agent_id: self.agent.id,
            success,
            output: if success {
                format!("Task completed successfully by {}", self.agent.name)
            } else {
                format!("Task failed - {} needs improvement", self.agent.name)
            },
            error_message: if success { None } else { Some("Task execution failed".to_string()) },
            execution_time,
            completed_at: Utc::now(),
            quality_score: Some(if success { 0.8 } else { 0.2 }),
            learned_insights: Vec::new(),
        })
    }

    /// Your custom task processing logic
    async fn perform_task_logic(&self, task: &Task) -> HiveResult<bool> {
        // Implement your specific task processing logic here
        // This is where the core functionality of your agent goes

        // Example: Simple success/failure based on agent proficiency
        let capability_score = self.agent.get_capability_score(&task.task_type);
        let success_probability = capability_score * 0.8 + 0.2; // 20% base success rate

        Ok(rand::random::<f64>() < success_probability)
    }

    /// Generate learning insights from task execution
    fn generate_insight(&self, success: bool, task: &Task) -> String {
        if success {
            format!("Successfully completed {} task", task.task_type)
        } else {
            format!("Failed {} task - need to improve {} capabilities",
                   task.task_type, task.task_type)
        }
    }

    /// Update performance metrics
    fn update_performance_metrics(&mut self, success: bool, execution_time: u64) {
        if success {
            self.performance_metrics.tasks_completed += 1;
        } else {
            self.performance_metrics.tasks_failed += 1;
        }

        // Update rolling average
        let total_tasks = self.performance_metrics.tasks_completed + self.performance_metrics.tasks_failed;
        let current_avg = self.performance_metrics.avg_processing_time_ms;
        self.performance_metrics.avg_processing_time_ms =
            (current_avg * (total_tasks - 1) as f64 + execution_time as f64) / total_tasks as f64;
    }

    /// Get agent health status
    #[must_use]
    pub fn health_status(&self) -> AgentHealth {
        let task_success_rate = if self.performance_metrics.tasks_completed + self.performance_metrics.tasks_failed > 0 {
            self.performance_metrics.tasks_completed as f64 /
            (self.performance_metrics.tasks_completed + self.performance_metrics.tasks_failed) as f64
        } else {
            1.0
        };

        AgentHealth {
            status: if task_success_rate > 0.8 && self.agent.energy > 50.0 {
                HealthStatus::Healthy
            } else if task_success_rate > 0.6 || self.agent.energy > 25.0 {
                HealthStatus::Degraded
            } else {
                HealthStatus::Unhealthy
            },
            task_success_rate,
            energy_level: self.agent.energy,
            queue_depth: self.custom_state.task_queue.len(),
        }
    }
}

/// Health status for the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHealth {
    pub status: HealthStatus,
    pub task_success_rate: f64,
    pub energy_level: f64,
    pub queue_depth: usize,
}

/// Health status levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[async_trait]
impl AgentBehavior for YourAgent {
    async fn execute_task(&mut self, task: Task) -> HiveResult<TaskResult> {
        // Validate we can perform this task
        if !self.agent.can_perform_task(&task) {
            return Err(crate::utils::error::HiveError::IncompatibleTask(
                format!("Agent {} cannot perform task type {}", self.agent.name, task.task_type)
            ));
        }

        // Check if we're at capacity
        if self.custom_state.task_queue.len() >= self.config.max_concurrent_tasks {
            return Err(crate::utils::error::HiveError::ResourceLimitExceeded(
                "Agent at maximum concurrent task capacity".to_string()
            ));
        }

        // Add to queue and process
        self.custom_state.task_queue.push(task.id);
        let result = self.process_task(&task).await;

        // Remove from queue
        if let Some(pos) = self.custom_state.task_queue.iter().position(|&id| id == task.id) {
            self.custom_state.task_queue.remove(pos);
        }

        result
    }

    async fn communicate(
        &mut self,
        message: &str,
        target_agent: Option<Uuid>,
    ) -> HiveResult<String> {
        // Use standardized communication delay
        self.agent.communication_delay(CommunicationComplexity::Standard);

        // Your custom communication logic here
        let response = match target_agent {
            Some(target) => format!(
                "Agent {} responding to {}: Acknowledged - {}",
                self.agent.name, target, message
            ),
            None => format!("Agent {} broadcasting: {}", self.agent.name, message),
        };

        Ok(response)
    }

    async fn learn(&mut self, nlp_processor: &NLPProcessor) -> HiveResult<()> {
        // Your custom learning logic here
        // This is called periodically to allow the agent to learn from experiences

        // Example: Analyze recent experiences for patterns
        let recent_experiences: Vec<_> = self.agent.memory.experiences
            .iter()
            .rev()
            .take(10)
            .collect();

        for experience in recent_experiences {
            if let Some(insight) = &experience.learned_insight {
                // Use NLP processor to analyze insights
                let tokens: Vec<String> = insight
                    .split_whitespace()
                    .map(ToString::to_string)
                    .collect();

                if !tokens.is_empty() {
                    let sentiment = nlp_processor.analyze_sentiment(&tokens);

                    // Update custom metrics based on learning
                    self.performance_metrics.custom_metrics
                        .insert("learning_sentiment".to_string(), sentiment);
                }
            }
        }

        Ok(())
    }

    async fn update_position(
        &mut self,
        swarm_center: (f64, f64),
        neighbors: &[Agent],
    ) -> HiveResult<()> {
        // Your custom position update logic
        // This implements swarm behavior like flocking

        // For now, use the default implementation
        self.agent.update_position(swarm_center, neighbors).await
    }
}

/// Factory function to create YourAgent instances
pub fn create_your_agent(name: String, config: Option<YourAgentConfig>) -> YourAgent {
    let config = config.unwrap_or_default();
    YourAgent::new(name, config)
}

/// Example usage and testing
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_agent_creation() {
        let agent = create_your_agent("TestAgent".to_string(), None);
        assert_eq!(agent.agent.name, "TestAgent");
        assert_eq!(agent.agent.state, AgentState::Idle);
        assert!(agent.agent.energy > 0.0);
    }

    #[test]
    async fn test_config_validation() {
        let config = YourAgentConfig {
            max_concurrent_tasks: 0, // Invalid
            ..Default::default()
        };
        let agent = YourAgent::new("TestAgent".to_string(), config);
        assert!(agent.validate_config().is_err());
    }

    #[test]
    async fn test_health_status() {
        let agent = create_your_agent("TestAgent".to_string(), None);
        let health = agent.health_status();
        assert!(matches!(health.status, HealthStatus::Healthy));
    }
}