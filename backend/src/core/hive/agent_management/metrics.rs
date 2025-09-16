//! # Agent Metrics Module
//!
//! This module handles agent performance metrics tracking, updates,
//! and analytics calculations.

use crate::infrastructure::cached_query::{CacheKey, CachedQueryManager};
use crate::utils::error::HiveResult;

use super::registry::AgentRegistry;
use super::types::AgentMetrics;
use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

/// Agent metrics management
///
/// Handles tracking and updating of agent performance metrics,
/// including task completion statistics and performance scoring.
#[derive(Clone)]
pub struct AgentMetricsManager {
    /// Reference to the agent registry
    registry: AgentRegistry,

    /// Cache manager for metrics data
    cache_manager: Arc<CachedQueryManager>,
}

impl AgentMetricsManager {
    /// Create a new agent metrics manager
    pub fn new(registry: AgentRegistry, cache_manager: Arc<CachedQueryManager>) -> Self {
        Self {
            registry,
            cache_manager,
        }
    }

    /// Update agent metrics after task execution
    ///
    /// Updates performance metrics for an agent after task completion.
    /// Calculates success rates, execution times, and performance scores.
    /// Automatically invalidates related cache entries to maintain consistency.
    pub async fn update_agent_metrics(
        &self,
        agent_id: Uuid,
        execution_time_ms: u64,
        success: bool,
    ) -> HiveResult<()> {
        // Get current metrics or create new ones
        let mut metrics = self
            .registry
            .get_agent_metrics(agent_id)
            .unwrap_or_else(|| AgentMetrics::default());

        // Update metrics based on task result
        if success {
            metrics.tasks_completed += 1;
        } else {
            metrics.tasks_failed += 1;
        }

        // Update execution time statistics
        metrics.total_execution_time_ms += execution_time_ms;
        let total_tasks = metrics.tasks_completed + metrics.tasks_failed;
        if total_tasks > 0 {
            metrics.average_execution_time_ms =
                metrics.total_execution_time_ms as f64 / total_tasks as f64;
        }

        // Update last activity timestamp
        metrics.last_activity = Some(Utc::now());

        // Calculate performance score
        metrics.performance_score = self.calculate_performance_score(&metrics);

        // Update metrics in registry
        self.registry.update_agent_metrics(agent_id, metrics);

        // Invalidate metrics cache to ensure fresh data
        let metrics_key = CacheKey::AgentMetrics(agent_id);
        if let Err(e) = self.cache_manager.invalidate_key(&metrics_key).await {
            tracing::warn!("Failed to invalidate agent metrics cache: {}", e);
        }

        Ok(())
    }

    /// Calculate performance score for an agent
    ///
    /// Computes a performance score based on success rate and execution speed.
    /// Higher scores indicate better performance with a maximum speed bonus cap.
    fn calculate_performance_score(&self, metrics: &AgentMetrics) -> f64 {
        let total_tasks = metrics.tasks_completed + metrics.tasks_failed;

        if total_tasks == 0 {
            return 0.0;
        }

        let success_rate = metrics.tasks_completed as f64 / total_tasks as f64;

        // Base score from success rate
        let base_score = success_rate;

        // Speed bonus (capped at 2x for very fast execution)
        let speed_bonus = if metrics.average_execution_time_ms > 0.0 {
            (1000.0 / metrics.average_execution_time_ms).min(1.0)
        } else {
            1.0 // Perfect speed bonus for instant execution
        };

        base_score * speed_bonus
    }

    /// Get agent performance statistics
    ///
    /// Returns detailed performance statistics for a specific agent.
    pub fn get_agent_performance(&self, agent_id: Uuid) -> Option<serde_json::Value> {
        self.registry.get_agent_metrics(agent_id).map(|metrics| {
            let total_tasks = metrics.tasks_completed + metrics.tasks_failed;
            let success_rate = if total_tasks > 0 {
                metrics.tasks_completed as f64 / total_tasks as f64
            } else {
                0.0
            };

            serde_json::json!({
                "agent_id": agent_id,
                "tasks_completed": metrics.tasks_completed,
                "tasks_failed": metrics.tasks_failed,
                "total_tasks": total_tasks,
                "success_rate": success_rate,
                "average_execution_time_ms": metrics.average_execution_time_ms,
                "performance_score": metrics.performance_score,
                "last_activity": metrics.last_activity
            })
        })
    }

    /// Get system-wide performance summary
    ///
    /// Aggregates performance metrics across all agents.
    pub fn get_system_performance_summary(&self) -> serde_json::Value {
        let mut total_tasks;
        let mut total_completed = 0u64;
        let mut total_failed = 0u64;
        let mut total_execution_time = 0u64;
        let mut agent_count = 0;
        let mut active_agents = 0;

        for entry in self.registry.agent_metrics.iter() {
            let metrics = entry.value();
            total_completed += metrics.tasks_completed;
            total_failed += metrics.tasks_failed;
            total_execution_time += metrics.total_execution_time_ms;
            agent_count += 1;

            if metrics.last_activity.is_some() {
                active_agents += 1;
            }
        }

        total_tasks = total_completed + total_failed;
        let overall_success_rate = if total_tasks > 0 {
            total_completed as f64 / total_tasks as f64
        } else {
            0.0
        };

        let average_execution_time = if total_completed > 0 {
            total_execution_time as f64 / total_completed as f64
        } else {
            0.0
        };

        serde_json::json!({
            "total_agents": agent_count,
            "active_agents": active_agents,
            "total_tasks": total_tasks,
            "completed_tasks": total_completed,
            "failed_tasks": total_failed,
            "overall_success_rate": overall_success_rate,
            "average_execution_time_ms": average_execution_time
        })
    }

    /// Reset agent metrics
    ///
    /// Resets all metrics for a specific agent to zero.
    /// Useful for testing or when an agent needs a fresh start.
    pub fn reset_agent_metrics(&self, agent_id: Uuid) {
        let empty_metrics = AgentMetrics::default();
        self.registry.update_agent_metrics(agent_id, empty_metrics);
    }

    /// Get top performing agents
    ///
    /// Returns a list of the top performing agents based on performance score.
    pub fn get_top_performers(&self, limit: usize) -> Vec<(Uuid, f64)> {
        let mut performers: Vec<(Uuid, f64)> = self
            .registry
            .agent_metrics
            .iter()
            .map(|entry| (*entry.key(), entry.value().performance_score))
            .collect();

        // Sort by performance score (descending)
        performers.sort_by(|a, b| match b.1.partial_cmp(&a.1) {
            Some(ordering) => ordering,
            None => std::cmp::Ordering::Equal,
        });

        // Take the top performers
        performers.into_iter().take(limit).collect()
    }
}
