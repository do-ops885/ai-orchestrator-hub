//! # Self-Healing Swarm Agent Demonstration
//!
//! This example demonstrates the capabilities of the self-healing swarm agent
//! including health monitoring, failure detection, and automatic recovery.

use multiagent_hive::agents::{
    Agent, AgentType, FailureType, HealthStatus, RecoveryStrategy, SelfHealingConfig,
    SelfHealingSwarmAgent,
};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::init();

    info!("🚀 Starting Self-Healing Swarm Agent Demonstration");

    // Create a self-healing swarm agent with custom configuration
    let config = SelfHealingConfig {
        health_check_interval: 10, // Check every 10 seconds
        degraded_threshold: 0.7,
        critical_threshold: 0.5,
        max_recovery_attempts: 3,
        min_swarm_size: 3,
        max_swarm_size: 15,
        enable_learning: true,
    };

    let mut healing_agent = SelfHealingSwarmAgent::new("SwarmHealer-Alpha".to_string(), config);

    info!(
        "✅ Self-healing agent created: {}",
        healing_agent.agent.name
    );

    // Create and register several worker agents for monitoring
    let worker_agents = create_worker_swarm(5).await;
    for agent in &worker_agents {
        healing_agent.register_agent(agent.id).await?;
        info!("📝 Registered agent: {} ({})", agent.name, agent.id);
    }

    // Display initial swarm health
    display_swarm_health(&healing_agent).await;

    // Simulate normal operations for a short period
    info!("🔄 Simulating normal operations...");
    sleep(Duration::from_secs(5)).await;

    // Simulate some agent failures
    info!("⚠️  Simulating agent failures for demonstration...");
    simulate_agent_failures(&mut healing_agent).await?;

    // Display health after failures
    display_swarm_health(&healing_agent).await;

    // Demonstrate recovery process
    info!("🔧 Demonstrating recovery process...");
    demonstrate_recovery_process(&mut healing_agent).await?;

    // Show final health status
    display_swarm_health(&healing_agent).await;

    // Display incident history
    display_incident_history(&healing_agent);

    // Demonstrate learning capabilities
    demonstrate_learning(&healing_agent);

    info!("🎉 Self-healing swarm demonstration completed successfully!");

    Ok(())
}

/// Creates a swarm of worker agents for testing
async fn create_worker_swarm(count: usize) -> Vec<Agent> {
    let mut agents = Vec::new();

    for i in 0..count {
        let agent = Agent::new(format!("Worker-{}", i + 1), AgentType::Worker);
        agents.push(agent);
    }

    info!("👥 Created swarm of {} worker agents", count);
    agents
}

/// Displays current swarm health summary
async fn display_swarm_health(healing_agent: &SelfHealingSwarmAgent) {
    let health_summary = healing_agent.get_swarm_health_summary().await;

    info!("📊 === Swarm Health Summary ===");
    for (status, count) in health_summary {
        let status_emoji = match status {
            HealthStatus::Healthy => "✅",
            HealthStatus::Degraded => "⚠️",
            HealthStatus::Critical => "🔴",
            HealthStatus::Failed => "💀",
        };
        info!("   {} {:?}: {} agents", status_emoji, status, count);
    }

    let active_recoveries = healing_agent.active_recoveries.len();
    if active_recoveries > 0 {
        info!("🔧 Active recoveries: {}", active_recoveries);
    }

    info!(
        "📈 Total incidents recorded: {}",
        healing_agent.incident_history.len()
    );
}

/// Simulates various types of agent failures
async fn simulate_agent_failures(
    healing_agent: &mut SelfHealingSwarmAgent,
) -> Result<(), Box<dyn std::error::Error>> {
    let agent_ids: Vec<Uuid> = {
        let health_metrics = healing_agent.health_metrics.read().await;
        health_metrics.keys().cloned().collect()
    };

    if agent_ids.len() < 3 {
        warn!("Not enough agents to simulate failures");
        return Ok(());
    }

    // Simulate different types of failures
    info!("💥 Simulating unresponsive agent...");
    simulate_unresponsive_agent(healing_agent, agent_ids[0]).await;

    info!("📉 Simulating performance degradation...");
    simulate_performance_degradation(healing_agent, agent_ids[1]).await;

    info!("🔥 Simulating resource exhaustion...");
    simulate_resource_exhaustion(healing_agent, agent_ids[2]).await;

    sleep(Duration::from_secs(2)).await;
    Ok(())
}

/// Simulates an unresponsive agent
async fn simulate_unresponsive_agent(healing_agent: &mut SelfHealingSwarmAgent, agent_id: Uuid) {
    let mut health_metrics = healing_agent.health_metrics.write().await;
    if let Some(metrics) = health_metrics.get_mut(&agent_id) {
        metrics.response_time = 8000.0; // Very high response time
        metrics.status = HealthStatus::Failed;
        metrics.task_success_rate = 0.0;
        warn!("Agent {} is now unresponsive", agent_id);
    }
}

/// Simulates performance degradation
async fn simulate_performance_degradation(
    healing_agent: &mut SelfHealingSwarmAgent,
    agent_id: Uuid,
) {
    let mut health_metrics = healing_agent.health_metrics.write().await;
    if let Some(metrics) = health_metrics.get_mut(&agent_id) {
        metrics.task_success_rate = 0.3; // Low success rate
        metrics.energy_level = 0.2; // Low energy
        metrics.status = HealthStatus::Critical;
        warn!("Agent {} is experiencing performance degradation", agent_id);
    }
}

/// Simulates resource exhaustion
async fn simulate_resource_exhaustion(healing_agent: &mut SelfHealingSwarmAgent, agent_id: Uuid) {
    let mut health_metrics = healing_agent.health_metrics.write().await;
    if let Some(metrics) = health_metrics.get_mut(&agent_id) {
        metrics.cpu_usage = 98.0; // High CPU usage
        metrics.memory_usage = 95.0; // High memory usage
        metrics.status = HealthStatus::Critical;
        warn!("Agent {} is experiencing resource exhaustion", agent_id);
    }
}

/// Demonstrates the recovery process
async fn demonstrate_recovery_process(
    healing_agent: &mut SelfHealingSwarmAgent,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔍 Analyzing health and initiating recovery...");

    // Perform health analysis and recovery
    healing_agent.analyze_and_recover().await?;

    sleep(Duration::from_secs(3)).await;

    // Simulate some recovery completions
    let agent_ids: Vec<Uuid> = {
        let health_metrics = healing_agent.health_metrics.read().await;
        health_metrics.keys().cloned().take(2).collect()
    };

    for agent_id in agent_ids {
        if healing_agent.active_recoveries.contains_key(&agent_id) {
            info!("✅ Simulating successful recovery for agent {}", agent_id);
            healing_agent.complete_recovery(agent_id, true).await?;

            // Restore agent health
            let mut health_metrics = healing_agent.health_metrics.write().await;
            if let Some(metrics) = health_metrics.get_mut(&agent_id) {
                metrics.status = HealthStatus::Healthy;
                metrics.task_success_rate = 0.9;
                metrics.response_time = 100.0;
                metrics.cpu_usage = 30.0;
                metrics.memory_usage = 40.0;
                metrics.energy_level = 0.9;
            }
        }
    }

    Ok(())
}

/// Displays incident history for analysis
fn display_incident_history(healing_agent: &SelfHealingSwarmAgent) {
    let incidents = healing_agent.get_incident_history();

    if incidents.is_empty() {
        info!("📝 No incidents recorded yet");
        return;
    }

    info!("📚 === Incident History ===");
    for (i, incident) in incidents.iter().enumerate() {
        let success_emoji = if incident.recovery_success {
            "✅"
        } else {
            "❌"
        };
        info!(
            "   {}. {} {:?} → {:?} {}",
            i + 1,
            success_emoji,
            incident.failure_type,
            incident.recovery_strategy,
            if incident.recovery_success {
                "SUCCESS"
            } else {
                "FAILED"
            }
        );

        if !incident.lessons_learned.is_empty() {
            info!("      Lessons: {}", incident.lessons_learned.join(", "));
        }
    }
}

/// Demonstrates learning capabilities
fn demonstrate_learning(healing_agent: &SelfHealingSwarmAgent) {
    info!("🧠 === Learning Summary ===");

    if healing_agent.learned_thresholds.is_empty() {
        info!("   No learning patterns established yet");
        return;
    }

    for (pattern, confidence) in &healing_agent.learned_thresholds {
        let confidence_level = if *confidence > 0.8 {
            "HIGH"
        } else if *confidence > 0.6 {
            "MEDIUM"
        } else {
            "LOW"
        };

        info!(
            "   {} → {} confidence ({:.2})",
            pattern, confidence_level, confidence
        );
    }
}

/// Example of integrating with existing swarm systems
async fn integrate_with_swarm_intelligence(
    healing_agent: &mut SelfHealingSwarmAgent,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔗 Demonstrating integration with swarm intelligence...");

    // Create some test agents for swarm formation
    let agents = create_worker_swarm(6).await;

    // Create a mock task for optimization
    use multiagent_hive::tasks::{RequiredCapability, Task, TaskPriority};

    let task = Task {
        id: Uuid::new_v4(),
        name: "Test Swarm Task".to_string(),
        description: "A test task for swarm formation optimization".to_string(),
        task_type: "swarm_coordination".to_string(),
        priority: TaskPriority::High,
        required_capabilities: vec![RequiredCapability {
            name: "coordination".to_string(),
            minimum_proficiency: 0.7,
        }],
        created_at: chrono::Utc::now(),
        deadline: None,
        dependencies: Vec::new(),
    };

    // Optimize formation using swarm intelligence
    match healing_agent
        .swarm_engine
        .optimize_formation(&agents, &task)
        .await
    {
        Ok(formation) => {
            info!(
                "✅ Optimal formation created: {} agents in {:?} formation",
                formation.agents.len(),
                formation.formation_type
            );
        }
        Err(e) => {
            error!("❌ Formation optimization failed: {}", e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod demo_tests {
    use super::*;

    #[tokio::test]
    async fn test_demo_functions() {
        // Test worker swarm creation
        let agents = create_worker_swarm(3).await;
        assert_eq!(agents.len(), 3);
        assert_eq!(agents[0].agent_type, AgentType::Worker);
    }

    #[tokio::test]
    async fn test_healing_agent_creation() {
        let config = SelfHealingConfig::default();
        let agent = SelfHealingSwarmAgent::new("test".to_string(), config);
        assert_eq!(agent.agent.agent_type, AgentType::Coordinator);
    }
}
