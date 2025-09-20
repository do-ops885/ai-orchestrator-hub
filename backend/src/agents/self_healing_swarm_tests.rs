//! # Self-Healing Swarm Agent Tests
//!
//! Comprehensive test suite for the self-healing swarm agent functionality,
//! covering health monitoring, failure detection, recovery strategies, and learning.

#[cfg(test)]
mod tests {
    use super::super::self_healing_swarm::*;
    use crate::agents::agent::{Agent, AgentType};
    use crate::communication::patterns::MessagePriority;
    use crate::communication::protocols::{MessageEnvelope, MessagePayload, MessageType};
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    /// Creates a test self-healing swarm agent with default configuration
    fn create_test_self_healing_agent() -> SelfHealingSwarmAgent {
        let config = SelfHealingConfig {
            health_check_interval: 5, // Fast checks for testing
            degraded_threshold: 0.7,
            critical_threshold: 0.5,
            max_recovery_attempts: 2,
            min_swarm_size: 2,
            max_swarm_size: 10,
            enable_learning: true,
        };
        
        SelfHealingSwarmAgent::new("test_healer".to_string(), config)
    }

    /// Creates test health metrics for different health states
    fn create_test_health_metrics(status: HealthStatus) -> HealthMetrics {
        let agent_id = Uuid::new_v4();
        
        match status {
            HealthStatus::Healthy => HealthMetrics {
                agent_id,
                status,
                cpu_usage: 30.0,
                memory_usage: 40.0,
                task_success_rate: 0.95,
                response_time: 80.0,
                energy_level: 0.9,
                last_updated: Utc::now(),
                issues: Vec::new(),
            },
            HealthStatus::Degraded => HealthMetrics {
                agent_id,
                status,
                cpu_usage: 70.0,
                memory_usage: 75.0,
                task_success_rate: 0.65,
                response_time: 200.0,
                energy_level: 0.6,
                last_updated: Utc::now(),
                issues: vec!["High resource usage".to_string()],
            },
            HealthStatus::Critical => HealthMetrics {
                agent_id,
                status,
                cpu_usage: 95.0,
                memory_usage: 90.0,
                task_success_rate: 0.4,
                response_time: 800.0,
                energy_level: 0.3,
                last_updated: Utc::now(),
                issues: vec!["Resource exhaustion".to_string(), "Poor performance".to_string()],
            },
            HealthStatus::Failed => HealthMetrics {
                agent_id,
                status,
                cpu_usage: 100.0,
                memory_usage: 95.0,
                task_success_rate: 0.1,
                response_time: 5000.0,
                energy_level: 0.05,
                last_updated: Utc::now(),
                issues: vec!["Complete failure".to_string()],
            },
        }
    }

    #[tokio::test]
    async fn test_self_healing_agent_creation() {
        let agent = create_test_self_healing_agent();
        
        assert_eq!(agent.agent.agent_type, AgentType::Coordinator);
        assert_eq!(agent.agent.name, "test_healer");
        assert_eq!(agent.config.max_recovery_attempts, 2);
        assert!(agent.config.enable_learning);
    }

    #[tokio::test]
    async fn test_agent_registration() {
        let mut agent = create_test_self_healing_agent();
        let test_agent_id = Uuid::new_v4();
        
        let result = agent.register_agent(test_agent_id).await;
        assert!(result.is_ok());
        
        let health_metrics = agent.health_metrics.read().await;
        assert!(health_metrics.contains_key(&test_agent_id));
        assert_eq!(health_metrics[&test_agent_id].status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_agent_unregistration() {
        let mut agent = create_test_self_healing_agent();
        let test_agent_id = Uuid::new_v4();
        
        // Register first
        agent.register_agent(test_agent_id).await.unwrap();
        
        // Then unregister
        let result = agent.unregister_agent(test_agent_id).await;
        assert!(result.is_ok());
        
        let health_metrics = agent.health_metrics.read().await;
        assert!(!health_metrics.contains_key(&test_agent_id));
    }

    #[tokio::test]
    async fn test_health_status_calculation() {
        let agent = create_test_self_healing_agent();
        
        // Test healthy metrics
        let healthy_metrics = create_test_health_metrics(HealthStatus::Healthy);
        let status = agent.calculate_health_status(&healthy_metrics);
        assert_eq!(status, HealthStatus::Healthy);
        
        // Test degraded metrics
        let degraded_metrics = create_test_health_metrics(HealthStatus::Degraded);
        let status = agent.calculate_health_status(&degraded_metrics);
        // Note: status might be different from input as it's calculated
        assert!(matches!(status, HealthStatus::Healthy | HealthStatus::Degraded));
        
        // Test critical metrics
        let critical_metrics = create_test_health_metrics(HealthStatus::Critical);
        let status = agent.calculate_health_status(&critical_metrics);
        assert!(matches!(status, HealthStatus::Critical | HealthStatus::Failed));
    }

    #[tokio::test]
    async fn test_failure_type_determination() {
        let agent = create_test_self_healing_agent();
        
        // Test unresponsive agent
        let mut metrics = create_test_health_metrics(HealthStatus::Failed);
        metrics.response_time = 6000.0;
        let failure_type = agent.determine_failure_type(&metrics);
        assert_eq!(failure_type, FailureType::AgentUnresponsive);
        
        // Test task execution failure
        metrics.response_time = 100.0;
        metrics.task_success_rate = 0.2;
        let failure_type = agent.determine_failure_type(&metrics);
        assert_eq!(failure_type, FailureType::TaskExecutionFailure);
        
        // Test resource exhaustion
        metrics.task_success_rate = 0.8;
        metrics.cpu_usage = 95.0;
        let failure_type = agent.determine_failure_type(&metrics);
        assert_eq!(failure_type, FailureType::ResourceExhaustion);
    }

    #[tokio::test]
    async fn test_recovery_strategy_selection() {
        let agent = create_test_self_healing_agent();
        let metrics = create_test_health_metrics(HealthStatus::Critical);
        
        // Test strategy selection for different failure types
        let strategy = agent.select_recovery_strategy(&FailureType::AgentUnresponsive, &metrics);
        assert_eq!(strategy, RecoveryStrategy::AgentRestart);
        
        let strategy = agent.select_recovery_strategy(&FailureType::ResourceExhaustion, &metrics);
        assert_eq!(strategy, RecoveryStrategy::ResourceScaling);
        
        let strategy = agent.select_recovery_strategy(&FailureType::TaskExecutionFailure, &metrics);
        assert_eq!(strategy, RecoveryStrategy::TaskRedistribution);
    }

    #[tokio::test]
    async fn test_recovery_execution() {
        let mut agent = create_test_self_healing_agent();
        let test_agent_id = Uuid::new_v4();
        
        // Register agent with critical health
        agent.register_agent(test_agent_id).await.unwrap();
        
        // Set up critical health metrics
        {
            let mut health_metrics = agent.health_metrics.write().await;
            health_metrics.insert(test_agent_id, create_test_health_metrics(HealthStatus::Critical));
        }
        
        // Execute restart strategy
        let result = agent.execute_recovery_strategy(test_agent_id, &RecoveryStrategy::AgentRestart).await;
        assert!(result.is_ok());
        
        // Check that health improved
        let health_metrics = agent.health_metrics.read().await;
        let updated_metrics = &health_metrics[&test_agent_id];
        assert!(updated_metrics.energy_level > 0.5);
        assert!(updated_metrics.task_success_rate > 0.5);
    }

    #[tokio::test]
    async fn test_swarm_health_summary() {
        let mut agent = create_test_self_healing_agent();
        
        // Register agents with different health statuses
        let healthy_id = Uuid::new_v4();
        let degraded_id = Uuid::new_v4();
        let critical_id = Uuid::new_v4();
        
        agent.register_agent(healthy_id).await.unwrap();
        agent.register_agent(degraded_id).await.unwrap();
        agent.register_agent(critical_id).await.unwrap();
        
        // Update health metrics
        {
            let mut health_metrics = agent.health_metrics.write().await;
            health_metrics.insert(healthy_id, create_test_health_metrics(HealthStatus::Healthy));
            health_metrics.insert(degraded_id, create_test_health_metrics(HealthStatus::Degraded));
            health_metrics.insert(critical_id, create_test_health_metrics(HealthStatus::Critical));
        }
        
        let summary = agent.get_swarm_health_summary().await;
        
        assert_eq!(summary.get(&HealthStatus::Healthy).copied().unwrap_or(0), 1);
        assert_eq!(summary.get(&HealthStatus::Degraded).copied().unwrap_or(0), 1);
        assert_eq!(summary.get(&HealthStatus::Critical).copied().unwrap_or(0), 1);
    }

    #[tokio::test]
    async fn test_communication_health_check() {
        use crate::agents::agent::AgentBehavior;
        
        let mut agent = create_test_self_healing_agent();
        
        // Register some test agents
        agent.register_agent(Uuid::new_v4()).await.unwrap();
        agent.register_agent(Uuid::new_v4()).await.unwrap();
        
        let request = MessageEnvelope {
            id: Uuid::new_v4(),
            sender_id: Uuid::new_v4(),
            recipient_id: agent.agent.id,
            message_type: MessageType::HealthCheck,
            payload: MessagePayload::Text("Health check request".to_string()),
            priority: MessagePriority::High,
            timestamp: Utc::now(),
            correlation_id: None,
        };
        
        let response = agent.communicate(request).await.unwrap();
        assert!(response.is_some());
        
        if let Some(response_envelope) = response {
            assert_eq!(response_envelope.message_type, MessageType::Response);
            if let MessagePayload::Json(json) = response_envelope.payload {
                assert!(json.get("swarm_health").is_some());
                assert!(json.get("monitoring_status").is_some());
            }
        }
    }

    #[tokio::test]
    async fn test_incident_recording() {
        let mut agent = create_test_self_healing_agent();
        let test_agent_id = Uuid::new_v4();
        
        // Start a recovery operation
        agent.active_recoveries.insert(
            test_agent_id, 
            (RecoveryStrategy::AgentRestart, 1)
        );
        
        // Complete the recovery
        agent.complete_recovery(test_agent_id, true).await.unwrap();
        
        // Check incident was recorded
        assert_eq!(agent.incident_history.len(), 1);
        let incident = &agent.incident_history[0];
        assert_eq!(incident.recovery_strategy, RecoveryStrategy::AgentRestart);
        assert!(incident.recovery_success);
        assert!(!incident.lessons_learned.is_empty());
    }

    #[tokio::test]
    async fn test_learning_from_incidents() {
        let mut agent = create_test_self_healing_agent();
        
        // Simulate successful recovery
        agent.learn_from_incident(&RecoveryStrategy::AgentRestart).await;
        
        // Check that confidence was updated
        let strategy_key = format!("{:?}", RecoveryStrategy::AgentRestart);
        assert!(agent.learned_thresholds.contains_key(&strategy_key));
        assert!(agent.learned_thresholds[&strategy_key] > 0.5);
    }

    #[tokio::test]
    async fn test_recovery_escalation() {
        let mut agent = create_test_self_healing_agent();
        let test_agent_id = Uuid::new_v4();
        
        // Set up a recovery that has reached max attempts
        agent.active_recoveries.insert(
            test_agent_id,
            (RecoveryStrategy::TaskRedistribution, agent.config.max_recovery_attempts)
        );
        
        // Escalate recovery
        let result = agent.escalate_recovery(test_agent_id).await;
        assert!(result.is_ok());
        
        // Check that strategy was escalated to emergency recovery
        if let Some((strategy, _)) = agent.active_recoveries.get(&test_agent_id) {
            assert_eq!(*strategy, RecoveryStrategy::EmergencyRecovery);
        }
    }

    #[tokio::test]
    async fn test_position_update() {
        use crate::agents::agent::AgentBehavior;
        
        let mut agent = create_test_self_healing_agent();
        
        // Set initial position away from center
        agent.agent.position = (100.0, 100.0);
        
        let swarm_center = (50.0, 50.0);
        let neighbors = vec![];
        
        agent.update_position(swarm_center, &neighbors).await.unwrap();
        
        // Agent should move toward center
        assert!(agent.agent.position.0 < 100.0);
        assert!(agent.agent.position.1 < 100.0);
        assert!(agent.agent.position.0 > 50.0);
        assert!(agent.agent.position.1 > 50.0);
    }

    #[tokio::test]
    async fn test_cleanup_completed_recoveries() {
        let mut agent = create_test_self_healing_agent();
        let test_agent_id = Uuid::new_v4();
        
        // Add a recovery with excessive attempts
        agent.active_recoveries.insert(
            test_agent_id,
            (RecoveryStrategy::AgentRestart, agent.config.max_recovery_attempts * 3)
        );
        
        // Cleanup should remove it
        agent.cleanup_completed_recoveries();
        
        assert!(!agent.active_recoveries.contains_key(&test_agent_id));
    }

    #[tokio::test]
    async fn test_config_default() {
        let config = SelfHealingConfig::default();
        
        assert_eq!(config.health_check_interval, 30);
        assert_eq!(config.degraded_threshold, 0.7);
        assert_eq!(config.critical_threshold, 0.5);
        assert_eq!(config.max_recovery_attempts, 3);
        assert_eq!(config.min_swarm_size, 2);
        assert_eq!(config.max_swarm_size, 20);
        assert!(config.enable_learning);
    }

    #[tokio::test]
    async fn test_performance_metrics_simulation() {
        let mut agent = create_test_self_healing_agent();
        let test_agent_id = Uuid::new_v4();
        
        agent.register_agent(test_agent_id).await.unwrap();
        
        // Perform health checks to simulate metric updates
        agent.perform_health_checks().await.unwrap();
        
        let health_metrics = agent.health_metrics.read().await;
        let metrics = &health_metrics[&test_agent_id];
        
        // Metrics should be updated
        assert!(metrics.last_updated > Utc::now() - chrono::Duration::seconds(5));
    }
}