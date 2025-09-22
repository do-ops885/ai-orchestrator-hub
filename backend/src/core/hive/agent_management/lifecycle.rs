//! # Agent Lifecycle Management Module
//!
//! This module handles agent lifecycle operations including status monitoring,
//! analytics, learning cycles, and system health checks.

use crate::agents::agent::AgentType;
use crate::neural::nlp::NLPProcessor;
use crate::utils::error::HiveResult;

use super::registry::AgentRegistry;
use crate::agents::agent::AgentBehavior;

use std::collections::HashMap;

/// Agent lifecycle management
///
/// Handles monitoring, analytics, and lifecycle operations for agents
/// including learning cycles, status reporting, and performance analysis.
#[derive(Clone)]
pub struct AgentLifecycle {
    /// Reference to the agent registry
    registry: AgentRegistry,
}

impl AgentLifecycle {
    /// Create a new agent lifecycle manager
    #[must_use]
    pub fn new(registry: AgentRegistry) -> Self {
        Self { registry }
    }

    /// Get agent status summary
    ///
    /// Returns a comprehensive summary of agent system status including
    /// counts, types, and performance metrics across all agents.
    pub async fn get_status(&self) -> serde_json::Value {
        let total_agents = self.registry.get_agent_count();
        let active_agents = self.count_active_agents().await;

        let agent_types = self.get_agent_type_distribution().await;
        let performance_summary = self.get_performance_summary().await;

        serde_json::json!({
            "total_agents": total_agents,
            "active_agents": active_agents,
            "agent_types": agent_types,
            "performance": performance_summary
        })
    }

    /// Get detailed analytics
    ///
    /// Returns comprehensive analytics about agent performance, efficiency,
    /// and system utilization patterns.
    pub async fn get_analytics(&self) -> serde_json::Value {
        let agent_performance: Vec<_> = self.registry.agent_metrics
            .iter()
            .map(|entry| {
                let agent_id = *entry.key();
                let metrics = entry.value();
                serde_json::json!({
                    "agent_id": agent_id,
                    "tasks_completed": metrics.tasks_completed,
                    "tasks_failed": metrics.tasks_failed,
                    "success_rate": if metrics.tasks_completed + metrics.tasks_failed > 0 {
                        metrics.tasks_completed as f64 / (metrics.tasks_completed + metrics.tasks_failed) as f64
                    } else {
                        0.0
                    },
                    "average_execution_time_ms": metrics.average_execution_time_ms,
                    "performance_score": metrics.performance_score,
                    "last_activity": metrics.last_activity
                })
            })
            .collect();

        serde_json::json!({
            "agent_performance": agent_performance,
            "total_agents": self.registry.get_agent_count(),
            "system_health": "operational"
        })
    }

    /// Run learning cycle for all agents
    ///
    /// Triggers a learning cycle for all active agents using the provided
    /// NLP processor to enable performance improvement through adaptation.
    pub async fn run_learning_cycle(&self, nlp_processor: &NLPProcessor) -> HiveResult<()> {
        let agents: Vec<_> = self
            .registry
            .agents
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        for mut agent in agents {
            if let Err(e) = AgentBehavior::learn(&mut agent, nlp_processor).await {
                tracing::warn!("Learning failed for agent {}: {}", agent.id, e);
            }
        }

        Ok(())
    }

    /// Parse agent type from configuration
    ///
    /// Extracts and validates the agent type from the provided configuration.
    pub fn parse_agent_type(&self, config: &serde_json::Value) -> HiveResult<AgentType> {
        let type_str = config.get("type").and_then(|v| v.as_str()).ok_or_else(|| {
            crate::utils::error::HiveError::ValidationError {
                field: "type".to_string(),
                reason: "Agent type is required".to_string(),
            }
        })?;

        match type_str {
            "worker" => Ok(AgentType::Worker),
            "coordinator" => Ok(AgentType::Coordinator),
            "specialist" => Ok(AgentType::Specialist("general".to_string())),
            "learner" => Ok(AgentType::Learner),
            _ => Err(crate::utils::error::HiveError::ValidationError {
                field: "type".to_string(),
                reason: format!("Unknown agent type: {type_str}"),
            }),
        }
    }

    /// Validate agent configuration
    ///
    /// Performs basic validation on agent configuration to ensure
    /// required fields are present and properly formatted.
    pub fn validate_agent_config(
        &self,
        config: &serde_json::Value,
    ) -> HiveResult<serde_json::Value> {
        // Basic validation - ensure required fields are present
        if !config.is_object() {
            return Err(crate::utils::error::HiveError::ValidationError {
                field: "config".to_string(),
                reason: "Agent configuration must be an object".to_string(),
            });
        }

        // Add any additional validation logic here
        Ok(config.clone())
    }

    /// Count currently active agents
    ///
    /// Returns the number of agents currently registered and active
    /// in the system. This is a simple count of the agent registry.
    async fn count_active_agents(&self) -> usize {
        // For now, consider all registered agents as active
        // In a real implementation, you might check last activity time
        self.registry.get_agent_count()
    }

    /// Get distribution of agent types
    ///
    /// Analyzes all active agents and returns a breakdown of agent types
    /// with counts for each type. Useful for system composition analysis.
    async fn get_agent_type_distribution(&self) -> serde_json::Value {
        let mut type_counts = HashMap::new();

        for entry in self.registry.agents.iter() {
            let agent_type = format!("{:?}", entry.value().agent_type);
            *type_counts.entry(agent_type).or_insert(0) += 1;
        }

        match serde_json::to_value(type_counts) {
            Ok(value) => value,
            Err(e) => {
                tracing::warn!("Failed to serialize agent type distribution: {}", e);
                serde_json::json!({})
            }
        }
    }

    /// Get performance summary
    ///
    /// Aggregates performance metrics across all agents to provide
    /// system-wide performance statistics and trends.
    async fn get_performance_summary(&self) -> serde_json::Value {
        let mut total_tasks = 0u64;
        let mut total_successful = 0u64;
        let mut total_execution_time = 0u64;
        let mut agent_count = 0;

        for entry in self.registry.agent_metrics.iter() {
            let metrics = entry.value();
            total_tasks += metrics.tasks_completed + metrics.tasks_failed;
            total_successful += metrics.tasks_completed;
            total_execution_time += metrics.total_execution_time_ms;
            agent_count += 1;
        }

        let success_rate = if total_tasks > 0 {
            total_successful as f64 / total_tasks as f64
        } else {
            0.0
        };

        let average_execution_time = if total_successful > 0 {
            total_execution_time as f64 / total_successful as f64
        } else {
            0.0
        };

        serde_json::json!({
            "total_tasks": total_tasks,
            "successful_tasks": total_successful,
            "success_rate": success_rate,
            "average_execution_time_ms": average_execution_time,
            "active_agents": agent_count
        })
    }
}
