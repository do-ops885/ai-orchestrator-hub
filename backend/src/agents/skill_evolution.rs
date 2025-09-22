//! # Agent Skill Evolution System
//!
//! This module provides dynamic skill learning and evolution capabilities for agents.
//! Agents can learn new skills, improve existing ones, and adapt their capabilities
//! based on task performance and environmental feedback.

use crate::agents::{Agent, AgentBehavior, AgentCapability, CommunicationComplexity};
use crate::communication::patterns::CommunicationConfig;
use crate::communication::protocols::{MessageEnvelope, MessagePayload, MessageType};
use crate::neural::NLPProcessor;
use crate::utils::error::HiveResult;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Skill evolution system that manages agent learning and capability development
#[derive(Debug)]
pub struct SkillEvolutionSystem {
    /// Learning patterns and skill templates
    skill_library: Arc<RwLock<SkillLibrary>>,
    /// Evolution policies and rules
    evolution_policies: Arc<RwLock<Vec<EvolutionPolicy>>>,
    /// Learning history and analytics
    learning_history: Arc<RwLock<Vec<LearningEvent>>>,
    /// NLP processor for skill analysis
    nlp_processor: Arc<NLPProcessor>,
    /// Configuration parameters
    config: SkillEvolutionConfig,
}

/// Library of available skills and learning patterns
#[derive(Debug, Clone)]
pub struct SkillLibrary {
    /// Available skill templates
    pub skill_templates: HashMap<String, SkillTemplate>,
    /// Learning pathways between skills
    pub learning_pathways: HashMap<String, Vec<SkillProgression>>,
    /// Skill categories and hierarchies
    pub skill_categories: HashMap<String, SkillCategory>,
}

/// Template for a learnable skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTemplate {
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub difficulty_level: u8, // 1-10
    pub prerequisites: Vec<String>,
    pub learning_time_hours: f64,
    pub base_proficiency: f64,
    pub max_proficiency: f64,
    pub learning_curve: LearningCurve,
    pub related_skills: Vec<String>,
}

/// How a skill's proficiency improves over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningCurve {
    /// Linear improvement
    Linear { rate: f64 },
    /// Exponential improvement (fast start, then plateaus)
    Exponential {
        initial_rate: f64,
        decay_factor: f64,
    },
    /// Logarithmic improvement (slow start, then accelerates)
    Logarithmic { base_rate: f64, acceleration: f64 },
    /// S-curve (slow start, rapid middle, plateau at end)
    SCurve {
        initial_rate: f64,
        peak_rate: f64,
        plateau_threshold: f64,
    },
}

/// Progression path between related skills
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProgression {
    pub from_skill: String,
    pub to_skill: String,
    pub transfer_efficiency: f64, // How much proficiency transfers (0.0-1.0)
    pub unlock_threshold: f64,    // Minimum proficiency needed to unlock next skill
    pub synergy_bonus: f64,       // Bonus when both skills are present
}

/// Category of related skills
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCategory {
    pub category_id: String,
    pub name: String,
    pub description: String,
    pub skills: Vec<String>,
    pub category_bonuses: Vec<CategoryBonus>,
}

/// Bonus effects for having multiple skills in a category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryBonus {
    pub required_skills: usize,
    pub bonus_type: BonusType,
    pub bonus_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BonusType {
    /// Increases learning rate for all skills in category
    LearningRateBonus,
    /// Increases maximum proficiency for all skills
    ProficiencyBonus,
    /// Reduces energy cost for using skills
    EfficiencyBonus,
    /// Unlocks special combined abilities
    SynergyUnlock(String),
}

/// Policy that governs when and how agents learn new skills
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionPolicy {
    pub policy_id: Uuid,
    pub name: String,
    pub triggers: Vec<LearningTrigger>,
    pub skill_selection: SkillSelectionStrategy,
    pub learning_parameters: LearningParameters,
    pub enabled: bool,
    pub priority: u8,
}

/// Conditions that trigger skill learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningTrigger {
    /// Task failure rate exceeds threshold
    TaskFailureThreshold {
        failure_rate: f64,
        skill_category: String,
    },
    /// Performance plateau detected
    PerformancePlateau { stagnation_period_hours: f64 },
    /// New task type encountered
    NewTaskType { task_pattern: String },
    /// Collaboration opportunity detected
    CollaborationOpportunity { required_skills: Vec<String> },
    /// Scheduled learning time
    ScheduledLearning { interval_hours: f64 },
    /// Peer agent has superior skills
    PeerComparison { skill_gap_threshold: f64 },
}

/// Strategy for selecting which skills to learn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillSelectionStrategy {
    /// Learn skills that address current weaknesses
    AddressWeaknesses,
    /// Learn skills that complement existing strengths
    BuildOnStrengths,
    /// Learn skills based on task demand patterns
    TaskDemandBased,
    /// Learn skills that enable collaboration
    CollaborationFocused,
    /// Learn skills randomly for exploration
    RandomExploration { exploration_rate: f64 },
}

/// Parameters that control the learning process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningParameters {
    pub max_concurrent_learning: usize,
    pub learning_time_multiplier: f64,
    pub proficiency_gain_rate: f64,
    pub forgetting_rate: f64,
    pub energy_cost_per_hour: f64,
}

/// Record of a learning event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvent {
    pub event_id: Uuid,
    pub agent_id: Uuid,
    pub skill_id: String,
    pub event_type: LearningEventType,
    pub timestamp: DateTime<Utc>,
    pub proficiency_before: f64,
    pub proficiency_after: f64,
    pub learning_time_hours: f64,
    pub energy_cost: f64,
    pub trigger_reason: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningEventType {
    Acquisition,
    Improvement,
    Transfer,
    Forgetting,
    Synergy,
}

/// Configuration for skill evolution behavior
#[derive(Debug, Clone)]
pub struct SkillEvolutionConfig {
    pub evaluation_interval_minutes: u64,
    pub max_skills_per_agent: usize,
    pub enable_skill_forgetting: bool,
    pub enable_skill_transfer: bool,
    pub enable_peer_learning: bool,
    pub learning_efficiency_factor: f64,
}

#[async_trait]
impl AgentBehavior for SkillEvolutionSystem {
    async fn execute_task(
        &mut self,
        _task: crate::tasks::Task,
    ) -> HiveResult<crate::tasks::TaskResult> {
        // Skill evolution systems don't execute tasks directly
        Err(crate::utils::error::HiveError::AgentExecutionFailed {
            reason: "SkillEvolutionSystem does not execute tasks directly".to_string(),
        })
    }

    async fn communicate(
        &mut self,
        envelope: MessageEnvelope,
    ) -> HiveResult<Option<MessageEnvelope>> {
        // Standardized communication pattern for skill evolution
        let complexity = match envelope.priority {
            crate::communication::patterns::MessagePriority::Low => CommunicationComplexity::Simple,
            crate::communication::patterns::MessagePriority::Normal => {
                CommunicationComplexity::Standard
            }
            crate::communication::patterns::MessagePriority::High => {
                CommunicationComplexity::Complex
            }
            crate::communication::patterns::MessagePriority::Critical => {
                CommunicationComplexity::Heavy
            }
        };

        // Use standardized delay based on complexity
        let delay_ms = match complexity {
            CommunicationComplexity::Simple => 50,
            CommunicationComplexity::Standard => 100,
            CommunicationComplexity::Complex => 200,
            CommunicationComplexity::Heavy => 500,
        };

        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

        match envelope.message_type {
            MessageType::Request => {
                let response_payload = match &envelope.payload {
                    MessagePayload::Text(text) => MessagePayload::Text(format!(
                        "Skill evolution system acknowledging: {text} - Managing skill development"
                    )),
                    MessagePayload::Json(json) => {
                        let stats = self.get_evolution_stats().await;
                        MessagePayload::Json(serde_json::json!({
                            "response": "Skill evolution system ready",
                            "evolution_stats": stats,
                            "original_request": json
                        }))
                    }
                    _ => MessagePayload::Text(
                        "Skill evolution system acknowledged message".to_string(),
                    ),
                };

                let response = MessageEnvelope::new_response(
                    &envelope,
                    uuid::Uuid::new_v4(),
                    response_payload,
                );
                Ok(Some(response))
            }
            MessageType::Broadcast => {
                tracing::info!(
                    "Skill evolution system received broadcast: {:?}",
                    envelope.payload
                );
                Ok(None)
            }
            MessageType::CoordinationRequest => {
                // Handle coordination for skill development
                if let MessagePayload::CoordinationData {
                    performance_metrics,
                    ..
                } = &envelope.payload
                {
                    tracing::info!(
                        "Received coordination data for skill evolution: {:?}",
                        performance_metrics
                    );
                }
                Ok(None)
            }
            _ => {
                let response = MessageEnvelope::new_response(
                    &envelope,
                    uuid::Uuid::new_v4(),
                    MessagePayload::Text(format!(
                        "Skill evolution system processed message of type {:?}",
                        envelope.message_type
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
        // Simulate processing time for skill evolution
        tokio::time::sleep(timeout / 4).await;

        let stats = self.get_evolution_stats().await;
        let response = MessageEnvelope::new_response(
            &request,
            uuid::Uuid::new_v4(),
            MessagePayload::Json(serde_json::json!({
                "response": "Skill evolution system processed request",
                "evolution_statistics": stats,
                "processing_timeout": timeout.as_millis()
            })),
        );

        Ok(response)
    }

    async fn learn(&mut self, _nlp_processor: &NLPProcessor) -> HiveResult<()> {
        // Skill evolution is inherently about learning
        debug!("Skill evolution system learning triggered - core functionality");
        Ok(())
    }

    async fn update_position(
        &mut self,
        _swarm_center: (f64, f64),
        _neighbors: &[Agent],
    ) -> HiveResult<()> {
        // Skill evolution systems don't participate in swarm positioning
        Ok(())
    }

    fn get_communication_config(&self) -> CommunicationConfig {
        CommunicationConfig {
            default_timeout: std::time::Duration::from_secs(25),
            max_retries: 3,
            retry_delay: std::time::Duration::from_millis(200),
            max_concurrent_messages: 60,
            buffer_size: 4096,
            enable_compression: true,
            delivery_guarantee: crate::communication::patterns::DeliveryGuarantee::AtLeastOnce,
        }
    }
}

impl SkillEvolutionSystem {
    /// Create a new skill evolution system
    pub fn new(nlp_processor: Arc<NLPProcessor>, config: SkillEvolutionConfig) -> Self {
        Self {
            skill_library: Arc::new(RwLock::new(SkillLibrary::create_default())),
            evolution_policies: Arc::new(RwLock::new(Self::create_default_policies())),
            learning_history: Arc::new(RwLock::new(Vec::new())),
            nlp_processor,
            config,
        }
    }

    /// Start the skill evolution evaluation loop
    pub fn start_skill_evolution(&self, agents: Arc<dashmap::DashMap<Uuid, Agent>>) {
        let skill_library = Arc::clone(&self.skill_library);
        let evolution_policies = Arc::clone(&self.evolution_policies);
        let learning_history = Arc::clone(&self.learning_history);
        let nlp_processor = Arc::clone(&self.nlp_processor);
        let config = self.config.clone();
        let interval_minutes = config.evaluation_interval_minutes;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                config.evaluation_interval_minutes * 60,
            ));

            loop {
                interval.tick().await;

                if let Err(e) = Self::evaluate_skill_evolution(
                    &skill_library,
                    &evolution_policies,
                    &learning_history,
                    &nlp_processor,
                    &config,
                    Arc::clone(&agents),
                )
                .await
                {
                    warn!("Skill evolution evaluation failed: {}", e);
                }
            }
        });

        info!(
            "Skill evolution system started with {} minute intervals",
            interval_minutes
        );
    }

    /// Evaluate all agents for skill learning opportunities
    async fn evaluate_skill_evolution(
        skill_library: &Arc<RwLock<SkillLibrary>>,
        evolution_policies: &Arc<RwLock<Vec<EvolutionPolicy>>>,
        learning_history: &Arc<RwLock<Vec<LearningEvent>>>,
        nlp_processor: &Arc<NLPProcessor>,
        config: &SkillEvolutionConfig,
        agents: Arc<dashmap::DashMap<Uuid, Agent>>,
    ) -> Result<()> {
        let policies = evolution_policies.read().await;
        let library = skill_library.read().await;

        for agent_entry in agents.iter() {
            let agent_id = *agent_entry.key();
            let agent = agent_entry.value().clone();

            // Evaluate each policy for this agent
            for policy in policies.iter() {
                if !policy.enabled {
                    continue;
                }

                // Check if policy triggers apply to this agent
                if Self::should_trigger_learning(&agent, policy, &library) {
                    info!(
                        "Learning policy '{}' triggered for agent {}",
                        policy.name, agent_id
                    );

                    // Select skill to learn
                    if let Some(skill_to_learn) =
                        Self::select_skill_to_learn(&agent, policy, &library, nlp_processor)
                    {
                        // Execute learning
                        let learning_result = Self::execute_skill_learning(
                            agent_id,
                            &skill_to_learn,
                            &policy.learning_parameters,
                            &library,
                            config,
                        );

                        // Record learning event
                        let mut history = learning_history.write().await;
                        let learning_event = LearningEvent {
                            event_id: Uuid::new_v4(),
                            agent_id,
                            skill_id: skill_to_learn.to_string(),
                            event_type: LearningEventType::Acquisition,
                            timestamp: Utc::now(),
                            proficiency_before: 0.0,
                            proficiency_after: learning_result
                                .as_ref()
                                .map(|r| r.new_proficiency)
                                .unwrap_or(0.0),
                            learning_time_hours: learning_result
                                .as_ref()
                                .map(|r| r.learning_time)
                                .unwrap_or(0.0),
                            energy_cost: learning_result
                                .as_ref()
                                .map(|r| r.energy_cost)
                                .unwrap_or(0.0),
                            trigger_reason: format!("Policy '{}' triggered", policy.name),
                            success: learning_result.is_ok(),
                        };

                        history.push(learning_event);

                        if let Ok(result) = learning_result {
                            // Update agent with new skill
                            if let Some(mut agent_ref) = agents.get_mut(&agent_id) {
                                let new_capability = AgentCapability {
                                    name: skill_to_learn.to_string(),
                                    proficiency: result.new_proficiency,
                                    learning_rate: result.learning_rate,
                                };
                                agent_ref.add_capability(new_capability);
                                info!(
                                    "Agent {} learned new skill with proficiency {:.2}",
                                    agent_id, result.new_proficiency
                                );
                            }
                        } else {
                            warn!(
                                "Skill learning failed for agent {}: {:?}",
                                agent_id, learning_result
                            );
                        }

                        break; // Only execute one learning action per evaluation cycle
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if learning should be triggered for an agent
    fn should_trigger_learning(
        agent: &Agent,
        policy: &EvolutionPolicy,
        _library: &SkillLibrary,
    ) -> bool {
        for trigger in &policy.triggers {
            match trigger {
                LearningTrigger::TaskFailureThreshold { failure_rate, .. } => {
                    // Calculate agent's recent failure rate
                    let recent_failures = agent
                        .memory
                        .experiences
                        .iter()
                        .rev()
                        .take(10)
                        .filter(|exp| !exp.success)
                        .count();
                    let recent_tasks = agent.memory.experiences.len().min(10);

                    if recent_tasks > 0 {
                        let agent_failure_rate = recent_failures as f64 / recent_tasks as f64;
                        if agent_failure_rate > *failure_rate {
                            return true;
                        }
                    }
                }
                LearningTrigger::PerformancePlateau {
                    stagnation_period_hours,
                } => {
                    // Check if performance has plateaued
                    let hours_since_improvement = 24.0; // Placeholder calculation
                    if hours_since_improvement > *stagnation_period_hours {
                        return true;
                    }
                }
                LearningTrigger::NewTaskType { .. }
                | LearningTrigger::CollaborationOpportunity { .. } => {
                    // Check if agent has encountered new task types or collaboration opportunities
                    // This would analyze recent task patterns or opportunities
                    return false; // Placeholder
                }
                LearningTrigger::ScheduledLearning { interval_hours } => {
                    // Check if it's time for scheduled learning
                    let hours_since_last_learning = 25.0; // Placeholder calculation
                    if hours_since_last_learning >= *interval_hours {
                        return true;
                    }
                }
                LearningTrigger::PeerComparison {
                    skill_gap_threshold,
                } => {
                    // Compare with peer agents
                    let skill_gap = 0.3; // Placeholder calculation
                    if skill_gap > *skill_gap_threshold {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Select which skill an agent should learn
    fn select_skill_to_learn(
        agent: &Agent,
        policy: &EvolutionPolicy,
        library: &SkillLibrary,
        _nlp_processor: &Arc<NLPProcessor>,
    ) -> Option<String> {
        let existing_skills: Vec<String> = agent
            .capabilities
            .iter()
            .map(|cap| cap.name.clone())
            .collect();

        match &policy.skill_selection {
            SkillSelectionStrategy::AddressWeaknesses => {
                // Find skills that could improve weak areas
                for (skill_id, template) in &library.skill_templates {
                    if !existing_skills.contains(skill_id) {
                        // Check if prerequisites are met
                        if template
                            .prerequisites
                            .iter()
                            .all(|prereq| existing_skills.contains(prereq))
                        {
                            return Some(skill_id.clone());
                        }
                    }
                }
            }
            SkillSelectionStrategy::BuildOnStrengths => {
                // Find skills that complement existing strong skills
                let strongest_skill = agent.capabilities.iter().max_by(|a, b| {
                    a.proficiency
                        .partial_cmp(&b.proficiency)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                if let Some(strong_skill) = strongest_skill {
                    if let Some(template) = library.skill_templates.get(&strong_skill.name) {
                        for related_skill in &template.related_skills {
                            if !existing_skills.contains(related_skill) {
                                if let Some(related_template) =
                                    library.skill_templates.get(related_skill)
                                {
                                    if related_template
                                        .prerequisites
                                        .iter()
                                        .all(|prereq| existing_skills.contains(prereq))
                                    {
                                        return Some(related_skill.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            SkillSelectionStrategy::TaskDemandBased => {
                // Analyze recent task patterns to determine needed skills
                // This would use NLP to analyze task descriptions
                return Some("task_analysis".to_string()); // Placeholder
            }
            SkillSelectionStrategy::CollaborationFocused => {
                // Select skills that enable better collaboration
                return Some("communication".to_string()); // Placeholder
            }
            SkillSelectionStrategy::RandomExploration { .. } => {
                // Randomly select an available skill
                let available_skills: Vec<String> = library
                    .skill_templates
                    .keys()
                    .filter(|skill_id| !existing_skills.contains(*skill_id))
                    .cloned()
                    .collect();

                if !available_skills.is_empty() {
                    let random_index = rand::random::<usize>() % available_skills.len();
                    return Some(available_skills[random_index].clone());
                }
            }
        }

        None
    }

    /// Execute the skill learning process
    fn execute_skill_learning(
        _agent_id: Uuid,
        skill_id: &str,
        learning_params: &LearningParameters,
        library: &SkillLibrary,
        _config: &SkillEvolutionConfig,
    ) -> Result<SkillLearningResult> {
        let template = library
            .skill_templates
            .get(skill_id)
            .ok_or_else(|| anyhow::anyhow!("Skill template not found: {skill_id}"))?;

        // Calculate learning time and proficiency gain
        let learning_time = template.learning_time_hours * learning_params.learning_time_multiplier;
        let energy_cost = learning_time * learning_params.energy_cost_per_hour;

        // Apply learning curve
        let new_proficiency = match &template.learning_curve {
            LearningCurve::Linear { rate } => {
                (template.base_proficiency + rate * learning_time).min(template.max_proficiency)
            }
            LearningCurve::Exponential {
                initial_rate,
                decay_factor,
            } => {
                let progress = 1.0 - (-decay_factor * learning_time).exp();
                template.base_proficiency
                    + (template.max_proficiency - template.base_proficiency)
                        * progress
                        * initial_rate
            }
            LearningCurve::Logarithmic {
                base_rate,
                acceleration,
            } => template.base_proficiency + base_rate * (1.0 + learning_time * acceleration).ln(),
            LearningCurve::SCurve {
                initial_rate,
                peak_rate,
                plateau_threshold,
            } => {
                let progress = learning_time / template.learning_time_hours;
                let rate = if progress < 0.3 {
                    initial_rate
                } else if progress < 0.7 {
                    peak_rate
                } else {
                    &(initial_rate * (1.0 - (progress - 0.7) / 0.3 * (1.0 - plateau_threshold)))
                };
                (template.base_proficiency + rate * learning_time).min(template.max_proficiency)
            }
        };

        Ok(SkillLearningResult {
            new_proficiency,
            learning_time,
            energy_cost,
            learning_rate: learning_params.proficiency_gain_rate,
        })
    }

    /// Create default evolution policies
    fn create_default_policies() -> Vec<EvolutionPolicy> {
        vec![
            EvolutionPolicy {
                policy_id: Uuid::new_v4(),
                name: "Address Task Failures".to_string(),
                triggers: vec![LearningTrigger::TaskFailureThreshold {
                    failure_rate: 0.3,
                    skill_category: "general".to_string(),
                }],
                skill_selection: SkillSelectionStrategy::AddressWeaknesses,
                learning_parameters: LearningParameters {
                    max_concurrent_learning: 2,
                    learning_time_multiplier: 1.0,
                    proficiency_gain_rate: 0.1,
                    forgetting_rate: 0.01,
                    energy_cost_per_hour: 5.0,
                },
                enabled: true,
                priority: 1,
            },
            EvolutionPolicy {
                policy_id: Uuid::new_v4(),
                name: "Scheduled Skill Development".to_string(),
                triggers: vec![LearningTrigger::ScheduledLearning {
                    interval_hours: 48.0,
                }],
                skill_selection: SkillSelectionStrategy::BuildOnStrengths,
                learning_parameters: LearningParameters {
                    max_concurrent_learning: 1,
                    learning_time_multiplier: 0.8,
                    proficiency_gain_rate: 0.15,
                    forgetting_rate: 0.005,
                    energy_cost_per_hour: 3.0,
                },
                enabled: true,
                priority: 2,
            },
        ]
    }

    /// Get skill evolution statistics
    pub async fn get_evolution_stats(&self) -> serde_json::Value {
        let history = self.learning_history.read().await;
        let library = self.skill_library.read().await;

        let total_learning_events = history.len();
        let successful_events = history.iter().filter(|e| e.success).count();
        let skills_learned = history
            .iter()
            .filter(|e| matches!(e.event_type, LearningEventType::Acquisition))
            .count();

        let skill_popularity: HashMap<String, usize> =
            history.iter().fold(HashMap::new(), |mut acc, event| {
                *acc.entry(event.skill_id.clone()).or_insert(0) += 1;
                acc
            });

        serde_json::json!({
            "total_learning_events": total_learning_events,
            "successful_events": successful_events,
            "skills_learned": skills_learned,
            "success_rate": if total_learning_events > 0 {
                successful_events as f64 / total_learning_events as f64
            } else { 0.0 },
            "available_skills": library.skill_templates.len(),
            "skill_categories": library.skill_categories.len(),
            "most_popular_skills": skill_popularity.iter()
                .map(|(skill, count)| serde_json::json!({ "skill": skill, "learning_count": count }))
                .collect::<Vec<_>>()
        })
    }
}

/// Result of a skill learning attempt
#[derive(Debug)]
pub struct SkillLearningResult {
    pub new_proficiency: f64,
    pub learning_time: f64,
    pub energy_cost: f64,
    pub learning_rate: f64,
}

impl SkillLibrary {
    /// Create a default skill library with common skills
    #[must_use]
    pub fn create_default() -> Self {
        let mut skill_templates = HashMap::new();
        let learning_pathways = HashMap::new();
        let mut skill_categories = HashMap::new();

        // Define basic skills
        skill_templates.insert(
            "communication".to_string(),
            SkillTemplate {
                skill_id: "communication".to_string(),
                name: "Communication".to_string(),
                description: "Ability to communicate effectively with other agents".to_string(),
                category: "social".to_string(),
                difficulty_level: 3,
                prerequisites: vec![],
                learning_time_hours: 8.0,
                base_proficiency: 0.3,
                max_proficiency: 0.95,
                learning_curve: LearningCurve::Linear { rate: 0.08 },
                related_skills: vec!["collaboration".to_string(), "negotiation".to_string()],
            },
        );

        skill_templates.insert(
            "problem_solving".to_string(),
            SkillTemplate {
                skill_id: "problem_solving".to_string(),
                name: "Problem Solving".to_string(),
                description: "Analytical thinking and problem resolution skills".to_string(),
                category: "cognitive".to_string(),
                difficulty_level: 5,
                prerequisites: vec![],
                learning_time_hours: 12.0,
                base_proficiency: 0.2,
                max_proficiency: 0.9,
                learning_curve: LearningCurve::SCurve {
                    initial_rate: 0.05,
                    peak_rate: 0.15,
                    plateau_threshold: 0.8,
                },
                related_skills: vec!["analysis".to_string(), "creativity".to_string()],
            },
        );

        skill_templates.insert(
            "task_optimization".to_string(),
            SkillTemplate {
                skill_id: "task_optimization".to_string(),
                name: "Task Optimization".to_string(),
                description: "Ability to optimize task execution and resource usage".to_string(),
                category: "technical".to_string(),
                difficulty_level: 6,
                prerequisites: vec!["problem_solving".to_string()],
                learning_time_hours: 15.0,
                base_proficiency: 0.1,
                max_proficiency: 0.85,
                learning_curve: LearningCurve::Exponential {
                    initial_rate: 0.12,
                    decay_factor: 0.1,
                },
                related_skills: vec!["efficiency".to_string(), "resource_management".to_string()],
            },
        );

        // Define skill categories
        skill_categories.insert(
            "social".to_string(),
            SkillCategory {
                category_id: "social".to_string(),
                name: "Social Skills".to_string(),
                description: "Skills related to interaction and collaboration".to_string(),
                skills: vec!["communication".to_string()],
                category_bonuses: vec![CategoryBonus {
                    required_skills: 2,
                    bonus_type: BonusType::LearningRateBonus,
                    bonus_value: 0.2,
                }],
            },
        );

        skill_categories.insert(
            "cognitive".to_string(),
            SkillCategory {
                category_id: "cognitive".to_string(),
                name: "Cognitive Skills".to_string(),
                description: "Mental processing and analytical abilities".to_string(),
                skills: vec!["problem_solving".to_string()],
                category_bonuses: vec![CategoryBonus {
                    required_skills: 3,
                    bonus_type: BonusType::ProficiencyBonus,
                    bonus_value: 0.15,
                }],
            },
        );

        Self {
            skill_templates,
            learning_pathways,
            skill_categories,
        }
    }
}

impl Default for SkillEvolutionConfig {
    fn default() -> Self {
        Self {
            evaluation_interval_minutes: 60, // Check every hour
            max_skills_per_agent: 10,
            enable_skill_forgetting: true,
            enable_skill_transfer: true,
            enable_peer_learning: true,
            learning_efficiency_factor: 1.0,
        }
    }
}
