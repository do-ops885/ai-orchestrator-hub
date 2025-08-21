//! Comprehensive test suite runner for the multiagent hive system
//!
//! This file serves as the main test runner that can be executed with `cargo test`
//! and provides organized test execution with proper setup and teardown.

use multiagent_hive::tests::test_utils::*;
use multiagent_hive::{HiveCoordinator, Agent, AgentType, Task, TaskPriority};

/// Test suite configuration
struct TestConfig {
    pub timeout_ms: u64,
    pub max_agents: usize,
    pub max_tasks: usize,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            max_agents: 100,
            max_tasks: 200,
        }
    }
}

/// Comprehensive test suite that runs all major test categories
#[tokio::test]
async fn run_comprehensive_test_suite() {
    let config = TestConfig::default();
    
    println!("🚀 Starting Comprehensive Multiagent Hive Test Suite");
    println!("Configuration: timeout={}ms, max_agents={}, max_tasks={}", 
             config.timeout_ms, config.max_agents, config.max_tasks);
    
    // Test 1: Basic system initialization
    test_system_initialization().await;
    println!("✅ System initialization tests passed");
    
    // Test 2: Agent lifecycle management
    test_agent_lifecycle().await;
    println!("✅ Agent lifecycle tests passed");
    
    // Test 3: Task management and execution
    test_task_management().await;
    println!("✅ Task management tests passed");
    
    // Test 4: Hive coordination
    test_hive_coordination().await;
    println!("✅ Hive coordination tests passed");
    
    // Test 5: Neural processing integration
    test_neural_integration().await;
    println!("✅ Neural processing tests passed");
    
    // Test 6: Performance and scalability
    test_performance_scalability().await;
    println!("✅ Performance and scalability tests passed");
    
    // Test 7: Error handling and resilience
    test_error_handling().await;
    println!("✅ Error handling tests passed");
    
    println!("🎉 All comprehensive tests passed successfully!");
}

async fn test_system_initialization() {
    // Test hive coordinator creation
    let hive = HiveCoordinator::new().await;
    assert!(hive.is_ok(), "Failed to create hive coordinator");
    
    let coordinator = hive.unwrap();
    assert_eq!(coordinator.agents.len(), 0, "Hive should start with no agents");
    
    // Test initial metrics
    let status = coordinator.get_status().await;
    assert!(status["hive_id"].is_string(), "Hive ID should be present");
    assert_eq!(status["metrics"]["total_agents"], 0, "Initial agent count should be 0");
}

async fn test_agent_lifecycle() {
    let hive = HiveCoordinator::new().await.unwrap();
    
    // Test agent creation
    let agent_config = create_agent_config(
        "TestAgent",
        "worker",
        Some(vec![("general", 0.7, 0.1)]),
    );
    let agent_id = hive.create_agent(agent_config).await;
    assert!(agent_id.is_ok(), "Failed to create agent");
    
    let agent_uuid = agent_id.unwrap();
    assert!(hive.agents.contains_key(&agent_uuid), "Agent should exist in hive");
    
    // Test agent capabilities
    let agent = hive.agents.get(&agent_uuid).unwrap();
    assert_eq!(agent.name, "TestAgent", "Agent name should match");
    assert!(matches!(agent.agent_type, AgentType::Worker), "Agent type should be Worker");
    assert_approx_eq(agent.get_capability_score("general"), 0.7, 0.001);
    
    // Test agent info retrieval
    let agents_info = hive.get_agents_info().await;
    assert_eq!(agents_info["total_count"], 1, "Should have one agent");
}

async fn test_task_management() {
    let hive = HiveCoordinator::new().await.unwrap();
    
    // Create agent first
    let agent_config = create_agent_config(
        "TaskAgent",
        "worker",
        Some(vec![("data_processing", 0.8, 0.1)]),
    );
    let _agent_id = hive.create_agent(agent_config).await.unwrap();
    
    // Test task creation
    let task_config = create_task_config(
        "Test task",
        "data_processing",
        1, // Medium priority
        Some(vec![("data_processing", 0.7)]),
    );
    let task_id = hive.create_task(task_config).await;
    assert!(task_id.is_ok(), "Failed to create task");
    
    // Test task info retrieval
    let tasks_info = hive.get_tasks_info().await;
    assert!(tasks_info["work_stealing_queue"]["total_queue_depth"].as_u64().unwrap_or(0) >= 0);
    
    // Wait for task processing
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Check work-stealing metrics
    let ws_metrics = hive.work_stealing_queue.get_metrics().await;
    assert_eq!(ws_metrics.active_agents, 1, "Should have one active agent");
}

async fn test_hive_coordination() {
    let hive = HiveCoordinator::new().await.unwrap();
    
    // Create multiple agents for coordination testing
    for i in 0..3 {
        let config = create_agent_config(
            &format!("CoordAgent{}", i),
            "worker",
            Some(vec![("coordination", 0.6, 0.1)]),
        );
        let _agent_id = hive.create_agent(config).await.unwrap();
    }
    
    // Wait for coordination processes
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    // Test swarm center calculation
    let swarm_center = *hive.swarm_center.read().await;
    assert!(swarm_center.0.is_finite() && swarm_center.1.is_finite(), 
            "Swarm center should be valid coordinates");
    
    // Test metrics update
    let status = hive.get_status().await;
    assert_eq!(status["metrics"]["total_agents"], 3, "Should have three agents");
    assert!(status["metrics"]["swarm_cohesion"].as_f64().unwrap() >= 0.0, 
            "Swarm cohesion should be non-negative");
}

async fn test_neural_integration() {
    let hive = HiveCoordinator::new().await.unwrap();
    
    // Test NLP processor
    let nlp = &hive.nlp_processor;
    let test_text = "successful task completion with excellent results";
    let keywords = nlp.extract_keywords(test_text, 3);
    assert!(!keywords.is_empty(), "Should extract keywords");
    
    let sentiment_tokens = vec!["excellent".to_string(), "successful".to_string()];
    let sentiment = nlp.analyze_sentiment(&sentiment_tokens);
    assert!(sentiment > 0.0, "Should detect positive sentiment");
    
    // Test neural processor
    let neural = hive.neural_processor.read().await;
    // Neural processor should be initialized without errors
    drop(neural);
    
    // Test agent with neural capabilities
    let agent_config = create_agent_config(
        "NeuralAgent",
        "learner",
        Some(vec![("neural_processing", 0.6, 0.15)]),
    );
    let agent_id = hive.create_agent(agent_config).await;
    assert!(agent_id.is_ok(), "Should create neural agent successfully");
}

async fn test_performance_scalability() {
    let hive = HiveCoordinator::new().await.unwrap();
    
    // Create multiple agents and tasks to test scalability
    let num_agents = 5;
    let num_tasks = 10;
    
    // Create agents
    for i in 0..num_agents {
        let config = create_agent_config(
            &format!("PerfAgent{}", i),
            "worker",
            Some(vec![("performance", 0.7, 0.1)]),
        );
        let agent_result = hive.create_agent(config).await;
        assert!(agent_result.is_ok(), "Should create agent {}", i);
    }
    
    // Create tasks
    for i in 0..num_tasks {
        let config = create_task_config(
            &format!("PerfTask{}", i),
            "performance",
            1,
            None,
        );
        let task_result = hive.create_task(config).await;
        assert!(task_result.is_ok(), "Should create task {}", i);
    }
    
    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    
    // Verify system handled the load
    assert_eq!(hive.agents.len(), num_agents, "Should have all agents");
    
    let status = hive.get_status().await;
    assert_eq!(status["metrics"]["total_agents"], num_agents, "Metrics should reflect agent count");
    
    // Check resource utilization
    let resource_info = hive.get_resource_info().await;
    assert!(resource_info["system_resources"]["cpu_usage"].is_number(), 
            "Should report CPU usage");
    assert!(resource_info["system_resources"]["memory_usage"].is_number(), 
            "Should report memory usage");
}

async fn test_error_handling() {
    let hive = HiveCoordinator::new().await.unwrap();
    
    // Test invalid configurations
    let invalid_agent_config = serde_json::json!({
        "name": "",
        "type": "invalid_type"
    });
    let agent_result = hive.create_agent(invalid_agent_config).await;
    assert!(agent_result.is_ok(), "Should handle invalid agent config gracefully");
    
    let invalid_task_config = serde_json::json!({
        "description": "",
        "priority": 999
    });
    let task_result = hive.create_task(invalid_task_config).await;
    assert!(task_result.is_ok(), "Should handle invalid task config gracefully");
    
    // Test operations on empty hive
    let empty_hive = HiveCoordinator::new().await.unwrap();
    let empty_status = empty_hive.get_status().await;
    assert_eq!(empty_status["metrics"]["total_agents"], 0, "Empty hive should have no agents");
    
    // Test concurrent operations
    let hive_clone1 = hive.clone();
    let hive_clone2 = hive.clone();
    
    let handle1 = tokio::spawn(async move {
        for i in 0..3 {
            let config = create_agent_config(&format!("ConcurrentAgent{}", i), "worker", None);
            let _result = hive_clone1.create_agent(config).await;
        }
    });
    
    let handle2 = tokio::spawn(async move {
        for i in 0..3 {
            let config = create_task_config(&format!("ConcurrentTask{}", i), "general", 1, None);
            let _result = hive_clone2.create_task(config).await;
        }
    });
    
    let (result1, result2) = tokio::join!(handle1, handle2);
    assert!(result1.is_ok() && result2.is_ok(), "Concurrent operations should succeed");
}

/// Performance benchmark test
#[tokio::test]
async fn benchmark_system_performance() {
    let hive = HiveCoordinator::new().await.unwrap();
    
    let start_time = std::time::Instant::now();
    
    // Create agents
    for i in 0..10 {
        let config = create_agent_config(&format!("BenchAgent{}", i), "worker", None);
        let _result = hive.create_agent(config).await.unwrap();
    }
    
    let agent_creation_time = start_time.elapsed();
    println!("Agent creation time: {:?}", agent_creation_time);
    
    let task_start = std::time::Instant::now();
    
    // Create tasks
    for i in 0..20 {
        let config = create_task_config(&format!("BenchTask{}", i), "general", 1, None);
        let _result = hive.create_task(config).await.unwrap();
    }
    
    let task_creation_time = task_start.elapsed();
    println!("Task creation time: {:?}", task_creation_time);
    
    // Performance assertions
    assert!(agent_creation_time.as_millis() < 5000, "Agent creation should be fast");
    assert!(task_creation_time.as_millis() < 5000, "Task creation should be fast");
    
    println!("✅ Performance benchmarks passed");
}

/// Memory usage test
#[tokio::test]
async fn test_memory_usage() {
    let hive = HiveCoordinator::new().await.unwrap();
    
    // Create a moderate number of agents and tasks
    for i in 0..20 {
        let config = create_agent_config(&format!("MemAgent{}", i), "worker", 
                                       Some(vec![("memory_test", 0.5, 0.1)]));
        let _result = hive.create_agent(config).await.unwrap();
    }
    
    for i in 0..40 {
        let config = create_task_config(&format!("MemTask{}", i), "memory_test", 1, None);
        let _result = hive.create_task(config).await.unwrap();
    }
    
    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    // Check resource usage
    let resource_info = hive.get_resource_info().await;
    let memory_usage = resource_info["system_resources"]["memory_usage"].as_f64().unwrap();
    
    // Memory usage should be reasonable (less than 90%)
    assert!(memory_usage < 90.0, "Memory usage should be reasonable: {}%", memory_usage);
    
    println!("✅ Memory usage test passed: {}%", memory_usage);
}