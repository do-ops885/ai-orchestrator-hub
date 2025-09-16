//! Integration tests for the hive system
//!
//! This module contains comprehensive integration tests that verify
//! the interaction between multiple modules.

#[cfg(test)]
mod tests {
    use super::super::*;
    use serde_json;
    use std::sync::Arc;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_full_module_integration() -> Result<(), Box<dyn std::error::Error>> {
        // Test full integration using the main HiveCoordinator
        let coordinator = HiveCoordinator::new().await?;

        // Test that all modules work together through the coordinator
        let status = coordinator.get_status().await;
        assert!(status.is_object());
        assert!(status.get("agents").is_some());
        assert!(status.get("tasks").is_some());
        assert!(status.get("metrics").is_some());

        // Create an agent through the coordinator
        let agent_config = serde_json::json!({
            "type": "worker",
            "name": "full_integration_agent"
        });
        let agent_id = coordinator.create_agent(agent_config).await?;

        // Create a task through the coordinator
        let task_config = serde_json::json!({
            "type": "computation",
            "title": "Full Integration Task"
        });
        let task_id = coordinator.create_task(task_config).await?;

        // Execute task through the coordinator
        let result = coordinator
            .execute_task_with_verification(task_id, agent_id)
            .await?;
        assert!(result.is_object());

        // Verify through coordinator methods
        let agents = coordinator.get_all_agents().await;
        assert_eq!(agents.len(), 1);

        let agent = coordinator.get_agent(agent_id).await;
        assert!(agent.is_some());

        // Check enhanced analytics
        let analytics = coordinator.get_enhanced_analytics().await;
        assert!(analytics.is_object());

        Ok(())
    }

    #[tokio::test]
    async fn test_module_error_handling_integration() -> Result<(), Box<dyn std::error::Error>> {
        // Test that errors are properly handled across modules
        let coordinator = HiveCoordinator::new().await?;

        // Test invalid agent creation
        let invalid_agent_config = serde_json::json!({
            "name": "invalid_agent"
            // Missing "type" field
        });
        let agent_result = coordinator.create_agent(invalid_agent_config).await;
        assert!(agent_result.is_err());

        // Test invalid task creation
        let invalid_task_config = serde_json::json!("invalid_task");
        let task_result = coordinator.create_task(invalid_task_config).await;
        assert!(task_result.is_err());

        // Test removing non-existent agent
        let fake_agent_id = uuid::Uuid::new_v4();
        let remove_result = coordinator.remove_agent(fake_agent_id).await;
        assert!(remove_result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_module_status_integration() -> Result<(), Box<dyn std::error::Error>> {
        // Test that status information is consistent across modules
        let coordinator = HiveCoordinator::new().await?;

        // Get initial status
        let initial_status = coordinator.get_status().await;
        let empty_map = serde_json::Map::new();
        let initial_agents = initial_status
            .get("agents")
            .and_then(|v| v.as_object())
            .unwrap_or(&empty_map);
        let initial_tasks = initial_status
            .get("tasks")
            .and_then(|v| v.as_object())
            .unwrap_or(&empty_map);

        // Create agent and task
        let agent_config = serde_json::json!({
            "type": "worker",
            "name": "status_test_agent"
        });
        let agent_id = coordinator.create_agent(agent_config).await?;

        let task_config = serde_json::json!({
            "type": "computation",
            "title": "Status Test Task"
        });
        let task_id = coordinator.create_task(task_config).await?;

        // Execute task
        coordinator
            .execute_task_with_verification(task_id, agent_id)
            .await?;

        // Get updated status
        let updated_status = coordinator.get_status().await;
        let updated_agents = updated_status
            .get("agents")
            .and_then(|v| v.as_object())
            .unwrap_or(&empty_map);
        let updated_tasks = updated_status
            .get("tasks")
            .and_then(|v| v.as_object())
            .unwrap_or(&empty_map);

        // Verify status is updated
        let updated_total = updated_agents
            .get("total_agents")
            .and_then(|v| v.as_u64())
            .unwrap_or_default();
        let initial_total = initial_agents
            .get("total_agents")
            .and_then(|v| v.as_u64())
            .unwrap_or_default();
        assert!(updated_total > initial_total);

        // Tasks status should be updated (though exact format may vary)
        assert!(updated_status.get("tasks").is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_module_coordination_message_flow() -> Result<(), Box<dyn std::error::Error>> {
        // Test that coordination messages flow properly between modules
        let coordinator = HiveCoordinator::new().await?;

        // Create agent (should send AgentRegistered message)
        let agent_config = serde_json::json!({
            "type": "worker",
            "name": "coordination_test_agent"
        });
        let agent_id = coordinator.create_agent(agent_config).await?;

        // Create and execute task (should send TaskCompleted message)
        let task_config = serde_json::json!({
            "type": "computation",
            "title": "Coordination Test Task"
        });
        let task_id = coordinator.create_task(task_config).await?;
        coordinator
            .execute_task_with_verification(task_id, agent_id)
            .await?;

        // Give time for message processing
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Check that metrics were updated (indicating messages were processed)
        let status = coordinator.get_status().await;
        let empty_map = serde_json::Map::new();
        let metrics = status
            .get("metrics")
            .and_then(|v| v.as_object())
            .unwrap_or(&empty_map);
        assert!(metrics.get("agent_metrics").is_some());
        assert!(metrics.get("task_metrics").is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_module_shutdown_integration() -> Result<(), Box<dyn std::error::Error>> {
        // Test that shutdown works across all modules
        let coordinator = HiveCoordinator::new().await?;

        // Create some resources
        let agent_config = serde_json::json!({
            "type": "worker",
            "name": "shutdown_test_agent"
        });
        coordinator.create_agent(agent_config).await?;

        // Shutdown should complete without error
        coordinator.shutdown().await?;

        // After shutdown, operations should still work (graceful shutdown)
        let status = coordinator.get_status().await;
        assert!(status.is_object());

        Ok(())
    }
}