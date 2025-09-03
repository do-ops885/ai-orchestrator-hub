//! # Intelligent Fallback System
//!
//! This module implements a sophisticated multi-tier fallback mechanism for the Universal Orchestrator.
//! The system provides graceful degradation when specialized agents are unavailable, ensuring
//! tasks are completed with appropriate quality standards.
//!
//! ## Fallback Tiers
//!
//! 1. **Primary Tier**: Specialized agents with exact capability matches
//! 2. **Secondary Tier**: General-purpose agents with sufficient capabilities
//! 3. **Tertiary Tier**: Cross-trained agents with partial capability matches
//! 4. **Emergency Tier**: Any available agent with basic capabilities
//!
//! ## Key Features
//!
//! - **Real-time Agent Availability**: Continuous monitoring of agent states and capabilities
//! - **Capability Matching**: Intelligent matching across agent types and proficiency levels
//! - **Progressive Degradation**: Graceful quality reduction with minimum thresholds
//! - **Context Preservation**: Maintains task context and requirements through fallback
//! - **Emergency Generalization**: Enhanced monitoring for critical situations
//! - **Transparent Logging**: Comprehensive tracking of fallback decisions and outcomes

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::agents::{Agent, AgentCapability, AgentState, AgentType};
use crate::tasks::{Task, TaskRequiredCapability};

/// Configuration for the intelligent fallback system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    /// Enable/disable the fallback system
    pub enabled: bool,
    /// Maximum number of fallback attempts per task
    pub max_fallback_attempts: usize,
    /// Minimum quality threshold for fallback acceptance (0.0 to 1.0)
    pub min_quality_threshold: f64,
    /// Emergency mode quality threshold (lower for critical tasks)
    pub emergency_quality_threshold: f64,
    /// Enable emergency generalization for critical tasks
    pub enable_emergency_generalization: bool,
    /// Time window for agent availability checks (in seconds)
    pub availability_check_window: u64,
    /// Enable detailed fallback logging
    pub detailed_logging: bool,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_fallback_attempts: 3,
            min_quality_threshold: 0.6,
            emergency_quality_threshold: 0.3,
            enable_emergency_generalization: true,
            availability_check_window: 300, // 5 minutes
            detailed_logging: true,
        }
    }
}

/// Represents the different tiers of fallback options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FallbackTier {
    /// Primary: Specialized agents with exact capability matches
    Primary,
    /// Secondary: General-purpose agents with sufficient capabilities
    Secondary,
    /// Tertiary: Cross-trained agents with partial capability matches
    Tertiary,
    /// Emergency: Any available agent with basic capabilities
    Emergency,
}

impl FallbackTier {
    /// Get the priority level of the tier (lower number = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            FallbackTier::Primary => 1,
            FallbackTier::Secondary => 2,
            FallbackTier::Tertiary => 3,
            FallbackTier::Emergency => 4,
        }
    }

    /// Get the quality degradation factor for this tier
    pub fn quality_factor(&self) -> f64 {
        match self {
            FallbackTier::Primary => 1.0,
            FallbackTier::Secondary => 0.9,
            FallbackTier::Tertiary => 0.7,
            FallbackTier::Emergency => 0.5,
        }
    }
}

/// Result of a fallback attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackResult {
    /// Whether a suitable agent was found
    pub success: bool,
    /// The tier used for the fallback
    pub tier: FallbackTier,
    /// ID of the selected agent (if any)
    pub selected_agent: Option<Uuid>,
    /// Quality score of the selected agent for this task
    pub quality_score: f64,
    /// Reason for the fallback decision
    pub reason: String,
    /// Timestamp of the fallback attempt
    pub timestamp: DateTime<Utc>,
    /// Additional context about the fallback
    pub context: HashMap<String, String>,
}

/// Comprehensive fallback decision record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackDecision {
    /// Unique ID for this fallback attempt
    pub id: Uuid,
    /// Task that triggered the fallback
    pub task_id: Uuid,
    /// Original agent assignment attempt
    pub original_attempt: Option<Uuid>,
    /// Sequence of fallback results
    pub attempts: Vec<FallbackResult>,
    /// Final successful assignment (if any)
    pub final_assignment: Option<Uuid>,
    /// Total time spent on fallback process
    pub total_duration_ms: u64,
    /// Whether the fallback was successful
    pub successful: bool,
    /// Overall quality degradation
    pub quality_degradation: f64,
    /// Timestamp when fallback process started
    pub started_at: DateTime<Utc>,
    /// Timestamp when fallback process completed
    pub completed_at: Option<DateTime<Utc>>,
}

/// Statistics for fallback system performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackStats {
    /// Total number of fallback attempts
    pub total_attempts: u64,
    /// Number of successful fallbacks
    pub successful_fallbacks: u64,
    /// Number of failed fallbacks (no agent found)
    pub failed_fallbacks: u64,
    /// Average quality degradation across all fallbacks
    pub average_quality_degradation: f64,
    /// Distribution of fallback tiers used
    pub tier_distribution: HashMap<FallbackTier, u64>,
    /// Average time spent on fallback decisions
    pub average_decision_time_ms: f64,
    /// Success rate by tier
    pub tier_success_rates: HashMap<FallbackTier, f64>,
}

/// Intelligent fallback system for agent selection
pub struct IntelligentFallback {
    /// Configuration for the fallback system
    config: FallbackConfig,
    /// Statistics tracking
    stats: FallbackStats,
    /// Active fallback decisions
    active_decisions: HashMap<Uuid, FallbackDecision>,
    /// Historical fallback decisions
    decision_history: Vec<FallbackDecision>,
}

impl IntelligentFallback {
    /// Create a new intelligent fallback system
    pub fn new(config: FallbackConfig) -> Self {
        Self {
            config,
            stats: FallbackStats {
                total_attempts: 0,
                successful_fallbacks: 0,
                failed_fallbacks: 0,
                average_quality_degradation: 0.0,
                tier_distribution: HashMap::new(),
                average_decision_time_ms: 0.0,
                tier_success_rates: HashMap::new(),
            },
            active_decisions: HashMap::new(),
            decision_history: Vec::new(),
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(FallbackConfig::default())
    }

    /// Find the best agent for a task using intelligent fallback
    pub async fn find_agent_with_fallback(
        &mut self,
        task: &Task,
        available_agents: &[Agent],
        task_context: Option<&HashMap<String, String>>,
    ) -> FallbackDecision {
        let start_time = Utc::now();
        let decision_id = Uuid::new_v4();

        let mut decision = FallbackDecision {
            id: decision_id,
            task_id: task.id,
            original_attempt: None,
            attempts: Vec::new(),
            final_assignment: None,
            total_duration_ms: 0,
            successful: false,
            quality_degradation: 0.0,
            started_at: start_time,
            completed_at: None,
        };

        if !self.config.enabled {
            // Fallback disabled, try direct assignment
            if let Some(agent) = self.find_best_agent_direct(task, available_agents) {
                decision.final_assignment = Some(agent.id);
                decision.successful = true;
                decision.quality_degradation = 1.0;
            }
            decision.completed_at = Some(Utc::now());
            decision.total_duration_ms = (Utc::now() - start_time).num_milliseconds() as u64;
            return decision;
        }

        // Try each fallback tier in order
        let tiers = vec![
            FallbackTier::Primary,
            FallbackTier::Secondary,
            FallbackTier::Tertiary,
            FallbackTier::Emergency,
        ];

        for tier in tiers {
            if decision.attempts.len() >= self.config.max_fallback_attempts {
                break;
            }

            let result = self
                .try_fallback_tier(task, available_agents, &tier, task_context)
                .await;

            decision.attempts.push(result.clone());

            if result.success {
                decision.final_assignment = result.selected_agent;
                decision.successful = true;
                decision.quality_degradation = result.quality_score;
                break;
            }
        }

        // Update statistics
        self.update_stats(&decision);

        decision.completed_at = Some(Utc::now());
        decision.total_duration_ms = (Utc::now() - start_time).num_milliseconds() as u64;

        // Store decision
        self.active_decisions.insert(decision_id, decision.clone());
        self.decision_history.push(decision.clone());

        // Clean up old decisions (keep last 1000)
        if self.decision_history.len() > 1000 {
            self.decision_history.remove(0);
        }

        decision
    }

    /// Try to find an agent in a specific fallback tier
    async fn try_fallback_tier(
        &self,
        task: &Task,
        available_agents: &[Agent],
        tier: &FallbackTier,
        task_context: Option<&HashMap<String, String>>,
    ) -> FallbackResult {
        let start_time = Utc::now();

        // Filter agents based on tier criteria
        let eligible_agents: Vec<&Agent> = available_agents
            .iter()
            .filter(|agent| self.is_agent_eligible_for_tier(agent, tier))
            .collect();

        if eligible_agents.is_empty() {
            return FallbackResult {
                success: false,
                tier: tier.clone(),
                selected_agent: None,
                quality_score: 0.0,
                reason: format!("No agents available for {} tier", self.tier_name(tier)),
                timestamp: Utc::now(),
                context: HashMap::new(),
            };
        }

        // Find best agent in this tier
        let mut best_agent = None;
        let mut best_score = 0.0;
        let mut best_reason = String::new();

        for agent in &eligible_agents {
            let (score, reason) =
                self.calculate_agent_fitness_for_tier(agent, task, tier, task_context);

            if score > best_score {
                best_score = score;
                best_agent = Some(agent.id);
                best_reason = reason;
            }
        }

        let quality_threshold = if matches!(tier, FallbackTier::Emergency)
            && self.config.enable_emergency_generalization
        {
            self.config.emergency_quality_threshold
        } else {
            self.config.min_quality_threshold
        };

        let success = best_score >= quality_threshold * tier.quality_factor();

        FallbackResult {
            success,
            tier: tier.clone(),
            selected_agent: best_agent,
            quality_score: best_score,
            reason: if success {
                format!(
                    "Selected agent with score {:.2} in {} tier: {}",
                    best_score,
                    self.tier_name(tier),
                    best_reason
                )
            } else {
                format!(
                    "No agent met quality threshold {:.2} in {} tier (best: {:.2})",
                    quality_threshold * tier.quality_factor(),
                    self.tier_name(tier),
                    best_score
                )
            },
            timestamp: Utc::now(),
            context: {
                let mut ctx = HashMap::new();
                ctx.insert("tier".to_string(), self.tier_name(tier));
                ctx.insert(
                    "eligible_agents".to_string(),
                    eligible_agents.len().to_string(),
                );
                ctx.insert(
                    "quality_threshold".to_string(),
                    (quality_threshold * tier.quality_factor()).to_string(),
                );
                ctx.insert(
                    "decision_time_ms".to_string(),
                    (Utc::now() - start_time).num_milliseconds().to_string(),
                );
                ctx
            },
        }
    }

    /// Check if an agent is eligible for a specific fallback tier
    fn is_agent_eligible_for_tier(&self, agent: &Agent, tier: &FallbackTier) -> bool {
        // Check if agent is available
        if !matches!(agent.state, AgentState::Idle) {
            return false;
        }

        // Check if agent has sufficient energy
        if agent.energy < 10.0 {
            return false;
        }

        // Check tier-specific eligibility
        match tier {
            FallbackTier::Primary => {
                // Primary tier: any agent type
                true
            }
            FallbackTier::Secondary => {
                // Secondary tier: prefer general-purpose agents
                matches!(agent.agent_type, AgentType::Worker | AgentType::Coordinator)
            }
            FallbackTier::Tertiary => {
                // Tertiary tier: cross-trained agents (agents with multiple capabilities)
                agent.capabilities.len() >= 2
            }
            FallbackTier::Emergency => {
                // Emergency tier: any agent with basic capabilities
                !agent.capabilities.is_empty()
            }
        }
    }

    /// Calculate fitness score for an agent in a specific tier
    fn calculate_agent_fitness_for_tier(
        &self,
        agent: &Agent,
        task: &Task,
        tier: &FallbackTier,
        task_context: Option<&HashMap<String, String>>,
    ) -> (f64, String) {
        let mut total_score = 0.0;
        let mut total_weight = 0.0;
        let mut reasons = Vec::new();

        // Base capability matching
        for req_cap in &task.required_capabilities {
            let agent_proficiency = agent.get_capability_score(&req_cap.name);
            let weight = 1.0;

            if agent_proficiency >= req_cap.minimum_proficiency {
                total_score += agent_proficiency * weight;
                reasons.push(format!("{}: {:.2}", req_cap.name, agent_proficiency));
            } else {
                // Partial credit for close matches in higher tiers
                let partial_score = agent_proficiency / req_cap.minimum_proficiency;
                total_score += partial_score * weight * 0.5;
                reasons.push(format!(
                    "{}: {:.2} (partial)",
                    req_cap.name, agent_proficiency
                ));
            }

            total_weight += weight;
        }

        // Apply tier-specific modifiers
        let tier_modifier = match tier {
            FallbackTier::Primary => 1.0,
            FallbackTier::Secondary => {
                // Boost general-purpose agents
                if matches!(agent.agent_type, AgentType::Worker) {
                    1.1
                } else {
                    0.9
                }
            }
            FallbackTier::Tertiary => {
                // Boost agents with diverse capabilities
                let diversity_bonus = (agent.capabilities.len() as f64 / 5.0).min(1.2);
                diversity_bonus
            }
            FallbackTier::Emergency => {
                // Emergency: prioritize availability over specialization
                0.8
            }
        };

        total_score *= tier_modifier;

        // Context-based adjustments
        if let Some(context) = task_context {
            if let Some(priority) = context.get("priority") {
                if priority == "high" && matches!(tier, FallbackTier::Emergency) {
                    total_score *= 1.2; // Boost emergency tier for high priority tasks
                    reasons.push("High priority boost".to_string());
                }
            }
        }

        let final_score = if total_weight > 0.0 {
            (total_score / total_weight).min(1.0)
        } else {
            0.5 // Default score when no specific requirements
        };

        (final_score, reasons.join(", "))
    }

    /// Find best agent using direct assignment (no fallback)
    fn find_best_agent_direct(&self, task: &Task, available_agents: &[Agent]) -> Option<&Agent> {
        let mut best_agent = None;
        let mut best_fitness = 0.0;

        for agent in available_agents {
            if matches!(agent.state, AgentState::Idle) && agent.energy >= 10.0 {
                let fitness = agent.calculate_task_fitness(task);
                if fitness > best_fitness {
                    best_fitness = fitness;
                    best_agent = Some(agent);
                }
            }
        }

        best_agent
    }

    /// Update statistics based on a fallback decision
    fn update_stats(&mut self, decision: &FallbackDecision) {
        self.stats.total_attempts += 1;

        if decision.successful {
            self.stats.successful_fallbacks += 1;
        } else {
            self.stats.failed_fallbacks += 1;
        }

        // Update tier distribution
        for attempt in &decision.attempts {
            let count = self
                .stats
                .tier_distribution
                .entry(attempt.tier.clone())
                .or_insert(0);
            *count += 1;
        }

        // Update average quality degradation
        let total_degradation: f64 = self
            .decision_history
            .iter()
            .map(|d| d.quality_degradation)
            .sum();
        self.stats.average_quality_degradation =
            total_degradation / self.decision_history.len() as f64;

        // Update average decision time
        let total_time: u64 = self
            .decision_history
            .iter()
            .map(|d| d.total_duration_ms)
            .sum();
        self.stats.average_decision_time_ms =
            total_time as f64 / self.decision_history.len() as f64;
    }

    /// Get human-readable name for a fallback tier
    fn tier_name(&self, tier: &FallbackTier) -> &'static str {
        match tier {
            FallbackTier::Primary => "Primary",
            FallbackTier::Secondary => "Secondary",
            FallbackTier::Tertiary => "Tertiary",
            FallbackTier::Emergency => "Emergency",
        }
    }

    /// Get current fallback statistics
    pub fn get_stats(&self) -> &FallbackStats {
        &self.stats
    }

    /// Get recent fallback decisions
    pub fn get_recent_decisions(&self, limit: usize) -> Vec<&FallbackDecision> {
        self.decision_history.iter().rev().take(limit).collect()
    }

    /// Get active fallback decisions
    pub fn get_active_decisions(&self) -> Vec<&FallbackDecision> {
        self.active_decisions.values().collect()
    }

    /// Clean up completed fallback decisions
    pub fn cleanup_completed_decisions(&mut self) {
        let now = Utc::now();
        self.active_decisions.retain(|_, decision| {
            decision.completed_at.map_or(true, |completed| {
                (now - completed).num_seconds() < 3600 // Keep for 1 hour
            })
        });
    }

    /// Update configuration
    pub fn update_config(&mut self, config: FallbackConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &FallbackConfig {
        &self.config
    }
}

impl Default for IntelligentFallback {
    fn default() -> Self {
        Self::new(FallbackConfig::default())
    }
}
