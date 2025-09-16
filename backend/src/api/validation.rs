/// API-specific validation utilities
///
/// This module provides validation functions specific to API request handling.
/// Currently delegates to the main validation utilities in utils/validation.rs

use serde_json::Value;
use crate::utils::validation::InputValidator;

/// API validation wrapper functions
pub struct ApiValidator;

impl ApiValidator {
    /// Validate agent creation payload for API requests
    pub fn validate_agent_payload(payload: &Value) -> Result<(), String> {
        InputValidator::validate_agent_payload(payload)
            .map_err(|e| e.to_string())
    }

    /// Validate task creation payload for API requests
    pub fn validate_task_payload(payload: &Value) -> Result<(), String> {
        InputValidator::validate_task_payload(payload)
            .map_err(|e| e.to_string())
    }
}