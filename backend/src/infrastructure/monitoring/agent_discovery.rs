//! Agent Discovery System
//!
//! Discovers and tracks agents in the system

use super::types::*;
use crate::utils::error::HiveResult;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct AgentDiscovery {
    agents: Arc<RwLock<HashMap<Uuid, AgentInfo>>>,
    relationships: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
    last_discovery: Arc<RwLock<DateTime<Utc>>>,
}

impl Default for AgentDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentDiscovery {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            relationships: Arc::new(RwLock::new(HashMap::new())),
            last_discovery: Arc::new(RwLock::new(Utc::now())),
        }
    }

    pub async fn register_agent(&self, agent_info: AgentInfo) -> HiveResult<()> {
        self.agents.write().await.insert(agent_info.id, agent_info);
        *self.last_discovery.write().await = Utc::now();
        Ok(())
    }

    pub async fn unregister_agent(&self, agent_id: Uuid) -> HiveResult<()> {
        self.agents.write().await.remove(&agent_id);
        self.relationships.write().await.remove(&agent_id);
        *self.last_discovery.write().await = Utc::now();
        Ok(())
    }

    pub async fn get_agent(&self, agent_id: Uuid) -> HiveResult<Option<AgentInfo>> {
        Ok(self.agents.read().await.get(&agent_id).cloned())
    }

    pub async fn get_all_agents(&self) -> HiveResult<Vec<AgentInfo>> {
        Ok(self.agents.read().await.values().cloned().collect())
    }

    pub async fn discover_agents(&self) -> HiveResult<Vec<AgentInfo>> {
        // Placeholder for agent discovery logic
        self.get_all_agents().await
    }
}
