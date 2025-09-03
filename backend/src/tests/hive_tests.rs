//! Unit tests for the hive coordination system

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::agents::AgentType;
    use crate::core::{HiveCoordinator, SwarmMetrics};
    use crate::tasks::{Task, TaskPriority};
    use crate::tests::test_utils::{
        assert_approx_eq, create_agent_config, create_task_config, create_test_task,
    };

    #[tokio::test]
    async fn test_hive_coordinator_creation() {
        let hive = HiveCoordinator::new().await;
        assert!(hive.is_ok());

        let coordinator = hive.unwrap();
        assert_eq!(coordinator.agents.len(), 0);

        // Check initial metrics
        let metrics = coordinator.metrics.read().await;
        assert_eq!(metrics.total_agents, 0);
        assert_eq!(metrics.active_agents, 0);
        assert_eq!(metrics.completed_tasks, 0);
        assert_eq!(metrics.failed_tasks, 0);
        assert_approx_eq(metrics.average_performance, 0.0, 0.001);
        assert_approx_eq(metrics.swarm_cohesion, 0.0, 0.001);
        assert_approx_eq(metrics.learning_progress, 0.0, 0.001);
    }

    #[tokio::test]
    async fn test_hive_agent_creation() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Test creating a basic worker agent
        let worker_config = create_agent_config("TestWorker", "worker", None);
        let agent_id = hive.create_agent(worker_config).await;
        assert!(agent_id.is_ok());

        let worker_id = agent_id.unwrap();
        assert!(hive.agents.contains_key(&worker_id));

        let agent = hive.agents.get(&worker_id).unwrap();
        assert_eq!(agent.name, "TestWorker");
        assert!(matches!(agent.agent_type, AgentType::Worker));
        assert_eq!(agent.capabilities.len(), 0); // No capabilities specified
    }

    #[tokio::test]
    async fn test_hive_agent_creation_with_capabilities() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Test creating an agent with capabilities
        let capabilities = vec![("data_processing", 0.8, 0.1), ("analysis", 0.6, 0.15)];
        let agent_config = create_agent_config("SkillfulAgent", "worker", Some(capabilities));
        let agent_id = hive.create_agent(agent_config).await.unwrap();

        let agent = hive.agents.get(&agent_id).unwrap();
        assert_eq!(agent.capabilities.len(), 2);
        assert_approx_eq(agent.get_capability_score("data_processing"), 0.8, 0.001);
        assert_approx_eq(agent.get_capability_score("analysis"), 0.6, 0.001);
    }

    #[tokio::test]
    async fn test_hive_different_agent_types() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Test creating different agent types
        let coordinator_config = create_agent_config("Coordinator", "coordinator", None);
        let coordinator_id = hive.create_agent(coordinator_config).await.unwrap();

        let learner_config = create_agent_config("Learner", "learner", None);
        let learner_id = hive.create_agent(learner_config).await.unwrap();

        let specialist_config = create_agent_config("Specialist", "specialist:AI", None);
        let specialist_id = hive.create_agent(specialist_config).await.unwrap();

        // Verify agent types
        let coordinator = hive.agents.get(&coordinator_id).unwrap();
        assert!(matches!(coordinator.agent_type, AgentType::Coordinator));

        let learner = hive.agents.get(&learner_id).unwrap();
        assert!(matches!(learner.agent_type, AgentType::Learner));

        let specialist = hive.agents.get(&specialist_id).unwrap();
        if let AgentType::Specialist(domain) = &specialist.agent_type {
            assert_eq!(domain, "AI");
        } else {
            panic!("Expected specialist agent type");
        }
    }

    #[tokio::test]
    async fn test_hive_task_creation() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Test creating a basic task
        let task_config = create_task_config("Test task", "general", 1, None);
        let task_id = hive.create_task(task_config).await;
        assert!(task_id.is_ok());

        // Verify task was added to work-stealing queue
        let ws_metrics = hive.work_stealing_queue.get_metrics().await;
        assert!(ws_metrics.total_queue_depth > 0 || ws_metrics.global_queue_depth > 0);
    }

    #[tokio::test]
    async fn test_hive_task_creation_with_requirements() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Test creating a task with capability requirements
        let requirements = vec![("data_processing", 0.7), ("analysis", 0.5)];
        let task_config = create_task_config(
            "Complex task",
            "analysis",
            2, // High priority
            Some(requirements),
        );
        let task_id = hive.create_task(task_config).await;
        assert!(task_id.is_ok());
    }

    #[tokio::test]
    async fn test_hive_task_priority_handling() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Create tasks with different priorities
        let low_config = create_task_config("Low priority", "general", 0, None);
        let medium_config = create_task_config("Medium priority", "general", 1, None);
        let high_config = create_task_config("High priority", "general", 2, None);
        let critical_config = create_task_config("Critical priority", "general", 3, None);

        let _low_id = hive.create_task(low_config).await.unwrap();
        let _medium_id = hive.create_task(medium_config).await.unwrap();
        let _high_id = hive.create_task(high_config).await.unwrap();
        let _critical_id = hive.create_task(critical_config).await.unwrap();

        // All tasks should be created successfully
        let ws_metrics = hive.work_stealing_queue.get_metrics().await;
        assert!(ws_metrics.total_queue_depth >= 4 || ws_metrics.global_queue_depth >= 4);
    }

    #[tokio::test]
    async fn test_hive_agents_info() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Initially should have no agents
        let initial_info = hive.get_agents_info().await;
        assert_eq!(initial_info["total_count"].as_u64().unwrap(), 0);
        assert_eq!(initial_info["agents"].as_array().unwrap().len(), 0);

        // Add some agents
        let worker_config = create_agent_config("Worker1", "worker", None);
        let _worker_id = hive.create_agent(worker_config).await.unwrap();

        let coordinator_config = create_agent_config("Coordinator1", "coordinator", None);
        let _coordinator_id = hive.create_agent(coordinator_config).await.unwrap();

        // Check updated info
        let updated_info = hive.get_agents_info().await;
        assert_eq!(updated_info["total_count"].as_u64().unwrap(), 2);
        assert_eq!(updated_info["agents"].as_array().unwrap().len(), 2);

        // Verify agent information structure
        let agents_array = updated_info["agents"].as_array().unwrap();
        let first_agent = &agents_array[0];
        assert!(first_agent["id"].is_string());
        assert!(first_agent["name"].is_string());
        assert!(first_agent["type"].is_object() || first_agent["type"].is_string());
        assert!(first_agent["state"].is_string());
        assert!(first_agent["capabilities"].is_array());
        assert!(first_agent["position"].is_array());
        assert!(first_agent["energy"].is_number());
    }

    #[tokio::test]
    async fn test_hive_tasks_info() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Check initial task info
        let initial_info = hive.get_tasks_info().await;
        assert!(initial_info["legacy_queue"].is_object());
        assert!(initial_info["work_stealing_queue"].is_object());

        // Add some tasks
        let task_config1 = create_task_config("Task 1", "general", 1, None);
        let _task_id1 = hive.create_task(task_config1).await.unwrap();

        let task_config2 = create_task_config("Task 2", "analysis", 2, None);
        let _task_id2 = hive.create_task(task_config2).await.unwrap();

        // Check updated task info
        let updated_info = hive.get_tasks_info().await;
        let ws_queue_info = &updated_info["work_stealing_queue"];

        // Should have some tasks in the system
        assert!(
            ws_queue_info["total_queue_depth"].as_u64().unwrap_or(0) > 0
                || ws_queue_info["global_queue_depth"].as_u64().unwrap_or(0) > 0
        );
    }

    #[tokio::test]
    async fn test_hive_status() {
        let hive = HiveCoordinator::new().await.unwrap();

        let status = hive.get_status().await;

        // Verify status structure
        assert!(status["hive_id"].is_string());
        assert!(status["created_at"].is_string());
        assert!(status["last_update"].is_string());
        assert!(status["metrics"].is_object());
        assert!(status["swarm_center"].is_array());
        assert!(status["total_energy"].is_number());

        // Verify metrics structure
        let metrics = &status["metrics"];
        assert!(metrics["total_agents"].is_number());
        assert!(metrics["active_agents"].is_number());
        assert!(metrics["completed_tasks"].is_number());
        assert!(metrics["failed_tasks"].is_number());
        assert!(metrics["average_performance"].is_number());
        assert!(metrics["swarm_cohesion"].is_number());
        assert!(metrics["learning_progress"].is_number());

        // Verify swarm center is a 2-element array
        let swarm_center = status["swarm_center"].as_array().unwrap();
        assert_eq!(swarm_center.len(), 2);
        assert!(swarm_center[0].is_number());
        assert!(swarm_center[1].is_number());
    }

    #[tokio::test]
    async fn test_hive_resource_info() {
        let hive = HiveCoordinator::new().await.unwrap();

        let resource_info = hive.get_resource_info().await;

        // Verify resource info structure
        assert!(resource_info["system_resources"].is_object());
        assert!(resource_info["resource_profile"].is_object());
        assert!(resource_info["hardware_class"].is_string());
        assert_eq!(resource_info["phase_2_status"], "active");
        assert_eq!(resource_info["optimization_enabled"], true);

        // Verify system resources structure
        let system_resources = &resource_info["system_resources"];
        assert!(system_resources["cpu_usage"].is_number());
        assert!(system_resources["memory_usage"].is_number());

        // Verify resource profile structure
        let resource_profile = &resource_info["resource_profile"];
        assert!(resource_profile["profile_name"].is_string());
        assert!(resource_profile["max_agents"].is_number());
    }

    #[tokio::test]
    async fn test_hive_simple_verification_stats() {
        let hive = HiveCoordinator::new().await.unwrap();

        let verification_stats = hive.get_simple_verification_stats().await;

        // Verify verification stats structure
        assert!(verification_stats["total_verifications"].is_number());
        assert!(verification_stats["passed_verifications"].is_number());
        assert!(verification_stats["failed_verifications"].is_number());
        assert!(verification_stats["success_rate"].is_number());
        assert!(verification_stats["average_verification_time_ms"].is_number());
        assert!(verification_stats["average_confidence_score"].is_number());
        assert!(verification_stats["tier_usage"].is_object());
        assert!(verification_stats["rule_effectiveness"].is_object());

        // Initial stats should show no verifications
        assert_eq!(
            verification_stats["total_verifications"].as_u64().unwrap(),
            0
        );
        assert!((verification_stats["success_rate"].as_f64().unwrap() - 0.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_hive_simple_verification_config() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Test configuration
        let config = json!({
            "confidence_threshold": 0.8,
            "task_rules": {
                "analysis": {
                    "min_output_length": 100,
                    "required_keywords": ["data", "result"]
                }
            },
            "ai_reviewer_agent": "00000000-0000-0000-0000-000000000000"
        });

        let result = hive.configure_simple_verification(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_hive_auto_scaling_stats() {
        let hive = HiveCoordinator::new().await.unwrap();

        let auto_scaling_stats = hive.get_auto_scaling_stats().await;

        // Should return a valid JSON object
        assert!(auto_scaling_stats.is_object());
    }

    #[tokio::test]
    async fn test_hive_skill_evolution_stats() {
        let hive = HiveCoordinator::new().await.unwrap();

        let skill_evolution_stats = hive.get_skill_evolution_stats().await;

        // Should return a valid JSON object
        assert!(skill_evolution_stats.is_object());
    }

    #[tokio::test]
    async fn test_hive_enhanced_analytics() {
        let hive = HiveCoordinator::new().await.unwrap();

        let analytics = hive.get_enhanced_analytics().await;

        // Verify enhanced analytics structure
        assert!(analytics["hive_status"].is_object());
        assert!(analytics["auto_scaling"].is_object());
        assert!(analytics["skill_evolution"].is_object());
        assert!(analytics["resource_management"].is_object());
        assert!(analytics["enhanced_features"].is_object());

        // Verify enhanced features
        let enhanced_features = &analytics["enhanced_features"];
        assert_eq!(enhanced_features["dynamic_scaling_enabled"], true);
        assert_eq!(enhanced_features["skill_learning_enabled"], true);
        assert_eq!(enhanced_features["neural_coordination_active"], true);
        assert_eq!(enhanced_features["swarm_formations_active"], true);
    }

    #[tokio::test]
    async fn test_hive_execute_task_with_simple_verification() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Create an agent capable of performing tasks
        let agent_config =
            create_agent_config("TestAgent", "worker", Some(vec![("general", 0.8, 0.1)]));
        let _agent_id = hive.create_agent(agent_config).await.unwrap();

        // Create a task
        let task_config = create_task_config("Verification test", "general", 1, None);
        let task_id = hive.create_task(task_config).await.unwrap();

        // Add task to legacy queue for verification test
        {
            let task = create_test_task("Verification test", "general", TaskPriority::Medium);
            let mut queue = hive.task_queue.write().await;
            queue.add_task(task);
        }

        // Execute with verification
        let result = hive
            .execute_task_with_simple_verification(task_id, Some("Complete the general task"))
            .await;

        // Should succeed (though the specific task might not be found in pending queue)
        // This tests the method signature and basic functionality
        if result.is_err() {
            // Expected if task not found in pending queue (moved to work-stealing queue)
            let error_msg = result.unwrap_err().to_string();
            assert!(error_msg.contains("not found") || error_msg.contains("No suitable agent"));
        }
    }

    #[tokio::test]
    async fn test_hive_concurrent_agent_creation() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Create multiple agents concurrently
        let mut handles = vec![];
        for i in 0..5 {
            let hive_clone = hive.clone();
            let handle = tokio::spawn(async move {
                let config = create_agent_config(&format!("Agent{}", i), "worker", None);
                hive_clone.create_agent(config).await
            });
            handles.push(handle);
        }

        // Wait for all agents to be created
        let mut agent_ids = vec![];
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
            agent_ids.push(result.unwrap());
        }

        // Verify all agents were created with unique IDs
        assert_eq!(agent_ids.len(), 5);
        assert_eq!(hive.agents.len(), 5);

        // Check that all IDs are unique
        for i in 0..agent_ids.len() {
            for j in (i + 1)..agent_ids.len() {
                assert_ne!(agent_ids[i], agent_ids[j]);
            }
        }
    }

    #[tokio::test]
    async fn test_hive_concurrent_task_creation() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Create multiple tasks concurrently
        let mut handles = vec![];
        for i in 0..5 {
            let hive_clone = hive.clone();
            let handle = tokio::spawn(async move {
                let config = create_task_config(&format!("Task{}", i), "general", 1, None);
                hive_clone.create_task(config).await
            });
            handles.push(handle);
        }

        // Wait for all tasks to be created
        let mut task_ids = vec![];
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
            task_ids.push(result.unwrap());
        }

        // Verify all tasks were created with unique IDs
        assert_eq!(task_ids.len(), 5);

        // Check that all IDs are unique
        for i in 0..task_ids.len() {
            for j in (i + 1)..task_ids.len() {
                assert_ne!(task_ids[i], task_ids[j]);
            }
        }
    }

    #[tokio::test]
    async fn test_hive_metrics_update_with_agents() {
        let hive = HiveCoordinator::new().await.unwrap();

        // Add some agents
        for i in 0..3 {
            let config = create_agent_config(
                &format!("Agent{}", i),
                "worker",
                Some(vec![("general", 0.7, 0.1)]),
            );
            let _agent_id = hive.create_agent(config).await.unwrap();
        }

        // Wait a bit for metrics to update
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let status = hive.get_status().await;
        let metrics = &status["metrics"];

        // Should reflect the added agents
        assert_eq!(metrics["total_agents"].as_u64().unwrap(), 3);
        assert!(metrics["average_performance"].as_f64().unwrap() > 0.0);
    }

    #[test]
    fn test_swarm_metrics_serialization() {
        let metrics = SwarmMetrics {
            total_agents: 5,
            active_agents: 3,
            completed_tasks: 10,
            failed_tasks: 2,
            average_performance: 0.75,
            swarm_cohesion: 0.8,
            learning_progress: 0.6,
        };

        // Test serialization
        let serialized = serde_json::to_string(&metrics);
        assert!(serialized.is_ok());

        // Test deserialization
        let json_str = serialized.unwrap();
        let deserialized: Result<SwarmMetrics, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let restored_metrics = deserialized.unwrap();
        assert_eq!(restored_metrics.total_agents, metrics.total_agents);
        assert_eq!(restored_metrics.active_agents, metrics.active_agents);
        assert_approx_eq(
            restored_metrics.average_performance,
            metrics.average_performance,
            0.001,
        );
    }
}
