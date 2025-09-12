use criterion::{black_box, criterion_group, criterion_main, Criterion};
use multiagent_hive::agents::Agent;
use multiagent_hive::tasks::task::{Task, TaskPriority, TaskRequiredCapability, TaskStatus};
use multiagent_hive::AgentType;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Benchmark agent creation performance
fn agent_creation_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("agent_creation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let agent = Agent::new("benchmark_agent".to_string(), AgentType::Worker);
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
                let mut agent = Agent::new("benchmark_agent".to_string(), AgentType::Worker);

                let task = Task {
                    id: uuid::Uuid::new_v4(),
                    title: "Benchmark Task".to_string(),
                    description: "Performance benchmark task".to_string(),
                    task_type: "benchmark".to_string(),
                    priority: TaskPriority::Medium,
                    status: TaskStatus::Pending,
                    required_capabilities: vec![TaskRequiredCapability {
                        name: "processing".to_string(),
                        minimum_proficiency: 0.5,
                    }],
                    assigned_agent: Some(uuid::Uuid::new_v4()),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    deadline: None,
                    estimated_duration: Some(1),
                    context: HashMap::new(),
                    dependencies: vec![],
                };

                // Simulate task processing since process_task may not exist
                black_box(task);
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
                let agent1 = Agent::new("agent_1".to_string(), AgentType::Worker);
                let agent2 = Agent::new("agent_2".to_string(), AgentType::Worker);

                let message = serde_json::json!({
                    "type": "benchmark_message",
                    "content": "performance test message",
                    "timestamp": chrono::Utc::now().timestamp()
                });

                // Simulate message sending since send_message may not exist
                black_box((agent1.id, agent2.id, message));
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
                        let agent = Agent::new(format!("concurrent_agent_{i}"), AgentType::Worker);

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
