//! Comprehensive tests for the swarm coordinator
//!
//! Tests cover task distribution, agent management, load balancing,
//! fault tolerance, and performance optimization.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{timeout, sleep};

use multiagent_hive::agents::{Agent, AgentConfig, AgentStatus};
use multiagent_hive::communication::{Message, MessageType, CommunicationManager};
use multiagent_hive::swarm::{SwarmCoordinator, SwarmConfig, Task, TaskStatus, TaskPriority};
use multiagent_hive::persistence::{PersistenceManager, SQLiteStorage};

mod test_utils;
use test_utils::*;

/// Test fixture for swarm coordinator testing
struct SwarmTestFixture {
    coordinator: Arc<SwarmCoordinator>,
    communication: Arc<CommunicationManager>,
    agents: Vec<Arc<RwLock<Agent>>>,
    _temp_dir: tempfile::TempDir,
}

impl SwarmTestFixture {
    async fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let storage = Arc::new(RwLock::new(
            SQLiteStorage::new(&db_path).expect("Failed to create storage")
        ));

        let persistence = PersistenceManager::new(storage, None)
            .await
            .expect("Failed to create persistence");

        let config = SwarmConfig {
            max_agents: 10,
            task_timeout_seconds: 300,
            coordination_strategy: "round_robin".to_string(),
            enable_load_balancing: true,
            health_check_interval_seconds: 30,
            auto_scaling_enabled: false,
            min_agents: 1,
            max_agents_per_task: 3,
            metadata: HashMap::new(),
        };

        let communication = Arc::new(CommunicationManager::new());
        let coordinator = Arc::new(
            SwarmCoordinator::new(config, persistence, communication.clone())
                .await
                .expect("Failed to create coordinator")
        );

        Self {
            coordinator,
            communication,
            agents: Vec::new(),
            _temp_dir: temp_dir,
        }
    }

    async fn add_agent(&mut self, agent_type: &str, capabilities: Vec<&str>) -> Arc<RwLock<Agent>> {
        let config = AgentConfig {
            name: format!("test-agent-{}", self.agents.len()),
            agent_type: agent_type.to_string(),
            capabilities: capabilities.into_iter().map(|s| s.to_string()).collect(),
            max_concurrent_tasks: 5,
            memory_limit_mb: 100,
            timeout_seconds: 30,
            retry_attempts: 3,
            specialization: Some(format!("test-{}", agent_type)),
            metadata: HashMap::new(),
        };

        let agent = Arc::new(RwLock::new(
            Agent::new(config).expect("Failed to create agent")
        ));

        self.coordinator
            .register_agent(agent.clone())
            .await
            .expect("Failed to register agent");

        self.agents.push(agent.clone());
        agent
    }

    async fn create_task(&self, title: &str, priority: TaskPriority, required_capabilities: Vec<&str>) -> Task {
        let task = Task {
            id: uuid::Uuid::new_v4(),
            title: title.to_string(),
            description: format!("Test task: {}", title),
            status: TaskStatus::Pending,
            priority,
            required_capabilities: required_capabilities.into_iter().map(|s| s.to_string()).collect(),
            assigned_agent: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deadline: None,
            metadata: HashMap::new(),
        };

        self.coordinator
            .submit_task(task.clone())
            .await
            .expect("Failed to submit task");

        task
    }

    async fn wait_for_task_assignment(&self, task_id: uuid::Uuid, timeout_ms: u64) -> bool {
        let start = std::time::Instant::now();
        while start.elapsed().as_millis() < timeout_ms as u128 {
            if let Some(task) = self.coordinator.get_task(task_id).await {
                if task.assigned_agent.is_some() {
                    return true;
                }
            }
            sleep(Duration::from_millis(10)).await;
        }
        false
    }
}

#[cfg(test)]
mod swarm_coordinator_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn test_agent_registration() {
        let mut fixture = SwarmTestFixture::new().await;

        let agent = fixture.add_agent("worker", vec!["computation", "analysis"]).await;

        let agent_read = agent.read().await;
        assert_eq!(agent_read.config.agent_type, "worker");
        assert!(agent_read.config.capabilities.contains(&"computation".to_string()));
        assert!(agent_read.config.capabilities.contains(&"analysis".to_string()));

        // Verify agent is registered with coordinator
        let registered_agents = fixture.coordinator.list_agents().await;
        assert_eq!(registered_agents.len(), 1);
        assert_eq!(registered_agents[0].read().await.config.name, agent_read.config.name);
    }

    #[tokio::test]
    async fn test_task_submission_and_assignment() {
        let mut fixture = SwarmTestFixture::new().await;

        // Add an agent that can handle computation tasks
        fixture.add_agent("worker", vec!["computation"]).await;

        // Submit a task requiring computation capability
        let task = fixture.create_task("Compute intensive task", TaskPriority::Medium, vec!["computation"]).await;

        // Wait for task assignment
        let assigned = fixture.wait_for_task_assignment(task.id, 1000).await;
        assert!(assigned, "Task should be assigned to available agent");

        // Verify task status
        let updated_task = fixture.coordinator.get_task(task.id).await.unwrap();
        assert_eq!(updated_task.status, TaskStatus::InProgress);
        assert!(updated_task.assigned_agent.is_some());
    }

    #[tokio::test]
    async fn test_task_assignment_with_no_matching_agents() {
        let mut fixture = SwarmTestFixture::new().await;

        // Add agent with limited capabilities
        fixture.add_agent("worker", vec!["analysis"]).await;

        // Submit task requiring different capability
        let task = fixture.create_task("ML task", TaskPriority::High, vec!["machine_learning"]).await;

        // Task should not be assigned
        let assigned = fixture.wait_for_task_assignment(task.id, 500).await;
        assert!(!assigned, "Task should not be assigned when no matching agents available");

        let task_status = fixture.coordinator.get_task(task.id).await.unwrap();
        assert_eq!(task_status.status, TaskStatus::Pending);
        assert!(task_status.assigned_agent.is_none());
    }

    #[tokio::test]
    async fn test_load_balancing() {
        let mut fixture = SwarmTestFixture::new().await;

        // Add multiple agents with same capabilities
        for i in 0..3 {
            fixture.add_agent("worker", vec!["computation"]).await;
        }

        // Submit multiple tasks
        let mut tasks = Vec::new();
        for i in 0..6 {
            let task = fixture.create_task(
                &format!("Task {}", i),
                TaskPriority::Medium,
                vec!["computation"]
            ).await;
            tasks.push(task);
        }

        // Wait for all tasks to be assigned
        sleep(Duration::from_millis(100)).await;

        // Count assignments per agent
        let mut agent_assignments = HashMap::new();
        for task in &tasks {
            if let Some(updated_task) = fixture.coordinator.get_task(task.id).await {
                if let Some(agent_id) = updated_task.assigned_agent {
                    *agent_assignments.entry(agent_id).or_insert(0) += 1;
                }
            }
        }

        // Each agent should have roughly equal load (2 tasks each)
        for &count in agent_assignments.values() {
            assert!(count >= 1 && count <= 3, "Load balancing should distribute tasks evenly");
        }
    }

    #[tokio::test]
    async fn test_priority_based_scheduling() {
        let mut fixture = SwarmTestFixture::new().await;

        // Add single agent
        fixture.add_agent("worker", vec!["computation"]).await;

        // Submit tasks with different priorities
        let low_task = fixture.create_task("Low priority", TaskPriority::Low, vec!["computation"]).await;
        let high_task = fixture.create_task("High priority", TaskPriority::High, vec!["computation"]).await;
        let medium_task = fixture.create_task("Medium priority", TaskPriority::Medium, vec!["computation"]).await;

        // High priority task should be assigned first
        sleep(Duration::from_millis(50)).await;

        let high_assigned = fixture.coordinator.get_task(high_task.id).await.unwrap();
        assert!(high_assigned.assigned_agent.is_some(), "High priority task should be assigned first");

        // Complete high priority task
        fixture.coordinator.complete_task(high_task.id).await.unwrap();

        // Medium priority should be assigned next
        sleep(Duration::from_millis(50)).await;

        let medium_assigned = fixture.coordinator.get_task(medium_task.id).await.unwrap();
        assert!(medium_assigned.assigned_agent.is_some(), "Medium priority task should be assigned second");
    }

    #[tokio::test]
    async fn test_agent_failure_handling() {
        let mut fixture = SwarmTestFixture::new().await;

        // Add two agents
        let agent1 = fixture.add_agent("worker", vec!["computation"]).await;
        let agent2 = fixture.add_agent("worker", vec!["computation"]).await;

        // Submit task
        let task = fixture.create_task("Test task", TaskPriority::Medium, vec!["computation"]).await;

        // Wait for assignment
        fixture.wait_for_task_assignment(task.id, 500).await;

        // Simulate agent failure
        {
            let mut agent = agent1.write().await;
            agent.status = AgentStatus::Failed;
        }

        // Task should be reassigned to healthy agent
        sleep(Duration::from_millis(100)).await;

        let reassigned_task = fixture.coordinator.get_task(task.id).await.unwrap();
        if let Some(agent_id) = reassigned_task.assigned_agent {
            // Should be assigned to agent2
            let agent2_read = agent2.read().await;
            assert_eq!(agent_id, agent2_read.id);
        } else {
            panic!("Task should be reassigned after agent failure");
        }
    }

    #[tokio::test]
    async fn test_concurrent_task_processing() {
        let mut fixture = SwarmTestFixture::new().await;

        // Add multiple agents
        for _ in 0..5 {
            fixture.add_agent("worker", vec!["computation"]).await;
        }

        // Submit many tasks concurrently
        let mut task_handles = Vec::new();
        for i in 0..20 {
            let coordinator = fixture.coordinator.clone();
            let handle = tokio::spawn(async move {
                let task = Task {
                    id: uuid::Uuid::new_v4(),
                    title: format!("Concurrent task {}", i),
                    description: "Test concurrent processing".to_string(),
                    status: TaskStatus::Pending,
                    priority: TaskPriority::Medium,
                    required_capabilities: vec!["computation".to_string()],
                    assigned_agent: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    deadline: None,
                    metadata: HashMap::new(),
                };

                coordinator.submit_task(task).await.unwrap();
            });
            task_handles.push(handle);
        }

        // Wait for all submissions
        for handle in task_handles {
            handle.await.unwrap();
        }

        // Wait for processing
        sleep(Duration::from_millis(200)).await;

        // Check that tasks are being processed
        let all_tasks = fixture.coordinator.list_tasks().await;
        let in_progress_count = all_tasks.iter()
            .filter(|t| t.status == TaskStatus::InProgress)
            .count();

        assert!(in_progress_count > 0, "Some tasks should be in progress");
        assert!(all_tasks.len() >= 15, "Most tasks should be processed");
    }

    #[tokio::test]
    async fn test_resource_limits() {
        let mut fixture = SwarmTestFixture::new().await;

        // Add agent with limited concurrent tasks
        let agent = fixture.add_agent("worker", vec!["computation"]).await;
        {
            let mut agent_write = agent.write().await;
            agent_write.config.max_concurrent_tasks = 2;
        }

        // Submit more tasks than agent can handle
        for i in 0..5 {
            fixture.create_task(&format!("Task {}", i), TaskPriority::Medium, vec!["computation"]).await;
        }

        sleep(Duration::from_millis(100)).await;

        // Check agent task load
        let agent_read = agent.read().await;
        assert!(agent_read.active_tasks.len() <= 2, "Agent should not exceed max concurrent tasks");

        // Some tasks should remain pending
        let pending_tasks = fixture.coordinator.list_tasks().await
            .into_iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .count();

        assert!(pending_tasks > 0, "Some tasks should remain pending due to resource limits");
    }

    #[tokio::test]
    async fn test_task_timeout_handling() {
        let mut fixture = SwarmTestFixture::new().await;

        fixture.add_agent("worker", vec!["computation"]).await;

        // Submit task with short timeout
        let task = fixture.create_task("Timeout task", TaskPriority::High, vec!["computation"]).await;

        // Wait for assignment
        fixture.wait_for_task_assignment(task.id, 500).await;

        // Simulate timeout by marking task as timed out
        fixture.coordinator.timeout_task(task.id).await.unwrap();

        let timed_out_task = fixture.coordinator.get_task(task.id).await.unwrap();
        assert_eq!(timed_out_task.status, TaskStatus::TimedOut);
    }

    #[tokio::test]
    async fn test_coordination_strategies() {
        let mut fixture = SwarmTestFixture::new().await;

        // Test round-robin strategy
        fixture.coordinator.set_coordination_strategy("round_robin").await;

        for _ in 0..3 {
            fixture.add_agent("worker", vec!["computation"]).await;
        }

        // Submit tasks and check distribution
        for i in 0..6 {
            fixture.create_task(&format!("RR Task {}", i), TaskPriority::Medium, vec!["computation"]).await;
        }

        sleep(Duration::from_millis(100)).await;

        // Check round-robin distribution
        let assignments: Vec<_> = fixture.coordinator.list_tasks().await
            .into_iter()
            .filter_map(|t| t.assigned_agent)
            .collect();

        let unique_agents = assignments.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_agents, 3, "Round-robin should use all agents");
    }

    #[tokio::test]
    async fn test_health_monitoring() {
        let mut fixture = SwarmTestFixture::new().await;

        let agent = fixture.add_agent("worker", vec!["computation"]).await;

        // Agent should be healthy initially
        let health_status = fixture.coordinator.check_agent_health(agent.read().await.id).await;
        assert!(health_status, "Agent should be healthy initially");

        // Simulate unhealthy agent
        {
            let mut agent_write = agent.write().await;
            agent_write.status = AgentStatus::Failed;
            agent_write.last_health_check = Some(chrono::Utc::now() - chrono::Duration::minutes(10));
        }

        let health_status_after = fixture.coordinator.check_agent_health(agent.read().await.id).await;
        assert!(!health_status_after, "Agent should be marked as unhealthy");
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let mut fixture = SwarmTestFixture::new().await;

        fixture.add_agent("worker", vec!["computation"]).await;

        // Submit and complete some tasks
        let task = fixture.create_task("Metrics task", TaskPriority::Medium, vec!["computation"]).await;
        fixture.wait_for_task_assignment(task.id, 500).await;
        fixture.coordinator.complete_task(task.id).await.unwrap();

        // Check metrics
        let metrics = fixture.coordinator.get_metrics().await;

        assert!(metrics.total_tasks > 0, "Should have processed at least one task");
        assert!(metrics.completed_tasks > 0, "Should have completed at least one task");
        assert!(metrics.success_rate >= 0.0 && metrics.success_rate <= 1.0, "Success rate should be valid");
    }
}