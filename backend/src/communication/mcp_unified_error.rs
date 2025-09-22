use super::mcp::{error_codes, MCPError};
use crate::utils::error::HiveError;
use anyhow::Result;
use serde_json::{json, Value};
use thiserror::Error;
use tracing::{debug, error, warn};

/// Unified MCP Error System
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
        context_chain: Vec<String>,
    },

    #[error("Tool Execution Failed: {tool_name} - {reason}")]
    ToolExecution {
        tool_name: String,
        reason: String,
        params: Option<Value>,
        duration_ms: Option<u64>,
        context_chain: Vec<String>,
    },

    #[error("Resource Access Denied: {resource} - {reason}")]
    ResourceAccess {
        resource: String,
        reason: String,
        required_permissions: Vec<String>,
        context_chain: Vec<String>,
    },

    #[error("Rate Limit Exceeded: {limit} requests per {window}")]
    RateLimit {
        limit: u32,
        window: String,
        retry_after_ms: u64,
        client_id: Option<String>,
        context_chain: Vec<String>,
    },

    #[error("Validation Failed: {field} - {issue}")]
    Validation {
        field: String,
        issue: String,
        provided_value: Option<Value>,
        expected_format: Option<String>,
        context_chain: Vec<String>,
    },

    #[error("Cache Error: {operation} - {reason}")]
    Cache {
        operation: String,
        reason: String,
        cache_key: Option<String>,
        context_chain: Vec<String>,
    },

    #[error("Hive System Error: {component} - {error}")]
    HiveSystem {
        component: String,
        error: String,
        hive_error: Option<HiveError>,
        context_chain: Vec<String>,
    },

    #[error("Authentication Failed: {reason}")]
    Authentication {
        reason: String,
        token_expired: bool,
        required_permissions: Vec<String>,
        context_chain: Vec<String>,
    },

    #[error("Internal Server Error: {message}")]
    Internal {
        message: String,
        source_error: Option<String>,
        recovery_suggestion: Option<String>,
        context_chain: Vec<String>,
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
            context_chain: Vec::new(),
        }
    }

    /// Create a tool execution error with timing info
    #[must_use]
    pub fn tool_execution(
        tool_name: String,
        reason: String,
        params: Option<Value>,
        duration_ms: Option<u64>,
    ) -> Self {
        Self::ToolExecution {
            tool_name,
            reason,
            params,
            duration_ms,
            context_chain: Vec::new(),
        }
    }

    /// Create a validation error with expected format
    #[must_use]
    pub fn validation(
        field: String,
        issue: String,
        provided_value: Option<Value>,
        expected_format: Option<String>,
    ) -> Self {
        Self::Validation {
            field,
            issue,
            provided_value,
            expected_format,
            context_chain: Vec::new(),
        }
    }

    /// Create a rate limit error with retry information
    #[must_use]
    pub fn rate_limit(
        limit: u32,
        window: String,
        retry_after_ms: u64,
        client_id: Option<String>,
    ) -> Self {
        Self::RateLimit {
            limit,
            window,
            retry_after_ms,
            client_id,
            context_chain: Vec::new(),
        }
    }

    /// Create a cache error
    #[must_use]
    pub fn cache(operation: String, reason: String, cache_key: Option<String>) -> Self {
        Self::Cache {
            operation,
            reason,
            cache_key,
            context_chain: Vec::new(),
        }
    }

    /// Create a hive system error from `HiveError`
    #[must_use]
    pub fn from_hive_error(component: String, hive_error: HiveError) -> Self {
        Self::HiveSystem {
            component,
            error: hive_error.to_string(),
            hive_error: Some(hive_error),
            context_chain: Vec::new(),
        }
    }

    /// Get the severity level of the error
    #[must_use]
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Protocol { code, .. } => match *code {
                error_codes::PARSE_ERROR | error_codes::INVALID_REQUEST => ErrorSeverity::Warning,
                error_codes::METHOD_NOT_FOUND | error_codes::TOOL_NOT_FOUND => ErrorSeverity::Info,
                error_codes::PERMISSION_DENIED => ErrorSeverity::Warning,
                error_codes::RATE_LIMITED => ErrorSeverity::Info,
                _ => ErrorSeverity::Error,
            },
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
            Self::Protocol { code, .. } => match *code {
                error_codes::PARSE_ERROR => Some("Check JSON formatting and syntax".to_string()),
                error_codes::INVALID_REQUEST => {
                    Some("Verify request structure matches MCP specification".to_string())
                }
                error_codes::METHOD_NOT_FOUND => {
                    Some("Check available methods using tools/list".to_string())
                }
                error_codes::INVALID_PARAMS => {
                    Some("Validate parameter types and required fields".to_string())
                }
                _ => None,
            },
            Self::ToolExecution { .. } => {
                Some("Check tool parameters and system status".to_string())
            }
            Self::ResourceAccess {
                required_permissions,
                ..
            } => {
                if required_permissions.is_empty() {
                    Some("Contact administrator for access".to_string())
                } else {
                    Some(format!(
                        "Required permissions: {}",
                        required_permissions.join(", ")
                    ))
                }
            }
            Self::RateLimit { retry_after_ms, .. } => {
                Some(format!("Wait {retry_after_ms} ms before retrying"))
            }
            Self::Validation {
                expected_format, ..
            } => expected_format
                .as_ref()
                .map(|f| format!("Expected format: {f}")),
            Self::Cache { .. } => Some("Try clearing cache or retrying operation".to_string()),
            Self::HiveSystem { .. } => Some("Check hive system status and logs".to_string()),
            Self::Authentication { .. } => {
                Some("Refresh authentication token or check permissions".to_string())
            }
            Self::Internal {
                recovery_suggestion,
                ..
            } => recovery_suggestion.clone(),
        }
    }

    /// Add context to the error chain
    #[must_use]
    pub fn with_context(mut self, context: String) -> Self {
        match &mut self {
            Self::Protocol { context_chain, .. } => context_chain.push(context),
            Self::ToolExecution { context_chain, .. } => context_chain.push(context),
            Self::ResourceAccess { context_chain, .. } => context_chain.push(context),
            Self::RateLimit { context_chain, .. } => context_chain.push(context),
            Self::Validation { context_chain, .. } => context_chain.push(context),
            Self::Cache { context_chain, .. } => context_chain.push(context),
            Self::HiveSystem { context_chain, .. } => context_chain.push(context),
            Self::Authentication { context_chain, .. } => context_chain.push(context),
            Self::Internal { context_chain, .. } => context_chain.push(context),
        }
        self
    }

    /// Get the full context chain as a formatted string
    #[must_use]
    pub fn context_trace(&self) -> String {
        let chain = match self {
            Self::Protocol { context_chain, .. } => context_chain,
            Self::ToolExecution { context_chain, .. } => context_chain,
            Self::ResourceAccess { context_chain, .. } => context_chain,
            Self::RateLimit { context_chain, .. } => context_chain,
            Self::Validation { context_chain, .. } => context_chain,
            Self::Cache { context_chain, .. } => context_chain,
            Self::HiveSystem { context_chain, .. } => context_chain,
            Self::Authentication { context_chain, .. } => context_chain,
            Self::Internal { context_chain, .. } => context_chain,
        };

        if chain.is_empty() {
            "No additional context".to_string()
        } else {
            format!("Context trace: {}", chain.join(" -> "))
        }
    }

    /// Log the error with appropriate level and full context trace
    pub fn log_error(&self, request_id: Option<&str>, client_id: Option<&str>) {
        let context = format!(
            "request_id={} client_id={}",
            request_id.unwrap_or("unknown"),
            client_id.unwrap_or("unknown")
        );

        let trace = self.context_trace();

        match self.severity() {
            ErrorSeverity::Critical => error!("{} - {} | {}", context, self, trace),
            ErrorSeverity::Error => error!("{} - {} | {}", context, self, trace),
            ErrorSeverity::Warning => warn!("{} - {} | {}", context, self, trace),
            ErrorSeverity::Info => debug!("{} - {} | {}", context, self, trace),
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

        let context_trace = err.context_trace();

        match err {
            MCPUnifiedError::Protocol {
                code,
                message,
                data,
                context,
                ref context_chain,
            } => MCPError {
                code,
                message: if let Some(ctx) = context {
                    format!("{message} (Context: {ctx})")
                } else {
                    message
                },
                data: Some(json!({
                    "original_data": data,
                    "context_chain": context_chain,
                    "context_trace": context_trace
                })),
            },
            MCPUnifiedError::ToolExecution {
                tool_name,
                reason,
                params,
                duration_ms,
                ref context_chain,
            } => MCPError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Tool '{tool_name}' failed: {reason}"),
                data: Some(json!({
                    "tool": tool_name,
                    "reason": reason,
                    "params": params,
                    "duration_ms": duration_ms,
                    "context_chain": context_chain,
                    "context_trace": context_trace,
                    "recovery": "Check tool parameters and system status"
                })),
            },
            MCPUnifiedError::ResourceAccess {
                resource,
                reason,
                required_permissions,
                ref context_chain,
            } => MCPError {
                code: error_codes::PERMISSION_DENIED,
                message: format!("Access denied to resource '{resource}': {reason}"),
                data: Some(json!({
                    "resource": resource,
                    "reason": reason,
                    "required_permissions": required_permissions,
                    "context_chain": context_chain,
                    "context_trace": context_trace,
                    "recovery": format!("Required permissions: {}", required_permissions.join(", "))
                })),
            },
            MCPUnifiedError::RateLimit {
                limit,
                window,
                retry_after_ms,
                client_id,
                ref context_chain,
            } => MCPError {
                code: error_codes::RATE_LIMITED,
                message: format!("Rate limit exceeded: {limit} requests per {window}"),
                data: Some(json!({
                    "limit": limit,
                    "window": window,
                    "retry_after_ms": retry_after_ms,
                    "client_id": client_id,
                    "context_chain": context_chain,
                    "context_trace": context_trace,
                    "recovery": format!("Wait {} ms before retrying", retry_after_ms)
                })),
            },
            MCPUnifiedError::Validation {
                field,
                issue,
                provided_value,
                expected_format,
                ref context_chain,
            } => MCPError {
                code: error_codes::INVALID_PARAMS,
                message: format!("Validation failed for '{field}': {issue}"),
                data: Some(json!({
                    "field": field,
                    "issue": issue,
                    "provided_value": provided_value,
                    "expected_format": expected_format,
                    "context_chain": context_chain,
                    "context_trace": context_trace,
                    "recovery": expected_format.map(|f| format!("Expected format: {f}"))
                })),
            },
            MCPUnifiedError::Cache {
                operation,
                reason,
                cache_key,
                ref context_chain,
            } => MCPError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Cache operation '{operation}' failed: {reason}"),
                data: Some(json!({
                    "operation": operation,
                    "reason": reason,
                    "cache_key": cache_key,
                    "context_chain": context_chain,
                    "context_trace": context_trace,
                    "recovery": "Try clearing cache or retrying operation"
                })),
            },
            MCPUnifiedError::HiveSystem {
                component,
                error,
                ref context_chain,
                ..
            } => MCPError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Hive system error in '{component}': {error}"),
                data: Some(json!({
                    "component": component,
                    "error": error,
                    "context_chain": context_chain,
                    "context_trace": context_trace,
                    "recovery": "Check hive system status and logs"
                })),
            },
            MCPUnifiedError::Authentication {
                reason,
                token_expired,
                required_permissions,
                ref context_chain,
            } => MCPError {
                code: error_codes::PERMISSION_DENIED,
                message: format!("Authentication failed: {reason}"),
                data: Some(json!({
                    "reason": reason,
                    "token_expired": token_expired,
                    "required_permissions": required_permissions,
                    "context_chain": context_chain,
                    "context_trace": context_trace,
                    "recovery": "Refresh authentication token or check permissions"
                })),
            },
            MCPUnifiedError::Internal {
                message,
                source_error,
                recovery_suggestion,
                ref context_chain,
            } => MCPError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Internal error: {message}"),
                data: Some(json!({
                    "message": message,
                    "source_error": source_error,
                    "recovery": recovery_suggestion.unwrap_or_else(|| "Contact system administrator".to_string()),
                    "context_chain": context_chain,
                    "context_trace": context_trace
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
            context_chain: Vec::new(),
        }
    }
}

/// Error handler utility for MCP operations
#[derive(Debug, Clone)]
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

    /// Handle a result, converting errors to `MCPUnifiedError` with context chaining
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
                let mut unified_error = if let Some(mcp_err) = err.downcast_ref::<MCPUnifiedError>()
                {
                    mcp_err.clone()
                } else if let Some(hive_err) = err.downcast_ref::<HiveError>() {
                    MCPUnifiedError::from_hive_error(context.to_string(), hive_err.clone())
                } else {
                    MCPUnifiedError::Internal {
                        message: format!("{context}: {err}"),
                        source_error: Some(format!("{err:?}")),
                        recovery_suggestion: Some("Check system logs for details".to_string()),
                        context_chain: Vec::new(),
                    }
                };

                // Add context to the chain
                unified_error = unified_error.with_context(context.to_string());

                if self.log_all_errors {
                    unified_error.log_error(request_id, client_id);
                }

                Err(unified_error)
            }
        }
    }

    /// Chain context to an existing MCPUnifiedError
    #[must_use]
    pub fn chain_context(self, error: MCPUnifiedError, context: String) -> MCPUnifiedError {
        error.with_context(context)
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
        let params_obj = params.as_object().ok_or_else(|| {
            MCPUnifiedError::validation(
                "params".to_string(),
                "Must be an object".to_string(),
                Some(params.clone()),
                Some("JSON object".to_string()),
            )
        })?;

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
        let all_allowed: Vec<&str> = required_fields
            .iter()
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
            Some("Missing jsonrpc field".to_string()),
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

        let data = mcp_error.data.expect("data should be some");
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

        let data = mcp_error.data.expect("data should be some");
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
        assert!(suggestion
            .expect("suggestion should be some")
            .contains("30000"));

        let validation_error = MCPUnifiedError::validation(
            "date".to_string(),
            "Invalid format".to_string(),
            Some(json!("2023-13-40")),
            Some("YYYY-MM-DD".to_string()),
        );

        let suggestion = validation_error.recovery_suggestion();
        assert!(suggestion.is_some());
        assert!(suggestion
            .expect("suggestion should be some")
            .contains("YYYY-MM-DD"));
    }
}
