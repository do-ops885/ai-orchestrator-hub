//! # Self-Healing Swarm Agent
//!
//! This module implements an autonomous self-healing swarm agent that monitors
//! system health, detects failures, and orchestrates recovery operations to
//! maintain swarm resilience and optimal performance.
//!
//! ## Key Features
//!
//! - **Proactive Health Monitoring**: Continuous monitoring of agent and system health
//! - **Failure Detection**: Early detection of performance degradation and failures
//! - **Autonomous Recovery**: Self-healing capabilities without human intervention
//! - **Swarm Coordination**: Manages swarm formation and rebalancing
//! - **Resource Management**: Optimizes resource allocation and prevents bottlenecks
//! - **Learning-Based Adaptation**: Improves healing strategies through experience
//!
//! ## Architecture
//!
//! The self-healing agent operates as a specialized coordinator that:
//! 1. Monitors health metrics from all swarm members
//! 2. Detects anomalies and performance degradation
//! 3. Initiates appropriate recovery strategies
//! 4. Coordinates with other agents for distributed healing
//! 5. Learns from past incidents to improve future responses

use crate::agents::agent::{Agent, AgentBehavior, AgentState, AgentType, CommunicationComplexity};
use crate::agents::recovery::AgentRecoveryManager;
use crate::communication::patterns::CommunicationConfig;
use crate::communication::protocols::{MessageEnvelope, MessagePayload, MessageType};
use crate::core::swarm_intelligence::SwarmIntelligenceEngine;
use crate::infrastructure::monitoring::HealthMonitor;
use crate::neural::NLPProcessor;
use crate::tasks::{Task, TaskResult};
use crate::utils::error::HiveResult;
use crate::utils::error_recovery::RecoveryHealthMonitor;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Health status levels for system components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Component is functioning optimally
    Healthy,
    /// Component is functional but showing early warning signs
    Degraded,
    /// Component is experiencing significant issues
    Critical,
    /// Component has failed and requires immediate attention
    Failed,
}

/// Types of failures that can occur in the swarm
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FailureType {
    /// Individual agent has stopped responding
    AgentUnresponsive,
    /// Agent is performing poorly
    AgentPerformanceDegraded,
    /// Communication between agents has failed
    CommunicationFailure,
    /// Resource exhaustion (memory, CPU, etc.)
    ResourceExhaustion,
    /// Task execution is failing repeatedly
    TaskExecutionFailure,
    /// Swarm formation is suboptimal
    SwarmFormationIssue,
    /// Network partition or connectivity issues
    NetworkPartition,
}

/// Recovery strategies available for different types of failures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Restart a failed agent
    AgentRestart,
    /// Redistribute tasks to healthy agents
    TaskRedistribution,
    /// Reform swarm with different configuration
    SwarmReformation,
    /// Scale up resources by adding new agents
    ResourceScaling,
    /// Emergency shutdown and recovery
    EmergencyRecovery,
    /// Graceful degradation mode
    GracefulDegradation,
}

/// Health metrics for a specific agent or system component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub agent_id: Uuid,
    pub status: HealthStatus,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub task_success_rate: f64,
    pub response_time: f64,
    pub energy_level: f64,
    pub last_updated: DateTime<Utc>,
    pub issues: Vec<String>,
}

/// Configuration for the self-healing system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfHealingConfig {
    /// How often to check system health (in seconds)
    pub health_check_interval: u64,
    /// Threshold for considering an agent degraded
    pub degraded_threshold: f64,
    /// Threshold for considering an agent critical
    pub critical_threshold: f64,
    /// Maximum number of recovery attempts before escalation
    pub max_recovery_attempts: u32,
    /// Minimum agents required for swarm operation
    pub min_swarm_size: usize,
    /// Maximum agents allowed in swarm
    pub max_swarm_size: usize,
    /// Enable learning from recovery incidents
    pub enable_learning: bool,
}

impl Default for SelfHealingConfig {
    fn default() -> Self {
        Self {
            health_check_interval: 30,
            degraded_threshold: 0.7,
            critical_threshold: 0.5,
            max_recovery_attempts: 3,
            min_swarm_size: 2,
            max_swarm_size: 20,
            enable_learning: true,
        }
    }
}

/// Incident record for learning and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentRecord {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub failure_type: FailureType,
    pub affected_agents: Vec<Uuid>,
    pub recovery_strategy: RecoveryStrategy,
    pub recovery_success: bool,
    pub recovery_time: u64,
    pub lessons_learned: Vec<String>,
}

/// Self-healing swarm agent that monitors and maintains system health
pub struct SelfHealingSwarmAgent {
    /// Basic agent information
    pub agent: Agent,
    /// Configuration for self-healing behavior
    pub config: SelfHealingConfig,
    /// Health metrics for all monitored agents
    pub health_metrics: Arc<RwLock<HashMap<Uuid, HealthMetrics>>>,
    /// Recovery manager for agent restoration
    pub recovery_manager: AgentRecoveryManager,
    /// Swarm intelligence engine for formation management
    pub swarm_engine: SwarmIntelligenceEngine,
    /// Health monitor for system-wide monitoring
    pub health_monitor: Arc<HealthMonitor>,
    /// Recovery health monitor for recovery-specific metrics
    pub recovery_health_monitor: Arc<RecoveryHealthMonitor>,
    /// Incident history for learning
    pub incident_history: Vec<IncidentRecord>,
    /// Active recovery operations
    pub active_recoveries: HashMap<Uuid, (RecoveryStrategy, u32)>,
    /// Performance thresholds learned from experience
    pub learned_thresholds: HashMap<String, f64>,
}

impl SelfHealingSwarmAgent {
    /// Creates a new self-healing swarm agent
    pub fn new(name: String, config: SelfHealingConfig) -> Self {
        let agent = Agent::new(name, AgentType::Coordinator);

        Self {
            agent,
            config,
            health_metrics: Arc::new(RwLock::new(HashMap::new())),
            recovery_manager: AgentRecoveryManager::new(),
            swarm_engine: SwarmIntelligenceEngine::new(),
            health_monitor: Arc::new(HealthMonitor::new()),
            recovery_health_monitor: Arc::new(RecoveryHealthMonitor::new()),
            incident_history: Vec::new(),
            active_recoveries: HashMap::new(),
            learned_thresholds: HashMap::new(),
        }
    }

    /// Starts the main health monitoring loop
    pub async fn start_health_monitoring(&mut self) -> HiveResult<()> {
        info!("Starting health monitoring for self-healing swarm agent");

        loop {
            // Perform health checks
            if let Err(e) = self.perform_health_checks().await {
                error!("Health check failed: {}", e);
            }

            // Analyze health status and trigger recovery if needed
            if let Err(e) = self.analyze_and_recover().await {
                error!("Analysis and recovery failed: {}", e);
            }

            // Clean up completed recovery operations
            self.cleanup_completed_recoveries();

            // Sleep until next health check
            tokio::time::sleep(tokio::time::Duration::from_secs(
                self.config.health_check_interval,
            ))
            .await;
        }
    }

    /// Performs comprehensive health checks on all monitored agents
    async fn perform_health_checks(&mut self) -> HiveResult<()> {
        debug!("Performing health checks on all monitored agents");

        let mut health_metrics = self.health_metrics.write().await;

        // For now, simulate health checks - in a real implementation,
        // this would query actual agent metrics
        for (agent_id, metrics) in health_metrics.iter_mut() {
            // Simulate health degradation over time
            metrics.task_success_rate *= 0.99; // Slight degradation
            metrics.response_time += rand::random::<f64>() * 10.0 - 5.0; // Random variation
            metrics.energy_level -= 0.01; // Energy depletion
            metrics.last_updated = Utc::now();

            // Update health status based on metrics
            metrics.status = self.calculate_health_status(metrics);

            // Record health metrics
            self.recovery_health_monitor
                .record_health_metric(&agent_id.to_string(), metrics.task_success_rate as f32)
                .await;
        }

        Ok(())
    }

    /// Analyzes current health status and triggers recovery operations
    async fn analyze_and_recover(&mut self) -> HiveResult<()> {
        let health_metrics = self.health_metrics.read().await;

        // Collect actions to perform after dropping the read lock
        let mut to_initiate_recovery = Vec::new();
        let mut to_complete_recovery = Vec::new();

        for (agent_id, metrics) in health_metrics.iter() {
            match metrics.status {
                HealthStatus::Critical | HealthStatus::Failed => {
                    if !self.active_recoveries.contains_key(agent_id) {
                        warn!(
                            "Agent {} is in {:?} state, initiating recovery",
                            agent_id, metrics.status
                        );
                        to_initiate_recovery.push((*agent_id, metrics.clone()));
                    }
                }
                HealthStatus::Degraded => {
                    // Check if degradation is worsening
                    if self.is_degradation_worsening(*agent_id).await {
                        info!(
                            "Agent {} degradation is worsening, initiating proactive recovery",
                            agent_id
                        );
                        to_initiate_recovery.push((*agent_id, metrics.clone()));
                    }
                }
                HealthStatus::Healthy => {
                    // Check if this agent was previously being recovered
                    if self.active_recoveries.contains_key(agent_id) {
                        info!("Agent {} has recovered successfully", agent_id);
                        to_complete_recovery.push(*agent_id);
                    }
                }
            }
        }

        // Drop the read lock
        drop(health_metrics);

        // Perform the actions
        for (agent_id, metrics) in to_initiate_recovery {
            self.initiate_recovery(agent_id, &metrics).await?;
        }

        for agent_id in to_complete_recovery {
            self.complete_recovery(agent_id, true).await?;
        }

        Ok(())
    }

    /// Initiates recovery for a specific agent
    async fn initiate_recovery(
        &mut self,
        agent_id: Uuid,
        metrics: &HealthMetrics,
    ) -> HiveResult<()> {
        let failure_type = self.determine_failure_type(metrics);
        let recovery_strategy = self.select_recovery_strategy(&failure_type, metrics);

        info!(
            "Initiating recovery for agent {} using strategy {:?}",
            agent_id, recovery_strategy
        );

        // Record the start of recovery operation
        self.active_recoveries
            .insert(agent_id, (recovery_strategy.clone(), 0));

        // Execute the recovery strategy
        match self
            .execute_recovery_strategy(agent_id, &recovery_strategy)
            .await
        {
            Ok(()) => {
                info!(
                    "Recovery strategy {:?} executed successfully for agent {}",
                    recovery_strategy, agent_id
                );
            }
            Err(e) => {
                error!(
                    "Recovery strategy {:?} failed for agent {}: {}",
                    recovery_strategy, agent_id, e
                );
                self.escalate_recovery(agent_id).await?;
            }
        }

        Ok(())
    }

    /// Determines the type of failure based on health metrics
    fn determine_failure_type(&self, metrics: &HealthMetrics) -> FailureType {
        if metrics.response_time > 5000.0 {
            FailureType::AgentUnresponsive
        } else if metrics.task_success_rate < 0.3 {
            FailureType::TaskExecutionFailure
        } else if metrics.cpu_usage > 90.0 || metrics.memory_usage > 90.0 {
            FailureType::ResourceExhaustion
        } else if metrics.energy_level < 0.1 {
            FailureType::AgentPerformanceDegraded
        } else {
            FailureType::AgentPerformanceDegraded
        }
    }

    /// Selects the appropriate recovery strategy for a given failure type
    fn select_recovery_strategy(
        &self,
        failure_type: &FailureType,
        _metrics: &HealthMetrics,
    ) -> RecoveryStrategy {
        // Use learned patterns if available
        if let Some(&threshold) = self.learned_thresholds.get(&format!("{:?}", failure_type)) {
            if threshold > 0.8 {
                return RecoveryStrategy::AgentRestart;
            }
        }

        // Default strategy selection based on failure type
        match failure_type {
            FailureType::AgentUnresponsive => RecoveryStrategy::AgentRestart,
            FailureType::AgentPerformanceDegraded => RecoveryStrategy::TaskRedistribution,
            FailureType::ResourceExhaustion => RecoveryStrategy::ResourceScaling,
            FailureType::TaskExecutionFailure => RecoveryStrategy::TaskRedistribution,
            FailureType::SwarmFormationIssue => RecoveryStrategy::SwarmReformation,
            FailureType::CommunicationFailure => RecoveryStrategy::SwarmReformation,
            FailureType::NetworkPartition => RecoveryStrategy::EmergencyRecovery,
        }
    }

    /// Executes a specific recovery strategy
    async fn execute_recovery_strategy(
        &mut self,
        agent_id: Uuid,
        strategy: &RecoveryStrategy,
    ) -> HiveResult<()> {
        match strategy {
            RecoveryStrategy::AgentRestart => {
                // Attempt to restart the agent using the recovery manager
                // In a real implementation, this would interface with the actual agent
                info!("Restarting agent {}", agent_id);

                // Simulate agent restart
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                // Update health metrics to reflect restart
                let mut health_metrics = self.health_metrics.write().await;
                if let Some(metrics) = health_metrics.get_mut(&agent_id) {
                    metrics.energy_level = 1.0;
                    metrics.task_success_rate = 0.8;
                    metrics.response_time = 100.0;
                    metrics.status = HealthStatus::Healthy;
                }
            }

            RecoveryStrategy::TaskRedistribution => {
                info!("Redistributing tasks from agent {}", agent_id);

                // Simulate task redistribution
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                // Mark agent as temporarily inactive
                let mut health_metrics = self.health_metrics.write().await;
                if let Some(metrics) = health_metrics.get_mut(&agent_id) {
                    metrics.status = HealthStatus::Degraded;
                }
            }

            RecoveryStrategy::SwarmReformation => {
                info!("Reforming swarm to exclude problematic agent {}", agent_id);

                // Use swarm intelligence engine to reform
                // This would involve creating new formations without the problematic agent
                tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
            }

            RecoveryStrategy::ResourceScaling => {
                info!("Scaling resources for agent {}", agent_id);

                // Simulate resource scaling
                tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

                let mut health_metrics = self.health_metrics.write().await;
                if let Some(metrics) = health_metrics.get_mut(&agent_id) {
                    metrics.cpu_usage *= 0.7;
                    metrics.memory_usage *= 0.7;
                    metrics.status = HealthStatus::Healthy;
                }
            }

            RecoveryStrategy::EmergencyRecovery => {
                warn!("Performing emergency recovery for agent {}", agent_id);

                // Emergency recovery involves more drastic measures
                tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;

                let mut health_metrics = self.health_metrics.write().await;
                if let Some(metrics) = health_metrics.get_mut(&agent_id) {
                    // Reset to baseline values
                    metrics.energy_level = 0.5;
                    metrics.task_success_rate = 0.6;
                    metrics.response_time = 200.0;
                    metrics.cpu_usage = 30.0;
                    metrics.memory_usage = 40.0;
                    metrics.status = HealthStatus::Degraded;
                }
            }

            RecoveryStrategy::GracefulDegradation => {
                info!("Enabling graceful degradation for agent {}", agent_id);

                // Reduce agent capabilities to maintain basic functionality
                tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

                let mut health_metrics = self.health_metrics.write().await;
                if let Some(metrics) = health_metrics.get_mut(&agent_id) {
                    metrics.status = HealthStatus::Degraded;
                }
            }
        }

        Ok(())
    }

    /// Escalates recovery when initial attempts fail
    async fn escalate_recovery(&mut self, agent_id: Uuid) -> HiveResult<()> {
        if let Some((current_strategy, attempts)) = self.active_recoveries.get_mut(&agent_id) {
            *attempts += 1;

            let strategy = current_strategy.clone();
            if *attempts >= self.config.max_recovery_attempts {
                warn!("Maximum recovery attempts reached for agent {}, escalating to emergency recovery", agent_id);
                *current_strategy = RecoveryStrategy::EmergencyRecovery;
                self.execute_recovery_strategy(agent_id, &strategy).await?;
            } else {
                info!("Retry attempt {} for agent {}", attempts, agent_id);
                self.execute_recovery_strategy(agent_id, &strategy).await?;
            }
        }

        Ok(())
    }

    /// Completes a recovery operation and records the outcome
    async fn complete_recovery(&mut self, agent_id: Uuid, success: bool) -> HiveResult<()> {
        if let Some((strategy, attempts)) = self.active_recoveries.remove(&agent_id) {
            let incident = IncidentRecord {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                failure_type: FailureType::AgentPerformanceDegraded, // Simplified for now
                affected_agents: vec![agent_id],
                recovery_strategy: strategy.clone(),
                recovery_success: success,
                recovery_time: 0, // Would be calculated from start time
                lessons_learned: if success {
                    vec![format!(
                        "Strategy {:?} successful for agent recovery",
                        strategy
                    )]
                } else {
                    vec![format!(
                        "Strategy {:?} failed after {} attempts",
                        strategy, attempts
                    )]
                },
            };

            self.incident_history.push(incident);

            // Learn from the incident if learning is enabled
            if self.config.enable_learning && success {
                self.learn_from_incident(&strategy).await;
            }

            info!(
                "Recovery completed for agent {} with result: {}",
                agent_id, success
            );
        }

        Ok(())
    }

    /// Learns from successful recovery incidents to improve future responses
    async fn learn_from_incident(&mut self, strategy: &RecoveryStrategy) {
        let strategy_key = format!("{:?}", strategy);
        let current_confidence = self
            .learned_thresholds
            .get(&strategy_key)
            .copied()
            .unwrap_or(0.5);
        let new_confidence = (current_confidence + 0.1).min(1.0);
        self.learned_thresholds.insert(strategy_key, new_confidence);

        debug!(
            "Updated confidence for strategy {:?} to {}",
            strategy, new_confidence
        );
    }

    /// Calculates health status based on metrics
    fn calculate_health_status(&self, metrics: &HealthMetrics) -> HealthStatus {
        let overall_score = (metrics.task_success_rate * 0.4)
            + ((100.0 - metrics.cpu_usage) / 100.0 * 0.2)
            + ((100.0 - metrics.memory_usage) / 100.0 * 0.2)
            + (metrics.energy_level * 0.2);

        if overall_score >= self.config.degraded_threshold {
            HealthStatus::Healthy
        } else if overall_score >= self.config.critical_threshold {
            HealthStatus::Degraded
        } else if overall_score > 0.2 {
            HealthStatus::Critical
        } else {
            HealthStatus::Failed
        }
    }

    /// Checks if agent degradation is worsening over time
    async fn is_degradation_worsening(&self, _agent_id: Uuid) -> bool {
        // In a real implementation, this would analyze historical trends
        // For now, return a simple probability
        rand::random::<f64>() < 0.3
    }

    /// Cleans up completed recovery operations
    fn cleanup_completed_recoveries(&mut self) {
        // Remove recoveries that have been running too long without completion
        let _cutoff_time = Utc::now() - chrono::Duration::minutes(30);
        self.active_recoveries
            .retain(|agent_id, (_strategy, attempts)| {
                if *attempts > self.config.max_recovery_attempts * 2 {
                    warn!(
                        "Abandoning recovery for agent {} after excessive attempts",
                        agent_id
                    );
                    false
                } else {
                    true
                }
            });
    }

    /// Registers a new agent for health monitoring
    pub async fn register_agent(&mut self, agent_id: Uuid) -> HiveResult<()> {
        let initial_metrics = HealthMetrics {
            agent_id,
            status: HealthStatus::Healthy,
            cpu_usage: 20.0,
            memory_usage: 30.0,
            task_success_rate: 0.9,
            response_time: 100.0,
            energy_level: 1.0,
            last_updated: Utc::now(),
            issues: Vec::new(),
        };

        let mut health_metrics = self.health_metrics.write().await;
        health_metrics.insert(agent_id, initial_metrics);

        info!("Registered agent {} for health monitoring", agent_id);
        Ok(())
    }

    /// Unregisters an agent from health monitoring
    pub async fn unregister_agent(&mut self, agent_id: Uuid) -> HiveResult<()> {
        let mut health_metrics = self.health_metrics.write().await;
        health_metrics.remove(&agent_id);

        // Also clean up any active recoveries
        self.active_recoveries.remove(&agent_id);

        info!("Unregistered agent {} from health monitoring", agent_id);
        Ok(())
    }

    /// Gets current health status for all monitored agents
    pub async fn get_swarm_health_summary(&self) -> HashMap<HealthStatus, usize> {
        let health_metrics = self.health_metrics.read().await;
        let mut summary = HashMap::new();

        for metrics in health_metrics.values() {
            *summary.entry(metrics.status).or_insert(0) += 1;
        }

        summary
    }

    /// Gets incident history for analysis
    pub fn get_incident_history(&self) -> &[IncidentRecord] {
        &self.incident_history
    }
}

#[async_trait]
impl AgentBehavior for SelfHealingSwarmAgent {
    async fn execute_task(&mut self, task: Task) -> HiveResult<TaskResult> {
        // Self-healing agents primarily coordinate rather than execute tasks directly
        self.agent
            .execute_with_state_management(AgentState::Working, |agent| {
                // Update timestamp
                agent.last_active = Utc::now();

                Ok(TaskResult {
                    task_id: task.id,
                    agent_id: agent.id,
                    success: true,
                    output: format!(
                        "Self-healing coordination completed for task type: {}",
                        task.task_type
                    ),
                    error_message: None,
                    execution_time: 100,
                    completed_at: Utc::now(),
                    quality_score: Some(0.95),
                    learned_insights: vec!["Swarm health monitoring active".to_string()],
                })
            })
    }

    async fn communicate(
        &mut self,
        envelope: MessageEnvelope,
    ) -> HiveResult<Option<MessageEnvelope>> {
        // Use standardized communication delay
        let complexity = match envelope.priority {
            crate::communication::patterns::MessagePriority::Critical => {
                CommunicationComplexity::Heavy
            }
            crate::communication::patterns::MessagePriority::High => {
                CommunicationComplexity::Complex
            }
            crate::communication::patterns::MessagePriority::Normal => {
                CommunicationComplexity::Standard
            }
            crate::communication::patterns::MessagePriority::Low => CommunicationComplexity::Simple,
        };

        self.agent.communication_delay(complexity).await;

        match envelope.message_type {
            MessageType::HealthCheck => {
                let health_summary = self.get_swarm_health_summary().await;
                let response = MessageEnvelope::new_response(
                    &envelope,
                    self.agent.id,
                    MessagePayload::Json(serde_json::json!({
                        "swarm_health": health_summary,
                        "active_recoveries": self.active_recoveries.len(),
                        "monitoring_status": "active"
                    })),
                );
                Ok(Some(response))
            }

            MessageType::Request => {
                let response_payload = MessagePayload::Text(format!(
                    "Self-healing swarm agent {} ready - monitoring {} agents",
                    self.agent.name,
                    self.health_metrics.read().await.len()
                ));

                let response =
                    MessageEnvelope::new_response(&envelope, self.agent.id, response_payload);
                Ok(Some(response))
            }

            _ => {
                // Delegate to the base agent behavior
                self.agent.communicate(envelope).await
            }
        }
    }

    async fn request_response(
        &mut self,
        request: MessageEnvelope,
        timeout: std::time::Duration,
    ) -> HiveResult<MessageEnvelope> {
        self.agent.request_response(request, timeout).await
    }

    async fn learn(&mut self, nlp_processor: &NLPProcessor) -> HiveResult<()> {
        // Learn from incident patterns
        if self.config.enable_learning && !self.incident_history.is_empty() {
            // Analyze recent incidents for patterns
            let recent_incidents: Vec<_> = self.incident_history.iter().rev().take(10).collect();

            for incident in recent_incidents {
                let pattern_key =
                    format!("{:?}_{}", incident.failure_type, incident.recovery_success);
                let current_confidence = self
                    .learned_thresholds
                    .get(&pattern_key)
                    .copied()
                    .unwrap_or(0.5);
                let adjustment = if incident.recovery_success {
                    0.05
                } else {
                    -0.05
                };
                let new_confidence = (current_confidence + adjustment).clamp(0.0, 1.0);
                self.learned_thresholds.insert(pattern_key, new_confidence);
            }
        }

        // Delegate to base agent learning
        self.agent.learn(nlp_processor).await
    }

    async fn update_position(
        &mut self,
        swarm_center: (f64, f64),
        _neighbors: &[Agent],
    ) -> HiveResult<()> {
        // Self-healing agents maintain central positions for optimal monitoring
        let center_x = swarm_center.0;
        let center_y = swarm_center.1;

        // Move gradually toward center
        let current_x = self.agent.position.0;
        let current_y = self.agent.position.1;

        let new_x = current_x + (center_x - current_x) * 0.1;
        let new_y = current_y + (center_y - current_y) * 0.1;

        self.agent.position = (new_x, new_y);
        Ok(())
    }

    fn get_communication_config(&self) -> CommunicationConfig {
        CommunicationConfig {
            default_timeout: std::time::Duration::from_secs(15),
            max_retries: 5,
            retry_delay: std::time::Duration::from_millis(200),
            max_concurrent_messages: 50,
            buffer_size: 2048,
            enable_compression: true,
            delivery_guarantee: crate::communication::patterns::DeliveryGuarantee::AtLeastOnce,
        }
    }
}
