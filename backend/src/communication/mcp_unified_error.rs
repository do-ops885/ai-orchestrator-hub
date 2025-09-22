use super::mcp::{MCPError, error_codes};
use crate::utils::error::HiveError;
use anyhow::Result;
use serde_json::{json, Value};
use thiserror::Error;
use tracing::{error, warn, debug};

/// Unified MCP Error System (Phase 2.2)
/// 
/// This provides a comprehensive error handling system that consolidates
/// all MCP-related errors into a unified hierarchy with proper context,
/// logging, and recovery suggestions.
#[derive(Debug, Clone, Error)]
pub enum MCPUnifiedError {
    #[error("MCP Protocol Error: {code} - {message}")]
    Protocol { 
        code: i32, 
        message: String, 
        data: Option<Value>,
        context: Option<String>,
    },
    
    #[error("Tool Execution Failed: {tool_name} - {reason}")]
    ToolExecution { 
        tool_name: String, 
        reason: String,
        params: Option<Value>,
        duration_ms: Option<u64>,
    },
    
    #[error("Resource Access Denied: {resource} - {reason}")]
    ResourceAccess { 
        resource: String, 
        reason: String,
        required_permissions: Vec<String>,
    },
    
    #[error("Rate Limit Exceeded: {limit} requests per {window}")]
    RateLimit { 
        limit: u32, 
        window: String,
        retry_after_ms: u64,
        client_id: Option<String>,
    },
    
    #[error("Validation Failed: {field} - {issue}")]
    Validation { 
        field: String, 
        issue: String,
        provided_value: Option<Value>,
        expected_format: Option<String>,
    },
    
    #[error("Cache Error: {operation} - {reason}")]
    Cache { 
        operation: String, 
        reason: String,
        cache_key: Option<String>,
    },
    
    #[error("Hive System Error: {component} - {error}")]
    HiveSystem { 
        component: String, 
        error: String,
        hive_error: Option<HiveError>,
    },
    
    #[error("Authentication Failed: {reason}")]
    Authentication { 
        reason: String,
        token_expired: bool,
        required_permissions: Vec<String>,
    },
    
    #[error("Internal Server Error: {message}")]
    Internal { 
        message: String,
        source_error: Option<String>,
        recovery_suggestion: Option<String>,
    },
}

impl MCPUnifiedError {
    /// Create a protocol error with context
    #[must_use] 
    pub fn protocol(code: i32, message: String, context: Option<String>) -> Self {
        Self::Protocol {
            code,
            message,
            data: None,
            context,
        }
    }

    /// Create a tool execution error with timing info
    #[must_use] 
    pub fn tool_execution(tool_name: String, reason: String, params: Option<Value>, duration_ms: Option<u64>) -> Self {
        Self::ToolExecution {
            tool_name,
            reason,
            params,
            duration_ms,
        }
    }

    /// Create a validation error with expected format
    #[must_use] 
    pub fn validation(field: String, issue: String, provided_value: Option<Value>, expected_format: Option<String>) -> Self {
        Self::Validation {
            field,
            issue,
            provided_value,
            expected_format,
        }
    }

    /// Create a rate limit error with retry information
    #[must_use] 
    pub fn rate_limit(limit: u32, window: String, retry_after_ms: u64, client_id: Option<String>) -> Self {
        Self::RateLimit {
            limit,
            window,
            retry_after_ms,
            client_id,
        }
    }

    /// Create a cache error
    #[must_use] 
    pub fn cache(operation: String, reason: String, cache_key: Option<String>) -> Self {
        Self::Cache {
            operation,
            reason,
            cache_key,
        }
    }

    /// Create a hive system error from `HiveError`
    #[must_use] 
    pub fn from_hive_error(component: String, hive_error: HiveError) -> Self {
        Self::HiveSystem {
            component,
            error: hive_error.to_string(),
            hive_error: Some(hive_error),
        }
    }

    /// Get the severity level of the error
    #[must_use] 
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Protocol { code, .. } => {
                match *code {
                    error_codes::PARSE_ERROR | error_codes::INVALID_REQUEST => ErrorSeverity::Warning,
                    error_codes::METHOD_NOT_FOUND | error_codes::TOOL_NOT_FOUND => ErrorSeverity::Info,
                    error_codes::PERMISSION_DENIED => ErrorSeverity::Warning,
                    error_codes::RATE_LIMITED => ErrorSeverity::Info,
                    _ => ErrorSeverity::Error,
                }
            }
            Self::ToolExecution { .. } => ErrorSeverity::Error,
            Self::ResourceAccess { .. } => ErrorSeverity::Warning,
            Self::RateLimit { .. } => ErrorSeverity::Info,
            Self::Validation { .. } => ErrorSeverity::Warning,
            Self::Cache { .. } => ErrorSeverity::Warning,
            Self::HiveSystem { .. } => ErrorSeverity::Error,
            Self::Authentication { .. } => ErrorSeverity::Warning,
            Self::Internal { .. } => ErrorSeverity::Critical,
        }
    }

    /// Get recovery suggestion for the error
    #[must_use] 
    pub fn recovery_suggestion(&self) -> Option<String> {
        match self {
            Self::Protocol { code, .. } => {
                match *code {
                    error_codes::PARSE_ERROR => Some("Check JSON formatting and syntax".to_string()),
                    error_codes::INVALID_REQUEST => Some("Verify request structure matches MCP specification".to_string()),
                    error_codes::METHOD_NOT_FOUND => Some("Check available methods using tools/list".to_string()),
                    error_codes::INVALID_PARAMS => Some("Validate parameter types and required fields".to_string()),
                    _ => None,
                }
            }
            Self::ToolExecution { .. } => Some("Check tool parameters and system status".to_string()),
            Self::ResourceAccess { required_permissions, .. } => {
                if required_permissions.is_empty() {
                    Some("Contact administrator for access".to_string())
                } else {
                    Some(format!("Required permissions: {}", required_permissions.join(", ")))
                }
            }
            Self::RateLimit { retry_after_ms, .. } => {
                Some(format!("Wait {retry_after_ms} ms before retrying"))
            }
            Self::Validation { expected_format, .. } => {
                expected_format.as_ref().map(|f| format!("Expected format: {f}"))
            }
            Self::Cache { .. } => Some("Try clearing cache or retrying operation".to_string()),
            Self::HiveSystem { .. } => Some("Check hive system status and logs".to_string()),
            Self::Authentication { .. } => Some("Refresh authentication token or check permissions".to_string()),
            Self::Internal { recovery_suggestion, .. } => recovery_suggestion.clone(),
        }
    }

    /// Log the error with appropriate level
    pub fn log_error(&self, request_id: Option<&str>, client_id: Option<&str>) {
        let context = format!(
            "request_id={} client_id={}", 
            request_id.unwrap_or("unknown"),
            client_id.unwrap_or("unknown")
        );

        match self.severity() {
            ErrorSeverity::Critical => error!("{} - {}", context, self),
            ErrorSeverity::Error => error!("{} - {}", context, self),
            ErrorSeverity::Warning => warn!("{} - {}", context, self),
            ErrorSeverity::Info => debug!("{} - {}", context, self),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl From<MCPUnifiedError> for MCPError {
    fn from(err: MCPUnifiedError) -> Self {
        // Log the error before conversion
        err.log_error(None, None);

        match err {
            MCPUnifiedError::Protocol { code, message, data, context } => MCPError {
                code,
                message: if let Some(ctx) = context {
                    format!("{message} (Context: {ctx})")
                } else {
                    message
                },
                data,
            },
            MCPUnifiedError::ToolExecution { tool_name, reason, params, duration_ms } => MCPError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Tool '{tool_name}' failed: {reason}"),
                data: Some(json!({
                    "tool": tool_name,
                    "reason": reason,
                    "params": params,
                    "duration_ms": duration_ms,
                    "recovery": "Check tool parameters and system status"
                })),
            },
            MCPUnifiedError::ResourceAccess { resource, reason, required_permissions } => MCPError {
                code: error_codes::PERMISSION_DENIED,
                message: format!("Access denied to resource '{resource}': {reason}"),
                data: Some(json!({
                    "resource": resource,
                    "reason": reason,
                    "required_permissions": required_permissions,
                    "recovery": format!("Required permissions: {}", required_permissions.join(", "))
                })),
            },
            MCPUnifiedError::RateLimit { limit, window, retry_after_ms, client_id } => MCPError {
                code: error_codes::RATE_LIMITED,
                message: format!("Rate limit exceeded: {limit} requests per {window}"),
                data: Some(json!({
                    "limit": limit,
                    "window": window,
                    "retry_after_ms": retry_after_ms,
                    "client_id": client_id,
                    "recovery": format!("Wait {} ms before retrying", retry_after_ms)
                })),
            },
            MCPUnifiedError::Validation { field, issue, provided_value, expected_format } => MCPError {
                code: error_codes::INVALID_PARAMS,
                message: format!("Validation failed for '{field}': {issue}"),
                data: Some(json!({
                    "field": field,
                    "issue": issue,
                    "provided_value": provided_value,
                    "expected_format": expected_format,
                    "recovery": expected_format.map(|f| format!("Expected format: {f}"))
                })),
            },
            MCPUnifiedError::Cache { operation, reason, cache_key } => MCPError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Cache operation '{operation}' failed: {reason}"),
                data: Some(json!({
                    "operation": operation,
                    "reason": reason,
                    "cache_key": cache_key,
                    "recovery": "Try clearing cache or retrying operation"
                })),
            },
            MCPUnifiedError::HiveSystem { component, error, .. } => MCPError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Hive system error in '{component}': {error}"),
                data: Some(json!({
                    "component": component,
                    "error": error,
                    "recovery": "Check hive system status and logs"
                })),
            },
            MCPUnifiedError::Authentication { reason, token_expired, required_permissions } => MCPError {
                code: error_codes::PERMISSION_DENIED,
                message: format!("Authentication failed: {reason}"),
                data: Some(json!({
                    "reason": reason,
                    "token_expired": token_expired,
                    "required_permissions": required_permissions,
                    "recovery": "Refresh authentication token or check permissions"
                })),
            },
            MCPUnifiedError::Internal { message, source_error, recovery_suggestion } => MCPError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Internal error: {message}"),
                data: Some(json!({
                    "message": message,
                    "source_error": source_error,
                    "recovery": recovery_suggestion.unwrap_or_else(|| "Contact system administrator".to_string())
                })),
            },
        }
    }
}

impl From<HiveError> for MCPUnifiedError {
    fn from(err: HiveError) -> Self {
        MCPUnifiedError::from_hive_error("hive_coordinator".to_string(), err)
    }
}

impl From<anyhow::Error> for MCPUnifiedError {
    fn from(err: anyhow::Error) -> Self {
        MCPUnifiedError::Internal {
            message: err.to_string(),
            source_error: Some(format!("{err:?}")),
            recovery_suggestion: Some("Check system logs for details".to_string()),
        }
    }
}

/// Error handler utility for MCP operations
#[derive(Clone)]
pub struct MCPErrorHandler {
    enable_detailed_errors: bool,
    log_all_errors: bool,
}

impl MCPErrorHandler {
    #[must_use] 
    pub fn new(enable_detailed_errors: bool, log_all_errors: bool) -> Self {
        Self {
            enable_detailed_errors,
            log_all_errors,
        }
    }

    /// Handle a result, converting errors to `MCPUnifiedError` with context
    pub fn handle_result<T>(
        &self,
        result: Result<T>,
        context: &str,
        request_id: Option<&str>,
        client_id: Option<&str>,
    ) -> Result<T, MCPUnifiedError> {
        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                let unified_error = if let Some(mcp_err) = err.downcast_ref::<MCPUnifiedError>() {
                    mcp_err.clone()
                } else if let Some(hive_err) = err.downcast_ref::<HiveError>() {
                    MCPUnifiedError::from_hive_error(context.to_string(), hive_err.clone())
                } else {
                    MCPUnifiedError::Internal {
                        message: format!("{context}: {err}"),
                        source_error: Some(format!("{err:?}")),
                        recovery_suggestion: Some("Check system logs for details".to_string()),
                    }
                };

                if self.log_all_errors {
                    unified_error.log_error(request_id, client_id);
                }

                Err(unified_error)
            }
        }
    }

    /// Validate tool parameters with detailed error reporting
    pub fn validate_tool_params(
        &self,
        tool_name: &str,
        params: &Value,
        required_fields: &[&str],
        optional_fields: &[&str],
    ) -> Result<(), MCPUnifiedError> {
        // Check that params is an object
        let params_obj = params.as_object()
            .ok_or_else(|| MCPUnifiedError::validation(
                "params".to_string(),
                "Must be an object".to_string(),
                Some(params.clone()),
                Some("JSON object".to_string()),
            ))?;

        // Check required fields
        for field in required_fields {
            if !params_obj.contains_key(*field) {
                return Err(MCPUnifiedError::validation(
                    (*field).to_string(),
                    "Required field missing".to_string(),
                    Some(params.clone()),
                    Some(format!("Required field: {field}")),
                ));
            }
        }

        // Check for unknown fields
        let all_allowed: Vec<&str> = required_fields.iter()
            .chain(optional_fields.iter())
            .copied()
            .collect();

        for key in params_obj.keys() {
            if !all_allowed.contains(&key.as_str()) {
                return Err(MCPUnifiedError::validation(
                    key.clone(),
                    "Unknown field".to_string(),
                    Some(json!(key)),
                    Some(format!("Allowed fields: {}", all_allowed.join(", "))),
                ));
            }
        }

        debug!("Tool '{}' parameters validated successfully", tool_name);
        Ok(())
    }
}

impl Default for MCPErrorHandler {
    fn default() -> Self {
        Self::new(true, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_protocol_error_conversion() {
        let error = MCPUnifiedError::protocol(
            error_codes::INVALID_REQUEST,
            "Invalid request format".to_string(),
            Some("Missing jsonrpc field".to_string())
        );

        let mcp_error: MCPError = error.into();
        assert_eq!(mcp_error.code, error_codes::INVALID_REQUEST);
        assert!(mcp_error.message.contains("Invalid request format"));
        assert!(mcp_error.message.contains("Context: Missing jsonrpc field"));
    }

    #[test]
    fn test_tool_execution_error() {
        let error = MCPUnifiedError::tool_execution(
            "create_agent".to_string(),
            "Invalid agent type".to_string(),
            Some(json!({"type": "invalid"})),
            Some(150),
        );

        let mcp_error: MCPError = error.into();
        assert_eq!(mcp_error.code, error_codes::INTERNAL_ERROR);
        assert!(mcp_error.message.contains("create_agent"));
        assert!(mcp_error.data.is_some());
        
        let data = mcp_error.data.unwrap();
        assert_eq!(data["tool"], "create_agent");
        assert_eq!(data["duration_ms"], 150);
    }

    #[test]
    fn test_validation_error() {
        let error = MCPUnifiedError::validation(
            "priority".to_string(),
            "Invalid priority level".to_string(),
            Some(json!("invalid")),
            Some("One of: Low, Medium, High, Critical".to_string()),
        );

        let mcp_error: MCPError = error.into();
        assert_eq!(mcp_error.code, error_codes::INVALID_PARAMS);
        assert!(mcp_error.message.contains("priority"));
        
        let data = mcp_error.data.unwrap();
        assert_eq!(data["field"], "priority");
        assert_eq!(data["provided_value"], "invalid");
    }

    #[test]
    fn test_error_severity() {
        let protocol_error = MCPUnifiedError::protocol(
            error_codes::METHOD_NOT_FOUND,
            "Method not found".to_string(),
            None,
        );
        assert_eq!(protocol_error.severity(), ErrorSeverity::Info);

        let internal_error = MCPUnifiedError::Internal {
            message: "Critical system failure".to_string(),
            source_error: None,
            recovery_suggestion: None,
        };
        assert_eq!(internal_error.severity(), ErrorSeverity::Critical);
    }

    #[test]
    fn test_error_handler_validation() {
        let handler = MCPErrorHandler::default();
        
        let params = json!({
            "type": "worker",
            "description": "Test agent"
        });

        let result = handler.validate_tool_params(
            "create_agent",
            &params,
            &["type"],
            &["description", "specialization"],
        );
        
        assert!(result.is_ok());

        // Test missing required field
        let invalid_params = json!({
            "description": "Test agent"
        });

        let result = handler.validate_tool_params(
            "create_agent",
            &invalid_params,
            &["type"],
            &["description"],
        );
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        if let MCPUnifiedError::Validation { field, .. } = err {
            assert_eq!(field, "type");
        } else {
            panic!("Expected validation error");
        }
    }

    #[test]
    fn test_recovery_suggestions() {
        let rate_limit_error = MCPUnifiedError::rate_limit(
            100,
            "minute".to_string(),
            30000,
            Some("client_123".to_string()),
        );
        
        let suggestion = rate_limit_error.recovery_suggestion();
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("30000"));

        let validation_error = MCPUnifiedError::validation(
            "date".to_string(),
            "Invalid format".to_string(),
            Some(json!("2023-13-40")),
            Some("YYYY-MM-DD".to_string()),
        );
        
        let suggestion = validation_error.recovery_suggestion();
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("YYYY-MM-DD"));
    }
}