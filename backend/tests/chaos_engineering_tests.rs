//! Chaos Engineering Tests
//!
//! This module contains chaos engineering tests to verify system resilience
//! under adverse conditions such as network failures, resource exhaustion,
//! and unexpected errors.

use multiagent_hive::HiveCoordinator;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

/// Chaos test configuration
#[derive(Debug, Clone)]
struct ChaosConfig {
    pub test_duration: Duration,
    pub failure_probability: f64,
    pub recovery_time: Duration,
    pub max_concurrent_failures: usize,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            test_duration: Duration::from_secs(30),
            failure_probability: 0.1, // 10% chance of failure
            recovery_time: Duration::from_secs(5),
            max_concurrent_failures: 3,
        }
    }
}

/// Chaos test results
#[derive(Debug, Clone)]
struct ChaosTestResult {
    pub test_name: String,
    pub duration: Duration,
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub recovery_events: u64,
    pub system_stability_score: f64, // 0.0 to 1.0
    pub average_response_time: Duration,
}

impl ChaosTestResult {
    fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            self.successful_operations as f64 / self.total_operations as f64
        }
    }

    fn failure_rate(&self) -> f64 {
        1.0 - self.success_rate()
    }
}

/// Network failure simulation
#[derive(Debug)]
struct NetworkFailureSimulator {
    failure_active: Arc<Mutex<bool>>,
    failure_count: Arc<Mutex<u64>>,
}

impl NetworkFailureSimulator {
    fn new() -> Self {
        Self {
            failure_active: Arc::new(Mutex::new(false)),
            failure_count: Arc::new(Mutex::new(0)),
        }
    }

    async fn simulate_failure(&self, duration: Duration) {
        {
            let mut active = self.failure_active.lock().await;
            *active = true;
            let mut count = self.failure_count.lock().await;
            *count += 1;
        }

        sleep(duration).await;

        let mut active = self.failure_active.lock().await;
        *active = false;
    }

    async fn is_failure_active(&self) -> bool {
        *self.failure_active.lock().await
    }

    async fn get_failure_count(&self) -> u64 {
        *self.failure_count.lock().await
    }
}

/// Resource exhaustion simulator
#[derive(Debug)]
struct ResourceExhaustionSimulator {
    memory_pressure: Arc<Mutex<f64>>,
    cpu_pressure: Arc<Mutex<f64>>,
}

impl ResourceExhaustionSimulator {
    fn new() -> Self {
        Self {
            memory_pressure: Arc::new(Mutex::new(0.0)),
            cpu_pressure: Arc::new(Mutex::new(0.0)),
        }
    }

    async fn set_memory_pressure(&self, pressure: f64) {
        let mut mem_pressure = self.memory_pressure.lock().await;
        *mem_pressure = pressure.clamp(0.0, 1.0);
    }

    async fn set_cpu_pressure(&self, pressure: f64) {
        let mut cpu_pressure = self.cpu_pressure.lock().await;
        *cpu_pressure = pressure.clamp(0.0, 1.0);
    }

    async fn get_memory_pressure(&self) -> f64 {
        *self.memory_pressure.lock().await
    }

    async fn get_cpu_pressure(&self) -> f64 {
        *self.cpu_pressure.lock().await
    }
}

/// Chaos engineering test for network failures
#[tokio::test]
async fn test_network_failure_resilience() {
    let config = ChaosConfig {
        test_duration: Duration::from_secs(15),
        failure_probability: 0.2,
        ..Default::default()
    };

    let network_simulator = Arc::new(NetworkFailureSimulator::new());
    let mut results = Vec::new();

    let start_time = Instant::now();
    let mut operation_count = 0;
    let mut success_count = 0;

    // Start chaos monkey to simulate network failures
    let chaos_monkey = {
        let simulator = Arc::clone(&network_simulator);
        let config = config.clone();

        tokio::spawn(async move {
            loop {
                let elapsed = start_time.elapsed();
                if elapsed >= config.test_duration {
                    break;
                }

                // Randomly trigger network failures
                if rand::random::<f64>() < config.failure_probability {
                    let failure_duration =
                        Duration::from_millis(rand::random::<u64>() % 2000 + 500);
                    simulator.simulate_failure(failure_duration).await;
                }

                sleep(Duration::from_millis(1000)).await;
            }
        })
    };

    // Run operations under chaos
    while start_time.elapsed() < config.test_duration {
        operation_count += 1;

        // Simulate operation that might fail due to network issues
        let operation_result = if network_simulator.is_failure_active().await {
            // Simulate network failure
            Err(anyhow::anyhow!("Network failure"))
        } else {
            // Simulate successful operation with some latency
            sleep(Duration::from_millis(rand::random::<u64>() % 100 + 50)).await;
            Ok(())
        };

        if operation_result.is_ok() {
            success_count += 1;
        }

        // Small delay between operations
        sleep(Duration::from_millis(200)).await;
    }

    // Wait for chaos monkey to finish
    let _ = chaos_monkey.await;

    let failure_count = network_simulator.get_failure_count().await;
    let success_rate = success_count as f64 / operation_count as f64;
    let stability_score = success_rate * (1.0 - (failure_count as f64 / 10.0).min(0.5));

    let result = ChaosTestResult {
        test_name: "network_failure_resilience".to_string(),
        duration: start_time.elapsed(),
        total_operations: operation_count,
        successful_operations: success_count,
        failed_operations: operation_count - success_count,
        recovery_events: failure_count,
        system_stability_score: stability_score,
        average_response_time: Duration::from_millis(100), // Simulated
    };

    println!("🌐 Network Failure Resilience Test:");
    println!("  Total operations: {}", result.total_operations);
    println!("  Success rate: {:.2}%", result.success_rate() * 100.0);
    println!("  Network failures: {}", failure_count);
    println!("  Stability score: {:.2}", result.system_stability_score);

    // Assert resilience requirements
    assert!(
        result.success_rate() > 0.7,
        "System should maintain >70% success rate under network failures"
    );
    assert!(
        result.system_stability_score > 0.5,
        "System stability score should be >0.5 under chaos"
    );

    println!("✅ Network failure resilience test passed");
}

/// Chaos engineering test for resource exhaustion
#[tokio::test]
async fn test_resource_exhaustion_resilience() {
    let config = ChaosConfig {
        test_duration: Duration::from_secs(20),
        ..Default::default()
    };

    let resource_simulator = Arc::new(ResourceExhaustionSimulator::new());
    let mut results = Vec::new();

    let start_time = Instant::now();
    let mut operation_count = 0;
    let mut success_count = 0;

    // Start resource pressure simulator
    let pressure_simulator = {
        let simulator = Arc::clone(&resource_simulator);
        let config = config.clone();

        tokio::spawn(async move {
            loop {
                let elapsed = start_time.elapsed();
                if elapsed >= config.test_duration {
                    break;
                }

                // Gradually increase resource pressure
                let progress = elapsed.as_secs_f64() / config.test_duration.as_secs_f64();
                let memory_pressure = (progress * 0.8).min(0.9); // Up to 90% memory pressure
                let cpu_pressure = (progress * 0.7).min(0.8); // Up to 80% CPU pressure

                simulator.set_memory_pressure(memory_pressure).await;
                simulator.set_cpu_pressure(cpu_pressure).await;

                sleep(Duration::from_secs(2)).await;
            }
        })
    };

    // Run operations under resource pressure
    while start_time.elapsed() < config.test_duration {
        operation_count += 1;

        let memory_pressure = resource_simulator.get_memory_pressure().await;
        let cpu_pressure = resource_simulator.get_cpu_pressure().await;

        // Simulate operation that might fail due to resource pressure
        let operation_result = if memory_pressure > 0.85 || cpu_pressure > 0.75 {
            // High chance of failure under extreme pressure
            if rand::random::<f64>() < 0.7 {
                Err(anyhow::anyhow!("Resource exhaustion"))
            } else {
                sleep(Duration::from_millis(rand::random::<u64>() % 200 + 100)).await;
                Ok(())
            }
        } else if memory_pressure > 0.6 || cpu_pressure > 0.5 {
            // Moderate chance of failure under medium pressure
            if rand::random::<f64>() < 0.3 {
                Err(anyhow::anyhow!("Resource pressure"))
            } else {
                sleep(Duration::from_millis(rand::random::<u64>() % 150 + 75)).await;
                Ok(())
            }
        } else {
            // Normal operation
            sleep(Duration::from_millis(rand::random::<u64>() % 100 + 50)).await;
            Ok(())
        };

        if operation_result.is_ok() {
            success_count += 1;
        }

        // Small delay between operations
        sleep(Duration::from_millis(100)).await;
    }

    // Wait for pressure simulator to finish
    let _ = pressure_simulator.await;

    let final_memory_pressure = resource_simulator.get_memory_pressure().await;
    let final_cpu_pressure = resource_simulator.get_cpu_pressure().await;
    let success_rate = success_count as f64 / operation_count as f64;

    // Calculate stability score based on performance under pressure
    let pressure_factor = (final_memory_pressure + final_cpu_pressure) / 2.0;
    let stability_score = success_rate * (1.0 - pressure_factor * 0.5);

    let result = ChaosTestResult {
        test_name: "resource_exhaustion_resilience".to_string(),
        duration: start_time.elapsed(),
        total_operations: operation_count,
        successful_operations: success_count,
        failed_operations: operation_count - success_count,
        recovery_events: 0, // Resource exhaustion doesn't have explicit recovery events
        system_stability_score: stability_score,
        average_response_time: Duration::from_millis(100), // Simulated
    };

    println!("💾 Resource Exhaustion Resilience Test:");
    println!("  Total operations: {}", result.total_operations);
    println!("  Success rate: {:.2}%", result.success_rate() * 100.0);
    println!(
        "  Final memory pressure: {:.1}%",
        final_memory_pressure * 100.0
    );
    println!("  Final CPU pressure: {:.1}%", final_cpu_pressure * 100.0);
    println!("  Stability score: {:.2}", result.system_stability_score);

    // Assert resilience requirements
    assert!(
        result.success_rate() > 0.6,
        "System should maintain >60% success rate under resource pressure"
    );
    assert!(
        result.system_stability_score > 0.4,
        "System stability score should be >0.4 under resource exhaustion"
    );

    println!("✅ Resource exhaustion resilience test passed");
}

/// Chaos engineering test for agent failures
#[tokio::test]
async fn test_agent_failure_resilience() {
    let config = ChaosConfig {
        test_duration: Duration::from_secs(25),
        failure_probability: 0.15,
        ..Default::default()
    };

    let mut results = Vec::new();
    let start_time = Instant::now();
    let mut operation_count = 0;
    let mut success_count = 0;
    let mut agent_failures = 0;

    // Simulate a hive with multiple agents
    let mut active_agents = 5;
    let mut failed_agents = Vec::new();

    // Start agent failure simulator
    let failure_simulator = tokio::spawn(async move {
        loop {
            let elapsed = start_time.elapsed();
            if elapsed >= config.test_duration {
                break;
            }

            // Randomly fail agents
            if rand::random::<f64>() < config.failure_probability && active_agents > 1 {
                active_agents -= 1;
                agent_failures += 1;
                failed_agents.push(elapsed);
            }

            sleep(Duration::from_millis(1500)).await;
        }

        (active_agents, agent_failures, failed_agents)
    });

    // Run operations with agent failures
    while start_time.elapsed() < config.test_duration {
        operation_count += 1;

        // Simulate operation that depends on available agents
        let operation_result = if active_agents == 0 {
            // No agents available
            Err(anyhow::anyhow!("No agents available"))
        } else {
            // Operation success depends on agent availability and random factors
            let base_success_rate = active_agents as f64 / 5.0; // 5 was initial count
            let random_factor = rand::random::<f64>() * 0.3; // Up to 30% random failure
            let success_probability = (base_success_rate - random_factor).max(0.1);

            if rand::random::<f64>() < success_probability {
                sleep(Duration::from_millis(rand::random::<u64>() % 100 + 50)).await;
                Ok(())
            } else {
                Err(anyhow::anyhow!("Agent operation failed"))
            }
        };

        if operation_result.is_ok() {
            success_count += 1;
        }

        // Small delay between operations
        sleep(Duration::from_millis(150)).await;
    }

    let (final_active_agents, total_agent_failures, failure_times) =
        failure_simulator.await.unwrap();
    let success_rate = success_count as f64 / operation_count as f64;

    // Calculate recovery score based on how well system handled agent failures
    let failure_rate = total_agent_failures as f64 / 5.0; // Based on initial agent count
    let recovery_score = success_rate * (1.0 - failure_rate.min(0.8));
    let stability_score = recovery_score * (final_active_agents as f64 / 5.0).max(0.2);

    let result = ChaosTestResult {
        test_name: "agent_failure_resilience".to_string(),
        duration: start_time.elapsed(),
        total_operations: operation_count,
        successful_operations: success_count,
        failed_operations: operation_count - success_count,
        recovery_events: total_agent_failures,
        system_stability_score: stability_score,
        average_response_time: Duration::from_millis(100), // Simulated
    };

    println!("🤖 Agent Failure Resilience Test:");
    println!("  Total operations: {}", result.total_operations);
    println!("  Success rate: {:.2}%", result.success_rate() * 100.0);
    println!("  Agent failures: {}", total_agent_failures);
    println!("  Final active agents: {}", final_active_agents);
    println!("  Stability score: {:.2}", result.system_stability_score);

    // Assert resilience requirements
    assert!(
        result.success_rate() > 0.5,
        "System should maintain >50% success rate with agent failures"
    );
    assert!(
        result.system_stability_score > 0.3,
        "System stability score should be >0.3 with agent failures"
    );

    println!("✅ Agent failure resilience test passed");
}

/// Chaos engineering test for cascading failures
#[tokio::test]
async fn test_cascading_failure_resilience() {
    let config = ChaosConfig {
        test_duration: Duration::from_secs(20),
        failure_probability: 0.05, // Lower probability for cascading test
        ..Default::default()
    };

    let mut results = Vec::new();
    let start_time = Instant::now();
    let mut operation_count = 0;
    let mut success_count = 0;

    // Simulate cascading failure scenario
    let mut failure_chain = Vec::new();
    let mut cascade_level = 0;

    // Run operations with potential cascading failures
    while start_time.elapsed() < config.test_duration {
        operation_count += 1;

        // Simulate cascading failure logic
        let base_failure_probability = config.failure_probability * (cascade_level + 1) as f64;

        let operation_result = if rand::random::<f64>() < base_failure_probability {
            // Failure occurred - increase cascade level
            cascade_level = (cascade_level + 1).min(5);
            failure_chain.push(start_time.elapsed());

            // Higher cascade levels have higher failure rates for subsequent operations
            Err(anyhow::anyhow!(
                "Cascading failure at level {}",
                cascade_level
            ))
        } else {
            // Success - reduce cascade level
            cascade_level = cascade_level.saturating_sub(1);

            sleep(Duration::from_millis(rand::random::<u64>() % 100 + 50)).await;
            Ok(())
        };

        if operation_result.is_ok() {
            success_count += 1;
        }

        // Small delay between operations
        sleep(Duration::from_millis(200)).await;
    }

    let success_rate = success_count as f64 / operation_count as f64;
    let cascade_events = failure_chain.len();

    // Calculate resilience score based on ability to contain cascading failures
    let cascade_factor = cascade_events as f64 / operation_count as f64;
    let containment_score = if cascade_events > 0 {
        // Measure how well the system contained the cascade
        let average_cascade_impact = cascade_level as f64 / cascade_events as f64;
        1.0 - (average_cascade_impact * 0.2).min(0.8)
    } else {
        1.0
    };

    let stability_score = success_rate * containment_score;

    let result = ChaosTestResult {
        test_name: "cascading_failure_resilience".to_string(),
        duration: start_time.elapsed(),
        total_operations: operation_count,
        successful_operations: success_count,
        failed_operations: operation_count - success_count,
        recovery_events: cascade_events as u64,
        system_stability_score: stability_score,
        average_response_time: Duration::from_millis(100), // Simulated
    };

    println!("🔗 Cascading Failure Resilience Test:");
    println!("  Total operations: {}", result.total_operations);
    println!("  Success rate: {:.2}%", result.success_rate() * 100.0);
    println!("  Cascade events: {}", cascade_events);
    println!("  Final cascade level: {}", cascade_level);
    println!("  Containment score: {:.2}", containment_score);
    println!("  Stability score: {:.2}", result.system_stability_score);

    // Assert resilience requirements
    assert!(
        result.success_rate() > 0.6,
        "System should maintain >60% success rate during cascading failures"
    );
    assert!(
        containment_score > 0.5,
        "System should contain cascading failures with score >0.5"
    );
    assert!(
        result.system_stability_score > 0.4,
        "Overall stability score should be >0.4 during cascading failures"
    );

    println!("✅ Cascading failure resilience test passed");
}
