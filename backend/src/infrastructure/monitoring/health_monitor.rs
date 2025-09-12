//! Health Monitoring System
//!
//! Monitors the health and status of agents and system components

use super::types::*;
use crate::utils::error::HiveResult;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct HealthMonitor {
    agent_health: Arc<RwLock<HashMap<Uuid, AgentHealth>>>,
    system_health: Arc<RwLock<SystemHealth>>,
    health_history: Arc<RwLock<Vec<HealthSnapshot>>>,
    monitoring_active: Arc<RwLock<bool>>,
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new() -> Self {
        Self {
            agent_health: Arc::new(RwLock::new(HashMap::new())),
            system_health: Arc::new(RwLock::new(SystemHealth {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                disk_usage: 0.0,
                active_connections: 0,
            })),
            health_history: Arc::new(RwLock::new(Vec::new())),
            monitoring_active: Arc::new(RwLock::new(false)),
        }
    }

    /// Start health monitoring
    pub async fn start(&self) -> HiveResult<()> {
        *self.monitoring_active.write().await = true;

        // Start background health check task
        let health_monitor = self.clone();
        tokio::spawn(async move {
            health_monitor.health_check_loop().await;
        });

        tracing::info!("Health monitoring started");
        Ok(())
    }

    /// Stop health monitoring
    pub async fn stop(&self) -> HiveResult<()> {
        *self.monitoring_active.write().await = false;
        tracing::info!("Health monitoring stopped");
        Ok(())
    }

    /// Add an agent to health monitoring
    pub async fn add_agent(&self, agent_id: Uuid) -> HiveResult<()> {
        let agent_health = AgentHealth {
            agent_id,
            status: HealthStatus::Unknown,
            last_heartbeat: Utc::now(),
            response_time: 0.0,
            error_rate: 0.0,
            resource_usage: ResourceHealth {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                disk_usage: 0.0,
                network_usage: 0.0,
            },
        };

        self.agent_health
            .write()
            .await
            .insert(agent_id, agent_health);
        tracing::debug!("Added agent {} to health monitoring", agent_id);
        Ok(())
    }

    /// Remove an agent from health monitoring
    pub async fn remove_agent(&self, agent_id: Uuid) -> HiveResult<()> {
        self.agent_health.write().await.remove(&agent_id);
        tracing::debug!("Removed agent {} from health monitoring", agent_id);
        Ok(())
    }

    /// Update agent health status
    pub async fn update_agent_health(&self, agent_health: AgentHealth) -> HiveResult<()> {
        self.agent_health
            .write()
            .await
            .insert(agent_health.agent_id, agent_health);
        Ok(())
    }

    /// Get health status for a specific agent
    pub async fn get_agent_health(&self, agent_id: Uuid) -> HiveResult<Option<AgentHealth>> {
        Ok(self.agent_health.read().await.get(&agent_id).cloned())
    }

    /// Get health snapshot for all monitored components
    pub async fn get_health_snapshot(&self) -> HiveResult<HealthSnapshot> {
        let agent_health: Vec<AgentHealth> =
            self.agent_health.read().await.values().cloned().collect();
        let system_health = self.system_health.read().await.clone();

        // Calculate overall status
        let overall_status = self
            .calculate_overall_status(&agent_health, &system_health)
            .await;

        let snapshot = HealthSnapshot {
            timestamp: Utc::now(),
            overall_status,
            agent_health,
            system_health,
        };

        // Store in history (keep last 100 snapshots)
        {
            let mut history = self.health_history.write().await;
            history.push(snapshot.clone());
            if history.len() > 100 {
                history.remove(0);
            }
        }

        Ok(snapshot)
    }

    /// Get health history
    pub async fn get_health_history(
        &self,
        limit: Option<usize>,
    ) -> HiveResult<Vec<HealthSnapshot>> {
        let history = self.health_history.read().await;
        let limit = limit.unwrap_or(history.len());
        Ok(history.iter().rev().take(limit).cloned().collect())
    }

    /// Check if an agent is healthy
    pub async fn is_agent_healthy(&self, agent_id: Uuid) -> HiveResult<bool> {
        if let Some(health) = self.get_agent_health(agent_id).await? {
            Ok(matches!(health.status, HealthStatus::Healthy))
        } else {
            Ok(false)
        }
    }

    /// Get unhealthy agents
    pub async fn get_unhealthy_agents(&self) -> HiveResult<Vec<Uuid>> {
        let agent_health = self.agent_health.read().await;
        let unhealthy: Vec<Uuid> = agent_health
            .iter()
            .filter(|(_, health)| !matches!(health.status, HealthStatus::Healthy))
            .map(|(id, _)| *id)
            .collect();
        Ok(unhealthy)
    }

    /// Update system health metrics
    pub async fn update_system_health(&self, system_health: SystemHealth) -> HiveResult<()> {
        *self.system_health.write().await = system_health;
        Ok(())
    }

    /// Get current system health
    pub async fn get_system_health(&self) -> HiveResult<SystemHealth> {
        Ok(self.system_health.read().await.clone())
    }

    /// Perform health check for a specific agent
    pub async fn check_agent_health(&self, agent_id: Uuid) -> HiveResult<HealthStatus> {
        // This would implement actual health checking logic
        // For now, return a placeholder implementation

        if let Some(health) = self.get_agent_health(agent_id).await? {
            let now = Utc::now();
            let time_since_heartbeat = now.signed_duration_since(health.last_heartbeat);

            // Consider agent unhealthy if no heartbeat for more than 5 minutes
            if time_since_heartbeat.num_minutes() > 5 {
                return Ok(HealthStatus::Critical);
            }

            // Check resource usage thresholds
            if health.resource_usage.cpu_usage > 90.0 || health.resource_usage.memory_usage > 90.0 {
                return Ok(HealthStatus::Warning);
            }

            // Check error rate
            if health.error_rate > 0.1 {
                return Ok(HealthStatus::Warning);
            }

            Ok(HealthStatus::Healthy)
        } else {
            Ok(HealthStatus::Unknown)
        }
    }

    /// Background health check loop
    async fn health_check_loop(&self) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

        while *self.monitoring_active.read().await {
            interval.tick().await;

            if let Err(e) = self.perform_health_checks().await {
                tracing::error!("Health check failed: {}", e);
            }
        }
    }

    /// Perform health checks for all monitored agents
    async fn perform_health_checks(&self) -> HiveResult<()> {
        let agent_ids: Vec<Uuid> = self.agent_health.read().await.keys().cloned().collect();

        for agent_id in agent_ids {
            let health_status = self.check_agent_health(agent_id).await?;

            // Update agent health status
            if let Some(mut agent_health) = self.get_agent_health(agent_id).await? {
                agent_health.status = health_status;
                self.update_agent_health(agent_health).await?;
            }
        }

        // Update system health
        self.update_system_metrics().await?;

        Ok(())
    }

    /// Update system-level health metrics
    async fn update_system_metrics(&self) -> HiveResult<()> {
        // This would implement actual system metrics collection
        // For now, use placeholder values

        let system_health = SystemHealth {
            cpu_usage: self.get_system_cpu_usage().await?,
            memory_usage: self.get_system_memory_usage().await?,
            disk_usage: self.get_system_disk_usage().await?,
            active_connections: self.get_active_connections().await?,
        };

        self.update_system_health(system_health).await?;
        Ok(())
    }

    /// Calculate overall system health status
    async fn calculate_overall_status(
        &self,
        agent_health: &[AgentHealth],
        system_health: &SystemHealth,
    ) -> HealthStatus {
        // Check if any agents are in critical state
        let critical_agents = agent_health
            .iter()
            .filter(|h| matches!(h.status, HealthStatus::Critical))
            .count();

        if critical_agents > 0 {
            return HealthStatus::Critical;
        }

        // Check system resource usage
        if system_health.cpu_usage > 90.0 || system_health.memory_usage > 90.0 {
            return HealthStatus::Critical;
        }

        // Check for warning conditions
        let warning_agents = agent_health
            .iter()
            .filter(|h| matches!(h.status, HealthStatus::Warning))
            .count();

        if warning_agents > 0 || system_health.cpu_usage > 70.0 || system_health.memory_usage > 70.0
        {
            return HealthStatus::Warning;
        }

        HealthStatus::Healthy
    }

    // Placeholder methods for system metrics collection
    async fn get_system_cpu_usage(&self) -> HiveResult<f64> {
        // Would implement actual CPU usage collection
        Ok(25.0) // Placeholder
    }

    async fn get_system_memory_usage(&self) -> HiveResult<f64> {
        // Would implement actual memory usage collection
        Ok(45.0) // Placeholder
    }

    async fn get_system_disk_usage(&self) -> HiveResult<f64> {
        // Would implement actual disk usage collection
        Ok(60.0) // Placeholder
    }

    async fn get_active_connections(&self) -> HiveResult<u32> {
        // Would implement actual connection counting
        Ok(42) // Placeholder
    }
}
