//! # Hive Coordinator Usage Examples
//!
//! This file contains practical examples demonstrating how to use the HiveCoordinator
//! for managing multiagent systems, coordinating tasks, and monitoring system health.

use crate::core::hive::coordinator::{HiveCoordinator, CoordinationMessage};
use crate::utils::error::HiveResult;
use serde_json;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Example: Basic Hive Coordinator Setup
/// Description: Initialize and start a basic hive coordinator with default configuration
/// Use case: Setting up a simple multiagent system for task processing
async fn basic_coordinator_setup() -> HiveResult<()> {
    // Create a new hive coordinator
    let coordinator = HiveCoordinator::new().await?;

    // Start the coordinator and all background processes
    coordinator.start().await?;

    println!("Hive coordinator started successfully with ID: {}", coordinator.id);

    // Get initial system status
    let status = coordinator.get_status().await;
    println!("Initial system status: {}", serde_json::to_string_pretty(&status)?);

    // Gracefully shutdown
    coordinator.shutdown().await?;

    Ok(())
}

/// Example: Agent Lifecycle Management
/// Description: Create, monitor, and remove agents through the coordinator
/// Use case: Dynamic agent management in response to workload changes
async fn agent_lifecycle_management() -> HiveResult<()> {
    let coordinator = HiveCoordinator::new().await?;
    coordinator.start().await?;

    // Create multiple agents with different configurations
    let agent_configs = vec![
        serde_json::json!({
            "type": "worker",
            "name": "data_processor",
            "capabilities": ["data_processing", "analytics"]
        }),
        serde_json::json!({
            "type": "coordinator",
            "name": "task_scheduler",
            "capabilities": ["scheduling", "optimization"]
        }),
        serde_json::json!({
            "type": "specialist",
            "name": "ml_expert",
            "capabilities": ["machine_learning", "neural_networks"]
        }),
    ];

    let mut agent_ids = Vec::new();

    // Create agents
    for config in agent_configs {
        let agent_id = coordinator.create_agent(config).await?;
        agent_ids.push(agent_id);
        println!("Created agent: {}", agent_id);
    }

    // Get all agents
    let all_agents = coordinator.get_all_agents().await;
    println!("Total agents created: {}", all_agents.len());

    // Get detailed agent information
    for (id, agent) in all_agents {
        println!("Agent {}: {} ({:?})", id, agent.name, agent.agent_type);
    }

    // Remove an agent
    if let Some(agent_to_remove) = agent_ids.first() {
        coordinator.remove_agent(*agent_to_remove).await?;
        println!("Removed agent: {}", agent_to_remove);
    }

    coordinator.shutdown().await?;
    Ok(())
}

/// Example: Task Creation and Execution
/// Description: Create tasks with different priorities and execute them
/// Use case: Processing various types of computational tasks
async fn task_creation_and_execution() -> HiveResult<()> {
    let coordinator = HiveCoordinator::new().await?;
    coordinator.start().await?;

    // Create an agent to execute tasks
    let agent_config = serde_json::json!({
        "type": "worker",
        "name": "task_executor"
    });
    let agent_id = coordinator.create_agent(agent_config).await?;

    // Create tasks with different priorities
    let tasks = vec![
        serde_json::json!({
            "type": "computation",
            "title": "High Priority Calculation",
            "description": "Critical mathematical computation",
            "priority": "high",
            "required_capabilities": ["math", "computation"]
        }),
        serde_json::json!({
            "type": "data_processing",
            "title": "Medium Priority Data Processing",
            "description": "Process incoming data stream",
            "priority": "medium",
            "required_capabilities": ["data_processing"]
        }),
        serde_json::json!({
            "type": "maintenance",
            "title": "Low Priority Maintenance",
            "description": "System cleanup and optimization",
            "priority": "low",
            "required_capabilities": ["maintenance"]
        }),
    ];

    // Create and execute tasks
    for task_config in tasks {
        let task_id = coordinator.create_task(task_config).await?;
        println!("Created task: {}", task_id);

        // Execute task with verification
        let result = coordinator.execute_task_with_verification(task_id, agent_id).await?;
        println!("Task {} execution result: {}", task_id, serde_json::to_string_pretty(&result)?);
    }

    coordinator.shutdown().await?;
    Ok(())
}

/// Example: System Monitoring and Analytics
/// Description: Monitor system performance and get detailed analytics
/// Use case: Real-time monitoring and performance analysis
async fn system_monitoring_and_analytics() -> HiveResult<()> {
    let coordinator = HiveCoordinator::new().await?;
    coordinator.start().await?;

    // Create some agents and tasks for monitoring
    for i in 0..3 {
        let agent_config = serde_json::json!({
            "type": "worker",
            "name": format!("monitor_agent_{}", i)
        });
        let agent_id = coordinator.create_agent(agent_config).await?;

        // Create and execute a task for each agent
        let task_config = serde_json::json!({
            "type": "computation",
            "title": format!("Monitoring Task {}", i),
            "description": "Task for monitoring demonstration"
        });
        let task_id = coordinator.create_task(task_config).await?;
        coordinator.execute_task_with_verification(task_id, agent_id).await?;
    }

    // Get comprehensive system status
    let status = coordinator.get_status().await;
    println!("System Status: {}", serde_json::to_string_pretty(&status)?);

    // Get enhanced analytics with performance metrics
    let analytics = coordinator.get_enhanced_analytics().await;
    println!("Enhanced Analytics: {}", serde_json::to_string_pretty(&analytics)?);

    // Get specific information
    let agents_info = coordinator.get_agents_info().await;
    let tasks_info = coordinator.get_tasks_info().await?;
    let resource_info = coordinator.get_resource_info().await?;
    let memory_stats = coordinator.get_memory_stats().await?;
    let queue_health = coordinator.check_queue_health().await?;
    let agent_health = coordinator.check_agent_health();

    println!("Agents Info: {}", serde_json::to_string_pretty(&agents_info)?);
    println!("Tasks Info: {}", serde_json::to_string_pretty(&tasks_info)?);
    println!("Resource Info: {}", serde_json::to_string_pretty(&resource_info)?);
    println!("Memory Stats: {}", serde_json::to_string_pretty(&memory_stats)?);
    println!("Queue Health: {}", serde_json::to_string_pretty(&queue_health)?);
    println!("Agent Health: {}", serde_json::to_string_pretty(&agent_health)?);

    coordinator.shutdown().await?;
    Ok(())
}

/// Example: Error Handling and Recovery
/// Description: Handle various error scenarios gracefully
/// Use case: Building robust systems that can recover from failures
async fn error_handling_and_recovery() -> HiveResult<()> {
    let coordinator = HiveCoordinator::new().await?;
    coordinator.start().await?;

    // Example 1: Invalid agent configuration
    let invalid_agent_config = serde_json::json!({
        "name": "invalid_agent"
        // Missing required "type" field
    });

    match coordinator.create_agent(invalid_agent_config).await {
        Ok(agent_id) => println!("Unexpected success: {}", agent_id),
        Err(e) => println!("Expected error creating invalid agent: {}", e),
    }

    // Example 2: Invalid task configuration
    let invalid_task_config = serde_json::json!("not_an_object");

    match coordinator.create_task(invalid_task_config).await {
        Ok(task_id) => println!("Unexpected success: {}", task_id),
        Err(e) => println!("Expected error creating invalid task: {}", e),
    }

    // Example 3: Attempting to remove non-existent agent
    let fake_agent_id = Uuid::new_v4();

    match coordinator.remove_agent(fake_agent_id).await {
        Ok(()) => println!("Unexpected success removing fake agent"),
        Err(e) => println!("Expected error removing non-existent agent: {}", e),
    }

    // Example 4: Attempting to execute task with non-existent agent
    let valid_task_config = serde_json::json!({
        "type": "computation",
        "title": "Test Task"
    });
    let task_id = coordinator.create_task(valid_task_config).await?;

    match coordinator.execute_task_with_verification(task_id, fake_agent_id).await {
        Ok(result) => println!("Unexpected success: {}", serde_json::to_string(&result)?),
        Err(e) => println!("Expected error executing task with invalid agent: {}", e),
    }

    coordinator.shutdown().await?;
    Ok(())
}

/// Example: Coordination Message Processing
/// Description: Monitor and respond to internal coordination messages
/// Use case: Advanced monitoring and custom coordination logic
async fn coordination_message_processing() -> HiveResult<()> {
    let coordinator = HiveCoordinator::new().await?;
    coordinator.start().await?;

    // Create an agent to trigger coordination messages
    let agent_config = serde_json::json!({
        "type": "worker",
        "name": "coordination_test_agent"
    });
    let agent_id = coordinator.create_agent(agent_config).await?;

    // Create and execute a task to trigger more coordination messages
    let task_config = serde_json::json!({
        "type": "computation",
        "title": "Coordination Test Task"
    });
    let task_id = coordinator.create_task(task_config).await?;
    coordinator.execute_task_with_verification(task_id, agent_id).await?;

    // Give time for coordination messages to be processed
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Get updated status to see the effects of coordination
    let status = coordinator.get_status().await;
    println!("Status after coordination: {}", serde_json::to_string_pretty(&status)?);

    coordinator.shutdown().await?;
    Ok(())
}

/// Example: High-Throughput Task Processing
/// Description: Process multiple tasks concurrently with load balancing
/// Use case: High-performance computing scenarios
async fn high_throughput_task_processing() -> HiveResult<()> {
    let coordinator = HiveCoordinator::new().await?;
    coordinator.start().await?;

    // Create multiple worker agents
    let mut agent_ids = Vec::new();
    for i in 0..5 {
        let agent_config = serde_json::json!({
            "type": "worker",
            "name": format!("worker_agent_{}", i),
            "capabilities": ["computation", "data_processing"]
        });
        let agent_id = coordinator.create_agent(agent_config).await?;
        agent_ids.push(agent_id);
    }

    // Create a batch of tasks
    let mut task_ids = Vec::new();
    for i in 0..20 {
        let task_config = serde_json::json!({
            "type": "computation",
            "title": format!("Batch Task {}", i),
            "description": format!("High-throughput task {}", i),
            "priority": if i % 3 == 0 { "high" } else { "medium" }
        });
        let task_id = coordinator.create_task(task_config).await?;
        task_ids.push(task_id);
    }

    // Execute tasks concurrently
    let mut handles = Vec::new();
    for (i, task_id) in task_ids.into_iter().enumerate() {
        let coordinator_clone = coordinator.clone();
        let agent_id = agent_ids[i % agent_ids.len()]; // Round-robin agent assignment

        let handle = tokio::spawn(async move {
            match coordinator_clone.execute_task_with_verification(task_id, agent_id).await {
                Ok(result) => println!("Task {} completed successfully", task_id),
                Err(e) => println!("Task {} failed: {}", task_id, e),
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await?;
    }

    // Get final analytics
    let analytics = coordinator.get_enhanced_analytics().await;
    println!("Final Analytics: {}", serde_json::to_string_pretty(&analytics)?);

    coordinator.shutdown().await?;
    Ok(())
}

/// Example: Custom Coordination Logic
/// Description: Extend coordinator functionality with custom message handling
/// Use case: Implementing domain-specific coordination patterns
async fn custom_coordination_logic() -> HiveResult<()> {
    let coordinator = HiveCoordinator::new().await?;
    coordinator.start().await?;

    // Create a custom coordination channel for monitoring
    let (custom_tx, mut custom_rx) = mpsc::unbounded_channel();

    // Spawn a custom message handler
    let custom_handler = tokio::spawn(async move {
        while let Some(message) = custom_rx.recv().await {
            match message {
                CoordinationMessage::AgentRegistered { agent_id } => {
                    println!("Custom handler: Agent {} registered", agent_id);
                    // Custom logic: Send welcome message, update external systems, etc.
                }
                CoordinationMessage::TaskCompleted { task_id, agent_id, success } => {
                    println!("Custom handler: Task {} completed by agent {} (success: {})",
                             task_id, agent_id, success);
                    // Custom logic: Update dashboards, trigger notifications, etc.
                }
                CoordinationMessage::ResourceAlert { resource, usage } => {
                    println!("Custom handler: Resource alert - {} at {:.1}%", resource, usage * 100.0);
                    // Custom logic: Scale resources, send alerts, etc.
                }
                _ => {}
            }
        }
    });

    // Create agents and tasks to trigger messages
    for i in 0..3 {
        let agent_config = serde_json::json!({
            "type": "worker",
            "name": format!("custom_agent_{}", i)
        });
        let agent_id = coordinator.create_agent(agent_config).await?;

        let task_config = serde_json::json!({
            "type": "computation",
            "title": format!("Custom Task {}", i)
        });
        let task_id = coordinator.create_task(task_config).await?;
        coordinator.execute_task_with_verification(task_id, agent_id).await?;
    }

    // Give time for message processing
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Cleanup
    custom_handler.abort();
    coordinator.shutdown().await?;

    Ok(())
}

/// Example: Graceful Shutdown and Cleanup
/// Description: Properly shutdown the coordinator and cleanup resources
/// Use case: Application shutdown, maintenance windows
async fn graceful_shutdown_and_cleanup() -> HiveResult<()> {
    let coordinator = HiveCoordinator::new().await?;
    coordinator.start().await?;

    // Create some resources
    let agent_config = serde_json::json!({
        "type": "worker",
        "name": "shutdown_test_agent"
    });
    let agent_id = coordinator.create_agent(agent_config).await?;

    let task_config = serde_json::json!({
        "type": "computation",
        "title": "Shutdown Test Task"
    });
    let task_id = coordinator.create_task(task_config).await?;

    // Execute task
    coordinator.execute_task_with_verification(task_id, agent_id).await?;

    // Get final status before shutdown
    let final_status = coordinator.get_status().await;
    println!("Final status before shutdown: {}", serde_json::to_string_pretty(&final_status)?);

    // Graceful shutdown
    coordinator.shutdown().await?;
    println!("Coordinator shutdown completed successfully");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_coordinator_setup() {
        basic_coordinator_setup().await.unwrap();
    }

    #[tokio::test]
    async fn test_agent_lifecycle_management() {
        agent_lifecycle_management().await.unwrap();
    }

    #[tokio::test]
    async fn test_task_creation_and_execution() {
        task_creation_and_execution().await.unwrap();
    }

    #[tokio::test]
    async fn test_system_monitoring_and_analytics() {
        system_monitoring_and_analytics().await.unwrap();
    }

    #[tokio::test]
    async fn test_error_handling_and_recovery() {
        error_handling_and_recovery().await.unwrap();
    }

    #[tokio::test]
    async fn test_coordination_message_processing() {
        coordination_message_processing().await.unwrap();
    }

    #[tokio::test]
    async fn test_high_throughput_task_processing() {
        high_throughput_task_processing().await.unwrap();
    }

    #[tokio::test]
    async fn test_graceful_shutdown_and_cleanup() {
        graceful_shutdown_and_cleanup().await.unwrap();
    }
}
