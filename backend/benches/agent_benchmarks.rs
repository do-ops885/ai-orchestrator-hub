use criterion::{black_box, criterion_group, criterion_main, Criterion};
use multiagent_hive::agents::{agent::AgentConfig, Agent};
use multiagent_hive::tasks::task::Task;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Benchmark agent creation performance
fn agent_creation_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("agent_creation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = AgentConfig {
                    id: uuid::Uuid::new_v4(),
                    name: "benchmark_agent".to_string(),
                    capabilities: vec!["processing".to_string(), "communication".to_string()],
                    max_concurrent_tasks: 5,
                    memory_limit_mb: 100,
                    timeout_seconds: 30,
                };

                let agent = Agent::new(config).await;
                black_box(agent);
            });
        });
    });
}

/// Benchmark task processing performance
fn task_processing_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("task_processing", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = AgentConfig {
                    id: uuid::Uuid::new_v4(),
                    name: "benchmark_agent".to_string(),
                    capabilities: vec!["processing".to_string()],
                    max_concurrent_tasks: 5,
                    memory_limit_mb: 100,
                    timeout_seconds: 30,
                };

                let mut agent = Agent::new(config).await.unwrap();

                let task = Task {
                    id: uuid::Uuid::new_v4(),
                    title: "Benchmark Task".to_string(),
                    description: "Performance benchmark task".to_string(),
                    priority: multiagent_hive::tasks::task::Priority::Medium,
                    status: multiagent_hive::tasks::task::TaskStatus::Pending,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    assigned_agent: Some(uuid::Uuid::new_v4()),
                    required_capabilities: vec!["processing".to_string()],
                    estimated_duration: std::time::Duration::from_secs(1),
                    actual_duration: None,
                    dependencies: vec![],
                    metadata: serde_json::json!({}),
                };

                let result = agent.process_task(Arc::new(task)).await;
                black_box(result);
            });
        });
    });
}

/// Benchmark agent communication performance
fn agent_communication_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("agent_communication", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config1 = AgentConfig {
                    id: uuid::Uuid::new_v4(),
                    name: "agent_1".to_string(),
                    capabilities: vec!["communication".to_string()],
                    max_concurrent_tasks: 5,
                    memory_limit_mb: 100,
                    timeout_seconds: 30,
                };

                let config2 = AgentConfig {
                    id: uuid::Uuid::new_v4(),
                    name: "agent_2".to_string(),
                    capabilities: vec!["communication".to_string()],
                    max_concurrent_tasks: 5,
                    memory_limit_mb: 100,
                    timeout_seconds: 30,
                };

                let agent1 = Agent::new(config1).await.unwrap();
                let agent2 = Agent::new(config2).await.unwrap();

                let message = serde_json::json!({
                    "type": "benchmark_message",
                    "content": "performance test message",
                    "timestamp": chrono::Utc::now().timestamp()
                });

                let result = agent1.send_message(&agent2.id(), message).await;
                black_box(result);
            });
        });
    });
}

/// Benchmark concurrent agent operations
fn concurrent_agents_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("concurrent_agents_10", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = vec![];

                for i in 0..10 {
                    let handle = tokio::spawn(async move {
                        let config = AgentConfig {
                            id: uuid::Uuid::new_v4(),
                            name: format!("concurrent_agent_{}", i),
                            capabilities: vec!["processing".to_string()],
                            max_concurrent_tasks: 5,
                            memory_limit_mb: 100,
                            timeout_seconds: 30,
                        };

                        let agent = Agent::new(config).await.unwrap();

                        // Simulate some work
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

                        agent
                    });

                    handles.push(handle);
                }

                for handle in handles {
                    let _agent = handle.await.unwrap();
                }
            });
        });
    });
}

criterion_group!(
    benches,
    agent_creation_benchmark,
    task_processing_benchmark,
    agent_communication_benchmark,
    concurrent_agents_benchmark
);
criterion_main!(benches);
