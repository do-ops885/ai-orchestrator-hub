//! # Agent Registry Module
//!
//! This module handles the core agent storage and retrieval operations
//! including registration, lookup, and basic lifecycle management.

use crate::agents::agent::Agent;
use crate::infrastructure::cache_invalidation::AgentCacheInvalidationManager;
use crate::infrastructure::cached_query::{CacheKey, CachedQueryManager};
use crate::infrastructure::resource_manager::ResourceManager;
use crate::utils::error::{HiveError, HiveResult};

use super::types::AgentMetrics;
use crate::core::hive::coordinator::CoordinationMessage;

use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Core agent registry functionality
///
/// Handles the fundamental operations for agent storage and retrieval
/// with intelligent caching and coordination messaging.
#[derive(Clone)]
pub struct AgentRegistry {
    /// Active agents in the system
    pub agents: Arc<DashMap<Uuid, Agent>>,

    /// Agent performance metrics
    pub agent_metrics: Arc<DashMap<Uuid, AgentMetrics>>,

    /// Resource manager for capacity planning
    resource_manager: Arc<ResourceManager>,

    /// Communication channel for coordination
    coordination_tx: mpsc::UnboundedSender<CoordinationMessage>,

    /// Intelligent cache manager for agent data
    cache_manager: Arc<CachedQueryManager>,

    /// Cache invalidation manager for agent data
    invalidation_manager: Arc<AgentCacheInvalidationManager>,
}

impl AgentRegistry {
    /// Create a new agent registry
    #[must_use] 
    pub fn new(
        resource_manager: Arc<ResourceManager>,
        coordination_tx: mpsc::UnboundedSender<CoordinationMessage>,
        cache_manager: Arc<CachedQueryManager>,
        invalidation_manager: Arc<AgentCacheInvalidationManager>,
    ) -> Self {
        Self {
            agents: Arc::new(DashMap::new()),
            agent_metrics: Arc::new(DashMap::new()),
            resource_manager,
            coordination_tx,
            cache_manager,
            invalidation_manager,
        }
    }

    /// Register an agent in the system
    ///
    /// Stores the agent in the registry and initializes its metrics.
    /// Sends coordination notification for system-wide updates.
    pub async fn register_agent(&self, agent: Agent) -> HiveResult<Uuid> {
        let agent_id = agent.id;

        // Register the agent
        self.agents.insert(agent_id, agent.clone());
        self.agent_metrics.insert(agent_id, AgentMetrics::default());

        // Cache the new agent data
        let cache_key = CacheKey::Agent(agent_id);
        let cache_entry = crate::infrastructure::cached_query::CacheEntry::new(
            agent,
            vec![], // New agents don't have dependencies initially
        );

        if let Err(e) = self.cache_manager.set_cached(cache_key, cache_entry).await {
            tracing::warn!("Failed to cache new agent data: {}", e);
        }

        // Notify coordination system
        if let Err(e) = self
            .coordination_tx
            .send(CoordinationMessage::AgentRegistered { agent_id })
        {
            tracing::warn!("Failed to send agent registration notification: {}", e);
        }

        tracing::info!("Agent {} registered and cached successfully", agent_id);
        Ok(agent_id)
    }

    /// Remove an agent from the registry
    ///
    /// Removes the agent and cleans up associated resources and cache entries.
    pub async fn unregister_agent(&self, agent_id: Uuid) -> HiveResult<()> {
        if let Some((_, agent)) = self.agents.remove(&agent_id) {
            // Cleanup agent resources
            drop(agent);
            self.agent_metrics.remove(&agent_id);

            // Invalidate cache entries for this agent
            if let Err(e) = self.invalidation_manager.invalidate_agent(agent_id).await {
                tracing::warn!("Failed to invalidate agent cache: {}", e);
            }

            // Notify coordination system
            if let Err(e) = self
                .coordination_tx
                .send(CoordinationMessage::AgentRemoved { agent_id })
            {
                tracing::warn!("Failed to send agent removal notification: {}", e);
            }

            tracing::info!(
                "Agent {} removed and cache invalidated successfully",
                agent_id
            );
            Ok(())
        } else {
            Err(HiveError::AgentNotFound {
                id: agent_id.to_string(),
            })
        }
    }

    /// Get an agent by ID with intelligent caching
    ///
    /// Retrieves an agent from cache first, falling back to direct lookup if not cached.
    /// Automatically caches the result for future requests.
    pub async fn get_agent(&self, agent_id: Uuid) -> Option<Agent> {
        let cache_key = CacheKey::Agent(agent_id);
        let dependencies = vec![]; // Agent data doesn't depend on other cache entries

        // Try to get from cache first
        if let Some(agent) = self.cache_manager.get_cached(&cache_key).await {
            return Some(agent);
        }

        // Cache miss - get from direct storage and cache the result
        if let Some(agent) = self.agents.get(&agent_id).map(|entry| entry.clone()) {
            // Cache the result for future requests
            let cache_entry =
                crate::infrastructure::cached_query::CacheEntry::new(agent.clone(), dependencies);

            if let Err(e) = self.cache_manager.set_cached(cache_key, cache_entry).await {
                tracing::warn!("Failed to cache agent data: {}", e);
            }

            Some(agent)
        } else {
            None
        }
    }

    /// Get all active agents
    ///
    /// Returns a complete list of all currently active agents in the system.
    #[must_use] 
    pub fn get_all_agents(&self) -> Vec<(Uuid, Agent)> {
        self.agents
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect()
    }

    /// Update an agent in the registry
    ///
    /// Updates an existing agent with new data.
    pub fn update_agent(&self, agent_id: Uuid, agent: Agent) {
        self.agents.insert(agent_id, agent);
    }

    /// Get agent count
    ///
    /// Returns the total number of active agents in the system.
    #[must_use] 
    pub fn get_agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Check if agent exists
    ///
    /// Returns true if the agent with the given ID exists in the registry.
    #[must_use] 
    pub fn agent_exists(&self, agent_id: Uuid) -> bool {
        self.agents.contains_key(&agent_id)
    }

    /// Get agent metrics
    ///
    /// Retrieves the performance metrics for a specific agent.
    #[must_use] 
    pub fn get_agent_metrics(&self, agent_id: Uuid) -> Option<AgentMetrics> {
        self.agent_metrics.get(&agent_id).map(|entry| entry.clone())
    }

    /// Update agent metrics
    ///
    /// Updates the performance metrics for a specific agent.
    pub fn update_agent_metrics(&self, agent_id: Uuid, metrics: AgentMetrics) {
        self.agent_metrics.insert(agent_id, metrics);
    }
}
