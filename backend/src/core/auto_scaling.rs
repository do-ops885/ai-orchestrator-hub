//! # Dynamic Agent Auto-Scaling System
//!
//! This module provides intelligent auto-scaling capabilities that automatically
//! spawn and remove agents based on workload, performance metrics, and resource availability.

use crate::agents::AgentType;
use crate::core::hive::HiveCoordinator;
use crate::infrastructure::ResourceManager;
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Auto-scaling system that manages agent population dynamically
#[derive(Debug)]
pub struct AutoScalingSystem {
    /// Scaling policies and rules
    scaling_policies: Arc<RwLock<Vec<ScalingPolicy>>>,
    /// Historical scaling decisions
    scaling_history: Arc<RwLock<Vec<ScalingEvent>>>,
    /// Current scaling state
    scaling_state: Arc<RwLock<ScalingState>>,
    /// Resource manager for system constraints
    resource_manager: Arc<ResourceManager>,
    /// Configuration parameters
    config: AutoScalingConfig,
}

/// Scaling policy that defines when and how to scale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPolicy {
    pub policy_id: Uuid,
    pub name: String,
    pub trigger_conditions: Vec<ScalingTrigger>,
    pub scaling_action: ScalingAction,
    pub cooldown_period: Duration,
    pub enabled: bool,
    pub priority: u8,
}

/// Conditions that trigger scaling actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingTrigger {
    /// Queue depth exceeds threshold
    QueueDepthThreshold {
        threshold: usize,
        duration_minutes: u32,
    },
    /// Agent utilization above/below threshold
    AgentUtilization {
        min_threshold: f64,
        max_threshold: f64,
    },
    /// Task failure rate exceeds threshold
    TaskFailureRate { threshold: f64, window_minutes: u32 },
    /// Response time exceeds threshold
    ResponseTimeThreshold { threshold_ms: u64, percentile: u8 },
    /// Resource utilization constraints
    ResourceConstraints {
        cpu_threshold: f64,
        memory_threshold: f64,
    },
    /// Time-based scaling (e.g., business hours)
    TimeBasedScaling { schedule: ScalingSchedule },
}

/// Scaling actions to take when triggered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingAction {
    /// Scale up by adding agents
    ScaleUp {
        agent_count: usize,
        agent_type: AgentType,
        capabilities: Vec<String>,
    },
    /// Scale down by removing agents
    ScaleDown {
        agent_count: usize,
        selection_strategy: AgentSelectionStrategy,
    },
    /// Replace underperforming agents
    ReplaceAgents {
        performance_threshold: f64,
        replacement_type: AgentType,
    },
}

/// Strategy for selecting agents to remove during scale-down
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentSelectionStrategy {
    /// Remove least recently used agents
    LeastRecentlyUsed,
    /// Remove lowest performing agents
    LowestPerformance,
    /// Remove agents with highest energy consumption
    HighestEnergyConsumption,
    /// Remove agents with fewest capabilities
    FewestCapabilities,
}

/// Time-based scaling schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingSchedule {
    pub timezone: String,
    pub daily_schedule: Vec<ScheduleEntry>,
    pub weekend_behavior: WeekendBehavior,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub start_hour: u8,
    pub end_hour: u8,
    pub target_agent_count: usize,
    pub agent_types: Vec<AgentType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeekendBehavior {
    SameAsWeekday,
    ReducedCapacity(f64), // Factor to multiply weekday capacity
    CustomSchedule(Vec<ScheduleEntry>),
}

/// Record of a scaling event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub policy_id: Uuid,
    pub trigger_reason: String,
    pub action_taken: ScalingAction,
    pub agents_affected: Vec<Uuid>,
    pub success: bool,
    pub error_message: Option<String>,
    pub metrics_before: ScalingMetrics,
    pub metrics_after: Option<ScalingMetrics>,
}

/// Current state of the auto-scaling system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingState {
    pub last_scaling_event: Option<DateTime<Utc>>,
    pub active_cooldowns: HashMap<Uuid, DateTime<Utc>>, // Policy ID -> Cooldown end time
    pub target_agent_count: usize,
    pub current_agent_count: usize,
    pub scaling_in_progress: bool,
    pub last_evaluation: DateTime<Utc>,
}

/// Metrics captured during scaling events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingMetrics {
    pub timestamp: DateTime<Utc>,
    pub agent_count: usize,
    pub queue_depth: usize,
    pub average_response_time_ms: f64,
    pub agent_utilization: f64,
    pub task_failure_rate: f64,
    pub cpu_usage: f64,
    pub memory_usage: f64,
}

/// Configuration for auto-scaling behavior
#[derive(Debug, Clone)]
pub struct AutoScalingConfig {
    pub evaluation_interval_seconds: u64,
    pub min_agents: usize,
    pub max_agents: usize,
    pub default_cooldown_minutes: u32,
    pub enable_predictive_scaling: bool,
    pub safety_margin_factor: f64,
}

impl AutoScalingSystem {
    /// Create a new auto-scaling system
    #[must_use]
    pub fn new(resource_manager: Arc<ResourceManager>, config: AutoScalingConfig) -> Self {
        Self {
            scaling_policies: Arc::new(RwLock::new(Self::create_default_policies())),
            scaling_history: Arc::new(RwLock::new(Vec::new())),
            scaling_state: Arc::new(RwLock::new(ScalingState {
                last_scaling_event: None,
                active_cooldowns: HashMap::new(),
                target_agent_count: config.min_agents,
                current_agent_count: 0,
                scaling_in_progress: false,
                last_evaluation: Utc::now(),
            })),
            resource_manager,
            config,
        }
    }

    /// Start the auto-scaling evaluation loop
    pub fn start_auto_scaling(&self, hive: Arc<RwLock<HiveCoordinator>>) {
        let policies = Arc::clone(&self.scaling_policies);
        let state = Arc::clone(&self.scaling_state);
        let history = Arc::clone(&self.scaling_history);
        let resource_manager = Arc::clone(&self.resource_manager);
        let config = self.config.clone();
        let interval_seconds = config.evaluation_interval_seconds;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                config.evaluation_interval_seconds,
            ));

            loop {
                interval.tick().await;

                if let Err(e) = Self::evaluate_scaling_policies(
                    &policies,
                    &state,
                    &history,
                    &resource_manager,
                    &config,
                    Arc::clone(&hive),
                )
                .await
                {
                    warn!("Auto-scaling evaluation failed: {}", e);
                }
            }
        });

        info!(
            "Auto-scaling system started with {} second intervals",
            interval_seconds
        );
    }

    /// Evaluate all scaling policies and take action if needed
    async fn evaluate_scaling_policies(
        policies: &Arc<RwLock<Vec<ScalingPolicy>>>,
        state: &Arc<RwLock<ScalingState>>,
        history: &Arc<RwLock<Vec<ScalingEvent>>>,
        resource_manager: &Arc<ResourceManager>,
        config: &AutoScalingConfig,
        hive: Arc<RwLock<HiveCoordinator>>,
    ) -> Result<()> {
        let mut scaling_state = state.write().await;

        // Skip if scaling is already in progress
        if scaling_state.scaling_in_progress {
            debug!("Scaling evaluation skipped - scaling in progress");
            return Ok(());
        }

        // Collect current metrics
        let current_metrics = Self::collect_scaling_metrics(&hive, resource_manager).await?;
        scaling_state.current_agent_count = current_metrics.agent_count;
        scaling_state.last_evaluation = Utc::now();

        // Clean up expired cooldowns
        let now = Utc::now();
        scaling_state
            .active_cooldowns
            .retain(|_, cooldown_end| *cooldown_end > now);

        // Evaluate each policy
        let policies_guard = policies.read().await;
        let mut triggered_policy = None;

        for policy in policies_guard.iter() {
            if !policy.enabled {
                continue;
            }

            // Check if policy is in cooldown
            if let Some(cooldown_end) = scaling_state.active_cooldowns.get(&policy.policy_id) {
                if *cooldown_end > now {
                    continue;
                }
            }

            // Evaluate trigger conditions
            if Self::should_trigger_policy(policy, &current_metrics, &scaling_state)? {
                info!("Scaling policy '{}' triggered", policy.name);
                triggered_policy = Some(policy.clone());
                break;
            }
        }
        drop(policies_guard);

        if let Some(policy) = triggered_policy {
            // Execute scaling action
            scaling_state.scaling_in_progress = true;
            drop(scaling_state);

            let scaling_result =
                Self::execute_scaling_action(&policy, &current_metrics, Arc::clone(&hive), config)
                    .await;

            // Record scaling event
            let mut history_guard = history.write().await;
            let mut state_guard = state.write().await;

            let scaling_event = ScalingEvent {
                event_id: Uuid::new_v4(),
                timestamp: now,
                policy_id: policy.policy_id,
                trigger_reason: format!("Policy '{}' conditions met", policy.name),
                action_taken: policy.scaling_action.clone(),
                agents_affected: scaling_result.as_ref().cloned().unwrap_or_default(),
                success: scaling_result.is_ok(),
                error_message: scaling_result.as_ref().err().map(ToString::to_string),
                metrics_before: current_metrics.clone(),
                metrics_after: None, // Will be updated after action completes
            };

            history_guard.push(scaling_event);
            state_guard.scaling_in_progress = false;
            state_guard.last_scaling_event = Some(now);

            // Set cooldown
            let cooldown_end = now + policy.cooldown_period;
            state_guard
                .active_cooldowns
                .insert(policy.policy_id, cooldown_end);

            if let Err(e) = scaling_result {
                warn!("Scaling action failed: {}", e);
            } else {
                info!("Scaling action completed successfully");
            }
        }

        Ok(())
    }

    /// Check if a policy's trigger conditions are met
    fn should_trigger_policy(
        policy: &ScalingPolicy,
        metrics: &ScalingMetrics,
        _state: &ScalingState,
    ) -> Result<bool> {
        for trigger in &policy.trigger_conditions {
            match trigger {
                ScalingTrigger::QueueDepthThreshold { threshold, .. } => {
                    if metrics.queue_depth >= *threshold {
                        return Ok(true);
                    }
                }
                ScalingTrigger::AgentUtilization {
                    min_threshold,
                    max_threshold,
                } => {
                    if metrics.agent_utilization < *min_threshold
                        || metrics.agent_utilization > *max_threshold
                    {
                        return Ok(true);
                    }
                }
                ScalingTrigger::TaskFailureRate { threshold, .. } => {
                    if metrics.task_failure_rate > *threshold {
                        return Ok(true);
                    }
                }
                ScalingTrigger::ResponseTimeThreshold { threshold_ms, .. } => {
                    if metrics.average_response_time_ms > *threshold_ms as f64 {
                        return Ok(true);
                    }
                }
                ScalingTrigger::ResourceConstraints {
                    cpu_threshold,
                    memory_threshold,
                } => {
                    if metrics.cpu_usage > *cpu_threshold
                        || metrics.memory_usage > *memory_threshold
                    {
                        return Ok(true);
                    }
                }
                ScalingTrigger::TimeBasedScaling { .. } => {
                    // Time-based scaling logic would go here
                    // For now, skip this trigger type
                }
            }
        }
        Ok(false)
    }

    /// Execute a scaling action
    async fn execute_scaling_action(
        policy: &ScalingPolicy,
        _metrics: &ScalingMetrics,
        hive: Arc<RwLock<HiveCoordinator>>,
        config: &AutoScalingConfig,
    ) -> Result<Vec<Uuid>> {
        let mut affected_agents = Vec::new();

        match &policy.scaling_action {
            ScalingAction::ScaleUp {
                agent_count,
                agent_type,
                capabilities,
            } => {
                let hive_guard = hive.read().await;

                for i in 0..*agent_count {
                    // Check if we're within limits
                    let agents_info = hive_guard.get_agents_info().await;
                    let current_count = agents_info["total_agents"].as_u64().unwrap_or(0) as usize;
                    if current_count >= config.max_agents {
                        warn!(
                            "Cannot scale up: maximum agent limit ({}) reached",
                            config.max_agents
                        );
                        break;
                    }

                    // Create agent configuration
                    let agent_config = serde_json::json!({
                        "name": format!("AutoScaled-{}-{}", agent_type.to_string(), i),
                        "type": agent_type.to_string(),
                        "capabilities": capabilities.iter().map(|cap| {
                            serde_json::json!({
                                "name": cap,
                                "proficiency": 0.7,
                                "learning_rate": 0.1
                            })
                        }).collect::<Vec<_>>()
                    });

                    // Create the agent
                    match hive_guard.create_agent(agent_config).await {
                        Ok(agent_id) => {
                            affected_agents.push(agent_id);
                            info!("Auto-scaled up: created agent {}", agent_id);
                        }
                        Err(e) => {
                            warn!("Failed to create auto-scaled agent: {}", e);
                        }
                    }
                }
            }
            ScalingAction::ScaleDown {
                agent_count,
                selection_strategy,
            } => {
                let hive_guard = hive.read().await;
                let agents_info = hive_guard.get_agents_info().await;
                let current_count = agents_info["total_agents"].as_u64().unwrap_or(0) as usize;

                if current_count <= config.min_agents {
                    warn!(
                        "Cannot scale down: minimum agent limit ({}) reached",
                        config.min_agents
                    );
                    return Ok(affected_agents);
                }

                // Select agents to remove based on strategy
                let agents_to_remove =
                    Self::select_agents_for_removal(&hive_guard, *agent_count, selection_strategy)
                        .await?;

                // Drop read lock and get write lock for modification
                drop(hive_guard);
                let hive_guard = hive.write().await;

                for agent_id in agents_to_remove {
                    if hive_guard.remove_agent(agent_id).await.is_ok() {
                        affected_agents.push(agent_id);
                        info!("Auto-scaled down: removed agent {}", agent_id);
                    }
                }
            }
            ScalingAction::ReplaceAgents {
                performance_threshold,
                replacement_type,
            } => {
                let hive_guard = hive.read().await;

                // Find underperforming agents
                let underperforming_agents =
                    Self::find_underperforming_agents(&hive_guard, *performance_threshold).await?;

                // Get agent details before dropping read lock
                let mut agent_details = Vec::new();
                for agent_id in &underperforming_agents {
                    if let Some(agent) = hive_guard.get_agent(*agent_id).await {
                        agent_details.push((*agent_id, agent));
                    }
                }

                // Drop read lock and get write lock for modification
                drop(hive_guard);
                let hive_guard = hive.write().await;

                for (agent_id, old_agent) in agent_details {
                    // Remove underperforming agent
                    if hive_guard.remove_agent(agent_id).await.is_ok() {
                        affected_agents.push(agent_id);

                        // Create replacement agent
                        let replacement_config = serde_json::json!({
                            "name": format!("Replacement-{}", old_agent.name),
                            "type": replacement_type.to_string(),
                            "capabilities": old_agent.capabilities.iter().map(|cap| {
                                serde_json::json!({
                                    "name": cap.name,
                                    "proficiency": cap.proficiency * 1.1, // Slight improvement
                                    "learning_rate": cap.learning_rate
                                })
                            }).collect::<Vec<_>>()
                        });

                        match hive_guard.create_agent(replacement_config).await {
                            Ok(new_agent_id) => {
                                info!(
                                    "Replaced underperforming agent {} with {}",
                                    agent_id, new_agent_id
                                );
                            }
                            Err(e) => {
                                warn!("Failed to create replacement agent: {}", e);
                            }
                        }
                    }
                }
            }
        }

        Ok(affected_agents)
    }

    /// Collect current system metrics for scaling decisions
    async fn collect_scaling_metrics(
        hive: &Arc<RwLock<HiveCoordinator>>,
        resource_manager: &Arc<ResourceManager>,
    ) -> Result<ScalingMetrics> {
        let hive_guard = hive.read().await;
        let task_info = hive_guard.get_tasks_info().await;
        let (system_resources, _, _) = resource_manager.get_system_info().await;

        // Calculate metrics
        let agents_info = hive_guard.get_agents_info().await;
        let agent_count = agents_info["total_agents"].as_u64().unwrap_or(0) as usize;
        let queue_depth = task_info?["work_stealing_queue"]["total_queue_depth"]
            .as_u64()
            .unwrap_or(0) as usize;

        // Calculate agent utilization
        let active_agents = agents_info["active_agents"].as_u64().unwrap_or(0) as usize;
        let agent_utilization = if agent_count > 0 {
            active_agents as f64 / agent_count as f64
        } else {
            0.0
        };

        Ok(ScalingMetrics {
            timestamp: Utc::now(),
            agent_count,
            queue_depth,
            average_response_time_ms: 150.0, // Placeholder - would calculate from actual metrics
            agent_utilization,
            task_failure_rate: 0.05, // Placeholder - would calculate from task history
            cpu_usage: system_resources.cpu_usage,
            memory_usage: system_resources.memory_usage,
        })
    }

    /// Select agents for removal based on strategy
    async fn select_agents_for_removal(
        hive: &HiveCoordinator,
        count: usize,
        strategy: &AgentSelectionStrategy,
    ) -> Result<Vec<Uuid>> {
        let agents = hive.get_all_agents().await;
        let mut candidates: Vec<_> = agents.into_iter().map(|(id, agent)| (id, agent)).collect();

        // Sort based on strategy
        match strategy {
            AgentSelectionStrategy::LeastRecentlyUsed => {
                // Sort by creation time as proxy for last used (since last_task_time doesn't exist)
                candidates.sort_by_key(|(_, agent)| agent.memory.experiences.len());
            }
            AgentSelectionStrategy::LowestPerformance => {
                candidates.sort_by(|(_, a), (_, b)| {
                    let a_perf = a.capabilities.iter().map(|c| c.proficiency).sum::<f64>();
                    let b_perf = b.capabilities.iter().map(|c| c.proficiency).sum::<f64>();
                    a_perf
                        .partial_cmp(&b_perf)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            AgentSelectionStrategy::HighestEnergyConsumption => {
                candidates.sort_by(|(_, a), (_, b)| {
                    b.energy
                        .partial_cmp(&a.energy)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            AgentSelectionStrategy::FewestCapabilities => {
                candidates.sort_by_key(|(_, agent)| agent.capabilities.len());
            }
        }

        Ok(candidates
            .into_iter()
            .take(count)
            .map(|(id, _)| id)
            .collect())
    }

    /// Find agents performing below threshold
    async fn find_underperforming_agents(
        hive: &HiveCoordinator,
        threshold: f64,
    ) -> Result<Vec<Uuid>> {
        let agents = hive.get_all_agents().await;
        let underperforming: Vec<Uuid> = agents
            .into_iter()
            .filter_map(|(id, agent)| {
                let avg_performance = if agent.capabilities.is_empty() {
                    0.0
                } else {
                    agent
                        .capabilities
                        .iter()
                        .map(|c| c.proficiency)
                        .sum::<f64>()
                        / agent.capabilities.len() as f64
                };

                if avg_performance < threshold {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();

        Ok(underperforming)
    }

    /// Create default scaling policies
    fn create_default_policies() -> Vec<ScalingPolicy> {
        vec![
            // Scale up when queue is deep
            ScalingPolicy {
                policy_id: Uuid::new_v4(),
                name: "High Queue Depth Scale Up".to_string(),
                trigger_conditions: vec![ScalingTrigger::QueueDepthThreshold {
                    threshold: 10,
                    duration_minutes: 2,
                }],
                scaling_action: ScalingAction::ScaleUp {
                    agent_count: 2,
                    agent_type: AgentType::Worker,
                    capabilities: vec!["general".to_string(), "task_processing".to_string()],
                },
                cooldown_period: Duration::minutes(5),
                enabled: true,
                priority: 1,
            },
            // Scale down when utilization is low
            ScalingPolicy {
                policy_id: Uuid::new_v4(),
                name: "Low Utilization Scale Down".to_string(),
                trigger_conditions: vec![ScalingTrigger::AgentUtilization {
                    min_threshold: 0.0,
                    max_threshold: 0.3,
                }],
                scaling_action: ScalingAction::ScaleDown {
                    agent_count: 1,
                    selection_strategy: AgentSelectionStrategy::LeastRecentlyUsed,
                },
                cooldown_period: Duration::minutes(10),
                enabled: true,
                priority: 2,
            },
        ]
    }

    /// Get scaling statistics
    pub async fn get_scaling_stats(&self) -> serde_json::Value {
        let history = self.scaling_history.read().await;
        let state = self.scaling_state.read().await;

        let total_events = history.len();
        let successful_events = history.iter().filter(|e| e.success).count();
        let scale_up_events = history
            .iter()
            .filter(|e| matches!(e.action_taken, ScalingAction::ScaleUp { .. }))
            .count();
        let scale_down_events = history
            .iter()
            .filter(|e| matches!(e.action_taken, ScalingAction::ScaleDown { .. }))
            .count();

        serde_json::json!({
            "total_scaling_events": total_events,
            "successful_events": successful_events,
            "scale_up_events": scale_up_events,
            "scale_down_events": scale_down_events,
            "success_rate": if total_events > 0 { successful_events as f64 / total_events as f64 } else { 0.0 },
            "current_state": {
                "target_agent_count": state.target_agent_count,
                "current_agent_count": state.current_agent_count,
                "scaling_in_progress": state.scaling_in_progress,
                "active_cooldowns": state.active_cooldowns.len(),
                "last_evaluation": state.last_evaluation,
                "last_scaling_event": state.last_scaling_event
            }
        })
    }
}

impl Default for AutoScalingConfig {
    fn default() -> Self {
        Self {
            evaluation_interval_seconds: 30,
            min_agents: 2,
            max_agents: 20,
            default_cooldown_minutes: 5,
            enable_predictive_scaling: false,
            safety_margin_factor: 1.2,
        }
    }
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Worker => write!(f, "worker"),
            AgentType::Coordinator => write!(f, "coordinator"),
            AgentType::Learner => write!(f, "learner"),
            AgentType::Specialist(spec) => write!(f, "specialist:{spec}"),
        }
    }
}
