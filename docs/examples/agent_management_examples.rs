//! # Agent Management Usage Examples
//!
//! This file contains practical examples demonstrating how to use the AgentManager
//! for agent lifecycle management, performance monitoring, and optimization.

use crate::core::hive::agent_management::{AgentManager, AgentRegistrationResult};
use crate::infrastructure::resource_manager::ResourceManager;
use crate::utils::error::HiveResult;
use crate::agents::agent::Agent;
use serde_json;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Example: Basic Agent Creation and Management
/// Description: Create agents with different types and manage their lifecycle
/// Use case: Setting up a basic multiagent system
async fn basic_agent_creation() -> HiveResult<()> {
    // Initialize dependencies
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let (tx, _rx) = mpsc::unbounded_channel();
    let agent_manager = AgentManager::new(resource_manager, tx).await?;

    // Create different types of agents
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

    // Create agents
    for config in agent_configs {
        let agent_id = agent_manager.create_agent(config).await?;
        agent_ids.push(agent_id);
        println!("Created agent: {}", agent_id);
    }

    // Verify agent count
    assert_eq!(agent_manager.get_agent_count(), 4);

    // Get all agents
    let all_agents = agent_manager.get_all_agents().await;
    println!("Total agents: {}", all_agents.len());

    for (id, agent) in all_agents {
        println!("Agent {}: {} ({:?})", id, agent.name, agent.agent_type);
    }

    // Cleanup: Remove all agents
    for agent_id in agent_ids {
        agent_manager.remove_agent(agent_id).await?;
    }

    assert_eq!(agent_manager.get_agent_count(), 0);
    Ok(())
}

/// Example: Agent Performance Monitoring
/// Description: Track and analyze agent performance metrics
/// Use case: Performance optimization and resource allocation
async fn agent_performance_monitoring() -> HiveResult<()> {
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let (tx, _rx) = mpsc::unbounded_channel();
    let agent_manager = AgentManager::new(resource_manager, tx).await?;

    // Create multiple agents
    let mut agent_ids = Vec::new();
    for i in 0..3 {
        let config = serde_json::json!({
            "type": "worker",
            "name": format!("perf_agent_{}", i)
        });
        let agent_id = agent_manager.create_agent(config).await?;
        agent_ids.push(agent_id);
    }

    // Simulate task execution with different performance characteristics
    let performance_scenarios = vec![
        (agent_ids[0], 100, true),  // Fast, successful
        (agent_ids[0], 150, true),  // Medium, successful
        (agent_ids[1], 200, true),  // Slow, successful
        (agent_ids[1], 50, false),  // Fast, failed
        (agent_ids[2], 120, true),  // Medium, successful
        (agent_ids[2], 180, true),  // Slow, successful
        (agent_ids[2], 90, false),  // Fast, failed
    ];

    // Record performance metrics
    for (agent_id, execution_time, success) in performance_scenarios {
        agent_manager.update_agent_metrics(agent_id, execution_time, success).await;
    }

    // Get performance analytics
    let analytics = agent_manager.get_analytics().await;
    println!("Performance Analytics: {}", serde_json::to_string_pretty(&analytics)?);

    // Get top performers
    let top_performers = analytics.get("top_performers").unwrap().as_array().unwrap();
    println!("Top Performers:");
    for performer in top_performers {
        println!("  Agent: {}, Score: {}",
                 performer.get("agent_id").unwrap(),
                 performer.get("performance_score").unwrap());
    }

    // Get resource utilization
    let resource_utilization = analytics.get("resource_utilization").unwrap();
    println!("Resource Utilization: {}", serde_json::to_string_pretty(resource_utilization)?);

    Ok(())
}

/// Example: Agent Health Monitoring
/// Description: Monitor agent health and detect issues
/// Use case: System reliability and fault detection
async fn agent_health_monitoring() -> HiveResult<()> {
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let (tx, _rx) = mpsc::unbounded_channel();
    let agent_manager = AgentManager::new(resource_manager, tx).await?;

    // Create agents
    let mut agent_ids = Vec::new();
    for i in 0..5 {
        let config = serde_json::json!({
            "type": "worker",
            "name": format!("health_agent_{}", i)
        });
        let agent_id = agent_manager.create_agent(config).await?;
        agent_ids.push(agent_id);
    }

    // Simulate mixed performance (some healthy, some struggling)
    for (i, &agent_id) in agent_ids.iter().enumerate() {
        let success_rate = if i < 3 { 0.9 } else { 0.3 }; // First 3 healthy, last 2 struggling

        for _ in 0..10 {
            let success = rand::random::<f64>() < success_rate;
            let execution_time = if success { 100 + (rand::random::<u64>() % 100) } else { 50 };
            agent_manager.update_agent_metrics(agent_id, execution_time, success).await;
        }
    }

    // Get status summary
    let status = agent_manager.get_status().await;
    println!("Agent Status Summary: {}", serde_json::to_string_pretty(&status)?);

    // Analyze performance distribution
    let analytics = agent_manager.get_analytics().await;
    let agent_performance = analytics.get("agent_performance").unwrap().as_array().unwrap();

    println!("Agent Health Analysis:");
    for perf in agent_performance {
        let agent_id = perf.get("agent_id").unwrap().as_str().unwrap();
        let success_rate = perf.get("success_rate").unwrap().as_f64().unwrap();
        let avg_time = perf.get("average_execution_time_ms").unwrap().as_f64().unwrap();

        let health_status = if success_rate > 0.8 && avg_time < 150.0 {
            "Healthy"
        } else if success_rate > 0.6 {
            "Warning"
        } else {
            "Critical"
        };

        println!("  Agent {}: {} (Success: {:.1}%, Avg Time: {:.1}ms)",
                 agent_id, health_status, success_rate * 100.0, avg_time);
    }

    Ok(())
}

/// Example: Dynamic Agent Scaling
/// Description: Add and remove agents based on workload
/// Use case: Auto-scaling in response to demand changes
async fn dynamic_agent_scaling() -> HiveResult<()> {
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let (tx, _rx) = mpsc::unbounded_channel();
    let agent_manager = AgentManager::new(resource_manager, tx).await?;

    println!("Starting with {} agents", agent_manager.get_agent_count());

    // Scale up: Add agents during high load
    println!("Scaling up due to high load...");
    let mut high_load_agents = Vec::new();
    for i in 0..5 {
        let config = serde_json::json!({
            "type": "worker",
            "name": format!("scale_up_agent_{}", i)
        });
        let agent_id = agent_manager.create_agent(config).await?;
        high_load_agents.push(agent_id);
    }

    println!("After scale up: {} agents", agent_manager.get_agent_count());

    // Simulate workload processing
    for &agent_id in &high_load_agents {
        for _ in 0..5 {
            agent_manager.update_agent_metrics(agent_id, 100, true).await;
        }
    }

    // Scale down: Remove underutilized agents
    println!("Scaling down due to low load...");
    while high_load_agents.len() > 2 {
        if let Some(agent_id) = high_load_agents.pop() {
            agent_manager.remove_agent(agent_id).await?;
            println!("Removed agent: {}", agent_id);
        }
    }

    println!("After scale down: {} agents", agent_manager.get_agent_count());

    // Get final status
    let final_status = agent_manager.get_status().await;
    println!("Final Status: {}", serde_json::to_string_pretty(&final_status)?);

    Ok(())
}

/// Example: Agent Configuration Management
/// Description: Manage agent configurations and capabilities
/// Use case: Customizing agent behavior for specific tasks
async fn agent_configuration_management() -> HiveResult<()> {
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let (tx, _rx) = mpsc::unbounded_channel();
    let agent_manager = AgentManager::new(resource_manager, tx).await?;

    // Create agents with different configurations
    let configs = vec![
        serde_json::json!({
            "type": "worker",
            "name": "basic_worker",
            "capabilities": ["computation"],
            "max_concurrent_tasks": 1
        }),
        serde_json::json!({
            "type": "specialist",
            "name": "data_scientist",
            "capabilities": ["machine_learning", "statistics", "data_analysis"],
            "specialization": "data_science",
            "experience_level": "expert"
        }),
        serde_json::json!({
            "type": "coordinator",
            "name": "project_manager",
            "capabilities": ["planning", "scheduling", "resource_management"],
            "team_size": 10,
            "management_style": "agile"
        }),
    ];

    let mut agent_ids = Vec::new();

    for config in configs {
        let agent_id = agent_manager.create_agent(config.clone()).await?;
        agent_ids.push(agent_id);

        // Get the created agent and display its configuration
        if let Some(agent) = agent_manager.get_agent(agent_id).await {
            println!("Created agent: {} ({:?})", agent.name, agent.agent_type);
            println!("  Configuration: {}", serde_json::to_string_pretty(&config)?);
        }
    }

    // Demonstrate agent lookup by capabilities
    let all_agents = agent_manager.get_all_agents().await;
    println!("Agent Capabilities Summary:");
    for (id, agent) in all_agents {
        println!("  {}: {:?}", agent.name, agent.agent_type);
    }

    Ok(())
}

/// Example: Error Handling in Agent Management
/// Description: Handle various error scenarios gracefully
/// Use case: Building robust agent management systems
async fn error_handling_in_agent_management() -> HiveResult<()> {
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let (tx, _rx) = mpsc::unbounded_channel();
    let agent_manager = AgentManager::new(resource_manager, tx).await?;

    // Test 1: Invalid agent type
    println!("Test 1: Invalid agent type");
    let invalid_type_config = serde_json::json!({
        "type": "invalid_type",
        "name": "test_agent"
    });

    match agent_manager.create_agent(invalid_type_config).await {
        Ok(id) => println!("Unexpected success: {}", id),
        Err(e) => println!("Expected error: {}", e),
    }

    // Test 2: Missing required fields
    println!("Test 2: Missing required fields");
    let missing_type_config = serde_json::json!({
        "name": "test_agent"
    });

    match agent_manager.create_agent(missing_type_config).await {
        Ok(id) => println!("Unexpected success: {}", id),
        Err(e) => println!("Expected error: {}", e),
    }

    // Test 3: Invalid configuration format
    println!("Test 3: Invalid configuration format");
    let invalid_config = serde_json::json!("not_an_object");

    match agent_manager.create_agent(invalid_config).await {
        Ok(id) => println!("Unexpected success: {}", id),
        Err(e) => println!("Expected error: {}", e),
    }

    // Test 4: Removing non-existent agent
    println!("Test 4: Removing non-existent agent");
    let fake_id = Uuid::new_v4();

    match agent_manager.remove_agent(fake_id).await {
        Ok(()) => println!("Unexpected success"),
        Err(e) => println!("Expected error: {}", e),
    }

    // Test 5: Updating metrics for non-existent agent
    println!("Test 5: Updating metrics for non-existent agent");
    agent_manager.update_agent_metrics(fake_id, 100, true).await;
    println!("Metrics update for non-existent agent completed (should be graceful)");

    // Test 6: Successful operations after errors
    println!("Test 6: Successful operations after errors");
    let valid_config = serde_json::json!({
        "type": "worker",
        "name": "recovery_agent"
    });

    let agent_id = agent_manager.create_agent(valid_config).await?;
    println!("Successfully created agent after error scenarios: {}", agent_id);

    // Verify the agent was created
    assert_eq!(agent_manager.get_agent_count(), 1);

    Ok(())
}

/// Example: Agent Learning and Adaptation
/// Description: Implement learning cycles for agent improvement
/// Use case: Continuous improvement and adaptation
async fn agent_learning_and_adaptation() -> HiveResult<()> {
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let (tx, _rx) = mpsc::unbounded_channel();
    let agent_manager = AgentManager::new(resource_manager, tx).await?;

    // Create learner agents
    let mut learner_ids = Vec::new();
    for i in 0..3 {
        let config = serde_json::json!({
            "type": "learner",
            "name": format!("learner_agent_{}", i)
        });
        let agent_id = agent_manager.create_agent(config).await?;
        learner_ids.push(agent_id);
    }

    // Simulate learning cycles with performance tracking
    for cycle in 0..5 {
        println!("Learning Cycle {}", cycle + 1);

        // Simulate task execution with improving performance
        for &agent_id in &learner_ids {
            let base_time = 200 - (cycle * 20); // Performance improves over cycles
            let success_rate = 0.6 + (cycle as f64 * 0.08); // Success rate improves

            for _ in 0..3 {
                let execution_time = base_time + (rand::random::<u64>() % 50);
                let success = rand::random::<f64>() < success_rate;
                agent_manager.update_agent_metrics(agent_id, execution_time, success).await;
            }
        }

        // Run learning cycle (would integrate with NLP processor in real implementation)
        // agent_manager.run_learning_cycle(&nlp_processor).await?;

        // Get current performance
        let analytics = agent_manager.get_analytics().await;
        let agent_performance = analytics.get("agent_performance").unwrap().as_array().unwrap();

        println!("  Current Performance:");
        for perf in agent_performance {
            let success_rate = perf.get("success_rate").unwrap().as_f64().unwrap();
            let avg_time = perf.get("average_execution_time_ms").unwrap().as_f64().unwrap();
            println!("    Success Rate: {:.1}%, Avg Time: {:.1}ms",
                     success_rate * 100.0, avg_time);
        }
    }

    // Final performance analysis
    let final_analytics = agent_manager.get_analytics().await;
    println!("Final Learning Results: {}", serde_json::to_string_pretty(&final_analytics)?);

    Ok(())
}

/// Example: Resource-Aware Agent Management
/// Description: Manage agents considering resource constraints
/// Use case: Resource optimization and capacity planning
async fn resource_aware_agent_management() -> HiveResult<()> {
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let (tx, _rx) = mpsc::unbounded_channel();
    let agent_manager = AgentManager::new(resource_manager.clone(), tx).await?;

    // Monitor system resources before creating agents
    let initial_resources = resource_manager.get_system_info().await;
    println!("Initial System Resources:");
    println!("  CPU Cores: {}", initial_resources.0.cpu_cores);
    println!("  Available Memory: {} MB", initial_resources.0.available_memory / 1_000_000);
    println!("  CPU Usage: {:.1}%", initial_resources.0.cpu_usage * 100.0);
    println!("  Memory Usage: {:.1}%", initial_resources.0.memory_usage * 100.0);

    // Create agents based on available resources
    let max_agents = if initial_resources.0.cpu_usage < 0.7 { 5 } else { 2 };
    println!("Creating {} agents based on resource availability", max_agents);

    let mut agent_ids = Vec::new();
    for i in 0..max_agents {
        let config = serde_json::json!({
            "type": "worker",
            "name": format!("resource_aware_agent_{}", i),
            "resource_requirements": {
                "cpu_cores": 1,
                "memory_mb": 256
            }
        });
        let agent_id = agent_manager.create_agent(config).await?;
        agent_ids.push(agent_id);
    }

    // Monitor resource usage after agent creation
    let post_creation_resources = resource_manager.get_system_info().await;
    println!("Resources After Agent Creation:");
    println!("  CPU Usage: {:.1}%", post_creation_resources.0.cpu_usage * 100.0);
    println!("  Memory Usage: {:.1}%", post_creation_resources.0.memory_usage * 100.0);

    // Get agent resource utilization
    let analytics = agent_manager.get_analytics().await;
    let resource_utilization = analytics.get("resource_utilization").unwrap();
    println!("Agent Resource Utilization: {}", serde_json::to_string_pretty(resource_utilization)?);

    // Simulate resource monitoring during operation
    for &agent_id in &agent_ids {
        agent_manager.update_agent_metrics(agent_id, 100, true).await;
    }

    // Final resource assessment
    let final_resources = resource_manager.get_system_info().await;
    println!("Final Resource Assessment:");
    println!("  CPU Usage: {:.1}%", final_resources.0.cpu_usage * 100.0);
    println!("  Memory Usage: {:.1}%", final_resources.0.memory_usage * 100.0);

    Ok(())
}

/// Example: Agent Performance Benchmarking
/// Description: Benchmark agent performance across different scenarios
/// Use case: Performance testing and optimization
async fn agent_performance_benchmarking() -> HiveResult<()> {
    let resource_manager = Arc::new(ResourceManager::new().await?);
    let (tx, _rx) = mpsc::unbounded_channel();
    let agent_manager = AgentManager::new(resource_manager, tx).await?;

    // Create agents of different types for benchmarking
    let benchmark_configs = vec![
        ("worker", "standard_worker"),
        ("specialist", "computation_specialist"),
        ("learner", "adaptive_learner"),
    ];

    let mut benchmark_agents = Vec::new();
    for (agent_type, name) in benchmark_configs {
        let config = serde_json::json!({
            "type": agent_type,
            "name": name
        });
        let agent_id = agent_manager.create_agent(config).await?;
        benchmark_agents.push((agent_id, name.to_string()));
    }

    // Define benchmark scenarios
    let scenarios = vec![
        ("light_computation", 50, 20),   // Fast tasks
        ("medium_computation", 150, 10), // Medium tasks
        ("heavy_computation", 300, 5),   // Slow tasks
    ];

    println!("Running Performance Benchmarks:");

    for (scenario_name, base_time, task_count) in scenarios {
        println!("Scenario: {}", scenario_name);

        for (agent_id, agent_name) in &benchmark_agents {
            let start_time = std::time::Instant::now();

            // Execute tasks for this scenario
            for _ in 0..task_count {
                let execution_time = base_time + (rand::random::<u64>() % (base_time / 2));
                let success = rand::random::<f64>() < 0.9; // 90% success rate
                agent_manager.update_agent_metrics(*agent_id, execution_time, success).await;
            }

            let duration = start_time.elapsed();
            println!("  {}: {} tasks in {:.2}s", agent_name, task_count, duration.as_secs_f64());
        }
    }

    // Generate benchmark report
    let analytics = agent_manager.get_analytics().await;
    let agent_performance = analytics.get("agent_performance").unwrap().as_array().unwrap();

    println!("Benchmark Results:");
    for perf in agent_performance {
        let agent_id = perf.get("agent_id").unwrap().as_str().unwrap();
        let tasks_completed = perf.get("tasks_completed").unwrap().as_u64().unwrap();
        let tasks_failed = perf.get("tasks_failed").unwrap().as_u64().unwrap();
        let success_rate = perf.get("success_rate").unwrap().as_f64().unwrap();
        let avg_time = perf.get("average_execution_time_ms").unwrap().as_f64().unwrap();

        println!("Agent {}: {} completed, {} failed, {:.1}% success, {:.1}ms avg",
                 agent_id, tasks_completed, tasks_failed, success_rate * 100.0, avg_time);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_agent_creation() {
        basic_agent_creation().await.unwrap();
    }

    #[tokio::test]
    async fn test_agent_performance_monitoring() {
        agent_performance_monitoring().await.unwrap();
    }

    #[tokio::test]
    async fn test_agent_health_monitoring() {
        agent_health_monitoring().await.unwrap();
    }

    #[tokio::test]
    async fn test_dynamic_agent_scaling() {
        dynamic_agent_scaling().await.unwrap();
    }

    #[tokio::test]
    async fn test_agent_configuration_management() {
        agent_configuration_management().await.unwrap();
    }

    #[tokio::test]
    async fn test_error_handling_in_agent_management() {
        error_handling_in_agent_management().await.unwrap();
    }

    #[tokio::test]
    async fn test_resource_aware_agent_management() {
        resource_aware_agent_management().await.unwrap();
    }
}
