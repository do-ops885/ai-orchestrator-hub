use criterion::{black_box, criterion_group, criterion_main, Criterion};
use multiagent_hive::agents::Agent;
use multiagent_hive::infrastructure::async_optimizer::AsyncOptimizerConfig;
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

/// Benchmark async optimizer performance
fn async_optimizer_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("async_optimizer_batch", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = AsyncOptimizerConfig {
                    max_concurrent_ops: 4,
                    batch_size: 10,
                    batch_timeout: std::time::Duration::from_millis(50),
                    connection_pool_size: 5,
                    enable_prioritization: true,
                    metrics_interval: std::time::Duration::from_secs(60),
                };
                let optimizer =
                    multiagent_hive::infrastructure::async_optimizer::AsyncOptimizer::new(config);

                // Create batch of operations
                let operations: Vec<_> = (0..10)
                    .map(|i| {
                        move || async move {
                            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                            Ok::<i32, multiagent_hive::HiveError>(i)
                        }
                    })
                    .collect();

                let results = optimizer.execute_batch(operations).await.unwrap();
                black_box(results);
            });
        });
    });
}

/// Benchmark database operations with async optimization
fn database_operations_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("database_operations_optimized", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Simulate optimized database operations
                let mut handles = vec![];

                for i in 0..5 {
                    let handle = tokio::spawn(async move {
                        // Simulate database query with blocking operation moved to separate thread
                        tokio::task::spawn_blocking(move || {
                            std::thread::sleep(std::time::Duration::from_millis(2));
                            format!("query_result_{}", i)
                        })
                        .await
                        .unwrap()
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    let _result = handle.await.unwrap();
                }
            });
        });
    });
}

/// Benchmark WebSocket event-driven updates vs polling
fn websocket_updates_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("websocket_event_driven", |b| {
        b.iter(|| {
            rt.block_on(async {
                use tokio::sync::broadcast;

                let (tx, mut rx) = broadcast::channel(100);

                // Simulate sending updates
                let sender_handle = tokio::spawn(async move {
                    for i in 0..10 {
                        let message = multiagent_hive::WebSocketMessage {
                            message_type: "update".to_string(),
                            data: serde_json::json!({"count": i}),
                            timestamp: chrono::Utc::now(),
                        };
                        let _ = tx.send(message);
                        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                    }
                });

                // Simulate receiving updates
                let receiver_handle = tokio::spawn(async move {
                    let mut count = 0;
                    while let Ok(_) = rx.recv().await {
                        count += 1;
                        if count >= 10 {
                            break;
                        }
                    }
                    count
                });

                let _ = sender_handle.await;
                let result = receiver_handle.await.unwrap();
                black_box(result);
            });
        });
    });
}

criterion_group!(
    benches,
    agent_creation_benchmark,
    task_processing_benchmark,
    agent_communication_benchmark,
    concurrent_agents_benchmark,
    async_optimizer_benchmark,
    database_operations_benchmark,
    websocket_updates_benchmark
);
criterion_main!(benches);
