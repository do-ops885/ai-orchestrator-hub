//! # Task Management Usage Examples
//!
//! This file contains practical examples demonstrating how to use the TaskDistributor
//! for task creation, execution, monitoring, and optimization.

use crate::core::hive::task_management::{TaskDistributor, TaskExecutionResult};
use crate::infrastructure::resource_manager::ResourceManager;
use crate::utils::error::HiveResult;
use crate::tasks::task::{TaskPriority, TaskRequiredCapability};
use serde_json;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Example: Basic Task Creation and Execution
/// Description: Create and execute basic tasks with different configurations
/// Use case: Simple task processing workflows
async fn basic_task_creation_and_execution() -> HiveResult<()> {
    // Initialize dependencies
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let (tx, _rx) = mpsc::unbounded_channel();
    let task_distributor = TaskDistributor::new(resource_manager, tx).await?;

    // Create tasks with different configurations
    let task_configs = vec![
        serde_json::json!({
            "type": "computation",
            "title": "Simple Calculation",
            "description": "Basic mathematical computation",
            "priority": "medium"
        }),
        serde_json::json!({
            "type": "data_processing",
            "title": "Data Transformation",
            "description": "Transform incoming data",
            "priority": "high"
        }),
        serde_json::json!({
            "type": "analysis",
            "title": "Data Analysis",
            "description": "Analyze processed data",
            "priority": "low"
        }),
    ];

    let mut task_ids = Vec::new();

    // Create tasks
    for config in task_configs {
        let task_id = task_distributor.create_task(config).await?;
        task_ids.push(task_id);
        println!("Created task: {}", task_id);
    }

    // Execute tasks with mock agent IDs
    for task_id in task_ids {
        let mock_agent_id = Uuid::new_v4();
        let result = task_distributor.execute_task_with_verification(task_id, mock_agent_id).await?;
        println!("Task {} execution result: {}", task_id, serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}

/// Example: Task Prioritization and Scheduling
/// Description: Create tasks with different priorities and observe execution order
/// Use case: Managing task queues with priority-based scheduling
async fn task_prioritization_and_scheduling() -> HiveResult<()> {
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let (tx, _rx) = mpsc::unbounded_channel();
    let task_distributor = TaskDistributor::new(resource_manager, tx).await?;

    // Create tasks with different priorities
    let priority_tasks = vec![
        ("Critical System Update", "critical", 1),
        ("High Priority Analysis", "high", 2),
        ("Medium Priority Processing", "medium", 3),
        ("Low Priority Cleanup", "low", 4),
        ("Another Critical Task", "critical", 5),
    ];

    let mut task_ids = Vec::new();

    for (title, priority, id) in priority_tasks {
        let config = serde_json::json!({
            "type": "computation",
            "title": title,
            "description": format!("Task {} with {} priority", id, priority),
            "priority": priority
        });
        let task_id = task_distributor.create_task(config).await?;
        task_ids.push((task_id, priority.to_string()));
        println!("Created {} priority task: {}", priority, task_id);
    }

    // Get current status to see task distribution
    let status = task_distributor.get_status().await;
    println!("Task Status: {}", serde_json::to_string_pretty(&status)?);

    // Execute tasks (in a real scenario, these would be picked up by agents)
    for (task_id, priority) in task_ids {
        let mock_agent_id = Uuid::new_v4();
        println!("Executing {} priority task: {}", priority, task_id);
        let result = task_distributor.execute_task_with_verification(task_id, mock_agent_id).await?;
        println!("  Result: {}", serde_json::to_string(&result)?);
    }

    Ok(())
}

/// Example: Task Execution with Required Capabilities
/// Description: Create tasks that require specific agent capabilities
/// Use case: Matching tasks to agents with appropriate skills
async fn task_execution_with_capabilities() -> HiveResult<()> {
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let (tx, _rx) = mpsc::unbounded_channel();
    let task_distributor = TaskDistributor::new(resource_manager, tx).await?;

    // Create tasks requiring different capabilities
    let capability_tasks = vec![
        serde_json::json!({
            "type": "machine_learning",
            "title": "ML Model Training",
            "description": "Train a neural network model",
            "required_capabilities": [
                {
                    "name": "machine_learning",
                    "minimum_proficiency": 0.8
                },
                {
                    "name": "gpu_computing",
                    "minimum_proficiency": 0.7
                }
            ]
        }),
        serde_json::json!({
            "type": "data_analysis",
            "title": "Statistical Analysis",
            "description": "Perform statistical analysis on dataset",
            "required_capabilities": [
                {
                    "name": "statistics",
                    "minimum_proficiency": 0.6
                },
                {
                    "name": "data_analysis",
                    "minimum_proficiency": 0.7
                }
            ]
        }),
        serde_json::json!({
            "type": "computation",
            "title": "Parallel Processing",
            "description": "Execute parallel computational tasks",
            "required_capabilities": [
                {
                    "name": "parallel_computing",
                    "minimum_proficiency": 0.5
                }
            ]
        }),
    ];

    let mut task_ids = Vec::new();

    for config in capability_tasks {
        let task_id = task_distributor.create_task(config.clone()).await?;
        task_ids.push(task_id);

        let title = config.get("title").unwrap().as_str().unwrap();
        println!("Created task requiring capabilities: {} ({})", title, task_id);
    }

    // Execute tasks (in practice, these would be matched to capable agents)
    for task_id in task_ids {
        let mock_agent_id = Uuid::new_v4();
        let result = task_distributor.execute_task_with_verification(task_id, mock_agent_id).await?;
        println!("Task {} completed: {}", task_id, result.get("result").unwrap());
    }

    Ok(())
}

/// Example: Task Execution Monitoring and Analytics
/// Description: Monitor task execution and analyze performance metrics
/// Use case: Performance monitoring and optimization
async fn task_execution_monitoring_and_analytics() -> HiveResult<()> {
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let (tx, _rx) = mpsc::unbounded_channel();
    let task_distributor = TaskDistributor::new(resource_manager, tx).await?;

    // Create and execute multiple tasks to generate analytics data
    let mut task_ids = Vec::new();
    for i in 0..10 {
        let config = serde_json::json!({
            "type": "computation",
            "title": format!("Analytics Task {}", i),
            "description": format!("Task for analytics demonstration {}", i),
            "priority": if i % 3 == 0 { "high" } else { "medium" }
        });
        let task_id = task_distributor.create_task(config).await?;
        task_ids.push(task_id);
    }

    // Execute tasks with varying performance characteristics
    for (i, task_id) in task_ids.into_iter().enumerate() {
        let mock_agent_id = Uuid::new_v4();

        // Simulate different execution times and success rates
        let result = task_distributor.execute_task_with_verification(task_id, mock_agent_id).await?;
        println!("Task {} result: {}", task_id, serde_json::to_string(&result)?);
    }

    // Get comprehensive analytics
    let analytics = task_distributor.get_analytics().await;
    println!("Task Analytics: {}", serde_json::to_string_pretty(&analytics)?);

    // Get current status
    let status = task_distributor.get_status().await;
    println!("Current Status: {}", serde_json::to_string_pretty(&status)?);

    Ok(())
}

/// Example: Batch Task Processing
/// Description: Process multiple tasks in batches for efficiency
/// Use case: High-throughput task processing scenarios
async fn batch_task_processing() -> HiveResult<()> {
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let (tx, _rx) = mpsc::unbounded_channel();
    let task_distributor = TaskDistributor::new(resource_manager, tx).await?;

    // Create a batch of tasks
    let batch_size = 20;
    let mut task_ids = Vec::new();

    println!("Creating batch of {} tasks...", batch_size);
    for i in 0..batch_size {
        let config = serde_json::json!({
            "type": "computation",
            "title": format!("Batch Task {}", i),
            "description": format!("Task {} in processing batch", i),
            "priority": "medium"
        });
        let task_id = task_distributor.create_task(config).await?;
        task_ids.push(task_id);
    }

    // Process tasks in parallel (simulated with async execution)
    println!("Processing {} tasks concurrently...", task_ids.len());
    let mut handles = Vec::new();

    for task_id in task_ids {
        let task_distributor_clone = task_distributor.clone();
        let handle = tokio::spawn(async move {
            let mock_agent_id = Uuid::new_v4();
            task_distributor_clone.execute_task_with_verification(task_id, mock_agent_id).await
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut completed = 0;
    let mut failed = 0;

    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => completed += 1,
            Ok(Err(_)) => failed += 1,
            Err(_) => failed += 1,
        }
    }

    println!("Batch processing completed: {} successful, {} failed", completed, failed);

    // Get final analytics
    let analytics = task_distributor.get_analytics().await;
    println!("Batch Processing Analytics: {}", serde_json::to_string_pretty(&analytics)?);

    Ok(())
}

/// Example: Error Handling in Task Management
/// Description: Handle various error scenarios in task processing
/// Use case: Building robust task processing systems
async fn error_handling_in_task_management() -> HiveResult<()> {
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let (tx, _rx) = mpsc::unbounded_channel();
    let task_distributor = TaskDistributor::new(resource_manager, tx).await?;

    // Test 1: Invalid task configuration
    println!("Test 1: Invalid task configuration");
    let invalid_config = serde_json::json!("not_an_object");

    match task_distributor.create_task(invalid_config).await {
        Ok(id) => println!("Unexpected success: {}", id),
        Err(e) => println!("Expected error: {}", e),
    }

    // Test 2: Missing required fields
    println!("Test 2: Missing required fields");
    let incomplete_config = serde_json::json!({
        "title": "Incomplete Task"
    });

    match task_distributor.create_task(incomplete_config).await {
        Ok(id) => println!("Unexpected success: {}", id),
        Err(e) => println!("Expected error: {}", e),
    }

    // Test 3: Executing non-existent task
    println!("Test 3: Executing non-existent task");
    let fake_task_id = Uuid::new_v4();
    let mock_agent_id = Uuid::new_v4();

    match task_distributor.execute_task_with_verification(fake_task_id, mock_agent_id).await {
        Ok(result) => println!("Unexpected success: {}", serde_json::to_string(&result)?),
        Err(e) => println!("Expected error: {}", e),
    }

    // Test 4: Valid task creation and execution
    println!("Test 4: Valid task creation and execution");
    let valid_config = serde_json::json!({
        "type": "computation",
        "title": "Valid Test Task",
        "description": "A properly configured task"
    });

    let task_id = task_distributor.create_task(valid_config).await?;
    println!("Successfully created task: {}", task_id);

    let result = task_distributor.execute_task_with_verification(task_id, mock_agent_id).await?;
    println!("Successfully executed task: {}", serde_json::to_string(&result)?);

    Ok(())
}

/// Example: Task Distribution to Multiple Agents
/// Description: Distribute tasks among multiple agents
/// Use case: Load balancing and parallel processing
async fn task_distribution_to_multiple_agents() -> HiveResult<()> {
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let (tx, _rx) = mpsc::unbounded_channel();
    let task_distributor = TaskDistributor::new(resource_manager, tx).await?;

    // Create multiple tasks
    let mut task_ids = Vec::new();
    for i in 0..15 {
        let config = serde_json::json!({
            "type": "computation",
            "title": format!("Distributed Task {}", i),
            "description": format!("Task for distribution test {}", i),
            "priority": if i % 5 == 0 { "high" } else { "medium" }
        });
        let task_id = task_distributor.create_task(config).await?;
        task_ids.push(task_id);
    }

    // Simulate multiple agents
    let agent_ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();

    // Distribute tasks among agents (round-robin for demonstration)
    for (i, task_id) in task_ids.into_iter().enumerate() {
        let agent_id = agent_ids[i % agent_ids.len()];
        println!("Assigning task {} to agent {}", task_id, agent_id);

        let result = task_distributor.execute_task_with_verification(task_id, agent_id).await?;
        println!("  Task completed by agent {}: {}", agent_id, result.get("result").unwrap());
    }

    // Get distribution analytics
    let analytics = task_distributor.get_analytics().await;
    let task_distribution = analytics.get("task_distribution").unwrap();
    println!("Task Distribution Analytics: {}", serde_json::to_string_pretty(task_distribution)?);

    Ok(())
}

/// Example: Task Performance Benchmarking
/// Description: Benchmark task execution performance under different conditions
/// Use case: Performance testing and optimization
async fn task_performance_benchmarking() -> HiveResult<()> {
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let (tx, _rx) = mpsc::unbounded_channel();
    let task_distributor = TaskDistributor::new(resource_manager, tx).await?;

    // Define benchmark scenarios
    let scenarios = vec![
        ("light_tasks", 10, 50),    // 10 light tasks, 50ms each
        ("medium_tasks", 5, 150),   // 5 medium tasks, 150ms each
        ("heavy_tasks", 3, 300),    // 3 heavy tasks, 300ms each
    ];

    println!("Running Task Performance Benchmarks:");

    for (scenario_name, task_count, base_time) in scenarios {
        println!("Benchmark: {}", scenario_name);

        let start_time = std::time::Instant::now();
        let mut task_ids = Vec::new();

        // Create tasks for this scenario
        for i in 0..task_count {
            let config = serde_json::json!({
                "type": "computation",
                "title": format!("{} Task {}", scenario_name, i),
                "description": format!("Benchmark task {} in scenario {}", i, scenario_name)
            });
            let task_id = task_distributor.create_task(config).await?;
            task_ids.push(task_id);
        }

        // Execute tasks sequentially (for accurate timing)
        for task_id in task_ids {
            let mock_agent_id = Uuid::new_v4();
            task_distributor.execute_task_with_verification(task_id, mock_agent_id).await?;
        }

        let duration = start_time.elapsed();
        println!("  Completed {} tasks in {:.2}s", task_count, duration.as_secs_f64());
        println!("  Average time per task: {:.2}ms", duration.as_millis() as f64 / task_count as f64);
    }

    // Get final performance analytics
    let analytics = task_distributor.get_analytics().await;
    println!("Benchmark Results: {}", serde_json::to_string_pretty(&analytics)?);

    Ok(())
}

/// Example: Task Queue Management
/// Description: Manage task queues and monitor queue health
/// Use case: Queue monitoring and maintenance
async fn task_queue_management() -> HiveResult<()> {
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let (tx, _rx) = mpsc::unbounded_channel();
    let task_distributor = TaskDistributor::new(resource_manager, tx).await?;

    // Create tasks to populate the queue
    for i in 0..25 {
        let config = serde_json::json!({
            "type": "computation",
            "title": format!("Queue Task {}", i),
            "description": format!("Task for queue management demo {}", i),
            "priority": match i % 4 {
                0 => "critical",
                1 => "high",
                2 => "medium",
                _ => "low",
            }
        });
        task_distributor.create_task(config).await?;
    }

    // Monitor queue status
    let initial_status = task_distributor.get_status().await;
    println!("Initial Queue Status: {}", serde_json::to_string_pretty(&initial_status)?);

    // Process some tasks
    let tasks_to_process = 10;
    for i in 0..tasks_to_process {
        // In a real scenario, tasks would be picked up by agents
        // Here we simulate by checking status
        let status = task_distributor.get_status().await;
        println!("Processing task {} - Queue size: {}",
                 i + 1,
                 status.get("legacy_queue_size").unwrap());
    }

    // Get final queue analytics
    let final_analytics = task_distributor.get_analytics().await;
    println!("Final Queue Analytics: {}", serde_json::to_string_pretty(&final_analytics)?);

    Ok(())
}

/// Example: Task Failure Recovery
/// Description: Handle task failures and implement retry logic
/// Use case: Building resilient task processing systems
async fn task_failure_recovery() -> HiveResult<()> {
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let (tx, _rx) = mpsc::unbounded_channel();
    let task_distributor = TaskDistributor::new(resource_manager, tx).await?;

    // Create tasks that might fail
    let task_configs = vec![
        serde_json::json!({
            "type": "computation",
            "title": "Reliable Task",
            "description": "Task that should succeed",
            "priority": "high"
        }),
        serde_json::json!({
            "type": "computation",
            "title": "Unreliable Task",
            "description": "Task that might fail",
            "priority": "medium"
        }),
        serde_json::json!({
            "type": "computation",
            "title": "Critical Task",
            "description": "Important task that needs to succeed",
            "priority": "critical"
        }),
    ];

    let mut task_ids = Vec::new();
    for config in task_configs {
        let task_id = task_distributor.create_task(config).await?;
        task_ids.push(task_id);
    }

    // Execute tasks with simulated retry logic
    for task_id in task_ids {
        let mock_agent_id = Uuid::new_v4();
        let mut attempts = 0;
        let max_attempts = 3;

        loop {
            attempts += 1;
            println!("Executing task {} (attempt {})", task_id, attempts);

            let result = task_distributor.execute_task_with_verification(task_id, mock_agent_id).await;

            match result {
                Ok(success_result) => {
                    println!("  Task {} succeeded on attempt {}", task_id, attempts);
                    break;
                }
                Err(e) => {
                    if attempts >= max_attempts {
                        println!("  Task {} failed after {} attempts: {}", task_id, attempts, e);
                        break;
                    } else {
                        println!("  Task {} failed on attempt {}, retrying...", task_id, attempts);
                        // In a real scenario, you might wait or try a different agent
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            }
        }
    }

    // Get final analytics including failure rates
    let analytics = task_distributor.get_analytics().await;
    println!("Recovery Analytics: {}", serde_json::to_string_pretty(&analytics)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_task_creation_and_execution() {
        basic_task_creation_and_execution().await.unwrap();
    }

    #[tokio::test]
    async fn test_task_prioritization_and_scheduling() {
        task_prioritization_and_scheduling().await.unwrap();
    }

    #[tokio::test]
    async fn test_task_execution_with_capabilities() {
        task_execution_with_capabilities().await.unwrap();
    }

    #[tokio::test]
    async fn test_task_execution_monitoring_and_analytics() {
        task_execution_monitoring_and_analytics().await.unwrap();
    }

    #[tokio::test]
    async fn test_batch_task_processing() {
        batch_task_processing().await.unwrap();
    }

    #[tokio::test]
    async fn test_error_handling_in_task_management() {
        error_handling_in_task_management().await.unwrap();
    }

    #[tokio::test]
    async fn test_task_distribution_to_multiple_agents() {
        task_distribution_to_multiple_agents().await.unwrap();
    }

    #[tokio::test]
    async fn test_task_performance_benchmarking() {
        task_performance_benchmarking().await.unwrap();
    }

    #[tokio::test]
    async fn test_task_queue_management() {
        task_queue_management().await.unwrap();
    }
}
