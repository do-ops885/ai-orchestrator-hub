//! Unit tests for the agent system

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use crate::agents::{Agent, AgentState, AgentType, Experience};
    use crate::neural::NLPProcessor;
    use crate::tasks::{Task, TaskPriority};
    use crate::tests::test_utils::{
        assert_approx_eq, create_test_agent, create_test_capability,
        create_test_required_capability, create_test_task, create_test_task_with_requirements,
    };

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new("TestAgent".to_string(), AgentType::Worker);

        assert_eq!(agent.name, "TestAgent");
        assert!(matches!(agent.agent_type, AgentType::Worker));
        assert!(matches!(agent.state, AgentState::Idle));
        assert_eq!(agent.capabilities.len(), 0);
        assert!((agent.energy - 100.0).abs() < f64::EPSILON);
        assert_eq!(agent.position, (0.0, 0.0));
        assert!(agent.memory.experiences.is_empty());
        assert!(agent.memory.learned_patterns.is_empty());
        assert!(agent.memory.social_connections.is_empty());
    }

    #[test]
    fn test_agent_types() {
        let worker = Agent::new("Worker".to_string(), AgentType::Worker);
        let coordinator = Agent::new("Coordinator".to_string(), AgentType::Coordinator);
        let learner = Agent::new("Learner".to_string(), AgentType::Learner);
        let specialist = Agent::new(
            "Specialist".to_string(),
            AgentType::Specialist("AI".to_string()),
        );

        assert!(matches!(worker.agent_type, AgentType::Worker));
        assert!(matches!(coordinator.agent_type, AgentType::Coordinator));
        assert!(matches!(learner.agent_type, AgentType::Learner));
        assert!(matches!(specialist.agent_type, AgentType::Specialist(_)));

        if let AgentType::Specialist(domain) = &specialist.agent_type {
            assert_eq!(domain, "AI");
        }
    }

    #[test]
    fn test_agent_capability_management() {
        let mut agent = create_test_agent("TestAgent", AgentType::Worker);

        // Test adding capabilities
        let capability = create_test_capability("data_processing", 0.8, 0.1);
        agent.add_capability(capability);

        assert_eq!(agent.capabilities.len(), 3); // 2 default + 1 added
        assert_approx_eq(agent.get_capability_score("data_processing"), 0.8, 0.001);
        assert_approx_eq(agent.get_capability_score("nonexistent"), 0.0, 0.001);
    }

    #[test]
    fn test_agent_capability_bounds() {
        let capability_high = create_test_capability("test", 1.5, 0.1); // Should be clamped to 1.0
        let capability_low = create_test_capability("test", -0.5, 0.1); // Should be clamped to 0.0

        assert_approx_eq(capability_high.proficiency, 1.0, 0.001);
        assert_approx_eq(capability_low.proficiency, 0.0, 0.001);
    }

    #[test]
    fn test_agent_learning_from_experience() {
        let mut agent = create_test_agent("TestAgent", AgentType::Worker);

        // Add a capability that can be improved
        agent.add_capability(create_test_capability("general", 0.5, 0.2));

        let initial_proficiency = agent.get_capability_score("general");

        // Create a successful experience
        let success_experience = Experience {
            timestamp: Utc::now(),
            task_type: "general".to_string(),
            success: true,
            context: "Test task".to_string(),
            learned_insight: Some("Learned something useful".to_string()),
        };

        agent.learn_from_experience(success_experience);

        // Proficiency should have improved
        let new_proficiency = agent.get_capability_score("general");
        assert!(new_proficiency > initial_proficiency);
        assert_eq!(agent.memory.experiences.len(), 1);
    }

    #[test]
    fn test_agent_learning_from_failure() {
        let mut agent = create_test_agent("TestAgent", AgentType::Worker);

        // Add a capability that can be degraded
        agent.add_capability(create_test_capability("general", 0.8, 0.2));

        let initial_proficiency = agent.get_capability_score("general");

        // Create a failed experience
        let failure_experience = Experience {
            timestamp: Utc::now(),
            task_type: "general".to_string(),
            success: false,
            context: "Failed task".to_string(),
            learned_insight: Some("Need to improve".to_string()),
        };

        agent.learn_from_experience(failure_experience);

        // Proficiency should have decreased
        let new_proficiency = agent.get_capability_score("general");
        assert!(new_proficiency < initial_proficiency);
        assert_eq!(agent.memory.experiences.len(), 1);
    }

    #[test]
    fn test_agent_memory_limit() {
        let mut agent = create_test_agent("TestAgent", AgentType::Worker);

        // Add more than 1000 experiences to test memory limit
        for i in 0..1100 {
            let experience = Experience {
                timestamp: Utc::now(),
                task_type: "general".to_string(),
                success: i % 2 == 0,
                context: format!("Task {}", i),
                learned_insight: Some(format!("Insight {}", i)),
            };
            agent.learn_from_experience(experience);
        }

        // Memory should be limited to 1000 experiences
        assert_eq!(agent.memory.experiences.len(), 1000);
    }

    #[test]
    fn test_agent_social_connections() {
        let mut agent = create_test_agent("TestAgent", AgentType::Worker);
        let other_agent_id = Uuid::new_v4();

        // Test successful interaction
        agent.update_social_connection(other_agent_id, true);
        assert_approx_eq(
            *agent
                .memory
                .social_connections
                .get(&other_agent_id)
                .unwrap(),
            0.6, // 0.5 + 0.1
            0.001,
        );

        // Test failed interaction
        agent.update_social_connection(other_agent_id, false);
        assert_approx_eq(
            *agent
                .memory
                .social_connections
                .get(&other_agent_id)
                .unwrap(),
            0.5, // 0.6 - 0.1
            0.001,
        );
    }

    #[test]
    fn test_agent_can_perform_task() {
        let mut agent = create_test_agent("TestAgent", AgentType::Worker);
        agent.add_capability(create_test_capability("data_processing", 0.8, 0.1));
        agent.add_capability(create_test_capability("analysis", 0.6, 0.1));

        // Task with no requirements - should be able to perform
        let simple_task = create_test_task("Simple task", "general", TaskPriority::Low);
        assert!(agent.can_perform_task(&simple_task));

        // Task with requirements agent can meet
        let suitable_task = create_test_task_with_requirements(
            "Data task",
            "data",
            TaskPriority::Medium,
            vec![create_test_required_capability("data_processing", 0.7)],
        );
        assert!(agent.can_perform_task(&suitable_task));

        // Task with requirements agent cannot meet
        let unsuitable_task = create_test_task_with_requirements(
            "Expert task",
            "expert",
            TaskPriority::High,
            vec![create_test_required_capability("data_processing", 0.9)],
        );
        assert!(!agent.can_perform_task(&unsuitable_task));

        // Task with multiple requirements
        let complex_task = create_test_task_with_requirements(
            "Complex task",
            "complex",
            TaskPriority::High,
            vec![
                create_test_required_capability("data_processing", 0.7),
                create_test_required_capability("analysis", 0.5),
            ],
        );
        assert!(agent.can_perform_task(&complex_task));

        // Task with one requirement not met
        let impossible_task = create_test_task_with_requirements(
            "Impossible task",
            "impossible",
            TaskPriority::Critical,
            vec![
                create_test_required_capability("data_processing", 0.7),
                create_test_required_capability("nonexistent", 0.5),
            ],
        );
        assert!(!agent.can_perform_task(&impossible_task));
    }

    #[test]
    fn test_agent_task_fitness_calculation() {
        let mut agent = create_test_agent("TestAgent", AgentType::Worker);
        agent.add_capability(create_test_capability("data_processing", 0.8, 0.1));
        agent.add_capability(create_test_capability("analysis", 0.6, 0.1));

        // Task with no requirements - should have default fitness
        let simple_task = create_test_task("Simple task", "general", TaskPriority::Low);
        assert_approx_eq(agent.calculate_task_fitness(&simple_task), 0.5, 0.001);

        // Task with one requirement
        let single_req_task = create_test_task_with_requirements(
            "Data task",
            "data",
            TaskPriority::Medium,
            vec![create_test_required_capability("data_processing", 0.7)],
        );
        assert_approx_eq(agent.calculate_task_fitness(&single_req_task), 0.8, 0.001);

        // Task with multiple requirements
        let multi_req_task = create_test_task_with_requirements(
            "Complex task",
            "complex",
            TaskPriority::High,
            vec![
                create_test_required_capability("data_processing", 0.7),
                create_test_required_capability("analysis", 0.5),
            ],
        );
        let expected_fitness = f64::midpoint(0.8, 0.6); // Average of capabilities
        assert_approx_eq(
            agent.calculate_task_fitness(&multi_req_task),
            expected_fitness,
            0.001,
        );
    }

    #[tokio::test]
    async fn test_agent_execute_task() {
        let mut agent = create_test_agent("TestAgent", AgentType::Worker);
        agent.add_capability(create_test_capability("general", 0.8, 0.1));

        let task = create_test_task("Test task", "general", TaskPriority::Medium);
        let task_id = task.id;

        let result = agent.execute_task(task).await;
        assert!(result.is_ok());

        let task_result = result.unwrap();
        assert_eq!(task_result.task_id, task_id);
        assert_eq!(task_result.agent_id, agent.id);
        assert!(task_result.execution_time > 0);

        // Agent should be back to idle state
        assert!(matches!(agent.state, AgentState::Idle));

        // Agent should have gained experience
        assert_eq!(agent.memory.experiences.len(), 1);
    }

    #[tokio::test]
    async fn test_agent_communication() {
        let mut agent = create_test_agent("TestAgent", AgentType::Worker);
        let target_agent_id = Uuid::new_v4();

        // Test targeted communication
        let response = agent.communicate("Hello", Some(target_agent_id)).await;
        assert!(response.is_ok());
        let response_text = response.unwrap();
        assert!(response_text.contains("TestAgent"));
        assert!(response_text.contains(&target_agent_id.to_string()));
        assert!(response_text.contains("Hello"));

        // Test broadcast communication
        let broadcast_response = agent.communicate("Broadcast message", None).await;
        assert!(broadcast_response.is_ok());
        let broadcast_text = broadcast_response.unwrap();
        assert!(broadcast_text.contains("TestAgent"));
        assert!(broadcast_text.contains("broadcasting"));
    }

    #[tokio::test]
    async fn test_agent_learning_with_nlp() {
        let mut agent = create_test_agent("TestAgent", AgentType::Learner);

        // Add some experiences first
        let experience = Experience {
            timestamp: Utc::now(),
            task_type: "analysis".to_string(),
            success: true,
            context: "Successful analysis task".to_string(),
            learned_insight: Some("Great work on data analysis".to_string()),
        };
        agent.learn_from_experience(experience);

        let nlp_processor = NLPProcessor::new().await.unwrap();
        let result = agent.learn(&nlp_processor).await;
        assert!(result.is_ok());

        // Agent should have learned patterns
        assert!(!agent.memory.learned_patterns.is_empty());
    }

    #[tokio::test]
    async fn test_agent_position_update() {
        let mut agent1 = create_test_agent("Agent1", AgentType::Worker);
        let mut agent2 = create_test_agent("Agent2", AgentType::Worker);
        let mut agent3 = create_test_agent("Agent3", AgentType::Worker);

        // Set initial positions
        agent1.position = (0.0, 0.0);
        agent2.position = (10.0, 0.0);
        agent3.position = (0.0, 10.0);

        let swarm_center = (5.0, 5.0);
        let neighbors = vec![agent2.clone(), agent3.clone()];

        let initial_position = agent1.position;
        let result = agent1.update_position(swarm_center, &neighbors).await;
        assert!(result.is_ok());

        // Position should have changed due to swarm forces
        assert_ne!(agent1.position, initial_position);
    }

    #[test]
    fn test_agent_state_transitions() {
        let mut agent = create_test_agent("TestAgent", AgentType::Worker);

        // Initial state should be Idle
        assert!(matches!(agent.state, AgentState::Idle));

        // Manually test state transitions
        agent.state = AgentState::Working;
        assert!(matches!(agent.state, AgentState::Working));

        agent.state = AgentState::Learning;
        assert!(matches!(agent.state, AgentState::Learning));

        agent.state = AgentState::Communicating;
        assert!(matches!(agent.state, AgentState::Communicating));

        agent.state = AgentState::Failed;
        assert!(matches!(agent.state, AgentState::Failed));
    }

    #[test]
    fn test_agent_energy_management() {
        let mut agent = create_test_agent("TestAgent", AgentType::Worker);

        // Initial energy should be 100.0
        assert_approx_eq(agent.energy, 100.0, 0.001);

        // Test energy bounds
        agent.energy = 150.0; // Over maximum
        assert!((agent.energy - 150.0).abs() < f64::EPSILON); // No automatic clamping in basic agent

        agent.energy = -10.0; // Below minimum
        assert!((agent.energy - -10.0).abs() < f64::EPSILON); // No automatic clamping in basic agent

        // Reset to valid range
        agent.energy = 50.0;
        assert_approx_eq(agent.energy, 50.0, 0.001);
    }

    #[test]
    fn test_agent_serialization() {
        let agent = create_test_agent("TestAgent", AgentType::Specialist("AI".to_string()));

        // Test serialization to JSON
        let serialized = serde_json::to_string(&agent);
        assert!(serialized.is_ok());

        // Test deserialization from JSON
        let json_str = serialized.unwrap();
        let deserialized: Result<Agent, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let restored_agent = deserialized.unwrap();
        assert_eq!(restored_agent.name, agent.name);
        assert_eq!(restored_agent.id, agent.id);
        // Note: AgentType doesn't implement PartialEq, so we can't directly compare
    }

    #[test]
    fn test_capability_learning_rate_bounds() {
        let mut capability = create_test_capability("test", 0.5, 1.5); // Learning rate > 1.0
        assert_approx_eq(capability.learning_rate, 1.0, 0.001);

        capability = create_test_capability("test", 0.5, -0.1); // Learning rate < 0.0
        assert_approx_eq(capability.learning_rate, 0.0, 0.001);
    }

    #[test]
    fn test_agent_unique_ids() {
        let agent1 = create_test_agent("Agent1", AgentType::Worker);
        let agent2 = create_test_agent("Agent2", AgentType::Worker);

        // Each agent should have a unique ID
        assert_ne!(agent1.id, agent2.id);
    }

    #[test]
    fn test_agent_timestamps() {
        let agent = create_test_agent("TestAgent", AgentType::Worker);

        // Created and last_active timestamps should be recent
        let now = Utc::now();
        let time_diff = (now - agent.created_at).num_seconds();
        assert!(time_diff < 5); // Should be created within last 5 seconds

        let last_active_diff = (now - agent.last_active).num_seconds();
        assert!(last_active_diff < 5); // Should be active within last 5 seconds
    }
}
