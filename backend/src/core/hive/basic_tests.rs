//! Basic module tests for the hive system
//!
//! This module contains basic tests for module exports and simple integration.

#[cfg(test)]
mod tests {
    use super::super::*;
    use serde_json;
    use std::sync::Arc;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_module_exports() {
        // Test that all expected types are exported and accessible
        let _coordinator: HiveCoordinator;
        let _agent_manager: AgentManager;
        let _task_distributor: TaskDistributor;
        let _process_manager: ProcessManager;
        let _metrics_collector: MetricsCollector;

        // Test that types can be used
        let _agent_result: AgentRegistrationResult;
        let _task_result: TaskExecutionResult;
        let _process_config: ProcessConfig;
        let _hive_metrics: HiveMetrics;
        let _swarm_metrics: SwarmMetrics;
    }

    #[tokio::test]
    async fn test_module_integration_basic() -> Result<(), Box<dyn std::error::Error>> {
        // Test basic integration between modules
        let resource_manager =
            Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);
        let (tx, _rx) = mpsc::unbounded_channel();

        // Create individual modules
        let agent_manager = AgentManager::new(Arc::clone(&resource_manager), tx.clone()).await?;
        let task_distributor =
            TaskDistributor::new(Arc::clone(&resource_manager), tx.clone()).await?;
        let metrics_collector = MetricsCollector::new(tx.clone()).await?;
        let process_manager = ProcessManager::new(tx).await?;

        // Verify they can be created and used together
        let agent_status = agent_manager.get_status().await;
        assert_eq!(agent_status["total_agents"].as_u64().unwrap_or_default(), 0);

        let task_status = task_distributor.get_status().await;
        assert_eq!(
            task_status["queue"]["legacy_queue_size"]
                .as_u64()
                .unwrap_or_default(),
            0
        );

        let status = metrics_collector.get_current_metrics().await;
        assert_eq!(status.agent_metrics.total_agents, 0);

        let process_status = process_manager.get_process_status().await;
        assert_eq!(process_status["total_processes"], 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_agent_task_integration() -> Result<(), Box<dyn std::error::Error>> {
        // Test integration between agent management and task distribution
        let resource_manager =
            Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);
        let (tx, _rx) = mpsc::unbounded_channel();

        let agent_manager = AgentManager::new(Arc::clone(&resource_manager), tx.clone()).await?;
        let task_distributor =
            TaskDistributor::new(Arc::clone(&resource_manager), tx.clone()).await?;
        let metrics_collector = MetricsCollector::new(tx).await?;

        // Create an agent
        let agent_config = serde_json::json!({
            "type": "worker",
            "name": "integration_test_agent"
        });
        let agent_id = agent_manager.create_agent(agent_config).await?;
        assert_eq!(agent_manager.get_agent_count(), 1);

        // Create a task
        let task_config = serde_json::json!({
            "type": "computation",
            "title": "Integration Test Task",
            "description": "Testing module integration"
        });
        let task_id = task_distributor.create_task(task_config).await?;

        // Execute the task
        let result = task_distributor
            .execute_task_with_verification(task_id, agent_id)
            .await?;
        assert!(result.is_object());

        // Check that metrics were recorded
        let metrics = metrics_collector.get_current_metrics().await;
        assert_eq!(metrics.agent_metrics.total_agents, 1);
        assert_eq!(metrics.task_metrics.total_tasks, 1);

        Ok(())
    }
}