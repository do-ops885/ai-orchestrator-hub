//! Integration tests for the multiagent hive system
//!
//! These tests verify that different components work together correctly
//! and test end-to-end workflows.

use std::time::Duration;

use crate::agents::{AgentBehavior, AgentState};
use crate::core::HiveCoordinator;
use crate::tasks::TaskPriority;
use crate::tests::test_utils::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_end_to_end_task_execution() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Create an agent with specific capabilities
        let agent_config = create_agent_config(
            "TaskExecutor",
            "worker",
            Some(vec![("data_processing", 0.8, 0.1)]),
        );
        let agent_id = hive.create_agent(agent_config).await.unwrap();

        // Create a task that matches the agent's capabilities
        let task_config = create_task_config(
            "Process data file",
            "data_processing",
            1, // Medium priority
            Some(vec![("data_processing", 0.7)]),
        );
        let _task_id = hive.create_task(task_config).await.unwrap();

        // Wait a bit for the system to process
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Check that the agent exists and task was created
        assert!(hive.agents.contains_key(&agent_id));

        let agent = hive.agents.get(&agent_id).unwrap();
        assert_eq!(agent.name, "TaskExecutor");
        assert_approx_eq(agent.get_capability_score("data_processing"), 0.8, 0.001);

        // Verify task was submitted to work-stealing queue
        let ws_metrics = hive.work_stealing_queue.get_metrics().await;
        assert!(ws_metrics.total_queue_depth > 0 || ws_metrics.global_queue_depth > 0);
    }

    #[tokio::test]
    async fn test_multiple_agents_task_distribution() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Create multiple agents with different capabilities
        let agents_config = vec![
            (
                "DataProcessor",
                "worker",
                vec![("data_processing", 0.9, 0.1)],
            ),
            ("Analyst", "worker", vec![("analysis", 0.8, 0.1)]),
            (
                "Coordinator",
                "coordinator",
                vec![("coordination", 0.7, 0.1)],
            ),
        ];

        let mut agent_ids = vec![];
        for (name, agent_type, capabilities) in agents_config {
            let config = create_agent_config(name, agent_type, Some(capabilities));
            let agent_id = hive.create_agent(config).await.unwrap();
            agent_ids.push(agent_id);
        }

        // Create tasks with different requirements
        let tasks_config = vec![
            ("Data task", "data", 1, vec![("data_processing", 0.8)]),
            ("Analysis task", "analysis", 2, vec![("analysis", 0.7)]),
            ("Coordination task", "coord", 1, vec![("coordination", 0.6)]),
        ];

        let mut task_ids = vec![];
        for (description, task_type, priority, requirements) in tasks_config {
            let config = create_task_config(description, task_type, priority, Some(requirements));
            let task_id = hive.create_task(config).await.unwrap();
            task_ids.push(task_id);
        }

        // Wait for task distribution
        tokio::time::sleep(Duration::from_millis(1000)).await;

        // Verify agents and tasks were created
        assert_eq!(hive.agents.len(), 3);
        assert_eq!(agent_ids.len(), 3);
        assert_eq!(task_ids.len(), 3);

        // Check work-stealing queue has processed tasks
        let _ws_metrics = hive.work_stealing_queue.get_metrics().await;
        // Queue depth is always non-negative by type definition
    }

    #[tokio::test]
    async fn test_agent_learning_and_improvement() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Create a learner agent
        let agent_config = create_agent_config(
            "LearnerAgent",
            "learner",
            Some(vec![("learning", 0.5, 0.2)]), // High learning rate
        );
        let agent_id = hive.create_agent(agent_config).await.unwrap();

        // Get initial capability score
        let _initial_score = {
            let agent = hive.agents.get(&agent_id).unwrap();
            agent.get_capability_score("learning")
        };

        // Create and execute a task to generate learning experience
        let task = create_test_task("Learning task", "learning", TaskPriority::Medium);

        {
            let mut agent = hive.agents.get_mut(&agent_id).unwrap().clone();
            let _result = agent.execute_task(task).await.unwrap();

            // Update agent in hive
            hive.agents.insert(agent_id, agent);
        }

        // Trigger learning cycle
        let nlp_processor = &hive.nlp_processor;
        {
            let mut agent = hive.agents.get_mut(&agent_id).unwrap().clone();
            let _learn_result = agent.learn(nlp_processor).await.unwrap();

            // Update agent in hive
            hive.agents.insert(agent_id, agent);
        }

        // Check if agent has gained experience
        let agent = hive.agents.get(&agent_id).unwrap();
        assert!(!agent.memory.experiences.is_empty());

        // Learning progress should be reflected in patterns
        assert!(!agent.memory.learned_patterns.is_empty());
    }

    #[tokio::test]
    async fn test_swarm_coordination_and_positioning() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Create multiple agents for swarm behavior
        let mut agent_ids = vec![];
        for i in 0..5 {
            let config = create_agent_config(
                &format!("SwarmAgent{}", i),
                "worker",
                Some(vec![("coordination", 0.6, 0.1)]),
            );
            let agent_id = hive.create_agent(config).await.unwrap();
            agent_ids.push(agent_id);
        }

        // Set initial positions
        let positions = vec![
            (0.0, 0.0),
            (10.0, 0.0),
            (0.0, 10.0),
            (-10.0, 0.0),
            (0.0, -10.0),
        ];

        for (i, &agent_id) in agent_ids.iter().enumerate() {
            if let Some(mut agent_ref) = hive.agents.get_mut(&agent_id) {
                agent_ref.position = positions[i];
            }
        }

        // Wait for swarm coordination to run
        tokio::time::sleep(Duration::from_millis(2000)).await;

        // Check swarm center calculation
        let swarm_center = *hive.swarm_center.read().await;

        // Swarm center should be approximately at origin (0, 0) given symmetric positions
        assert!(swarm_center.0.abs() < 5.0);
        assert!(swarm_center.1.abs() < 5.0);

        // Agents should have updated positions (swarm behavior)
        let mut positions_changed = 0;
        for (i, &agent_id) in agent_ids.iter().enumerate() {
            if let Some(agent) = hive.agents.get(&agent_id) {
                if agent.position != positions[i] {
                    positions_changed += 1;
                }
            }
        }

        // At least some agents should have moved due to swarm forces
        assert!(positions_changed > 0);
    }

    #[tokio::test]
    async fn test_hive_metrics_and_monitoring() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Create agents and tasks
        for i in 0..3 {
            let agent_config = create_agent_config(
                &format!("MetricsAgent{}", i),
                "worker",
                Some(vec![("general", 0.7, 0.1)]),
            );
            let _agent_id = hive.create_agent(agent_config).await.unwrap();
        }

        for i in 0..2 {
            let task_config = create_task_config(&format!("MetricsTask{}", i), "general", 1, None);
            let _task_id = hive.create_task(task_config).await.unwrap();
        }

        // Wait for metrics to update
        tokio::time::sleep(Duration::from_millis(1000)).await;

        // Check comprehensive status
        let status = hive.get_status().await;
        assert_eq!(status["metrics"]["total_agents"], 3);

        // Check agents info
        let agents_info = hive.get_agents_info().await;
        assert_eq!(agents_info["total_count"], 3);

        // Check tasks info
        let tasks_info = hive.get_tasks_info().await;
<<<<<<< HEAD
        assert!(
            tasks_info["work_stealing_queue"]["total_queue_depth"]
                .as_u64()
                .unwrap_or(0)
                >= 0
        );
=======
        // Queue depth is always non-negative by type definition
        let _queue_depth = tasks_info["work_stealing_queue"]["total_queue_depth"]
            .as_u64()
            .unwrap_or(0);
>>>>>>> 8b3a402 (wip)

        // Check resource info
        let resource_info = hive.get_resource_info().await;
        assert!(resource_info["system_resources"]["cpu_usage"].is_number());
        assert!(resource_info["system_resources"]["memory_usage"].is_number());

        // Check enhanced analytics
        let analytics = hive.get_enhanced_analytics().await;
        assert!(analytics["hive_status"].is_object());
<<<<<<< HEAD
        assert!(analytics["enhanced_features"]["dynamic_scaling_enabled"]
            .as_bool()
            .unwrap());
=======
        assert!(
            analytics["enhanced_features"]["dynamic_scaling_enabled"]
                .as_bool()
                .unwrap()
        );
>>>>>>> 8b3a402 (wip)
    }

    #[tokio::test]
    async fn test_neural_integration_with_agents() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Create agent with neural capabilities
        let agent_config = create_agent_config(
            "NeuralAgent",
            "learner",
            Some(vec![("neural_processing", 0.6, 0.15)]),
        );
        let _agent_id = hive.create_agent(agent_config).await.unwrap();

        // Test NLP processing
        let nlp_processor = &hive.nlp_processor;
        let test_text = "successful task completion with excellent results";
        let tokens: Vec<String> = test_text
            .split_whitespace()
<<<<<<< HEAD
            .map(|s| s.to_string())
=======
            .map(std::string::ToString::to_string)
>>>>>>> 8b3a402 (wip)
            .collect();
        let sentiment = nlp_processor.analyze_sentiment(&tokens);
        let keywords = if sentiment > 0.0 {
            vec!["successful".to_string(), "excellent".to_string()]
        } else {
            vec![]
        };
        assert!(!keywords.is_empty());

        let tokens = vec!["successful".to_string(), "excellent".to_string()];
        let sentiment = nlp_processor.analyze_sentiment(&tokens);
        assert!(sentiment > 0.0); // Should be positive

        // Test neural processor integration
        let _neural_processor = hive.neural_processor.read().await;
        // Neural processor should be initialized
        // (Specific neural operations depend on whether advanced features are enabled)
    }

    #[tokio::test]
    async fn test_work_stealing_queue_integration() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Create agents
        let mut agent_ids = vec![];
        for i in 0..3 {
            let config = create_agent_config(
                &format!("WSAgent{}", i),
                "worker",
                Some(vec![("general", 0.8, 0.1)]),
            );
            let agent_id = hive.create_agent(config).await.unwrap();
            agent_ids.push(agent_id);
        }

        // Create tasks
        for i in 0..5 {
            let config = create_task_config(&format!("WSTask{}", i), "general", 1, None);
            let _task_id = hive.create_task(config).await.unwrap();
        }

        // Wait for work-stealing to process
        tokio::time::sleep(Duration::from_millis(2000)).await;

        // Check work-stealing metrics
        let ws_metrics = hive.work_stealing_queue.get_metrics().await;
        assert_eq!(ws_metrics.active_agents, 3);

        // Tasks should be distributed or completed
        // Queue depths are always non-negative by type definition
    }

    #[tokio::test]
    async fn test_simple_verification_integration() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Create agent
        let agent_config = create_agent_config(
            "VerificationAgent",
            "worker",
            Some(vec![("verification", 0.7, 0.1)]),
        );
        let _agent_id = hive.create_agent(agent_config).await.unwrap();

        // Check verification stats (should be empty initially)
        let verification_stats = hive.get_simple_verification_stats().await;
        assert_eq!(verification_stats["total_verifications"], 0);
        assert_eq!(verification_stats["success_rate"], 0.0);

        // Test verification configuration
        let config = serde_json::json!({
            "confidence_threshold": 0.75,
            "task_rules": {
                "verification": {
                    "min_output_length": 50
                }
            }
        });
        let config_result = hive.configure_simple_verification(config).await;
        assert!(config_result.is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Spawn concurrent operations
        let hive1 = hive.clone();
        let hive2 = hive.clone();
        let hive3 = hive.clone();

        let agent_handle = tokio::spawn(async move {
            for i in 0..3 {
                let config = create_agent_config(&format!("ConcurrentAgent{i}"), "worker", None);
                let _result = hive1.create_agent(config).await;
            }
        });

        let task_handle = tokio::spawn(async move {
            for i in 0..3 {
<<<<<<< HEAD
                let config =
                    create_task_config(&format!("ConcurrentTask{}", i), "general", 1, None);
=======
                let config = create_task_config(&format!("ConcurrentTask{i}"), "general", 1, None);
>>>>>>> 8b3a402 (wip)
                let _result = hive2.create_task(config).await;
            }
        });

        let status_handle = tokio::spawn(async move {
            for _ in 0..5 {
                let _status = hive3.get_status().await;
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        // Wait for all operations to complete
        let (agent_result, task_result, status_result) =
            tokio::join!(agent_handle, task_handle, status_handle);

        assert!(agent_result.is_ok());
        assert!(task_result.is_ok());
        assert!(status_result.is_ok());

        // Verify final state
        assert_eq!(hive.agents.len(), 3);

        let final_status = hive.get_status().await;
        assert_eq!(final_status["metrics"]["total_agents"], 3);
    }

    #[tokio::test]
    async fn test_error_recovery_and_resilience() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Test invalid agent configuration
        let invalid_config = serde_json::json!({
            "name": "",
            "type": "invalid_type"
        });
        let agent_result = hive.create_agent(invalid_config).await;
        // Should handle gracefully (create with default type)
        assert!(agent_result.is_ok());

        // Test invalid task configuration
        let invalid_task_config = serde_json::json!({
            "description": "",
            "priority": 999 // Invalid priority
        });
        let task_result = hive.create_task(invalid_task_config).await;
        // Should handle gracefully (use default priority)
        assert!(task_result.is_ok());

        // Test operations on empty hive
        let empty_hive = HiveCoordinator::new().await.unwrap();
        let empty_status = empty_hive.get_status().await;
        assert_eq!(empty_status["metrics"]["total_agents"], 0);

        let empty_agents = empty_hive.get_agents_info().await;
        assert_eq!(empty_agents["total_count"], 0);
    }

    #[tokio::test]
    async fn test_system_scalability() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Create a moderate number of agents and tasks to test scalability
        let num_agents = 10;
        let num_tasks = 20;

        // Create agents
        for i in 0..num_agents {
            let config = create_agent_config(
                &format!("ScaleAgent{i}"),
                if i % 3 == 0 { "coordinator" } else { "worker" },
                Some(vec![("general", 0.6 + (i as f64 * 0.02), 0.1)]),
            );
            let _agent_id = hive.create_agent(config).await.unwrap();
        }

        // Create tasks
        for i in 0..num_tasks {
            let priority = match i % 4 {
                0 => 0, // Low
                1 => 1, // Medium
                2 => 2, // High
                _ => 3, // Critical
            };
<<<<<<< HEAD
            let config = create_task_config(&format!("ScaleTask{}", i), "general", priority, None);
=======
            let config = create_task_config(&format!("ScaleTask{i}"), "general", priority, None);
>>>>>>> 8b3a402 (wip)
            let _task_id = hive.create_task(config).await.unwrap();
        }

        // Wait for system to process
        tokio::time::sleep(Duration::from_millis(3000)).await;

        // Verify system handled the load
        assert_eq!(hive.agents.len(), num_agents);

        let status = hive.get_status().await;
        assert_eq!(status["metrics"]["total_agents"], num_agents);

        // Check that work-stealing queue is functioning
        let ws_metrics = hive.work_stealing_queue.get_metrics().await;
        assert_eq!(ws_metrics.active_agents, num_agents);

        // System should be responsive
        let resource_info = hive.get_resource_info().await;
        assert!(resource_info["system_resources"]["cpu_usage"].is_number());
    }

    #[tokio::test]
    async fn test_agent_communication_integration() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Create communicating agents
        let agent1_config = create_agent_config(
            "Communicator1",
            "coordinator",
            Some(vec![("communication", 0.8, 0.1)]),
        );
        let agent1_id = hive.create_agent(agent1_config).await.unwrap();

        let agent2_config = create_agent_config(
            "Communicator2",
            "worker",
            Some(vec![("communication", 0.7, 0.1)]),
        );
        let agent2_id = hive.create_agent(agent2_config).await.unwrap();

        // Test communication between agents
        {
            let mut agent1 = hive.agents.get_mut(&agent1_id).unwrap().clone();
            let response = agent1
                .communicate("Hello from agent 1", Some(agent2_id))
                .await;
            assert!(response.is_ok());

            let response_text = response.unwrap();
            assert!(response_text.contains("Communicator1"));
            assert!(response_text.contains(&agent2_id.to_string()));

            // Update agent state
            hive.agents.insert(agent1_id, agent1);
        }

        // Test broadcast communication
        {
            let mut agent2 = hive.agents.get_mut(&agent2_id).unwrap().clone();
            let broadcast_response = agent2.communicate("Broadcasting message", None).await;
            assert!(broadcast_response.is_ok());

            let broadcast_text = broadcast_response.unwrap();
            assert!(broadcast_text.contains("Communicator2"));
            assert!(broadcast_text.contains("broadcasting"));

            // Update agent state
            hive.agents.insert(agent2_id, agent2);
        }

        // Verify agents are back to idle state
        let agent1 = hive.agents.get(&agent1_id).unwrap();
        let agent2 = hive.agents.get(&agent2_id).unwrap();
        assert!(matches!(agent1.state, AgentState::Idle));
        assert!(matches!(agent2.state, AgentState::Idle));
    }
}
