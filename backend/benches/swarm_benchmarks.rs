use criterion::{black_box, criterion_group, criterion_main, Criterion};
use multiagent_hive::agents::Agent;
use multiagent_hive::core::hive::Hive;
use multiagent_hive::core::swarm_intelligence::SwarmIntelligence;
use multiagent_hive::tasks::task::Task;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Benchmark hive creation
fn hive_creation_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("hive_creation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = multiagent_hive::core::hive::HiveConfig {
                    max_agents: 10,
                    auto_scaling_enabled: true,
                    swarm_intelligence_enabled: true,
                    monitoring_enabled: false, // Disable for benchmark
                };

                let hive = Hive::new(config).await;
                black_box(hive);
            });
        });
    });
}

/// Benchmark swarm coordination
fn swarm_coordination_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("swarm_coordination", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = multiagent_hive::core::hive::HiveConfig {
                    max_agents: 5,
                    auto_scaling_enabled: true,
                    swarm_intelligence_enabled: true,
                    monitoring_enabled: false,
                };

                let mut hive = Hive::new(config).await.unwrap();
                let swarm = SwarmIntelligence::new(hive.get_agents().await.len());

                // Simulate coordination decisions
                let decisions = swarm.make_coordination_decisions().await;
                black_box(decisions);
            });
        });
    });
}

/// Benchmark task distribution
fn task_distribution_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("task_distribution", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = multiagent_hive::core::hive::HiveConfig {
                    max_agents: 10,
                    auto_scaling_enabled: true,
                    swarm_intelligence_enabled: true,
                    monitoring_enabled: false,
                };

                let mut hive = Hive::new(config).await.unwrap();

                // Create multiple tasks
                let tasks = (0..20)
                    .map(|i| {
                        Arc::new(Task {
                            id: uuid::Uuid::new_v4(),
                            title: format!("Benchmark Task {}", i),
                            description: "Performance benchmark task".to_string(),
                            priority: multiagent_hive::tasks::task::Priority::Medium,
                            status: multiagent_hive::tasks::task::TaskStatus::Pending,
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                            assigned_agent: None,
                            required_capabilities: vec!["processing".to_string()],
                            estimated_duration: std::time::Duration::from_secs(1),
                            actual_duration: None,
                            dependencies: vec![],
                            metadata: serde_json::json!({}),
                        })
                    })
                    .collect::<Vec<_>>();

                // Distribute tasks
                for task in tasks {
                    let _result = hive.assign_task(task).await;
                }

                black_box(hive);
            });
        });
    });
}

/// Benchmark agent communication in swarm
fn swarm_communication_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("swarm_communication", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = multiagent_hive::core::hive::HiveConfig {
                    max_agents: 20,
                    auto_scaling_enabled: true,
                    swarm_intelligence_enabled: true,
                    monitoring_enabled: false,
                };

                let mut hive = Hive::new(config).await.unwrap();

                // Simulate inter-agent communication
                let agents = hive.get_agents().await;
                if agents.len() >= 2 {
                    let agent1 = &agents[0];
                    let agent2 = &agents[1];

                    let message = serde_json::json!({
                        "type": "swarm_coordination",
                        "content": "benchmark coordination message",
                        "timestamp": chrono::Utc::now().timestamp()
                    });

                    let _result = agent1.send_message(&agent2.id(), message).await;
                }

                black_box(hive);
            });
        });
    });
}

/// Benchmark auto-scaling decisions
fn auto_scaling_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("auto_scaling_decisions", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = multiagent_hive::core::hive::HiveConfig {
                    max_agents: 50,
                    auto_scaling_enabled: true,
                    swarm_intelligence_enabled: true,
                    monitoring_enabled: false,
                };

                let mut hive = Hive::new(config).await.unwrap();

                // Simulate load that triggers auto-scaling
                for _ in 0..100 {
                    let task = Arc::new(Task {
                        id: uuid::Uuid::new_v4(),
                        title: "Load Task".to_string(),
                        description: "Task to trigger auto-scaling".to_string(),
                        priority: multiagent_hive::tasks::task::Priority::High,
                        status: multiagent_hive::tasks::task::TaskStatus::Pending,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        assigned_agent: None,
                        required_capabilities: vec!["processing".to_string()],
                        estimated_duration: std::time::Duration::from_millis(100),
                        actual_duration: None,
                        dependencies: vec![],
                        metadata: serde_json::json!({}),
                    });

                    let _result = hive.assign_task(task).await;
                }

                // Trigger auto-scaling evaluation
                let scaling_decision = hive.evaluate_auto_scaling().await;
                black_box(scaling_decision);
            });
        });
    });
}

/// Benchmark swarm intelligence algorithms
fn swarm_intelligence_algorithms_benchmark(c: &mut Criterion) {
    c.bench_function("swarm_algorithms", |b| {
        b.iter(|| {
            let swarm = SwarmIntelligence::new(10);

            // Test particle swarm optimization
            let particles = swarm.initialize_particles(20, 5);
            let best_solution = swarm.optimize_particles(particles, 100);

            black_box(best_solution);
        });
    });
}

/// Benchmark concurrent swarm operations
fn concurrent_swarm_operations_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("concurrent_swarm_ops", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = vec![];

                for i in 0..5 {
                    let handle = tokio::spawn(async move {
                        let config = multiagent_hive::core::hive::HiveConfig {
                            max_agents: 5,
                            auto_scaling_enabled: true,
                            swarm_intelligence_enabled: true,
                            monitoring_enabled: false,
                        };

                        let mut hive = Hive::new(config).await.unwrap();

                        // Add some tasks
                        for j in 0..10 {
                            let task = Arc::new(Task {
                                id: uuid::Uuid::new_v4(),
                                title: format!("Concurrent Task {} {}", i, j),
                                description: "Concurrent benchmark task".to_string(),
                                priority: multiagent_hive::tasks::task::Priority::Medium,
                                status: multiagent_hive::tasks::task::TaskStatus::Pending,
                                created_at: chrono::Utc::now(),
                                updated_at: chrono::Utc::now(),
                                assigned_agent: None,
                                required_capabilities: vec!["processing".to_string()],
                                estimated_duration: std::time::Duration::from_millis(50),
                                actual_duration: None,
                                dependencies: vec![],
                                metadata: serde_json::json!({}),
                            });

                            let _result = hive.assign_task(task).await;
                        }

                        hive.get_stats().await
                    });

                    handles.push(handle);
                }

                for handle in handles {
                    let _stats = handle.await.unwrap();
                }
            });
        });
    });
}

criterion_group!(
    benches,
    hive_creation_benchmark,
    swarm_coordination_benchmark,
    task_distribution_benchmark,
    swarm_communication_benchmark,
    auto_scaling_benchmark,
    swarm_intelligence_algorithms_benchmark,
    concurrent_swarm_operations_benchmark
);
criterion_main!(benches);
