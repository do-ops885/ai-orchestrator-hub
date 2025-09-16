//! # Test Utilities and Helpers
//!
//! This module contains test utilities and helper functions for the HiveCoordinator.
//! It includes mock implementations and test-specific functionality.

use super::core::HiveCoordinator;
use super::messages::CoordinationMessage;
use crate::utils::error::HiveResult;
use uuid::Uuid;

/// Helper function to create a test coordinator
pub async fn create_test_coordinator() -> HiveResult<HiveCoordinator> {
    HiveCoordinator::new().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::agent::Agent;
    use serde_json;

    #[tokio::test]
    async fn test_hive_coordinator_creation() -> Result<(), Box<dyn std::error::Error>> {
        let coordinator = create_test_coordinator().await?;
        assert!(!coordinator.id.is_nil());
        Ok(())
    }

    #[tokio::test]
    async fn test_coordination_message_processing() -> Result<(), Box<dyn std::error::Error>> {
        let coordinator = create_test_coordinator().await?;

        // Test sending a coordination message
        let agent_id = Uuid::new_v4();
        coordinator
            .coordination_tx
            .send(CoordinationMessage::AgentRegistered { agent_id })?;

        // Give some time for message processing
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        Ok(())
    }

    #[tokio::test]
    async fn test_create_agent_success() -> Result<(), Box<dyn std::error::Error>> {
        let coordinator = create_test_coordinator().await?;

        let config = serde_json::json!({
            "type": "worker",
            "name": "test_agent"
        });

        let agent_id = coordinator.create_agent(config).await?;
        assert!(!agent_id.is_nil());

        // Verify agent was created
        let agent = coordinator.get_agent(agent_id).await;
        assert!(agent.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_agent_invalid_config() -> Result<(), Box<dyn std::error::Error>> {
        let coordinator = create_test_coordinator().await?;

        // Test with missing type
        let config = serde_json::json!({
            "name": "test_agent"
        });

        let result = coordinator.create_agent(config).await;
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_remove_agent_success() -> Result<(), Box<dyn std::error::Error>> {
        let coordinator = create_test_coordinator().await?;

        let config = serde_json::json!({
            "type": "worker",
            "name": "test_agent"
        });

        let agent_id = coordinator.create_agent(config).await?;
        assert!(coordinator.get_agent(agent_id).await.is_some());

        // Remove the agent
        coordinator.remove_agent(agent_id).await?;
        assert!(coordinator.get_agent(agent_id).await.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_remove_agent_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let coordinator = create_test_coordinator().await?;

        let fake_id = Uuid::new_v4();
        let result = coordinator.remove_agent(fake_id).await;
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_all_agents() -> Result<(), Box<dyn std::error::Error>> {
        let coordinator = create_test_coordinator().await?;

        let config1 = serde_json::json!({
            "type": "worker",
            "name": "agent1"
        });
        let config2 = serde_json::json!({
            "type": "coordinator",
            "name": "agent2"
        });

        let agent_id1 = coordinator.create_agent(config1).await?;
        let agent_id2 = coordinator.create_agent(config2).await?;

        let all_agents = coordinator.get_all_agents().await;
        assert_eq!(all_agents.len(), 2);

        let agent_ids: Vec<Uuid> = all_agents.iter().map(|(id, _)| *id).collect();
        assert!(agent_ids.contains(&agent_id1));
        assert!(agent_ids.contains(&agent_id2));

        Ok(())
    }

    #[tokio::test]
    async fn test_create_task_success() -> Result<(), Box<dyn std::error::Error>> {
        let coordinator = create_test_coordinator().await?;

        let config = serde_json::json!({
            "type": "computation",
            "title": "Test Task",
            "description": "A test task"
        });

        let task_id = coordinator.create_task(config).await?;
        assert!(!task_id.is_nil());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_status() -> Result<(), Box<dyn std::error::Error>> {
        let coordinator = create_test_coordinator().await?;

        let status = coordinator.get_status().await;
        assert!(status.is_object());
        assert!(status.get("hive_id").is_some());
        assert!(status.get("agents").is_some());
        assert!(status.get("tasks").is_some());
        assert!(status.get("metrics").is_some());
        assert!(status.get("resources").is_some());
        assert!(status.get("timestamp").is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_enhanced_analytics() -> Result<(), Box<dyn std::error::Error>> {
        let coordinator = create_test_coordinator().await?;

        let analytics = coordinator.get_enhanced_analytics().await;
        assert!(analytics.is_object());
        assert!(analytics.get("hive_id").is_some());
        assert!(analytics.get("performance_metrics").is_some());
        assert!(analytics.get("agent_analytics").is_some());
        assert!(analytics.get("task_analytics").is_some());
        assert!(analytics.get("timestamp").is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_task_with_verification() -> Result<(), Box<dyn std::error::Error>> {
        let coordinator = create_test_coordinator().await?;

        // Create a task first
        let task_config = serde_json::json!({
            "type": "computation",
            "title": "Test Task",
            "description": "A test task"
        });
        let task_id = coordinator.create_task(task_config).await?;

        // Create an agent
        let agent_config = serde_json::json!({
            "type": "worker",
            "name": "test_agent"
        });
        let agent_id = coordinator.create_agent(agent_config).await?;

        // Execute the task
        let result = coordinator
            .execute_task_with_verification(task_id, agent_id)
            .await?;
        assert!(result.is_object());

        Ok(())
    }

    #[tokio::test]
    async fn test_shutdown() -> Result<(), Box<dyn std::error::Error>> {
        let coordinator = create_test_coordinator().await?;

        // Shutdown should complete without error
        coordinator.shutdown().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_new_testing_methods() -> Result<(), Box<dyn std::error::Error>> {
        let coordinator = create_test_coordinator().await?;

        // Test initial counts
        assert_eq!(coordinator.get_agent_count(), 0);
        assert_eq!(coordinator.get_task_count().await, 0);

        // Test initial metrics
        let initial_metrics = coordinator.get_metrics().await;
        assert_eq!(initial_metrics.agent_metrics.total_agents, 0);
        assert_eq!(initial_metrics.task_metrics.total_tasks, 0);

        // Create an agent
        let agent_config = serde_json::json!({
            "type": "worker",
            "name": "test_agent"
        });
        let agent_id = coordinator.create_agent(agent_config).await?;

        // Test agent count after creation
        assert_eq!(coordinator.get_agent_count(), 1);

        // Create a task
        let task_config = serde_json::json!({
            "type": "computation",
            "title": "Test Task",
            "description": "A test task"
        });
        let task_id = coordinator.create_task(task_config).await?;

        // Test task count after creation
        assert_eq!(coordinator.get_task_count().await, 1);

        // Test metrics after operations
        let metrics_after = coordinator.get_metrics().await;
        assert_eq!(metrics_after.agent_metrics.total_agents, 1);
        assert_eq!(metrics_after.agent_metrics.active_agents, 1);
        assert_eq!(metrics_after.task_metrics.total_tasks, 1);

        // Verify agent exists
        let agent = coordinator.get_agent(agent_id).await;
        assert!(agent.is_some());
        assert_eq!(agent.unwrap().name, "test_agent");

        Ok(())
    }

    #[tokio::test]
    async fn test_agent_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
        let coordinator = create_test_coordinator().await?;

        // Create agent
        let config = serde_json::json!({
            "type": "worker",
            "name": "lifecycle_test_agent"
        });
        let agent_id = coordinator.create_agent(config).await?;
        assert!(coordinator.get_agent(agent_id).await.is_some());

        // Verify in all agents list
        let all_agents = coordinator.get_all_agents().await;
        assert!(all_agents.iter().any(|(id, _)| *id == agent_id));

        // Remove agent
        coordinator.remove_agent(agent_id).await?;
        assert!(coordinator.get_agent(agent_id).await.is_none());

        // Verify removed from all agents list
        let all_agents_after = coordinator.get_all_agents().await;
        assert!(!all_agents_after.iter().any(|(id, _)| *id == agent_id));

        Ok(())
    }
}
