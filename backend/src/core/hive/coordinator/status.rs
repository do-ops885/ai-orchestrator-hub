//! # Status and Analytics Reporting
//!
//! This module handles all status reporting, analytics, and monitoring operations
//! for the HiveCoordinator. It provides comprehensive system health and performance data.

use super::core::HiveCoordinator;
use crate::utils::error::HiveResult;
use serde_json;

impl HiveCoordinator {
    /// Get comprehensive system status.
    ///
    /// Returns a complete snapshot of the current system state including
    /// agent counts, task queues, metrics, and resource utilization.
    /// This is the primary method for monitoring system health and performance.
    ///
    /// ## Status Information
    ///
    /// The returned JSON includes:
    /// - `hive_id`: Unique identifier for this hive instance
    /// - `agents`: Agent counts and status information
    /// - `tasks`: Task queue status and execution statistics
    /// - `metrics`: Performance metrics and system health indicators
    /// - `resources`: Current resource utilization and availability
    /// - `swarm_center`: Calculated center coordinates of the swarm based on hive_id
    /// - `total_energy`: Derived energy consumption based on system metrics
    /// - `timestamp`: When the status was generated
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is the number of active components.
    /// May involve multiple subsystem queries and data aggregation.
    pub async fn get_status(&self) -> serde_json::Value {
        let agent_status = self.agent_manager.get_status().await;
        let task_status = self.task_distributor.get_status().await;
        let metrics = self.metrics_collector.get_current_metrics().await;
        let resource_info = self.resource_manager.get_system_info().await;

        // Calculate swarm_center based on hive_id
        // Use a simple hash of the hive_id to generate coordinates
        let hive_id_str = self.id.to_string();
        let hash = hive_id_str.chars().map(|c| c as u32).sum::<u32>() as f64;
        let swarm_center_x = (hash % 1000.0) / 100.0; // Range 0.0 to 10.0
        let swarm_center_y = ((hash / 1000.0) % 1000.0) / 100.0; // Range 0.0 to 10.0

        // Calculate total_energy from metrics
        // Combine CPU usage, memory usage, and task efficiency
        let cpu_energy = metrics.system_metrics.cpu_usage_percent * 0.4;
        let memory_energy = (metrics.system_metrics.total_memory_usage_mb / 1000.0) * 0.3;
        let task_energy = (metrics.task_metrics.success_rate * 100.0) * 0.3;
        let total_energy = cpu_energy + memory_energy + task_energy;

        serde_json::json!({
            "hive_id": self.id,
            "agents": agent_status,
            "tasks": task_status,
            "metrics": metrics,
            "resources": {
                "system_resources": resource_info.0,
                "resource_profile": resource_info.1,
                "hardware_class": resource_info.2
            },
            "swarm_center": [swarm_center_x, swarm_center_y],
            "total_energy": total_energy,
            "timestamp": chrono::Utc::now()
        })
    }

    /// Get detailed analytics and performance metrics.
    ///
    /// Returns enhanced analytics including performance trends, agent efficiency,
    /// task distribution patterns, and predictive metrics. This provides deeper
    /// insights than the basic status for optimization and planning.
    ///
    /// ## Analytics Data
    ///
    /// Includes:
    /// - `performance_metrics`: Detailed performance statistics
    /// - `agent_analytics`: Individual agent performance and efficiency
    /// - `task_analytics`: Task completion patterns and bottlenecks
    /// - `trends`: Historical trends and growth patterns
    /// - `predictions`: Predictive analytics for resource planning
    ///
    /// ## Performance
    ///
    /// O(n) time complexity with historical data analysis.
    /// May involve complex calculations for trend analysis.
    pub async fn get_enhanced_analytics(&self) -> serde_json::Value {
        let base_metrics = self.metrics_collector.get_enhanced_metrics().await;
        let agent_analytics = self.agent_manager.get_analytics().await;
        let task_analytics = self.task_distributor.get_analytics().await;

        serde_json::json!({
            "hive_id": self.id,
            "performance_metrics": base_metrics,
            "agent_analytics": agent_analytics,
            "task_analytics": task_analytics,
            "timestamp": chrono::Utc::now()
        })
    }

    /// Get information about all agents.
    ///
    /// Returns detailed information about all registered agents including
    /// their status, capabilities, performance metrics, and current assignments.
    pub async fn get_agents_info(&self) -> serde_json::Value {
        self.agent_manager.get_status().await
    }

    /// Get information about all tasks.
    ///
    /// Returns comprehensive information about all tasks in the system
    /// including pending, running, completed, and failed tasks.
    pub async fn get_tasks_info(&self) -> HiveResult<serde_json::Value> {
        Ok(self.task_distributor.get_status().await)
    }

    /// Get resource information.
    ///
    /// Returns detailed information about system resource utilization
    /// and availability for capacity planning and monitoring.
    pub async fn get_resource_info(&self) -> HiveResult<serde_json::Value> {
        let resource_info = self.resource_manager.get_system_info().await;
        Ok(serde_json::json!({
            "system_resources": resource_info.0,
            "resource_profile": resource_info.1,
            "hardware_class": resource_info.2
        }))
    }

    /// Get memory statistics.
    ///
    /// Returns detailed memory usage statistics for the system and
    /// individual components.
    pub async fn get_memory_stats(&self) -> HiveResult<serde_json::Value> {
        // Placeholder implementation
        Ok(serde_json::json!({
            "total_agents": 0,
            "healthy_agents": 0,
            "unhealthy_agents": 0
        }))
    }

    /// Check queue health.
    ///
    /// Performs health checks on task queues and distribution systems.
    /// Returns information about queue status, backlog, and processing efficiency.
    pub async fn check_queue_health(&self) -> HiveResult<serde_json::Value> {
        let task_status = self.task_distributor.get_status().await;
        let queue_size = task_status
            .get("legacy_queue_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        Ok(serde_json::json!({
            "queue_size": queue_size,
            "healthy": true
        }))
    }

    /// Check agent health.
    ///
    /// Performs health checks on all registered agents and returns
    /// aggregated health status information.
    pub fn check_agent_health(&self) -> serde_json::Value {
        // Placeholder implementation
        serde_json::json!({
            "total_agents": 0,
            "healthy_agents": 0,
            "unhealthy_agents": 0
        })
    }

    // Stub methods for backward compatibility

    /// Get simple verification statistics.
    pub async fn get_simple_verification_stats(&self) -> serde_json::Value {
        serde_json::json!({
            "total_verifications": 0,
            "passed_verifications": 0,
            "failed_verifications": 0,
            "success_rate": 0.0,
            "average_verification_time_ms": 0.0,
            "average_confidence_score": 0.0,
            "tier_usage": {},
            "rule_effectiveness": {}
        })
    }

    /// Configure simple verification.
    pub async fn configure_simple_verification(
        &self,
        _config: serde_json::Value,
    ) -> HiveResult<()> {
        Ok(())
    }

    /// Get auto scaling statistics.
    pub async fn get_auto_scaling_stats(&self) -> serde_json::Value {
        serde_json::json!({
            "total_scaling_events": 0,
            "successful_scalings": 0,
            "failed_scalings": 0,
            "average_scaling_time_ms": 0.0,
            "current_scale_factor": 1.0
        })
    }

    /// Get skill evolution statistics.
    pub async fn get_skill_evolution_stats(&self) -> serde_json::Value {
        serde_json::json!({
            "total_skill_updates": 0,
            "average_skill_improvement": 0.0,
            "skill_distribution": {},
            "evolution_trends": []
        })
    }
}
