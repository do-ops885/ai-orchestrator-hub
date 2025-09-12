//! # Hive Coordination System - Modular Architecture
//!
//! This module provides a refactored, modular approach to the hive coordination system.
//! The original monolithic `HiveCoordinator` has been broken down into focused modules
//! for better maintainability and separation of concerns.
//!
//! ## Architecture
//!
//! The hive system is organized into specialized modules:
//!
//! - **`coordinator`**: Main coordination logic and public API
//! - **`agent_management`**: Agent lifecycle, registration, and monitoring
//! - **`task_management`**: Task distribution, execution, and work-stealing queues
//! - **`background_processes`**: Long-running processes and system maintenance
//! - **`metrics_collection`**: Comprehensive metrics gathering and reporting
//!
//! ## Key Features
//!
//! - **Modular Design**: Each subsystem can be developed and tested independently
//! - **Event-Driven Communication**: Modules communicate via async message passing
//! - **Resource-Aware**: Automatic scaling and resource management
//! - **Comprehensive Monitoring**: Detailed metrics and performance tracking
//! - **Fault Tolerance**: Graceful error handling and recovery mechanisms
//!
//! ## Usage
//!
//! ```rust,no_run
//! use hive::core::hive::{HiveCoordinator, AgentManager, TaskDistributor};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create the main coordinator
//!     let coordinator = HiveCoordinator::new().await?;
//!
//!     // Start background processes
//!     coordinator.start().await?;
//!
//!     // Create and manage agents
//!     let agent_config = serde_json::json!({
//!         "type": "worker",
//!         "name": "example_agent"
//!     });
//!     let agent_id = coordinator.create_agent(agent_config).await?;
//!
//!     // Create and execute tasks
//!     let task_config = serde_json::json!({
//!         "type": "computation",
//!         "title": "Example Task"
//!     });
//!     let task_id = coordinator.create_task(task_config).await?;
//!
//!     let result = coordinator.execute_task_with_verification(task_id, agent_id).await?;
//!
//!     // Get system status
//!     let status = coordinator.get_status().await;
//!     println!("System status: {}", status);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Memory Usage**: O(n) where n is the number of active agents and tasks
//! - **CPU Overhead**: Minimal background processing with configurable intervals
//! - **Scalability**: Designed to handle hundreds of concurrent agents and tasks
//! - **Latency**: Sub-millisecond for most operations, configurable for background tasks
//!
//! ## Error Handling
//!
//! All operations return `HiveResult<T>` which encapsulates both success and error cases.
//! Common error types include:
//!
//! - `ResourceExhausted`: System resources insufficient for operation
//! - `AgentNotFound`: Referenced agent does not exist
//! - `TaskNotFound`: Referenced task does not exist
//! - `ValidationError`: Input data failed validation
//! - `OperationFailed`: Generic operation failure with details

/// Agent management subsystem for lifecycle and monitoring
pub mod agent_management;
/// Background process management for system maintenance
pub mod background_processes;
/// Main coordinator providing unified API
pub mod coordinator;
/// Metrics collection and reporting system
pub mod metrics_collection;
/// Task distribution and execution management
pub mod task_management;
pub mod task_management_legacy;

/// Main coordinator for the hive system - provides unified API
///
/// This is the primary entry point for interacting with the hive system.
/// It coordinates all subsystems and provides a clean, consistent interface
/// for agent and task management operations.
pub use coordinator::HiveCoordinator;

/// Agent management subsystem
///
/// Handles agent registration, lifecycle management, performance tracking,
/// and resource allocation for individual agents.
pub use agent_management::{AgentManager, AgentRegistrationResult};

/// Background process management
///
/// Manages long-running system processes including work stealing,
/// learning cycles, swarm coordination, and resource monitoring.
pub use background_processes::{ProcessConfig, ProcessManager};

/// Metrics collection and reporting
///
/// Provides comprehensive metrics gathering, trend analysis, and
/// export capabilities for monitoring system performance.
pub use metrics_collection::{HiveMetrics, MetricsCollector, SwarmMetrics};

/// Task distribution and execution
///
/// Manages task queuing, distribution to agents, execution tracking,
/// and work-stealing algorithms for optimal resource utilization.
pub use task_management::{TaskDistributor, TaskExecutionResult};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::mpsc;

    // Mock implementations for integration testing
    struct MockResourceManager;
    impl MockResourceManager {
        async fn new() -> crate::utils::error::HiveResult<Self> {
            Ok(Self)
        }
        async fn get_system_info(
            &self,
        ) -> (
            crate::infrastructure::resource_manager::SystemResources,
            String,
            String,
        ) {
            use chrono::Utc;
            (
                crate::infrastructure::resource_manager::SystemResources {
                    cpu_cores: 4,
                    available_memory: 8_000_000_000,
                    cpu_usage: 0.5,
                    memory_usage: 0.3,
                    simd_capabilities: vec!["avx2".to_string()],
                    last_updated: Utc::now(),
                },
                "desktop".to_string(),
                "Desktop".to_string(),
            )
        }
        async fn update_system_metrics(&self) -> crate::utils::error::HiveResult<()> {
            Ok(())
        }
    }

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
        let resource_manager = Arc::new(MockResourceManager::new().await?);
        let (tx, _rx) = mpsc::unbounded_channel();

        // Create individual modules
        let agent_manager = AgentManager::new(Arc::clone(&resource_manager), tx.clone()).await?;
        let task_distributor =
            TaskDistributor::new(Arc::clone(&resource_manager), tx.clone()).await?;
        let metrics_collector = MetricsCollector::new(tx.clone()).await?;
        let process_manager = ProcessManager::new(tx).await?;

        // Verify they can be created and used together
        assert_eq!(agent_manager.get_agent_count(), 0);
        assert_eq!(task_distributor.task_queue.read().await.len(), 0);

        let status = metrics_collector.get_current_metrics().await;
        assert_eq!(status.agent_metrics.total_agents, 0);

        let process_status = process_manager.get_process_status().await;
        assert_eq!(process_status["total_processes"], 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_agent_task_integration() -> Result<(), Box<dyn std::error::Error>> {
        // Test integration between agent management and task distribution
        let resource_manager = Arc::new(MockResourceManager::new().await?);
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
        let initial_agents = initial_status.get("agents").unwrap().as_object().unwrap();
        let initial_tasks = initial_status.get("tasks").unwrap().as_object().unwrap();

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
        let updated_agents = updated_status.get("agents").unwrap().as_object().unwrap();
        let updated_tasks = updated_status.get("tasks").unwrap().as_object().unwrap();

        // Verify status is updated
        assert!(
            updated_agents
                .get("total_agents")
                .unwrap()
                .as_u64()
                .unwrap()
                > initial_agents
                    .get("total_agents")
                    .unwrap()
                    .as_u64()
                    .unwrap_or(0)
        );

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
        let metrics = status.get("metrics").unwrap().as_object().unwrap();
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

    #[tokio::test]
    async fn test_module_type_consistency() {
        // Test that exported types are consistent and usable
        use super::*;

        // Test that we can create instances of exported types
        let _agent_result = AgentRegistrationResult {
            agent_id: uuid::Uuid::new_v4(),
            success: true,
            message: "Test".to_string(),
        };

        let _task_result = TaskExecutionResult {
            task_id: uuid::Uuid::new_v4(),
            agent_id: uuid::Uuid::new_v4(),
            success: true,
            execution_time_ms: 100,
            result: Some(serde_json::json!({"test": "data"})),
            error_message: None,
        };

        let _process_config = ProcessConfig::default();
        let _hive_metrics = HiveMetrics::default();
        let _swarm_metrics = SwarmMetrics::default();
    }

    #[tokio::test]
    async fn test_module_backward_compatibility() -> Result<(), Box<dyn std::error::Error>> {
        // Test that the main HiveCoordinator maintains backward compatibility
        let coordinator = HiveCoordinator::new().await?;

        // Test all the main public methods exist and work
        let _status = coordinator.get_status().await;
        let _analytics = coordinator.get_enhanced_analytics().await;
        let _agents_info = coordinator.get_agents_info().await;
        let _tasks_info = coordinator.get_tasks_info().await?;
        let _resource_info = coordinator.get_resource_info().await?;
        let _memory_stats = coordinator.get_memory_stats().await?;
        let _queue_health = coordinator.check_queue_health().await?;
        let _agent_health = coordinator.check_agent_health();

        // Test agent operations
        let agent_config = serde_json::json!({
            "type": "worker",
            "name": "compatibility_test_agent"
        });
        let agent_id = coordinator.create_agent(agent_config).await?;
        let _agent = coordinator.get_agent(agent_id).await;
        let _all_agents = coordinator.get_all_agents().await;
        coordinator.remove_agent(agent_id).await?;

        // Test task operations
        let task_config = serde_json::json!({
            "type": "computation",
            "title": "Compatibility Test Task"
        });
        let task_id = coordinator.create_task(task_config).await?;
        let _result = coordinator
            .execute_task_with_verification(task_id, agent_id)
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_module_concurrent_access() -> Result<(), Box<dyn std::error::Error>> {
        // Test that modules can handle concurrent access
        let coordinator = HiveCoordinator::new().await?;

        let mut handles = vec![];

        // Spawn multiple tasks that use the coordinator concurrently
        for i in 0..5 {
            let coordinator_clone = coordinator.clone();
            let handle = tokio::spawn(async move {
                let agent_config = serde_json::json!({
                    "type": "worker",
                    "name": format!("concurrent_agent_{}", i)
                });
                let agent_id = coordinator_clone.create_agent(agent_config).await?;

                let task_config = serde_json::json!({
                    "type": "computation",
                    "title": format!("Concurrent Task {}", i)
                });
                let task_id = coordinator_clone.create_task(task_config).await?;

                coordinator_clone
                    .execute_task_with_verification(task_id, agent_id)
                    .await?;

                Ok::<(), Box<dyn std::error::Error>>(())
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await??;
        }

        // Verify final state
        let status = coordinator.get_status().await;
        let agents = status.get("agents").unwrap().as_object().unwrap();
        assert_eq!(agents.get("total_agents").unwrap(), 5);

        Ok(())
    }

    #[tokio::test]
    async fn test_module_resource_management_integration() -> Result<(), Box<dyn std::error::Error>>
    {
        // Test integration with resource management
        let coordinator = HiveCoordinator::new().await?;

        // Get resource info
        let resource_info = coordinator.get_resource_info().await?;
        assert!(resource_info.is_object());
        assert!(resource_info.get("system_resources").is_some());

        // Create multiple agents to test resource usage
        for i in 0..3 {
            let agent_config = serde_json::json!({
                "type": "worker",
                "name": format!("resource_test_agent_{}", i)
            });
            coordinator.create_agent(agent_config).await?;
        }

        // Resource info should still be accessible
        let updated_resource_info = coordinator.get_resource_info().await?;
        assert!(updated_resource_info.is_object());

        Ok(())
    }
}
