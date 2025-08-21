//! Unit tests for the task system

use chrono::{Duration, Utc};
use uuid::Uuid;
use crate::tasks::{Task, TaskPriority, TaskQueue, TaskRequiredCapability, TaskResult, TaskStatus};
use crate::tests::test_utils::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "Test Task".to_string(),
            "A test task description".to_string(),
            "general".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        assert_eq!(task.title, "Test Task");
        assert_eq!(task.description, "A test task description");
        assert_eq!(task.task_type, "general");
        assert!(matches!(task.priority, TaskPriority::Medium));
        assert!(matches!(task.status, TaskStatus::Pending));
        assert!(task.assigned_agent.is_none());
        assert!(task.deadline.is_none());
        assert!(task.estimated_duration.is_none());
        assert!(task.context.is_empty());
        assert!(task.dependencies.is_empty());
        assert!(task.required_capabilities.is_empty());
    }

    #[test]
    fn test_task_with_requirements() {
        let requirements = vec![
            create_test_required_capability("data_processing", 0.8),
            create_test_required_capability("analysis", 0.6),
        ];

        let task = Task::new(
            "Complex Task".to_string(),
            "A complex task with requirements".to_string(),
            "analysis".to_string(),
            TaskPriority::High,
            requirements.clone(),
        );

        assert_eq!(task.required_capabilities.len(), 2);
        assert_eq!(task.required_capabilities[0].name, "data_processing");
        assert_approx_eq(task.required_capabilities[0].minimum_proficiency, 0.8, 0.001);
        assert_eq!(task.required_capabilities[1].name, "analysis");
        assert_approx_eq(task.required_capabilities[1].minimum_proficiency, 0.6, 0.001);
    }

    #[test]
    fn test_task_builder_pattern() {
        let deadline = Utc::now() + Duration::hours(24);
        let dependency_id = Uuid::new_v4();

        let task = Task::new(
            "Builder Task".to_string(),
            "Task using builder pattern".to_string(),
            "builder".to_string(),
            TaskPriority::Low,
            vec![],
        )
        .with_deadline(deadline)
        .with_duration(3600) // 1 hour
        .with_context("environment".to_string(), "test".to_string())
        .with_context("priority_level".to_string(), "normal".to_string())
        .with_dependencies(vec![dependency_id]);

        assert!(task.deadline.is_some());
        assert_eq!(task.deadline.unwrap(), deadline);
        assert_eq!(task.estimated_duration, Some(3600));
        assert_eq!(task.context.len(), 2);
        assert_eq!(task.context.get("environment"), Some(&"test".to_string()));
        assert_eq!(task.context.get("priority_level"), Some(&"normal".to_string()));
        assert_eq!(task.dependencies.len(), 1);
        assert_eq!(task.dependencies[0], dependency_id);
    }

    #[test]
    fn test_task_dependency_checking() {
        let completed_task_id = Uuid::new_v4();
        let incomplete_task_id = Uuid::new_v4();

        // Create completed task results
        let completed_results = vec![TaskResult::success(
            completed_task_id,
            Uuid::new_v4(),
            "Completed successfully".to_string(),
            1000,
        )];

        // Task with no dependencies should be ready
        let independent_task = create_test_task("Independent", "general", TaskPriority::Low);
        assert!(independent_task.is_ready_to_execute(&completed_results));

        // Task with satisfied dependencies should be ready
        let dependent_task = Task::new(
            "Dependent Task".to_string(),
            "Task with dependencies".to_string(),
            "dependent".to_string(),
            TaskPriority::Medium,
            vec![],
        )
        .with_dependencies(vec![completed_task_id]);
        assert!(dependent_task.is_ready_to_execute(&completed_results));

        // Task with unsatisfied dependencies should not be ready
        let blocked_task = Task::new(
            "Blocked Task".to_string(),
            "Task with unsatisfied dependencies".to_string(),
            "blocked".to_string(),
            TaskPriority::High,
            vec![],
        )
        .with_dependencies(vec![incomplete_task_id]);
        assert!(!blocked_task.is_ready_to_execute(&completed_results));

        // Task with mixed dependencies (some satisfied, some not) should not be ready
        let mixed_task = Task::new(
            "Mixed Task".to_string(),
            "Task with mixed dependencies".to_string(),
            "mixed".to_string(),
            TaskPriority::Critical,
            vec![],
        )
        .with_dependencies(vec![completed_task_id, incomplete_task_id]);
        assert!(!mixed_task.is_ready_to_execute(&completed_results));
    }

    #[test]
    fn test_task_result_creation() {
        let task_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();

        // Test successful result
        let success_result = TaskResult::success(
            task_id,
            agent_id,
            "Task completed successfully".to_string(),
            2500,
        );

        assert_eq!(success_result.task_id, task_id);
        assert_eq!(success_result.agent_id, agent_id);
        assert!(success_result.success);
        assert_eq!(success_result.output, "Task completed successfully");
        assert!(success_result.error_message.is_none());
        assert_eq!(success_result.execution_time, 2500);
        assert!(success_result.quality_score.is_none());
        assert!(success_result.learned_insights.is_empty());

        // Test failure result
        let failure_result = TaskResult::failure(
            task_id,
            agent_id,
            "Task failed due to insufficient resources".to_string(),
            1200,
        );

        assert_eq!(failure_result.task_id, task_id);
        assert_eq!(failure_result.agent_id, agent_id);
        assert!(!failure_result.success);
        assert!(failure_result.output.is_empty());
        assert_eq!(
            failure_result.error_message,
            Some("Task failed due to insufficient resources".to_string())
        );
        assert_eq!(failure_result.execution_time, 1200);
    }

    #[test]
    fn test_task_result_builder_pattern() {
        let task_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();

        let result = TaskResult::success(
            task_id,
            agent_id,
            "Great work!".to_string(),
            1500,
        )
        .with_quality_score(0.95)
        .with_insights(vec![
            "Learned efficient algorithm".to_string(),
            "Improved error handling".to_string(),
        ]);

        assert_approx_eq(result.quality_score.unwrap(), 0.95, 0.001);
        assert_eq!(result.learned_insights.len(), 2);
        assert_eq!(result.learned_insights[0], "Learned efficient algorithm");
        assert_eq!(result.learned_insights[1], "Improved error handling");
    }

    #[test]
    fn test_task_result_quality_score_bounds() {
        let task_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();

        // Test quality score clamping
        let high_score_result = TaskResult::success(task_id, agent_id, "output".to_string(), 1000)
            .with_quality_score(1.5); // Should be clamped to 1.0
        assert_approx_eq(high_score_result.quality_score.unwrap(), 1.0, 0.001);

        let low_score_result = TaskResult::success(task_id, agent_id, "output".to_string(), 1000)
            .with_quality_score(-0.5); // Should be clamped to 0.0
        assert_approx_eq(low_score_result.quality_score.unwrap(), 0.0, 0.001);
    }

    #[test]
    fn test_task_queue_creation() {
        let queue = TaskQueue::new();

        assert_eq!(queue.get_pending_count(), 0);
        assert_eq!(queue.get_assigned_count(), 0);
        assert_eq!(queue.get_completed_count(), 0);
        assert_eq!(queue.get_failed_count(), 0);
    }

    #[test]
    fn test_task_queue_priority_ordering() {
        let mut queue = TaskQueue::new();

        // Add tasks in non-priority order
        let low_task = create_test_task("Low Priority", "general", TaskPriority::Low);
        let critical_task = create_test_task("Critical", "urgent", TaskPriority::Critical);
        let medium_task = create_test_task("Medium Priority", "normal", TaskPriority::Medium);
        let high_task = create_test_task("High Priority", "important", TaskPriority::High);

        queue.add_task(low_task.clone());
        queue.add_task(critical_task.clone());
        queue.add_task(medium_task.clone());
        queue.add_task(high_task.clone());

        // Tasks should be retrieved in priority order: Critical, High, Medium, Low
        let first = queue.get_next_task().unwrap();
        assert_eq!(first.title, "Critical");

        let second = queue.get_next_task().unwrap();
        assert_eq!(second.title, "High Priority");

        let third = queue.get_next_task().unwrap();
        assert_eq!(third.title, "Medium Priority");

        let fourth = queue.get_next_task().unwrap();
        assert_eq!(fourth.title, "Low Priority");

        // Queue should be empty now
        assert!(queue.get_next_task().is_none());
    }

    #[test]
    fn test_task_queue_assignment() {
        let mut queue = TaskQueue::new();
        let agent_id = Uuid::new_v4();

        let task = create_test_task("Test Task", "general", TaskPriority::Medium);
        let task_id = task.id;

        queue.assign_task(task, agent_id);

        assert_eq!(queue.get_assigned_count(), 1);
        assert_eq!(queue.get_pending_count(), 0);

        let assigned_task = queue.get_task_by_agent(&agent_id).unwrap();
        assert_eq!(assigned_task.id, task_id);
        assert_eq!(assigned_task.assigned_agent, Some(agent_id));
        assert!(matches!(assigned_task.status, TaskStatus::Assigned));
    }

    #[test]
    fn test_task_queue_completion() {
        let mut queue = TaskQueue::new();
        let agent_id = Uuid::new_v4();
        let task_id = Uuid::new_v4();

        // First assign a task
        let task = create_test_task("Test Task", "general", TaskPriority::Medium);
        queue.assign_task(task, agent_id);

        // Then complete it successfully
        let success_result = TaskResult::success(
            task_id,
            agent_id,
            "Completed successfully".to_string(),
            1000,
        );

        queue.complete_task(success_result);

        assert_eq!(queue.get_assigned_count(), 0);
        assert_eq!(queue.get_completed_count(), 1);
        assert_eq!(queue.get_failed_count(), 0);
    }

    #[test]
    fn test_task_queue_failure() {
        let mut queue = TaskQueue::new();
        let agent_id = Uuid::new_v4();
        let task_id = Uuid::new_v4();

        // First assign a task
        let task = create_test_task("Test Task", "general", TaskPriority::Medium);
        queue.assign_task(task, agent_id);

        // Then fail it
        let failure_result = TaskResult::failure(
            task_id,
            agent_id,
            "Task failed".to_string(),
            500,
        );

        queue.complete_task(failure_result);

        assert_eq!(queue.get_assigned_count(), 0);
        assert_eq!(queue.get_completed_count(), 0);
        assert_eq!(queue.get_failed_count(), 1);
    }

    #[test]
    fn test_task_queue_suitable_tasks_filtering() {
        let queue = TaskQueue::new();

        // Create agent capabilities
        let agent_capabilities = vec![
            create_test_capability("data_processing", 0.8, 0.1),
            create_test_capability("analysis", 0.6, 0.1),
            create_test_capability("communication", 0.7, 0.1),
        ];

        // This should be empty since we haven't added any tasks
        let suitable_tasks = queue.find_suitable_tasks(&agent_capabilities);
        assert!(suitable_tasks.is_empty());
    }

    #[test]
    fn test_task_queue_suitable_tasks_with_tasks() {
        let mut queue = TaskQueue::new();

        // Add tasks with different requirements
        let easy_task = create_test_task("Easy Task", "general", TaskPriority::Low);
        
        let medium_task = create_test_task_with_requirements(
            "Medium Task",
            "analysis",
            TaskPriority::Medium,
            vec![create_test_required_capability("data_processing", 0.5)],
        );
        
        let hard_task = create_test_task_with_requirements(
            "Hard Task",
            "expert",
            TaskPriority::High,
            vec![
                create_test_required_capability("data_processing", 0.9),
                create_test_required_capability("analysis", 0.8),
            ],
        );

        queue.add_task(easy_task);
        queue.add_task(medium_task);
        queue.add_task(hard_task);

        // Create agent capabilities
        let agent_capabilities = vec![
            create_test_capability("data_processing", 0.8, 0.1),
            create_test_capability("analysis", 0.6, 0.1),
        ];

        let suitable_tasks = queue.find_suitable_tasks(&agent_capabilities);
        
        // Should find easy task (no requirements) and medium task (requirements met)
        // Should NOT find hard task (analysis requirement too high: 0.8 > 0.6)
        assert_eq!(suitable_tasks.len(), 2);
        
        let task_titles: Vec<&str> = suitable_tasks.iter().map(|t| t.title.as_str()).collect();
        assert!(task_titles.contains(&"Easy Task"));
        assert!(task_titles.contains(&"Medium Task"));
        assert!(!task_titles.contains(&"Hard Task"));
    }

    #[test]
    fn test_task_priority_enum() {
        // Test that priority enum variants exist
        let _low = TaskPriority::Low;
        let _medium = TaskPriority::Medium;
        let _high = TaskPriority::High;
        let _critical = TaskPriority::Critical;
    }

    #[test]
    fn test_task_status_enum() {
        // Test that status enum variants exist and can be compared
        assert_eq!(TaskStatus::Pending, TaskStatus::Pending);
        assert_ne!(TaskStatus::Pending, TaskStatus::Assigned);
        
        let _pending = TaskStatus::Pending;
        let _assigned = TaskStatus::Assigned;
        let _in_progress = TaskStatus::InProgress;
        let _completed = TaskStatus::Completed;
        let _failed = TaskStatus::Failed;
        let _cancelled = TaskStatus::Cancelled;
    }

    #[test]
    fn test_task_serialization() {
        let task = create_test_task_with_requirements(
            "Serialization Test",
            "test",
            TaskPriority::High,
            vec![create_test_required_capability("test_capability", 0.7)],
        );

        // Test serialization to JSON
        let serialized = serde_json::to_string(&task);
        assert!(serialized.is_ok());

        // Test deserialization from JSON
        let json_str = serialized.unwrap();
        let deserialized: Result<Task, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let restored_task = deserialized.unwrap();
        assert_eq!(restored_task.title, task.title);
        assert_eq!(restored_task.description, task.description);
        assert_eq!(restored_task.task_type, task.task_type);
        assert_eq!(restored_task.id, task.id);
    }

    #[test]
    fn test_task_result_serialization() {
        let result = TaskResult::success(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Test output".to_string(),
            1500,
        )
        .with_quality_score(0.85)
        .with_insights(vec!["Test insight".to_string()]);

        // Test serialization to JSON
        let serialized = serde_json::to_string(&result);
        assert!(serialized.is_ok());

        // Test deserialization from JSON
        let json_str = serialized.unwrap();
        let deserialized: Result<TaskResult, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let restored_result = deserialized.unwrap();
        assert_eq!(restored_result.task_id, result.task_id);
        assert_eq!(restored_result.agent_id, result.agent_id);
        assert_eq!(restored_result.success, result.success);
        assert_eq!(restored_result.output, result.output);
        assert_approx_eq(
            restored_result.quality_score.unwrap(),
            result.quality_score.unwrap(),
            0.001
        );
    }

    #[test]
    fn test_task_unique_ids() {
        let task1 = create_test_task("Task 1", "general", TaskPriority::Low);
        let task2 = create_test_task("Task 2", "general", TaskPriority::Low);

        // Each task should have a unique ID
        assert_ne!(task1.id, task2.id);
    }

    #[test]
    fn test_task_timestamps() {
        let task = create_test_task("Timestamp Test", "general", TaskPriority::Medium);

        // Created and updated timestamps should be recent and equal initially
        let now = Utc::now();
        let created_diff = (now - task.created_at).num_seconds();
        let updated_diff = (now - task.updated_at).num_seconds();
        
        assert!(created_diff < 5); // Should be created within last 5 seconds
        assert!(updated_diff < 5); // Should be updated within last 5 seconds
        assert_eq!(task.created_at, task.updated_at); // Should be equal initially
    }

    #[test]
    fn test_required_capability_bounds() {
        let req_cap_high = TaskRequiredCapability {
            name: "test".to_string(),
            minimum_proficiency: 1.5, // No automatic clamping in basic struct
        };
        assert_eq!(req_cap_high.minimum_proficiency, 1.5);

        let req_cap_low = TaskRequiredCapability {
            name: "test".to_string(),
            minimum_proficiency: -0.5, // No automatic clamping in basic struct
        };
        assert_eq!(req_cap_low.minimum_proficiency, -0.5);

        // Test with clamped values using helper function
        let req_cap_clamped = create_test_required_capability("test", 1.5);
        assert_approx_eq(req_cap_clamped.minimum_proficiency, 1.0, 0.001);
    }
}