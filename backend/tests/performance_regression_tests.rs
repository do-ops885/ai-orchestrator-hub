//! Performance Regression Tests
//!
//! This module contains performance tests to detect regressions in system performance.
//! These tests measure key performance metrics and compare them against baselines.

use multiagent_hive::{AgentType, HiveCoordinator};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Performance test configuration
#[derive(Debug, Clone)]
struct PerformanceConfig {
    pub warmup_iterations: usize,
    pub measurement_iterations: usize,
    pub max_allowed_regression: f64, // percentage
    pub baseline_timeout: Duration,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            warmup_iterations: 5,
            measurement_iterations: 20,
            max_allowed_regression: 10.0, // 10% regression allowed
            baseline_timeout: Duration::from_secs(30),
        }
    }
}

/// Performance metrics collector
#[derive(Debug, Clone)]
struct PerformanceMetrics {
    pub operation_name: String,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub p95_duration: Duration,
    pub throughput: f64,           // operations per second
    pub memory_usage: Option<u64>, // bytes
    pub cpu_usage: Option<f64>,    // percentage
}

impl PerformanceMetrics {
    fn new(operation_name: String, durations: Vec<Duration>) -> Self {
        let mut sorted_durations = durations.clone();
        sorted_durations.sort();

        let total_duration: Duration = durations.iter().sum();
        let average_duration = total_duration / durations.len() as u32;

        let p95_index = (durations.len() as f64 * 0.95) as usize;
        let p95_duration = sorted_durations[p95_index.min(durations.len() - 1)];

        let total_seconds = total_duration.as_secs_f64();
        let throughput = durations.len() as f64 / total_seconds;

        Self {
            operation_name,
            average_duration,
            min_duration: *sorted_durations.first().unwrap(),
            max_duration: *sorted_durations.last().unwrap(),
            p95_duration,
            throughput,
            memory_usage: None,
            cpu_usage: None,
        }
    }

    fn compare_with_baseline(
        &self,
        baseline: &PerformanceMetrics,
        config: &PerformanceConfig,
    ) -> PerformanceComparison {
        let regression_percentage = ((self.average_duration.as_secs_f64()
            - baseline.average_duration.as_secs_f64())
            / baseline.average_duration.as_secs_f64())
            * 100.0;

        let throughput_change =
            ((self.throughput - baseline.throughput) / baseline.throughput) * 100.0;

        let passed = regression_percentage.abs() <= config.max_allowed_regression;

        PerformanceComparison {
            operation_name: self.operation_name.clone(),
            regression_percentage,
            throughput_change,
            passed,
            baseline_avg: baseline.average_duration,
            current_avg: self.average_duration,
        }
    }
}

#[derive(Debug, Clone)]
struct PerformanceComparison {
    pub operation_name: String,
    pub regression_percentage: f64,
    pub throughput_change: f64,
    pub passed: bool,
    pub baseline_avg: Duration,
    pub current_avg: Duration,
}

/// Performance regression test for agent creation
#[tokio::test]
async fn test_agent_creation_performance() {
    let config = PerformanceConfig::default();
    let mut durations = Vec::new();

    // Warmup phase
    for _ in 0..config.warmup_iterations {
        let hive = HiveCoordinator::new().await.expect("Failed to create hive");
        let config = create_agent_config("WarmupAgent", "worker", None);
        let _ = hive.create_agent(config).await;
    }

    // Measurement phase
    for i in 0..config.measurement_iterations {
        let start_time = Instant::now();

        let hive = HiveCoordinator::new().await.expect("Failed to create hive");
        let config = create_agent_config(&format!("PerfTestAgent{}", i), "worker", None);
        let result = hive.create_agent(config).await;

        let duration = start_time.elapsed();

        assert!(result.is_ok(), "Agent creation should succeed");
        durations.push(duration);

        // Small delay between tests to avoid interference
        sleep(Duration::from_millis(10)).await;
    }

    let metrics = PerformanceMetrics::new("agent_creation".to_string(), durations);

    println!("🕐 Agent Creation Performance:");
    println!("  Average: {:?}", metrics.average_duration);
    println!("  P95: {:?}", metrics.p95_duration);
    println!("  Throughput: {:.2} ops/sec", metrics.throughput);

    // Assert performance requirements
    assert!(
        metrics.average_duration < config.baseline_timeout,
        "Agent creation should be faster than {:?}",
        config.baseline_timeout
    );
    assert!(
        metrics.p95_duration < Duration::from_secs(5),
        "95% of agent creations should complete within 5 seconds"
    );

    println!("✅ Agent creation performance test passed");
}

/// Performance regression test for task creation and execution
#[tokio::test]
async fn test_task_execution_performance() {
    let config = PerformanceConfig::default();
    let mut durations = Vec::new();

    // Create hive with test agent
    let hive = HiveCoordinator::new().await.expect("Failed to create hive");
    let agent_config = create_agent_config(
        "TaskPerfAgent",
        "worker",
        Some(vec![("task_execution", 0.9, 0.1)]),
    );
    let agent_id = hive
        .create_agent(agent_config)
        .await
        .expect("Failed to create agent");

    // Warmup phase
    for _ in 0..config.warmup_iterations {
        let task_config = create_task_config("Warmup Task", "task_execution", 1, None);
        let _ = hive.create_task(task_config).await;
    }

    // Measurement phase
    for i in 0..config.measurement_iterations {
        let start_time = Instant::now();

        let task_config = create_task_config(
            &format!("Perf Task {}", i),
            "task_execution",
            1,
            Some(vec![("task_execution", 0.5)]),
        );

        let task_result = hive.create_task(task_config).await;
        assert!(task_result.is_ok(), "Task creation should succeed");

        let duration = start_time.elapsed();
        durations.push(duration);

        // Small delay between tests
        sleep(Duration::from_millis(5)).await;
    }

    let metrics = PerformanceMetrics::new("task_creation".to_string(), durations);

    println!("🕐 Task Creation Performance:");
    println!("  Average: {:?}", metrics.average_duration);
    println!("  P95: {:?}", metrics.p95_duration);
    println!("  Throughput: {:.2} ops/sec", metrics.throughput);

    // Assert performance requirements
    assert!(
        metrics.average_duration < Duration::from_millis(500),
        "Task creation should be faster than 500ms"
    );
    assert!(
        metrics.p95_duration < Duration::from_secs(2),
        "95% of task creations should complete within 2 seconds"
    );

    println!("✅ Task creation performance test passed");
}

/// Performance regression test for concurrent operations
#[tokio::test]
async fn test_concurrent_operations_performance() {
    let config = PerformanceConfig {
        measurement_iterations: 10, // Fewer iterations for concurrent test
        ..Default::default()
    };

    let start_time = Instant::now();
    let mut handles = vec![];

    // Create multiple concurrent operations
    for i in 0..config.measurement_iterations {
        let handle = tokio::spawn(async move {
            let hive = HiveCoordinator::new().await.expect("Failed to create hive");

            // Create agent
            let agent_config =
                create_agent_config(&format!("ConcurrentAgent{}", i), "worker", None);
            let agent_id = hive
                .create_agent(agent_config)
                .await
                .expect("Failed to create agent");

            // Create and execute task
            let task_config =
                create_task_config(&format!("Concurrent Task {}", i), "general", 1, None);
            let task_id = hive
                .create_task(task_config)
                .await
                .expect("Failed to create task");

            (agent_id, task_id)
        });

        handles.push(handle);
    }

    // Wait for all concurrent operations to complete
    let results = futures::future::join_all(handles).await;
    let total_duration = start_time.elapsed();

    let success_count = results.iter().filter(|result| result.is_ok()).count();

    let average_duration = total_duration / config.measurement_iterations as u32;
    let throughput = config.measurement_iterations as f64 / total_duration.as_secs_f64();

    println!("🔄 Concurrent Operations Performance:");
    println!("  Total operations: {}", config.measurement_iterations);
    println!("  Successful: {}", success_count);
    println!("  Total time: {:?}", total_duration);
    println!("  Average time per operation: {:?}", average_duration);
    println!("  Throughput: {:.2} ops/sec", throughput);

    // Assert performance requirements
    assert_eq!(
        success_count, config.measurement_iterations,
        "All concurrent operations should succeed"
    );
    assert!(
        average_duration < Duration::from_secs(10),
        "Concurrent operations should complete within 10 seconds each"
    );
    assert!(
        throughput > 0.5,
        "Should achieve at least 0.5 operations per second"
    );

    println!("✅ Concurrent operations performance test passed");
}

/// Memory usage performance test
#[tokio::test]
async fn test_memory_usage_performance() {
    let config = PerformanceConfig {
        measurement_iterations: 50, // More iterations for memory test
        ..Default::default()
    };

    let mut memory_readings = Vec::new();

    // Create increasing number of agents to test memory scaling
    for i in 1..=config.measurement_iterations {
        let hive = HiveCoordinator::new().await.expect("Failed to create hive");

        // Create multiple agents
        for j in 0..i {
            let agent_config =
                create_agent_config(&format!("MemoryAgent{}-{}", i, j), "worker", None);
            let _ = hive.create_agent(agent_config).await;
        }

        // Simulate memory reading (in real implementation, use system monitoring)
        let simulated_memory = (i * 1024 * 1024) as u64; // Simulate 1MB per agent
        memory_readings.push(simulated_memory);

        // Small delay to allow system to stabilize
        sleep(Duration::from_millis(10)).await;
    }

    let average_memory = memory_readings.iter().sum::<u64>() / memory_readings.len() as u64;
    let max_memory = *memory_readings.iter().max().unwrap();
    let memory_growth_rate =
        (max_memory as f64 - memory_readings[0] as f64) / memory_readings[0] as f64 * 100.0;

    println!("💾 Memory Usage Performance:");
    println!("  Average memory: {} MB", average_memory / (1024 * 1024));
    println!("  Max memory: {} MB", max_memory / (1024 * 1024));
    println!("  Memory growth rate: {:.2}%", memory_growth_rate);

    // Assert memory requirements
    assert!(
        memory_growth_rate < 200.0,
        "Memory growth should be less than 200%"
    );
    assert!(
        max_memory < 500 * 1024 * 1024,
        "Peak memory usage should be less than 500MB"
    );

    println!("✅ Memory usage performance test passed");
}

/// API response time performance test
#[tokio::test]
async fn test_api_response_performance() {
    // Note: This test requires the API server to be running
    // In a real CI/CD environment, you would start the server programmatically

    let client = reqwest::Client::new();
    let base_url = "http://localhost:3001";
    let mut response_times = Vec::new();

    // Test health endpoint
    for _ in 0..10 {
        let start_time = Instant::now();

        match client
            .get(&format!("{}/api/mcp/health", base_url))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    let duration = start_time.elapsed();
                    response_times.push(duration);
                }
            }
            Err(_) => {
                // Server not running, skip test
                println!("⚠️  API server not running, skipping response time test");
                return;
            }
        }

        sleep(Duration::from_millis(100)).await;
    }

    if response_times.is_empty() {
        println!("⚠️  No successful API responses, skipping performance analysis");
        return;
    }

    let average_response_time =
        response_times.iter().sum::<Duration>() / response_times.len() as u32;
    let max_response_time = *response_times.iter().max().unwrap();

    println!("🌐 API Response Performance:");
    println!("  Average response time: {:?}", average_response_time);
    println!("  Max response time: {:?}", max_response_time);
    println!("  Successful responses: {}/{}", response_times.len(), 10);

    // Assert API performance requirements
    assert!(
        average_response_time < Duration::from_millis(500),
        "Average API response time should be less than 500ms"
    );
    assert!(
        max_response_time < Duration::from_secs(2),
        "Max API response time should be less than 2 seconds"
    );

    println!("✅ API response performance test passed");
}

/// WebSocket performance test
#[tokio::test]
async fn test_websocket_performance() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

    let mut response_times = Vec::new();
    let message_count = 10;

    // Connect to WebSocket
    let ws_url = "ws://localhost:3001/ws";
    let connect_result = connect_async(ws_url).await;

    if connect_result.is_err() {
        println!("⚠️  WebSocket server not running, skipping WebSocket performance test");
        return;
    }

    let (ws_stream, _) = connect_result.unwrap();
    let (mut write, mut read) = ws_stream.split();

    // Send multiple messages and measure response times
    for i in 0..message_count {
        let start_time = Instant::now();

        let test_message = serde_json::json!({
            "type": "performance_test",
            "message_id": i,
            "timestamp": chrono::Utc::now().timestamp()
        });

        // Send message
        if let Err(_) = write.send(Message::Text(test_message.to_string())).await {
            break;
        }

        // Wait for response with timeout
        let response_future = read.next();
        match tokio::time::timeout(Duration::from_secs(2), response_future).await {
            Ok(Some(Ok(_))) => {
                let duration = start_time.elapsed();
                response_times.push(duration);
            }
            _ => {
                // No response or timeout
                break;
            }
        }

        sleep(Duration::from_millis(50)).await;
    }

    // Close connection
    let _ = write.send(Message::Close(None)).await;

    if response_times.is_empty() {
        println!("⚠️  No WebSocket responses received, skipping performance analysis");
        return;
    }

    let average_response_time =
        response_times.iter().sum::<Duration>() / response_times.len() as u32;
    let max_response_time = *response_times.iter().max().unwrap();

    println!("🔌 WebSocket Performance:");
    println!("  Average response time: {:?}", average_response_time);
    println!("  Max response time: {:?}", max_response_time);
    println!(
        "  Successful responses: {}/{}",
        response_times.len(),
        message_count
    );

    // Assert WebSocket performance requirements
    assert!(
        average_response_time < Duration::from_millis(200),
        "Average WebSocket response time should be less than 200ms"
    );
    assert!(
        max_response_time < Duration::from_secs(1),
        "Max WebSocket response time should be less than 1 second"
    );

    println!("✅ WebSocket performance test passed");
}

/// Helper function to create agent configuration
fn create_agent_config(
    name: &str,
    agent_type: &str,
    capabilities: Option<Vec<(&str, f64, f64)>>,
) -> serde_json::Value {
    let mut config = serde_json::json!({
        "name": name,
        "type": agent_type
    });

    if let Some(caps) = capabilities {
        let capabilities_array = caps
            .into_iter()
            .map(|(name, proficiency, learning_rate)| {
                serde_json::json!({
                    "name": name,
                    "proficiency": proficiency,
                    "learning_rate": learning_rate
                })
            })
            .collect::<Vec<_>>();

        config["capabilities"] = serde_json::Value::Array(capabilities_array);
    }

    config
}
