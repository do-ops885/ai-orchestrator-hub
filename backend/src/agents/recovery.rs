use crate::agents::agent::{Agent, AgentMemory, AgentState};
use crate::utils::error::{HiveError, HiveResult};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

pub struct AgentRecoveryManager {
    max_retry_attempts: u32,
    base_retry_delay: Duration,
    max_retry_delay: Duration,
}

impl AgentRecoveryManager {
    pub fn new() -> Self {
        Self {
            max_retry_attempts: 3,
            base_retry_delay: Duration::from_millis(100),
            max_retry_delay: Duration::from_secs(5),
        }
    }

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

        let mut attempts = 0;
        let mut delay = self.base_retry_delay;

        while attempts < self.max_retry_attempts {
            match self.attempt_recovery(agent).await {
                Ok(()) => {
                    agent.state = AgentState::Idle;
                    info!(
                        "Successfully recovered agent {} after {} attempts",
                        agent.id,
                        attempts + 1
                    );
                    return Ok(());
                }
                Err(e) => {
                    attempts += 1;
                    warn!(
                        "Recovery attempt {} failed for agent {}: {}",
                        attempts, agent.id, e
                    );

                    if attempts >= self.max_retry_attempts {
                        error!(
                            "Recovery failed for agent {} after {} attempts",
                            agent.id, attempts
                        );
                        return Err(HiveError::AgentExecutionFailed {
                            reason: format!("Recovery failed after {} attempts: {}", attempts, e),
                        });
                    }

                    sleep(delay).await;
                    delay = std::cmp::min(delay * 2, self.max_retry_delay);
                }
            }
        }

        Err(HiveError::AgentExecutionFailed {
            reason: "Recovery exhausted".to_string(),
        })
    }

    async fn attempt_recovery(&self, agent: &mut Agent) -> HiveResult<()> {
        // Reset agent state
        agent.energy = agent.energy.max(0.1); // Minimum energy to function

        // Clear any stuck operations by resetting state
        if matches!(agent.state, AgentState::Failed) {
            agent.state = AgentState::Idle;
        }

        // Reset capabilities if they've been corrupted
        self.validate_and_repair_capabilities(agent)?;

        // Validate agent can perform basic operations
        self.validate_agent_health(agent).await?;

        info!("Agent {} recovery attempt successful", agent.id);
        Ok(())
    }

    fn validate_and_repair_capabilities(&self, agent: &mut Agent) -> HiveResult<()> {
        // Remove any capabilities with invalid proficiency
        agent.capabilities.retain(|cap| {
            cap.proficiency >= 0.0 && cap.proficiency <= 1.0 && cap.learning_rate >= 0.0
        });

        // If no capabilities remain, add a basic one
        if agent.capabilities.is_empty() {
            use crate::agents::agent::AgentCapability;
            agent.capabilities.push(AgentCapability {
                name: "basic_processing".to_string(),
                proficiency: 0.5,
                learning_rate: 0.1,
            });
        }

        Ok(())
    }

    async fn validate_agent_health(&self, agent: &Agent) -> HiveResult<()> {
        // Basic health checks
        if agent.energy <= 0.0 {
            return Err(HiveError::AgentExecutionFailed {
                reason: "No energy".to_string(),
            });
        }

        if agent.capabilities.is_empty() {
            return Err(HiveError::AgentExecutionFailed {
                reason: "No capabilities".to_string(),
            });
        }

        // Validate position is reasonable
        if agent.position.0.is_nan() || agent.position.1.is_nan() {
            return Err(HiveError::AgentExecutionFailed {
                reason: "Invalid position".to_string(),
            });
        }

        // Check for reasonable capability values
        for capability in &agent.capabilities {
            if capability.proficiency < 0.0 || capability.proficiency > 1.0 {
                return Err(HiveError::AgentExecutionFailed {
                    reason: format!("Invalid capability proficiency: {}", capability.proficiency),
                });
            }
        }

        Ok(())
    }

    pub async fn emergency_reset(&self, agent: &mut Agent) -> HiveResult<()> {
        warn!("Performing emergency reset for agent {}", agent.id);

        // Reset to safe defaults
        agent.energy = 0.5;
        agent.state = AgentState::Idle;
        agent.position = (0.0, 0.0);

        // Clear and rebuild capabilities
        agent.capabilities.clear();
        use crate::agents::agent::AgentCapability;
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

    pub fn can_recover(&self, agent: &Agent) -> bool {
        // Check if agent is in a recoverable state
        match agent.state {
            AgentState::Failed => true,
            AgentState::Idle
            | AgentState::Working
            | AgentState::Learning
            | AgentState::Communicating => false,
        }
    }

    pub async fn diagnose_agent(&self, agent: &Agent) -> Vec<String> {
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

        let issues = recovery_manager.diagnose_agent(&agent).await;
        assert!(issues.len() >= 2); // Should detect low energy and invalid position
    }
}
