use crate::agent_error;
use crate::agents::agent::{Agent, AgentBehavior, AgentState, CommunicationComplexity};
use crate::communication::patterns::CommunicationConfig;
use crate::communication::protocols::{MessageEnvelope, MessagePayload, MessageType};
use crate::neural::NLPProcessor;
use crate::utils::error::{HiveError, HiveResult};
use crate::utils::error_recovery::ContextAwareRecovery;
use async_trait::async_trait;
use std::time::Duration;

use tracing::{debug, error, info, warn};

pub struct AgentRecoveryManager {
    max_retry_attempts: u32,
    base_retry_delay: Duration,
    max_retry_delay: Duration,
}

#[async_trait]
impl AgentBehavior for AgentRecoveryManager {
    async fn execute_task(
        &mut self,
        task: crate::tasks::Task,
    ) -> HiveResult<crate::tasks::TaskResult> {
        // Recovery managers don't execute tasks directly
        Err(crate::utils::error::HiveError::AgentExecutionFailed {
            reason: "AgentRecoveryManager does not execute tasks directly".to_string(),
        })
    }

    async fn communicate(
        &mut self,
        envelope: MessageEnvelope,
    ) -> HiveResult<Option<MessageEnvelope>> {
        // Standardized communication pattern for agent recovery
        let complexity = match envelope.priority {
            crate::communication::patterns::MessagePriority::Low => CommunicationComplexity::Simple,
            crate::communication::patterns::MessagePriority::Normal => {
                CommunicationComplexity::Standard
            }
            crate::communication::patterns::MessagePriority::High => {
                CommunicationComplexity::Complex
            }
            crate::communication::patterns::MessagePriority::Critical => {
                CommunicationComplexity::Heavy
            }
        };

        // Use standardized delay based on complexity
        let delay_ms = match complexity {
            CommunicationComplexity::Simple => 50,
            CommunicationComplexity::Standard => 100,
            CommunicationComplexity::Complex => 200,
            CommunicationComplexity::Heavy => 500,
        };

        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

        match envelope.message_type {
            MessageType::Request => {
                let response_payload = match &envelope.payload {
                    MessagePayload::Text(text) => MessagePayload::Text(format!(
                        "Agent recovery manager acknowledging: {} - Ready to recover agents",
                        text
                    )),
                    MessagePayload::Json(json) => MessagePayload::Json(serde_json::json!({
                        "response": "Agent recovery manager ready",
                        "recovery_config": {
                            "max_retry_attempts": self.max_retry_attempts,
                            "base_retry_delay_ms": self.base_retry_delay.as_millis(),
                            "max_retry_delay_ms": self.max_retry_delay.as_millis()
                        },
                        "original_request": json
                    })),
                    _ => MessagePayload::Text(
                        "Agent recovery manager acknowledged message".to_string(),
                    ),
                };

                let response = MessageEnvelope::new_response(
                    &envelope,
                    uuid::Uuid::new_v4(),
                    response_payload,
                );
                Ok(Some(response))
            }
            MessageType::Broadcast => {
                tracing::info!(
                    "Agent recovery manager received broadcast: {:?}",
                    envelope.payload
                );
                Ok(None)
            }
            MessageType::Error => {
                // Handle error recovery coordination
                if let MessagePayload::ErrorInfo { error_message, .. } = &envelope.payload {
                    tracing::info!(
                        "Received error notification for recovery coordination: {}",
                        error_message
                    );
                }
                Ok(None)
            }
            _ => {
                let response = MessageEnvelope::new_response(
                    &envelope,
                    uuid::Uuid::new_v4(),
                    MessagePayload::Text(format!(
                        "Agent recovery manager processed message of type {:?}",
                        envelope.message_type
                    )),
                );
                Ok(Some(response))
            }
        }
    }

    async fn request_response(
        &mut self,
        request: MessageEnvelope,
        timeout: std::time::Duration,
    ) -> HiveResult<MessageEnvelope> {
        // Simulate processing time for recovery coordination
        tokio::time::sleep(timeout / 4).await;

        let response = MessageEnvelope::new_response(
            &request,
            uuid::Uuid::new_v4(),
            MessagePayload::Json(serde_json::json!({
                "response": "Agent recovery manager processed request",
                "recovery_capabilities": {
                    "can_recover_failed_agents": true,
                    "supports_emergency_reset": true,
                    "max_retry_attempts": self.max_retry_attempts
                },
                "processing_timeout": timeout.as_millis()
            })),
        );

        Ok(response)
    }

    async fn learn(&mut self, _nlp_processor: &NLPProcessor) -> HiveResult<()> {
        // Recovery manager learning could involve improving recovery strategies
        debug!("Agent recovery manager learning triggered");
        Ok(())
    }

    async fn update_position(
        &mut self,
        _swarm_center: (f64, f64),
        _neighbors: &[Agent],
    ) -> HiveResult<()> {
        // Recovery managers don't participate in swarm positioning
        Ok(())
    }

    fn get_communication_config(&self) -> CommunicationConfig {
        CommunicationConfig {
            default_timeout: std::time::Duration::from_secs(10),
            max_retries: self.max_retry_attempts,
            retry_delay: self.base_retry_delay,
            max_concurrent_messages: 25,
            buffer_size: 1024,
            enable_compression: false,
            delivery_guarantee: crate::communication::patterns::DeliveryGuarantee::AtLeastOnce,
        }
    }
}

impl AgentRecoveryManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            max_retry_attempts: 3,
            base_retry_delay: Duration::from_millis(100),
            max_retry_delay: Duration::from_secs(5),
        }
    }

    #[must_use]
    pub fn with_config(
        max_retry_attempts: u32,
        base_retry_delay: Duration,
        max_retry_delay: Duration,
    ) -> Self {
        Self {
            max_retry_attempts,
            base_retry_delay,
            max_retry_delay,
        }
    }

    pub async fn recover_agent(&self, agent: &mut Agent) -> HiveResult<()> {
        info!("Starting recovery for agent {}", agent.id);

        // Use agent-specific recovery with enhanced error handling
        let result = Self::attempt_recovery(agent);

        match result {
            Ok(()) => {
                agent.state = AgentState::Idle;
                info!(
                    "Successfully recovered agent {} using enhanced recovery",
                    agent.id
                );
                Ok(())
            }
            Err(e) => {
                error!("Enhanced recovery failed for agent {}: {}", agent.id, e);
                // Try emergency reset as fallback
                self.emergency_reset(agent).await
            }
        }
    }

    fn attempt_recovery(agent: &mut Agent) -> HiveResult<()> {
        // Reset agent state
        agent.energy = agent.energy.max(0.1); // Minimum energy to function

        // Clear any stuck operations by resetting state
        if matches!(agent.state, AgentState::Failed) {
            agent.state = AgentState::Idle;
        }

        // Reset capabilities if they've been corrupted
        Self::validate_and_repair_capabilities(agent)?;

        // Validate agent can perform basic operations
        Self::validate_agent_health(agent)?;

        info!("Agent {} recovery attempt successful", agent.id);
        Ok(())
    }

    fn validate_and_repair_capabilities(agent: &mut Agent) -> HiveResult<()> {
        // Remove any capabilities with invalid proficiency
        let original_count = agent.capabilities.len();
        agent.capabilities.retain(|cap| {
            cap.proficiency >= 0.0 && cap.proficiency <= 1.0 && cap.learning_rate >= 0.0
        });

        let removed_count = original_count - agent.capabilities.len();
        if removed_count > 0 {
            warn!(
                "Removed {} corrupted capabilities from agent {}",
                removed_count, agent.id
            );
        }

        // If no capabilities remain, add a basic one
        if agent.capabilities.is_empty() {
            use crate::agents::agent::AgentCapability;
            agent.capabilities.push(AgentCapability {
                name: "basic_processing".to_string(),
                proficiency: 0.5,
                learning_rate: 0.1,
            });
            info!(
                "Added basic capability to agent {} due to corruption",
                agent.id
            );
        }

        Ok(())
    }

    fn validate_agent_health(agent: &Agent) -> HiveResult<()> {
        // Basic health checks
        if agent.energy <= 0.0 {
            return Err(agent_error!(
                resource_starvation,
                agent.id,
                "energy",
                1,
                agent.energy as u64
            ));
        }

        if agent.capabilities.is_empty() {
            return Err(agent_error!(
                adaptation_failed,
                agent.id,
                "capability_validation",
                "No capabilities available"
            ));
        }

        // Validate position is reasonable
        if agent.position.0.is_nan() || agent.position.1.is_nan() {
            return Err(HiveError::AgentExecutionFailed {
                reason: "Invalid position coordinates".to_string(),
            });
        }

        // Check for reasonable capability values
        for capability in &agent.capabilities {
            if capability.proficiency < 0.0 || capability.proficiency > 1.0 {
                return Err(agent_error!(
                    skill_evolution_failed,
                    agent.id,
                    &capability.name,
                    "Invalid proficiency value"
                ));
            }
            if capability.learning_rate < 0.0 {
                return Err(agent_error!(
                    learning_failed,
                    agent.id,
                    "Invalid learning rate"
                ));
            }
        }

        Ok(())
    }

    pub async fn emergency_reset(&self, agent: &mut Agent) -> HiveResult<()> {
        use crate::agents::agent::AgentCapability;
        warn!("Performing emergency reset for agent {}", agent.id);

        // Use recovery mechanism for emergency reset
        let _recovery = ContextAwareRecovery::new();

        // Reset to safe defaults
        agent.energy = 0.5;
        agent.state = AgentState::Idle;
        agent.position = (0.0, 0.0);

        // Clear and rebuild capabilities
        agent.capabilities.clear();
        agent.capabilities.push(AgentCapability {
            name: "basic_processing".to_string(),
            proficiency: 0.3,
            learning_rate: 0.05,
        });

        // Reset counters
        agent.memory.experiences.clear();
        agent.memory.social_connections.clear();

        info!("Emergency reset completed for agent {}", agent.id);
        Ok(())
    }

    #[must_use]
    pub fn can_recover(agent: &Agent) -> bool {
        // Check if agent is in a recoverable state
        match agent.state {
            AgentState::Failed | AgentState::Inactive => true,
            AgentState::Idle
            | AgentState::Working
            | AgentState::Learning
            | AgentState::Communicating => false,
        }
    }

    #[must_use]
    pub fn diagnose_agent(&self, agent: &Agent) -> Vec<String> {
        let mut issues = Vec::new();

        if agent.energy <= 0.1 {
            issues.push("Low energy level".to_string());
        }

        if agent.capabilities.is_empty() {
            issues.push("No capabilities available".to_string());
        }

        if agent.position.0.is_nan() || agent.position.1.is_nan() {
            issues.push("Invalid position coordinates".to_string());
        }

        for (i, capability) in agent.capabilities.iter().enumerate() {
            if capability.proficiency < 0.0 || capability.proficiency > 1.0 {
                issues.push(format!(
                    "Invalid proficiency in capability {}: {}",
                    i, capability.proficiency
                ));
            }
            if capability.learning_rate < 0.0 {
                issues.push(format!(
                    "Invalid learning rate in capability {}: {}",
                    i, capability.learning_rate
                ));
            }
        }

        if matches!(agent.state, AgentState::Failed) {
            issues.push("Agent is in failed state".to_string());
        }

        issues
    }
}

impl Default for AgentRecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::agent::{AgentCapability, AgentType};
    use crate::agents::{Agent, AgentMemory, AgentState};
    use uuid::Uuid;

    fn create_test_agent() -> Agent {
        Agent {
            id: Uuid::new_v4(),
            name: "test_agent".to_string(),
            agent_type: AgentType::Worker,
            state: AgentState::Failed,
            capabilities: vec![AgentCapability {
                name: "test_capability".to_string(),
                proficiency: 0.8,
                learning_rate: 0.1,
            }],
            position: (1.0, 2.0),
            energy: 0.0,
            memory: AgentMemory::new(),
            created_at: chrono::Utc::now(),
            last_active: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_agent_recovery_success() {
        let recovery_manager = AgentRecoveryManager::new();
        let mut agent = create_test_agent();
        agent.energy = 0.5; // Give some energy

        let result = recovery_manager.recover_agent(&mut agent).await;
        assert!(result.is_ok());
        assert_eq!(agent.state, AgentState::Idle);
    }

    #[tokio::test]
    async fn test_agent_recovery_with_no_energy() {
        let recovery_manager = AgentRecoveryManager::new();
        let mut agent = create_test_agent();
        agent.energy = 0.0;

        let result = recovery_manager.recover_agent(&mut agent).await;
        assert!(result.is_ok()); // Should succeed because recovery sets minimum energy
        assert!(agent.energy >= 0.1);
    }

    #[tokio::test]
    async fn test_emergency_reset() {
        let recovery_manager = AgentRecoveryManager::new();
        let mut agent = create_test_agent();
        agent.position = (f64::NAN, f64::NAN);

        let result = recovery_manager.emergency_reset(&mut agent).await;
        assert!(result.is_ok());
        assert_eq!(agent.state, AgentState::Idle);
        assert_eq!(agent.position, (0.0, 0.0));
        assert!(!agent.capabilities.is_empty());
    }

    #[tokio::test]
    async fn test_diagnose_agent() {
        let recovery_manager = AgentRecoveryManager::new();
        let mut agent = create_test_agent();
        agent.energy = 0.05; // Low energy
        agent.position = (f64::NAN, 1.0); // Invalid position

        let issues = recovery_manager.diagnose_agent(&agent);
        assert!(issues.len() >= 2); // Should detect low energy and invalid position
    }
}
