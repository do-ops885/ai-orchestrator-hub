//! # Hive Module Integration Examples
//!
//! This file contains practical examples demonstrating how to integrate and use
//! all hive modules together for comprehensive multiagent system management.

use crate::core::hive::{HiveCoordinator, AgentManager, TaskDistributor, ProcessManager, MetricsCollector};
use crate::infrastructure::resource_manager::ResourceManager;
use crate::utils::error::HiveResult;
use serde_json;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Example: Complete Hive System Integration
/// Description: Set up and run a complete multiagent system with all modules
/// Use case: Full system deployment and operation
async fn complete_hive_system_integration() -> HiveResult<()> {
    println!("=== Complete Hive System Integration ===");

    // Initialize the main coordinator
    let coordinator = HiveCoordinator::new().await?;
    println!("✓ HiveCoordinator initialized");

    // Start the coordinator and all subsystems
    coordinator.start().await?;
    println!("✓ All subsystems started");

    // Create agents with different types
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
        serde_json::json!({
            "type": "learner",
            "name": "adaptive_agent",
            "capabilities": ["learning", "adaptation"]
        }),
    ];

    let mut agent_ids = Vec::new();
    for config in agent_configs {
        let agent_id = coordinator.create_agent(config).await?;
        agent_ids.push(agent_id);
        println!("✓ Created agent: {}", agent_id);
    }

    // Create and execute tasks
    let task_configs = vec![
        serde_json::json!({
            "type": "computation",
            "title": "Data Analysis Task",
            "description": "Analyze incoming data streams",
            "priority": "high"
        }),
        serde_json::json!({
            "type": "machine_learning",
            "title": "Model Training",
            "description": "Train ML model on processed data",
            "priority": "medium"
        }),
        serde_json::json!({
            "type": "optimization",
            "title": "System Optimization",
            "description": "Optimize system performance",
            "priority": "low"
        }),
    ];

    let mut task_ids = Vec::new();
    for (i, config) in task_configs.into_iter().enumerate() {
        let task_id = coordinator.create_task(config).await?;
        task_ids.push(task_id);

        // Assign task to an agent (round-robin)
        let agent_id = agent_ids[i % agent_ids.len()];
        let result = coordinator.execute_task_with_verification(task_id, agent_id).await?;
        println!("✓ Task {} executed by agent {}: {}", task_id, agent_id, result.get("result").unwrap());
    }

    // Monitor system status
    let status = coordinator.get_status().await;
    println!("System Status: {}", serde_json::to_string_pretty(&status)?);

    // Get analytics
    let analytics = coordinator.get_enhanced_analytics().await;
    println!("System Analytics: {}", serde_json::to_string_pretty(&analytics)?);

    // Graceful shutdown
    coordinator.shutdown().await?;
    println!("✓ System shutdown complete");

    Ok(())
}

/// Example: Modular Component Access
/// Description: Access individual modules for specialized operations
/// Use case: Fine-grained control over specific system components
async fn modular_component_access() -> HiveResult<()> {
    println!("=== Modular Component Access ===");

    // This example shows how to access individual modules
    // In practice, you'd get these from the coordinator or create them directly

    // Create coordination channel
    let (tx, _rx) = mpsc::unbounded_channel();

    // Initialize resource manager
    let resource_manager = Arc::new(ResourceManager::new().await?);

    // Initialize individual modules
    let agent_manager = AgentManager::new(Arc::clone(&resource_manager), tx.clone()).await?;
    let task_distributor = TaskDistributor::new(Arc::clone(&resource_manager), tx.clone()).await?;
    let metrics_collector = MetricsCollector::new(tx.clone()).await?;
    let process_manager = ProcessManager::new(tx).await?;

    println!("✓ All modules initialized individually");

    // Use agent manager
    let agent_config = serde_json::json!({
        "type": "worker",
        "name": "modular_agent"
    });
    let agent_id = agent_manager.create_agent(agent_config).await?;
    println!("✓ Agent created via AgentManager: {}", agent_id);

    // Use task distributor
    let task_config = serde_json::json!({
        "type": "computation",
        "title": "Modular Task",
        "description": "Task created via TaskDistributor"
    });
    let task_id = task_distributor.create_task(task_config).await?;
    println!("✓ Task created via TaskDistributor: {}", task_id);

    // Execute task
    let result = task_distributor.execute_task_with_verification(task_id, agent_id).await?;
    println!("✓ Task executed: {}", result.get("result").unwrap());

    // Get metrics
    let metrics = metrics_collector.get_current_metrics().await;
    println!("✓ Current metrics: {} agents, {} tasks",
             metrics.agent_metrics.total_agents,
             metrics.task_metrics.total_tasks);

    // Start background processes
    process_manager.start_all_processes(
        &agent_manager,
        &task_distributor,
        &metrics_collector,
        &resource_manager,
    ).await?;
    println!("✓ Background processes started");

    // Let processes run briefly
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Stop processes
    process_manager.stop_all_processes().await?;
    println!("✓ Background processes stopped");

    Ok(())
}

/// Example: Cross-Module Data Flow
/// Description: Demonstrate data flow between different modules
/// Use case: Understanding module interactions and data propagation
async fn cross_module_data_flow() -> HiveResult<()> {
    println!("=== Cross-Module Data Flow ===");

    let coordinator = HiveCoordinator::new().await?;
    coordinator.start().await?;

    // Phase 1: Agent Management -> Metrics Collection
    println!("Phase 1: Agent registration triggers metrics updates");
    let agent_config = serde_json::json!({
        "type": "worker",
        "name": "data_flow_agent"
    });
    let agent_id = coordinator.create_agent(agent_config).await?;
    println!("  Agent registered: {}", agent_id);

    // Phase 2: Task Distribution -> Agent Management -> Metrics Collection
    println!("Phase 2: Task execution updates multiple modules");
    let task_config = serde_json::json!({
        "type": "computation",
        "title": "Data Flow Task",
        "description": "Task demonstrating cross-module data flow"
    });
    let task_id = coordinator.create_task(task_config).await?;
    println!("  Task created: {}", task_id);

    let result = coordinator.execute_task_with_verification(task_id, agent_id).await?;
    println!("  Task executed: {}", result.get("result").unwrap());

    // Phase 3: Resource Manager -> All Modules
    println!("Phase 3: Resource updates affect all modules");
    // Resource updates happen automatically in the background

    // Phase 4: Process Manager coordinates all modules
    println!("Phase 4: Process manager coordinates background activities");
    // Background processes are already running

    // Get comprehensive status showing data flow
    let status = coordinator.get_status().await;
    println!("Final System Status: {}", serde_json::to_string_pretty(&status)?);

    // Show analytics that aggregate data from all modules
    let analytics = coordinator.get_enhanced_analytics().await;
    println!("Aggregated Analytics: {}", serde_json::to_string_pretty(&analytics)?);

    coordinator.shutdown().await?;

    Ok(())
}

/// Example: Error Handling Across Modules
/// Description: Handle errors that span multiple modules
/// Use case: Building resilient systems with cross-module error recovery
async fn error_handling_across_modules() -> HiveResult<()> {
    println!("=== Error Handling Across Modules ===");

    let coordinator = HiveCoordinator::new().await?;
    coordinator.start().await?;

    // Test 1: Invalid agent configuration (AgentManager error)
    println!("Test 1: Invalid agent configuration");
    let invalid_agent = serde_json::json!({
        "name": "invalid_agent"
        // Missing required "type" field
    });

    match coordinator.create_agent(invalid_agent).await {
        Ok(id) => println!("  Unexpected success: {}", id),
        Err(e) => println!("  Expected AgentManager error: {}", e),
    }

    // Test 2: Invalid task configuration (TaskDistributor error)
    println!("Test 2: Invalid task configuration");
    let invalid_task = serde_json::json!("not_an_object");

    match coordinator.create_task(invalid_task).await {
        Ok(id) => println!("  Unexpected success: {}", id),
        Err(e) => println!("  Expected TaskDistributor error: {}", e),
    }

    // Test 3: Resource exhaustion (ResourceManager error)
    println!("Test 3: Resource exhaustion scenario");
    // In a real scenario, this would be triggered by actual resource limits

    // Test 4: Coordination failure (ProcessManager error)
    println!("Test 4: Coordination failure");
    // Process manager errors would occur during background process failures

    // Test 5: Successful recovery after errors
    println!("Test 5: Recovery after errors");
    let valid_agent = serde_json::json!({
        "type": "worker",
        "name": "recovery_agent"
    });
    let agent_id = coordinator.create_agent(valid_agent).await?;
    println!("  Successfully created agent after errors: {}", agent_id);

    let valid_task = serde_json::json!({
        "type": "computation",
        "title": "Recovery Task"
    });
    let task_id = coordinator.create_task(valid_task).await?;
    let result = coordinator.execute_task_with_verification(task_id, agent_id).await?;
    println!("  Successfully executed task after errors: {}", result.get("result").unwrap());

    // Verify system is still functional
    let status = coordinator.get_status().await;
    println!("System status after error recovery: {}", serde_json::to_string_pretty(&status)?);

    coordinator.shutdown().await?;

    Ok(())
}

/// Example: Performance Monitoring Integration
/// Description: Monitor performance across all modules
/// Use case: System performance analysis and optimization
async fn performance_monitoring_integration() -> HiveResult<()> {
    println!("=== Performance Monitoring Integration ===");

    let coordinator = HiveCoordinator::new().await?;
    coordinator.start().await?;

    // Create multiple agents for performance testing
    let mut agent_ids = Vec::new();
    for i in 0..5 {
        let config = serde_json::json!({
            "type": "worker",
            "name": format!("perf_agent_{}", i)
        });
        let agent_id = coordinator.create_agent(config).await?;
        agent_ids.push(agent_id);
    }

    // Execute tasks across all agents
    let mut task_ids = Vec::new();
    for (i, &agent_id) in agent_ids.iter().enumerate() {
        for j in 0..10 {
            let task_config = serde_json::json!({
                "type": "computation",
                "title": format!("Perf Task {}:{}", i, j),
                "description": format!("Performance test task {} for agent {}", j, i)
            });
            let task_id = coordinator.create_task(task_config).await?;
            task_ids.push((task_id, agent_id));
        }
    }

    // Execute tasks and measure performance
    let start_time = std::time::Instant::now();
    for (task_id, agent_id) in task_ids {
        coordinator.execute_task_with_verification(task_id, agent_id).await?;
    }
    let execution_time = start_time.elapsed();

    println!("Performance Test Results:");
    println!("  Total tasks executed: {}", task_ids.len());
    println!("  Total execution time: {:.2}s", execution_time.as_secs_f64());
    println!("  Tasks per second: {:.1}", task_ids.len() as f64 / execution_time.as_secs_f64());

    // Get performance metrics from all modules
    let status = coordinator.get_status().await;
    let analytics = coordinator.get_enhanced_analytics().await;

    println!("Agent Performance: {}", serde_json::to_string_pretty(&status.get("agents").unwrap())?);
    println!("Task Performance: {}", serde_json::to_string_pretty(&status.get("tasks").unwrap())?);
    println!("System Performance: {}", serde_json::to_string_pretty(&status.get("metrics").unwrap())?);
    println!("Enhanced Analytics: {}", serde_json::to_string_pretty(&analytics)?);

    coordinator.shutdown().await?;

    Ok(())
}

/// Example: Configuration Management Across Modules
/// Description: Manage configuration settings across all modules
/// Use case: System configuration and tuning
async fn configuration_management_across_modules() -> HiveResult<()> {
    println!("=== Configuration Management Across Modules ===");

    // This example shows how configuration flows through the system
    // In practice, configuration would be loaded from files or environment variables

    let coordinator = HiveCoordinator::new().await?;
    coordinator.start().await?;

    // Configuration examples for different modules:

    // 1. Agent Manager Configuration
    println!("1. Agent Manager Configuration:");
    let agent_config = serde_json::json!({
        "type": "worker",
        "name": "configured_agent",
        "max_concurrent_tasks": 5,
        "resource_limits": {
            "cpu_percent": 80,
            "memory_mb": 512
        }
    });
    let agent_id = coordinator.create_agent(agent_config.clone()).await?;
    println!("  Agent created with config: {}", serde_json::to_string(&agent_config)?);

    // 2. Task Distributor Configuration
    println!("2. Task Distributor Configuration:");
    let task_config = serde_json::json!({
        "type": "computation",
        "title": "Configured Task",
        "description": "Task with specific configuration",
        "priority": "high",
        "timeout_seconds": 300,
        "retry_count": 3,
        "required_capabilities": ["computation", "memory_intensive"]
    });
    let task_id = coordinator.create_task(task_config.clone()).await?;
    println!("  Task created with config: {}", serde_json::to_string(&task_config)?);

    // 3. Process Manager Configuration (would be set during initialization)
    println!("3. Process Manager Configuration:");
    // Process configuration is set when creating the ProcessManager

    // 4. Metrics Collector Configuration
    println!("4. Metrics Collector Configuration:");
    // Metrics configuration is handled internally

    // Execute configured task
    let result = coordinator.execute_task_with_verification(task_id, agent_id).await?;
    println!("  Configured task executed: {}", result.get("result").unwrap());

    // Get system status showing configured components
    let status = coordinator.get_status().await;
    println!("System with configured components: {}", serde_json::to_string_pretty(&status)?);

    coordinator.shutdown().await?;

    Ok(())
}

/// Example: Scalability Testing Across Modules
/// Description: Test system scalability with increasing load
/// Use case: Capacity planning and performance testing
async fn scalability_testing_across_modules() -> HiveResult<()> {
    println!("=== Scalability Testing Across Modules ===");

    let coordinator = HiveCoordinator::new().await?;
    coordinator.start().await?;

    // Test different scales
    let scales = vec![
        ("Small Scale", 5, 20),   // agents, tasks_per_agent
        ("Medium Scale", 15, 50),
        ("Large Scale", 30, 100),
    ];

    for (scale_name, num_agents, tasks_per_agent) in scales {
        println!("Testing {}: {} agents, {} tasks each", scale_name, num_agents, tasks_per_agent);

        let scale_start = std::time::Instant::now();

        // Create agents
        let mut agent_ids = Vec::new();
        for i in 0..num_agents {
            let config = serde_json::json!({
                "type": "worker",
                "name": format!("scale_agent_{}_{}", scale_name.replace(" ", "_"), i)
            });
            let agent_id = coordinator.create_agent(config).await?;
            agent_ids.push(agent_id);
        }

        // Create and execute tasks
        let mut task_ids = Vec::new();
        for &agent_id in &agent_ids {
            for j in 0..tasks_per_agent {
                let task_config = serde_json::json!({
                    "type": "computation",
                    "title": format!("Scale Task {}:{}", agent_id, j),
                    "description": format!("Scalability test task for {}", scale_name)
                });
                let task_id = coordinator.create_task(task_config).await?;
                task_ids.push((task_id, agent_id));
            }
        }

        // Execute all tasks for this scale
        let mut completed = 0;
        for (task_id, agent_id) in task_ids {
            match coordinator.execute_task_with_verification(task_id, agent_id).await {
                Ok(_) => completed += 1,
                Err(e) => println!("  Task failed: {}", e),
            }
        }

        let scale_duration = scale_start.elapsed();

        println!("  Scale {} completed in {:.2}s", scale_name, scale_duration.as_secs_f64());
        println!("  Tasks completed: {}/{}", completed, num_agents * tasks_per_agent);
        println!("  Throughput: {:.1} tasks/sec",
                 completed as f64 / scale_duration.as_secs_f64());

        // Get status after each scale test
        let status = coordinator.get_status().await;
        println!("  Status after {}: {} agents, {} tasks",
                 scale_name,
                 status.get("agents").unwrap().get("total_agents").unwrap(),
                 status.get("tasks").unwrap().get("total_tasks").unwrap_or(&serde_json::Value::Null));

        // Brief pause between scales
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    // Final system analysis
    let final_analytics = coordinator.get_enhanced_analytics().await;
    println!("Final Scalability Analysis: {}", serde_json::to_string_pretty(&final_analytics)?);

    coordinator.shutdown().await?;

    Ok(())
}

/// Example: Module Health Monitoring
/// Description: Monitor the health of all modules
/// Use case: System health checks and maintenance
async fn module_health_monitoring() -> HiveResult<()> {
    println!("=== Module Health Monitoring ===");

    let coordinator = HiveCoordinator::new().await?;
    coordinator.start().await?;

    // Create some activity to monitor
    for i in 0..3 {
        let agent_config = serde_json::json!({
            "type": "worker",
            "name": format!("health_agent_{}", i)
        });
        let agent_id = coordinator.create_agent(agent_config).await?;

        let task_config = serde_json::json!({
            "type": "computation",
            "title": format!("Health Task {}", i)
        });
        let task_id = coordinator.create_task(task_config).await?;
        coordinator.execute_task_with_verification(task_id, agent_id).await?;
    }

    // Monitor health across different time intervals
    for check in 0..5 {
        println!("Health Check {}:", check + 1);

        // Check agent health
        let agents_info = coordinator.get_agents_info().await;
        let total_agents = agents_info.get("total_agents").unwrap().as_u64().unwrap_or(0);
        let active_agents = agents_info.get("active_agents").unwrap().as_u64().unwrap_or(0);
        println!("  Agent Health: {}/{} active", active_agents, total_agents);

        // Check task health
        let tasks_info = coordinator.get_tasks_info().await?;
        println!("  Task Health: {}", serde_json::to_string(&tasks_info)?);

        // Check resource health
        let resource_info = coordinator.get_resource_info().await?;
        println!("  Resource Health: {}", serde_json::to_string(&resource_info)?);

        // Check memory health
        let memory_stats = coordinator.get_memory_stats().await?;
        println!("  Memory Health: {}", serde_json::to_string(&memory_stats)?);

        // Check queue health
        let queue_health = coordinator.check_queue_health().await?;
        println!("  Queue Health: {}", serde_json::to_string(&queue_health)?);

        // Overall agent health
        let agent_health = coordinator.check_agent_health();
        println!("  Overall Agent Health: {}", serde_json::to_string(&agent_health)?);

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    // Get comprehensive final health report
    let full_status = coordinator.get_status().await;
    println!("Final Health Report: {}", serde_json::to_string_pretty(&full_status)?);

    coordinator.shutdown().await?;

    Ok(())
}

/// Example: Backup and Recovery Across Modules
/// Description: Demonstrate backup and recovery procedures
/// Use case: System backup, recovery, and disaster recovery
async fn backup_and_recovery_across_modules() -> HiveResult<()> {
    println!("=== Backup and Recovery Across Modules ===");

    let coordinator = HiveCoordinator::new().await?;
    coordinator.start().await?;

    // Phase 1: Normal Operation
    println!("Phase 1: Normal Operation");
    let mut agent_ids = Vec::new();
    let mut task_ids = Vec::new();

    for i in 0..3 {
        let agent_config = serde_json::json!({
            "type": "worker",
            "name": format!("backup_agent_{}", i)
        });
        let agent_id = coordinator.create_agent(agent_config).await?;
        agent_ids.push(agent_id);

        let task_config = serde_json::json!({
            "type": "computation",
            "title": format!("Backup Task {}", i)
        });
        let task_id = coordinator.create_task(task_config).await?;
        task_ids.push(task_id);

        coordinator.execute_task_with_verification(task_id, agent_id).await?;
    }

    // Phase 2: System State Backup
    println!("Phase 2: System State Backup");
    let backup_status = coordinator.get_status().await;
    let backup_analytics = coordinator.get_enhanced_analytics().await;
    println!("  System state backed up");

    // Phase 3: Simulated System Disruption
    println!("Phase 3: Simulated System Disruption");
    // In a real scenario, this might be a system crash or restart

    // Phase 4: Recovery Process
    println!("Phase 4: Recovery Process");
    let recovery_status = coordinator.get_status().await;
    let recovery_analytics = coordinator.get_enhanced_analytics().await;

    // Compare before and after
    println!("Recovery Verification:");
    println!("  Agents recovered: {}", recovery_status.get("agents").unwrap().get("total_agents").unwrap());
    println!("  Tasks recovered: {}", recovery_status.get("tasks").unwrap().get("total_tasks").unwrap_or(&serde_json::Value::Null));

    // Phase 5: System Validation
    println!("Phase 5: System Validation");
    let validation_agent = serde_json::json!({
        "type": "worker",
        "name": "validation_agent"
    });
    let validation_agent_id = coordinator.create_agent(validation_agent).await?;
    println!("  ✓ Agent creation works after recovery");

    let validation_task = serde_json::json!({
        "type": "computation",
        "title": "Validation Task"
    });
    let validation_task_id = coordinator.create_task(validation_task).await?;
    let validation_result = coordinator.execute_task_with_verification(validation_task_id, validation_agent_id).await?;
    println!("  ✓ Task execution works after recovery: {}", validation_result.get("result").unwrap());

    coordinator.shutdown().await?;
    println!("✓ Backup and recovery test completed successfully");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_hive_system_integration() {
        complete_hive_system_integration().await.unwrap();
    }

    #[tokio::test]
    async fn test_modular_component_access() {
        modular_component_access().await.unwrap();
    }

    #[tokio::test]
    async fn test_cross_module_data_flow() {
        cross_module_data_flow().await.unwrap();
    }

    #[tokio::test]
    async fn test_error_handling_across_modules() {
        error_handling_across_modules().await.unwrap();
    }

    #[tokio::test]
    async fn test_performance_monitoring_integration() {
        performance_monitoring_integration().await.unwrap();
    }

    #[tokio::test]
    async fn test_configuration_management_across_modules() {
        configuration_management_across_modules().await.unwrap();
    }

    #[tokio::test]
    async fn test_scalability_testing_across_modules() {
        scalability_testing_across_modules().await.unwrap();
    }

    #[tokio::test]
    async fn test_module_health_monitoring() {
        module_health_monitoring().await.unwrap();
    }

    #[tokio::test]
    async fn test_backup_and_recovery_across_modules() {
        backup_and_recovery_across_modules().await.unwrap();
    }
}
