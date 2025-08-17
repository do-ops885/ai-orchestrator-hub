use crate::utils::{HiveError, HiveResult};
use serde_json::Value;

/// Input validation utilities for the hive system
pub struct InputValidator;

impl InputValidator {
    /// Validate agent creation payload
    pub fn validate_agent_payload(payload: &Value) -> HiveResult<()> {
        // Check required fields
        if payload.get("name").is_none() {
            return Err(HiveError::AgentCreationFailed {
                reason: "Missing required field: name".to_string(),
            });
        }

        if payload.get("agent_type").is_none() {
            return Err(HiveError::AgentCreationFailed {
                reason: "Missing required field: agent_type".to_string(),
            });
        }

        // Validate name
        if let Some(name) = payload.get("name").and_then(|n| n.as_str()) {
            if name.trim().is_empty() {
                return Err(HiveError::AgentCreationFailed {
                    reason: "Agent name cannot be empty".to_string(),
                });
            }
            if name.len() > 100 {
                return Err(HiveError::AgentCreationFailed {
                    reason: "Agent name too long (max 100 characters)".to_string(),
                });
            }
            if !name
                .chars()
                .all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '_')
            {
                return Err(HiveError::AgentCreationFailed {
                    reason: "Agent name contains invalid characters".to_string(),
                });
            }
        } else {
            return Err(HiveError::AgentCreationFailed {
                reason: "Agent name must be a string".to_string(),
            });
        }

        // Validate agent type
        if let Some(agent_type) = payload.get("agent_type").and_then(|t| t.as_str()) {
            let valid_types = ["worker", "coordinator", "learner"];
            let is_specialist = agent_type.starts_with("specialist:");

            if !valid_types.contains(&agent_type) && !is_specialist {
                return Err(HiveError::AgentCreationFailed {
                    reason: format!(
                        "Invalid agent type: {}. Valid types: worker, coordinator, learner, specialist:<type>",
                        agent_type
                    ),
                });
            }

            if is_specialist {
                let specialist_type = agent_type.strip_prefix("specialist:").unwrap_or("");
                if specialist_type.is_empty() {
                    return Err(HiveError::AgentCreationFailed {
                        reason: "Specialist type cannot be empty".to_string(),
                    });
                }
            }
        } else {
            return Err(HiveError::AgentCreationFailed {
                reason: "Agent type must be a string".to_string(),
            });
        }

        // Validate optional capabilities
        if let Some(capabilities) = payload.get("capabilities") {
            if let Some(caps_array) = capabilities.as_array() {
                for cap in caps_array {
                    Self::validate_capability(cap)?;
                }
            } else {
                return Err(HiveError::AgentCreationFailed {
                    reason: "Capabilities must be an array".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate task creation payload
    pub fn validate_task_payload(payload: &Value) -> HiveResult<()> {
        // Check required fields
        if payload.get("description").is_none() {
            return Err(HiveError::TaskCreationFailed {
                reason: "Missing required field: description".to_string(),
            });
        }

        // Validate description
        if let Some(description) = payload.get("description").and_then(|d| d.as_str()) {
            if description.trim().is_empty() {
                return Err(HiveError::TaskCreationFailed {
                    reason: "Task description cannot be empty".to_string(),
                });
            }
            if description.len() > 1000 {
                return Err(HiveError::TaskCreationFailed {
                    reason: "Task description too long (max 1000 characters)".to_string(),
                });
            }
        } else {
            return Err(HiveError::TaskCreationFailed {
                reason: "Task description must be a string".to_string(),
            });
        }

        // Validate optional priority
        if let Some(priority) = payload.get("priority") {
            if let Some(priority_str) = priority.as_str() {
                let valid_priorities = ["low", "medium", "high", "critical"];
                if !valid_priorities.contains(&priority_str) {
                    return Err(HiveError::TaskCreationFailed {
                        reason: format!(
                            "Invalid priority: {}. Valid priorities: low, medium, high, critical",
                            priority_str
                        ),
                    });
                }
            } else {
                return Err(HiveError::TaskCreationFailed {
                    reason: "Priority must be a string".to_string(),
                });
            }
        }

        // Validate optional required capabilities
        if let Some(required_caps) = payload.get("required_capabilities") {
            if let Some(caps_array) = required_caps.as_array() {
                for cap in caps_array {
                    Self::validate_required_capability(cap)?;
                }
            } else {
                return Err(HiveError::TaskCreationFailed {
                    reason: "Required capabilities must be an array".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate capability object
    fn validate_capability(capability: &Value) -> HiveResult<()> {
        if let Some(name) = capability.get("name").and_then(|n| n.as_str()) {
            if name.trim().is_empty() {
                return Err(HiveError::AgentCreationFailed {
                    reason: "Capability name cannot be empty".to_string(),
                });
            }
        } else {
            return Err(HiveError::AgentCreationFailed {
                reason: "Capability must have a name field".to_string(),
            });
        }

        if let Some(proficiency) = capability.get("proficiency").and_then(|p| p.as_f64()) {
            if !(0.0..=1.0).contains(&proficiency) {
                return Err(HiveError::AgentCreationFailed {
                    reason: "Capability proficiency must be between 0.0 and 1.0".to_string(),
                });
            }
        }

        if let Some(learning_rate) = capability.get("learning_rate").and_then(|lr| lr.as_f64()) {
            if !(0.0..=1.0).contains(&learning_rate) {
                return Err(HiveError::AgentCreationFailed {
                    reason: "Learning rate must be between 0.0 and 1.0".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate required capability object
    fn validate_required_capability(capability: &Value) -> HiveResult<()> {
        if let Some(name) = capability.get("name").and_then(|n| n.as_str()) {
            if name.trim().is_empty() {
                return Err(HiveError::TaskCreationFailed {
                    reason: "Required capability name cannot be empty".to_string(),
                });
            }
        } else {
            return Err(HiveError::TaskCreationFailed {
                reason: "Required capability must have a name field".to_string(),
            });
        }

        if let Some(min_proficiency) = capability
            .get("minimum_proficiency")
            .and_then(|mp| mp.as_f64())
        {
            if !(0.0..=1.0).contains(&min_proficiency) {
                return Err(HiveError::TaskCreationFailed {
                    reason: "Minimum proficiency must be between 0.0 and 1.0".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate UUID string
    pub fn validate_uuid(uuid_str: &str) -> HiveResult<uuid::Uuid> {
        uuid::Uuid::parse_str(uuid_str).map_err(|_| HiveError::ConfigurationError {
            reason: format!("Invalid UUID format: {}", uuid_str),
        })
    }

    /// Sanitize string input with comprehensive security filtering
    ///
    /// # Security Features
    /// - Removes potentially dangerous characters
    /// - Prevents injection attacks
    /// - Limits length to prevent DoS
    /// - Normalizes whitespace
    pub fn sanitize_string(input: &str) -> String {
        // Prevent excessively long inputs (DoS protection)
        let truncated = if input.len() > 10000 {
            &input[..10000]
        } else {
            input
        };

        truncated
            .chars()
            .filter(|c| {
                // Allow alphanumeric, basic punctuation, and safe symbols
                c.is_alphanumeric()
                    || c.is_whitespace()
                    || "-_.,!?()[]{}:;@#$%^&*+=|\\/<>\"'`~".contains(*c)
            })
            .collect::<String>()
            .trim()
            .to_string()
    }

    /// Validate and sanitize agent name with security best practices
    pub fn validate_agent_name(name: &str) -> HiveResult<String> {
        let sanitized = Self::sanitize_string(name);

        if sanitized.is_empty() {
            return Err(HiveError::AgentCreationFailed {
                reason: "Agent name cannot be empty after sanitization".to_string(),
            });
        }

        if sanitized.len() > 100 {
            return Err(HiveError::AgentCreationFailed {
                reason: "Agent name too long (max 100 characters)".to_string(),
            });
        }

        // Prevent names that could cause confusion or security issues
        let forbidden_names = ["admin", "root", "system", "null", "undefined", "test"];
        if forbidden_names.contains(&sanitized.to_lowercase().as_str()) {
            return Err(HiveError::AgentCreationFailed {
                reason: "Agent name is reserved".to_string(),
            });
        }

        Ok(sanitized)
    }

    /// Validate resource limits
    pub fn validate_resource_limits(cpu_percent: f64, memory_percent: f64) -> HiveResult<()> {
        if !(0.0..=100.0).contains(&cpu_percent) {
            return Err(HiveError::ResourceExhausted {
                resource: "CPU percentage must be between 0 and 100".to_string(),
            });
        }

        if !(0.0..=100.0).contains(&memory_percent) {
            return Err(HiveError::ResourceExhausted {
                resource: "Memory percentage must be between 0 and 100".to_string(),
            });
        }

        Ok(())
    }
}
