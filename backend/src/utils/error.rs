use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Custom error types for the multiagent hive system
///
/// This enum provides comprehensive error handling for all system components
/// with structured error information and proper error chaining.
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum HiveError {
    /// Agent-related errors
    #[error("Agent not found: {id}")]
    AgentNotFound { id: String },

    #[error("Agent creation failed: {reason}")]
    AgentCreationFailed { reason: String },

    #[error("Agent execution failed: {reason}")]
    AgentExecutionFailed { reason: String },

    /// Task-related errors
    #[error("Task not found: {id}")]
    TaskNotFound { id: String },

    #[error("Task creation failed: {reason}")]
    TaskCreationFailed { reason: String },

    #[error("Task execution failed: {reason}")]
    TaskExecutionFailed { reason: String },

    /// Resource management errors
    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },

    #[error("Resource initialization failed: {reason}")]
    ResourceInitializationFailed { reason: String },

    /// Communication errors
    #[error("WebSocket error: {reason}")]
    WebSocketError { reason: String },

    #[error("Message parsing error: {reason}")]
    MessageParsingError { reason: String },

    /// System errors
    #[error("System overloaded: {reason}")]
    SystemOverloaded { reason: String },

    #[error("Configuration error: {reason}")]
    ConfigurationError { reason: String },

    #[error("Database error: {reason}")]
    DatabaseError { reason: String },

    /// Neural processing errors
    #[error("Neural processing error: {reason}")]
    NeuralProcessingError { reason: String },

    #[error("NLP error: {reason}")]
    NLPError { reason: String },

    /// Circuit breaker errors
    #[error("Circuit breaker open: {reason}")]
    CircuitBreakerOpen { reason: String },

    #[error("Operation failed: {reason}")]
    OperationFailed { reason: String },

    /// Validation errors
    #[error("Invalid input: {field} - {reason}")]
    ValidationError { field: String, reason: String },

    /// Timeout errors
    #[error("Operation timed out: {operation} after {duration_ms}ms")]
    TimeoutError { operation: String, duration_ms: u64 },
}

// Note: anyhow already provides a blanket implementation for std::error::Error

/// Result type alias for the hive system
pub type HiveResult<T> = Result<T, HiveError>;

/// Error context for better debugging and observability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// The operation that was being performed when the error occurred
    pub operation: String,
    /// The component/module where the error originated
    pub component: String,
    /// When the error occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional contextual information
    pub additional_info: std::collections::HashMap<String, String>,
    /// Request ID for tracing (if applicable)
    pub request_id: Option<String>,
    /// User ID for user-specific errors (if applicable)
    pub user_id: Option<String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(operation: &str, component: &str) -> Self {
        Self {
            operation: operation.to_string(),
            component: component.to_string(),
            timestamp: chrono::Utc::now(),
            additional_info: std::collections::HashMap::new(),
            request_id: None,
            user_id: None,
        }
    }

    /// Add additional information to the error context
    pub fn with_info(mut self, key: &str, value: &str) -> Self {
        self.additional_info
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Add request ID for tracing
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Add user ID for user-specific errors
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
}

/// Helper macros for creating errors with context
#[macro_export]
macro_rules! hive_error {
    ($error_type:ident, $($field:ident: $value:expr),*) => {
        HiveError::$error_type {
            $($field: $value.to_string()),*
        }
    };
}

/// Helper function to convert anyhow errors to HiveError
pub fn anyhow_to_hive_error(err: anyhow::Error, operation: &str) -> HiveError {
    HiveError::OperationFailed {
        reason: format!("{} failed: {}", operation, err),
    }
}

/// Helper trait for adding context to Results
pub trait ResultExt<T> {
    fn with_context(self, operation: &str, component: &str) -> Result<T, HiveError>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn with_context(self, operation: &str, component: &str) -> Result<T, HiveError> {
        self.map_err(|e| HiveError::OperationFailed {
            reason: format!("{} in {}: {}", operation, component, e),
        })
    }
}
