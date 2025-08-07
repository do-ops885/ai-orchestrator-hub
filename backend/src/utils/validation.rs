use serde_json::Value;
use crate::utils::{HiveError, HiveResult};

/// Input validation utilities for the hive system
pub struct InputValidator;

impl InputValidator {
    /// Validate agent creation payload
    pub fn validate_agent_payload(payload: &Value) -> HiveResult<()> {
        // Check required fields
        if payload.get("name").is_none() {
            return Err(HiveError::AgentCreationFailed("Missing required field: name".to_string()));
        }
        
        if payload.get("agent_type").is_none() {
            return Err(HiveError::AgentCreationFailed("Missing required field: agent_type".to_string()));
        }
        
        // Validate name
        if let Some(name) = payload.get("name").and_then(|n| n.as_str()) {
            if name.trim().is_empty() {
                return Err(HiveError::AgentCreationFailed("Agent name cannot be empty".to_string()));
            }
            if name.len() > 100 {
                return Err(HiveError::AgentCreationFailed("Agent name too long (max 100 characters)".to_string()));
            }
            if !name.chars().all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '_') {
                return Err(HiveError::AgentCreationFailed("Agent name contains invalid characters".to_string()));
            }
        } else {
            return Err(HiveError::AgentCreationFailed("Agent name must be a string".to_string()));
        }
        
        // Validate agent type
        if let Some(agent_type) = payload.get("agent_type").and_then(|t| t.as_str()) {
            let valid_types = ["worker", "coordinator", "learner"];
            let is_specialist = agent_type.starts_with("specialist:");
            
            if !valid_types.contains(&agent_type) && !is_specialist {
                return Err(HiveError::AgentCreationFailed(
                    format!("Invalid agent type: {}. Valid types: worker, coordinator, learner, specialist:<type>", agent_type)
                ));
            }
            
            if is_specialist {
                let specialist_type = agent_type.strip_prefix("specialist:").unwrap_or("");
                if specialist_type.is_empty() {
                    return Err(HiveError::AgentCreationFailed("Specialist type cannot be empty".to_string()));
                }
            }
        } else {
            return Err(HiveError::AgentCreationFailed("Agent type must be a string".to_string()));
        }
        
        // Validate optional capabilities
        if let Some(capabilities) = payload.get("capabilities") {
            if let Some(caps_array) = capabilities.as_array() {
                for cap in caps_array {
                    Self::validate_capability(cap)?;
                }
            } else {
                return Err(HiveError::AgentCreationFailed("Capabilities must be an array".to_string()));
            }
        }
        
        Ok(())
    }
    
    /// Validate task creation payload
    pub fn validate_task_payload(payload: &Value) -> HiveResult<()> {
        // Check required fields
        if payload.get("description").is_none() {
            return Err(HiveError::TaskCreationFailed("Missing required field: description".to_string()));
        }
        
        // Validate description
        if let Some(description) = payload.get("description").and_then(|d| d.as_str()) {
            if description.trim().is_empty() {
                return Err(HiveError::TaskCreationFailed("Task description cannot be empty".to_string()));
            }
            if description.len() > 1000 {
                return Err(HiveError::TaskCreationFailed("Task description too long (max 1000 characters)".to_string()));
            }
        } else {
            return Err(HiveError::TaskCreationFailed("Task description must be a string".to_string()));
        }
        
        // Validate optional priority
        if let Some(priority) = payload.get("priority") {
            if let Some(priority_str) = priority.as_str() {
                let valid_priorities = ["low", "medium", "high", "critical"];
                if !valid_priorities.contains(&priority_str) {
                    return Err(HiveError::TaskCreationFailed(
                        format!("Invalid priority: {}. Valid priorities: low, medium, high, critical", priority_str)
                    ));
                }
            } else {
                return Err(HiveError::TaskCreationFailed("Priority must be a string".to_string()));
            }
        }
        
        // Validate optional required capabilities
        if let Some(required_caps) = payload.get("required_capabilities") {
            if let Some(caps_array) = required_caps.as_array() {
                for cap in caps_array {
                    Self::validate_required_capability(cap)?;
                }
            } else {
                return Err(HiveError::TaskCreationFailed("Required capabilities must be an array".to_string()));
            }
        }
        
        Ok(())
    }
    
    /// Validate capability object
    fn validate_capability(capability: &Value) -> HiveResult<()> {
        if let Some(name) = capability.get("name").and_then(|n| n.as_str()) {
            if name.trim().is_empty() {
                return Err(HiveError::AgentCreationFailed("Capability name cannot be empty".to_string()));
            }
        } else {
            return Err(HiveError::AgentCreationFailed("Capability must have a name field".to_string()));
        }
        
        if let Some(proficiency) = capability.get("proficiency").and_then(|p| p.as_f64()) {
            if !(0.0..=1.0).contains(&proficiency) {
                return Err(HiveError::AgentCreationFailed("Capability proficiency must be between 0.0 and 1.0".to_string()));
            }
        }
        
        if let Some(learning_rate) = capability.get("learning_rate").and_then(|lr| lr.as_f64()) {
            if !(0.0..=1.0).contains(&learning_rate) {
                return Err(HiveError::AgentCreationFailed("Learning rate must be between 0.0 and 1.0".to_string()));
            }
        }
        
        Ok(())
    }
    
    /// Validate required capability object
    fn validate_required_capability(capability: &Value) -> HiveResult<()> {
        if let Some(name) = capability.get("name").and_then(|n| n.as_str()) {
            if name.trim().is_empty() {
                return Err(HiveError::TaskCreationFailed("Required capability name cannot be empty".to_string()));
            }
        } else {
            return Err(HiveError::TaskCreationFailed("Required capability must have a name field".to_string()));
        }
        
        if let Some(min_proficiency) = capability.get("minimum_proficiency").and_then(|mp| mp.as_f64()) {
            if !(0.0..=1.0).contains(&min_proficiency) {
                return Err(HiveError::TaskCreationFailed("Minimum proficiency must be between 0.0 and 1.0".to_string()));
            }
        }
        
        Ok(())
    }
    
    /// Validate UUID string
    pub fn validate_uuid(uuid_str: &str) -> HiveResult<uuid::Uuid> {
        uuid::Uuid::parse_str(uuid_str)
            .map_err(|_| HiveError::ConfigurationError(format!("Invalid UUID format: {}", uuid_str)))
    }
    
    /// Sanitize string input
    pub fn sanitize_string(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || "-_.,!?".contains(*c))
            .collect::<String>()
            .trim()
            .to_string()
    }
    
    /// Validate resource limits
    pub fn validate_resource_limits(cpu_percent: f64, memory_percent: f64) -> HiveResult<()> {
        if !(0.0..=100.0).contains(&cpu_percent) {
            return Err(HiveError::ResourceExhausted("CPU percentage must be between 0 and 100".to_string()));
        }
        
        if !(0.0..=100.0).contains(&memory_percent) {
            return Err(HiveError::ResourceExhausted("Memory percentage must be between 0 and 100".to_string()));
        }
        
        Ok(())
    }
}