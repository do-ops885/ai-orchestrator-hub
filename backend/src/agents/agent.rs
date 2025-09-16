//! # Agent System
//!
//! This module defines the core agent types, behaviors, and capabilities within
//! the multiagent hive system. Agents are autonomous entities that can execute
//! tasks, learn from experiences, and communicate with other agents.
//!
//! ## Agent Types
//!
//! - **Worker**: General-purpose agents for task execution
//! - **Coordinator**: Leadership agents that manage other agents
//! - **Specialist**: Domain-specific agents with specialized capabilities
//! - **Learner**: Agents focused on continuous learning and adaptation
//!
//! ## Agent States
//!
//! Agents transition through various states during their lifecycle:
//! - **Idle**: Available for task assignment
//! - **Working**: Actively executing tasks
//! - **Learning**: Processing new patterns and insights
//! - **Communicating**: Coordinating with other agents
//! - **Failed**: Error state requiring intervention

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::communication::patterns::{CommunicationConfig, MessagePriority};
use crate::communication::protocols::{MessageEnvelope, MessagePayload, MessageType};
use crate::neural::NLPProcessor;
use crate::tasks::{Task, TaskResult};
use crate::utils::error::HiveResult;

/// Defines the different types of agents in the hive system.
///
/// Each agent type has specific roles and capabilities within the swarm:
/// - Workers handle general task execution
/// - Coordinators manage and direct other agents
/// - Specialists have domain-specific expertise
/// - Learners focus on continuous improvement and adaptation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentType {
    /// General-purpose agent for task execution
    Worker,
    /// Leadership agent that coordinates other agents
    Coordinator,
    /// Specialized agent with domain-specific expertise
    /// The String parameter specifies the specialization area
    Specialist(String),
    /// Agent focused on learning and adaptation
    Learner,
}

/// Represents the current operational state of an agent.
///
/// Agents transition between these states based on their current activities
/// and the tasks they are assigned. State transitions are managed by the
/// hive coordinator and can be monitored for system health.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AgentState {
    /// Agent is available and waiting for task assignment
    Idle,
    /// Agent is actively executing a task
    Working,
    /// Agent is processing new information or improving capabilities
    Learning,
    /// Agent is coordinating with other agents
    Communicating,
    /// Agent has encountered an error and requires intervention
    Failed,
    /// Agent is temporarily inactive
    Inactive,
}

impl std::fmt::Display for AgentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentState::Idle => write!(f, "idle"),
            AgentState::Working => write!(f, "working"),
            AgentState::Learning => write!(f, "learning"),
            AgentState::Communicating => write!(f, "communicating"),
            AgentState::Failed => write!(f, "failed"),
            AgentState::Inactive => write!(f, "inactive"),
        }
    }
}

/// Represents a specific capability that an agent possesses.
///
/// Capabilities define what an agent can do and how well they can do it.
/// They can improve over time through learning and experience.
///
/// # Examples
///
/// ```rust
/// use multiagent_hive::AgentCapability;
///
/// let capability = AgentCapability {
///     name: "data_processing".to_string(),
///     proficiency: 0.85,
///     learning_rate: 0.1,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    /// Name of the capability (e.g., "`data_processing`", "`communication`")
    pub name: String,
    /// Current proficiency level from 0.0 (novice) to 1.0 (expert)
    pub proficiency: f64,
    /// Rate at which this capability improves through experience (0.0 to 1.0)
    pub learning_rate: f64,
}

/// Legacy agent memory system - kept for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMemory {
    /// Past experiences and task outcomes
    pub experiences: Vec<Experience>,
    /// Learned patterns with confidence scores
    pub learned_patterns: HashMap<String, f64>,
    /// Social connections with trust scores (`agent_id` -> `trust_score`)
    pub social_connections: HashMap<Uuid, f64>,
}

impl Default for AgentMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentMemory {
    #[must_use]
    pub fn new() -> Self {
        Self {
            experiences: Vec::new(),
            learned_patterns: HashMap::new(),
            social_connections: HashMap::new(),
        }
    }
}

/// A recorded experience from task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    /// When the experience occurred
    pub timestamp: DateTime<Utc>,
    /// Type of task that was executed
    pub task_type: String,
    /// Whether the task was completed successfully
    pub success: bool,
    /// Contextual information about the task
    pub context: String,
    /// Any insights learned from this experience
    pub learned_insight: Option<String>,
}

/// Communication complexity levels for consistent processing delays
#[derive(Debug, Clone, Copy)]
pub enum CommunicationComplexity {
    /// Simple acknowledgment or status update
    Simple,
    /// Standard message processing
    Standard,
    /// Complex analysis or multi-step processing
    Complex,
    /// Heavy computation or large data processing
    Heavy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
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
    #[must_use]
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

    /// Standardized async operation wrapper for consistent state management.
    ///
    /// This helper ensures all agent operations follow the same pattern:
    /// 1. Save previous state
    /// 2. Set working state
    /// 3. Update last_active timestamp
    /// 4. Execute operation
    /// 5. Restore to Idle state (or previous state on error)
    ///
    /// # Arguments
    /// * `working_state` - The state to set during operation (e.g., Working, Communicating)
    /// * `operation` - The async operation to execute
    ///
    /// # Returns
    /// Result of the operation, with guaranteed state restoration
    pub fn execute_with_state_management<T, F>(
        &mut self,
        working_state: AgentState,
        operation: F,
    ) -> HiveResult<T>
    where
        F: FnOnce(&mut Self) -> HiveResult<T>,
    {
        let _previous_state = &self.state;
        self.state = working_state;
        self.last_active = Utc::now();

        let result = operation(self);

        // Always restore state, even on error
        self.state = AgentState::Idle;

        result
    }

    /// Standardized communication delay helper.
    ///
    /// Provides consistent processing delays based on operation complexity.
    /// Use this instead of direct tokio::time::sleep calls for consistency.
    pub async fn communication_delay(&self, complexity: CommunicationComplexity) {
        let delay_ms = match complexity {
            CommunicationComplexity::Simple => 50,
            CommunicationComplexity::Standard => 100,
            CommunicationComplexity::Complex => 200,
            CommunicationComplexity::Heavy => 500,
        };

        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
    }
    /// Add a capability to the agent
    pub fn add_capability(&mut self, capability: AgentCapability) {
        self.capabilities.push(capability);
    }

    /// Get the proficiency score for a specific capability
    #[must_use]
    pub fn get_capability_score(&self, capability_name: &str) -> f64 {
        self.capabilities
            .iter()
            .find(|c| c.name == capability_name)
            .map_or(0.0, |c| c.proficiency)
    }

    /// Learn from task execution experience
    pub fn learn_from_experience(&mut self, experience: Experience) {
        // Update capability proficiency based on experience
        if let Some(capability) = self
            .capabilities
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

    /// Update social connection trust score with another agent
    pub fn update_social_connection(&mut self, agent_id: Uuid, interaction_success: bool) {
        let current_trust = self
            .memory
            .social_connections
            .get(&agent_id)
            .copied()
            .unwrap_or(0.5);
        let adjustment = if interaction_success { 0.1 } else { -0.1 };
        let new_trust = (current_trust + adjustment).clamp(0.0, 1.0);
        self.memory.social_connections.insert(agent_id, new_trust);
    }

    /// Check if agent can perform a given task
    #[must_use]
    pub fn can_perform_task(&self, task: &Task) -> bool {
        if task.required_capabilities.is_empty() {
            true
        } else {
            let required = &task.required_capabilities;
            required.iter().all(|req_cap| {
                self.get_capability_score(&req_cap.name) >= req_cap.minimum_proficiency
            })
        }
    }

    /// Calculate how well-suited this agent is for a task
    #[must_use]
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

/// Standardized agent behavior interface with consistent async communication patterns.
///
/// ## Standardized Async Patterns:
/// - **State Management**: Always save previous state, set appropriate working state, restore to Idle
/// - **Error Handling**: Use HiveResult for all operations
/// - **Timing**: Use tokio::time::sleep for processing delays
/// - **Performance**: Track execution time and update last_active timestamp
/// - **Resource Management**: Clean up resources and restore state in all code paths
/// - **Communication**: Use standardized MessageEnvelope for all inter-agent communication
///
/// ## Implementation Guidelines:
/// 1. Always update `last_active` timestamp at start of operation
/// 2. Use consistent state transitions: Idle → Working → Idle
/// 3. Handle errors gracefully without leaving agent in inconsistent state
/// 4. Use appropriate processing delays based on operation complexity
/// 5. Update performance metrics when applicable
/// 6. Use MessageEnvelope for all communication instead of raw strings
#[async_trait]
pub trait AgentBehavior {
    /// Execute a task with standardized state management and error handling.
    async fn execute_task(&mut self, task: Task) -> HiveResult<TaskResult>;

    /// Communicate with other agents using standardized message envelopes.
    async fn communicate(
        &mut self,
        envelope: MessageEnvelope,
    ) -> HiveResult<Option<MessageEnvelope>>;

    /// Send a message and wait for response with timeout.
    async fn request_response(
        &mut self,
        request: MessageEnvelope,
        timeout: std::time::Duration,
    ) -> HiveResult<MessageEnvelope>;

    /// Learn from experiences with consistent processing patterns.
    async fn learn(&mut self, nlp_processor: &NLPProcessor) -> HiveResult<()>;

    /// Update agent position in swarm with optimized calculations.
    async fn update_position(
        &mut self,
        swarm_center: (f64, f64),
        neighbors: &[Agent],
    ) -> HiveResult<()>;

    /// Get communication configuration for this agent.
    fn get_communication_config(&self) -> CommunicationConfig {
        CommunicationConfig::default()
    }
}

#[async_trait]
impl AgentBehavior for Agent {
    async fn execute_task(&mut self, task: Task) -> HiveResult<TaskResult> {
        // Standardized state management
        let _previous_state = &self.state;
        self.state = AgentState::Working;
        self.last_active = Utc::now();

        let start_time = std::time::Instant::now();

        // Simulate task execution with some randomness
        let fitness = self.calculate_task_fitness(&task);
        let success_probability = fitness * 0.8 + 0.2; // 20% base success rate

        // Simulate processing time
        let processing_time = rand::random::<u64>() % 5000 + 500; // 500-5500ms
        tokio::time::sleep(tokio::time::Duration::from_millis(processing_time)).await;

        let success = rand::random::<f64>() < success_probability;
        let execution_time = start_time.elapsed().as_millis() as u64;

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

        // Standardized state restoration
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
            error_message: if success {
                None
            } else {
                Some("Task execution failed".to_string())
            },
            execution_time,
            completed_at: Utc::now(),
            quality_score: Some(if success { 0.8 } else { 0.2 }),
            learned_insights: Vec::new(),
        })
    }

    async fn communicate(
        &mut self,
        envelope: MessageEnvelope,
    ) -> HiveResult<Option<MessageEnvelope>> {
        // Use standardized communication pattern
        // Use standardized communication delay based on message priority
        let complexity = match envelope.priority {
            MessagePriority::Low => CommunicationComplexity::Simple,
            MessagePriority::Normal => CommunicationComplexity::Standard,
            MessagePriority::High => CommunicationComplexity::Complex,
            MessagePriority::Critical => CommunicationComplexity::Heavy,
        };
        self.communication_delay(complexity).await;

        // Process the message based on type
        match envelope.message_type {
            MessageType::Request => {
                // Create a response envelope
                let response_payload = match &envelope.payload {
                    MessagePayload::Text(text) => MessagePayload::Text(format!(
                        "Agent {} responding to {}: Acknowledged - {}",
                        self.name, envelope.sender_id, text
                    )),
                    MessagePayload::Json(json) => MessagePayload::Json(serde_json::json!({
                        "response": format!("Agent {} acknowledged request", self.name),
                        "original_request": json
                    })),
                    _ => MessagePayload::Text(format!("Agent {} acknowledged message", self.name)),
                };

                let response = MessageEnvelope::new_response(&envelope, self.id, response_payload);
                Ok(Some(response))
            }
            MessageType::Broadcast => {
                // For broadcasts, we don't send a response but log the message
                tracing::info!(
                    "Agent {} received broadcast from {}: {:?}",
                    self.name,
                    envelope.sender_id,
                    envelope.payload
                );
                Ok(None)
            }
            MessageType::TaskAssigned => {
                // Handle task assignment
                if let MessagePayload::TaskInfo { task_id, .. } = &envelope.payload {
                    tracing::info!("Agent {} received task assignment: {}", self.name, task_id);
                }
                Ok(None)
            }
            _ => {
                // Default response for other message types
                let response = MessageEnvelope::new_response(
                    &envelope,
                    self.id,
                    MessagePayload::Text(format!(
                        "Agent {} processed message of type {:?}",
                        self.name, envelope.message_type
                    )),
                );
                Ok(Some(response))
            }
        }
    }

    async fn request_response(
        &mut self,
        request: MessageEnvelope,
        timeout: std::time::Duration,
    ) -> HiveResult<MessageEnvelope> {
        // For now, simulate a simple request-response pattern
        // In a real implementation, this would use the communication manager
        tokio::time::sleep(timeout / 2).await; // Simulate network delay

        let response = MessageEnvelope::new_response(
            &request,
            self.id,
            MessagePayload::Text(format!(
                "Agent {} processed request with timeout {:?}",
                self.name, timeout
            )),
        );

        Ok(response)
    }

    async fn learn(&mut self, nlp_processor: &NLPProcessor) -> HiveResult<()> {
        // Standardized state management
        let _previous_state = &self.state;
        self.state = AgentState::Learning;
        self.last_active = Utc::now();

        let start_time = std::time::Instant::now();

        // Analyze recent experiences for patterns
        let recent_experiences: Vec<_> = self.memory.experiences.iter().rev().take(10).collect();

        for experience in &recent_experiences {
            if let Some(insight) = &experience.learned_insight {
                let tokens: Vec<String> = insight
                    .split_whitespace()
                    .map(ToString::to_string)
                    .collect();
                let sentiment = nlp_processor.analyze_sentiment(&tokens);
                let pattern_key = format!(
                    "{}_{}",
                    experience.task_type,
                    if experience.success {
                        "success"
                    } else {
                        "failure"
                    }
                );

                let current_pattern_strength = self
                    .memory
                    .learned_patterns
                    .get(&pattern_key)
                    .unwrap_or(&0.0);

                let new_strength = current_pattern_strength + sentiment * 0.1;
                self.memory
                    .learned_patterns
                    .insert(pattern_key, new_strength.clamp(-1.0, 1.0));
            }
        }

        // Simulate learning processing time
        let learning_time = std::cmp::max(50, recent_experiences.len() as u64 * 20);
        tokio::time::sleep(tokio::time::Duration::from_millis(learning_time)).await;

        // Standardized state restoration
        self.state = AgentState::Idle;
        Ok(())
    }

    async fn update_position(
        &mut self,
        swarm_center: (f64, f64),
        neighbors: &[Agent],
    ) -> HiveResult<()> {
        // Implement boids-like flocking behavior
        let mut separation = (0.0, 0.0);
        let mut alignment = (0.0, 0.0);
        let mut cohesion = (0.0, 0.0);

        let mut neighbor_count = 0;

        for neighbor in neighbors {
            if neighbor.id == self.id {
                continue;
            }

            let distance = ((neighbor.position.0 - self.position.0).powi(2)
                + (neighbor.position.1 - self.position.1).powi(2))
            .sqrt();

            if distance < 50.0 {
                // Interaction radius
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
            alignment.0 /= f64::from(neighbor_count);
            alignment.1 /= f64::from(neighbor_count);
            cohesion.0 /= f64::from(neighbor_count);
            cohesion.1 /= f64::from(neighbor_count);
        }

        // Apply forces with weights
        let new_x = self.position.0
            + separation.0 * 0.1
            + alignment.0 * 0.05
            + cohesion.0 * 0.05
            + (swarm_center.0 - self.position.0) * 0.01;

        let new_y = self.position.1
            + separation.1 * 0.1
            + alignment.1 * 0.05
            + cohesion.1 * 0.05
            + (swarm_center.1 - self.position.1) * 0.01;

        self.position = (new_x, new_y);
        Ok(())
    }
}
