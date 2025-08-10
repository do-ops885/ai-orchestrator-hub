use std::fmt;
use serde::{Deserialize, Serialize};

/// Custom error types for the multiagent hive system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HiveError {
    /// Agent-related errors
    AgentNotFound(String),
    AgentCreationFailed(String),
    AgentExecutionFailed(String),
    
    /// Task-related errors
    TaskNotFound(String),
    TaskCreationFailed(String),
    TaskExecutionFailed(String),
    
    /// Resource management errors
    ResourceExhausted(String),
    ResourceInitializationFailed(String),
    
    /// Communication errors
    WebSocketError(String),
    MessageParsingError(String),
    
    /// System errors
    SystemOverloaded(String),
    ConfigurationError(String),
    DatabaseError(String),
    
    /// Neural processing errors
    NeuralProcessingError(String),
    NLPError(String),
    
    /// Circuit breaker errors
    CircuitBreakerOpen(String),
    OperationFailed(String),
}

impl fmt::Display for HiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HiveError::AgentNotFound(id) => write!(f, "Agent not found: {}", id),
            HiveError::AgentCreationFailed(reason) => write!(f, "Agent creation failed: {}", reason),
            HiveError::AgentExecutionFailed(reason) => write!(f, "Agent execution failed: {}", reason),
            HiveError::TaskNotFound(id) => write!(f, "Task not found: {}", id),
            HiveError::TaskCreationFailed(reason) => write!(f, "Task creation failed: {}", reason),
            HiveError::TaskExecutionFailed(reason) => write!(f, "Task execution failed: {}", reason),
            HiveError::ResourceExhausted(resource) => write!(f, "Resource exhausted: {}", resource),
            HiveError::ResourceInitializationFailed(reason) => write!(f, "Resource initialization failed: {}", reason),
            HiveError::WebSocketError(reason) => write!(f, "WebSocket error: {}", reason),
            HiveError::MessageParsingError(reason) => write!(f, "Message parsing error: {}", reason),
            HiveError::SystemOverloaded(reason) => write!(f, "System overloaded: {}", reason),
            HiveError::ConfigurationError(reason) => write!(f, "Configuration error: {}", reason),
            HiveError::DatabaseError(reason) => write!(f, "Database error: {}", reason),
            HiveError::NeuralProcessingError(reason) => write!(f, "Neural processing error: {}", reason),
            HiveError::NLPError(reason) => write!(f, "NLP error: {}", reason),
            HiveError::CircuitBreakerOpen(reason) => write!(f, "Circuit breaker open: {}", reason),
            HiveError::OperationFailed(reason) => write!(f, "Operation failed: {}", reason),
        }
    }
}

impl std::error::Error for HiveError {}

// Note: anyhow already provides a blanket implementation for std::error::Error

/// Result type alias for the hive system
pub type HiveResult<T> = Result<T, HiveError>;

/// Error context for better debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub operation: String,
    pub component: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub additional_info: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(operation: &str, component: &str) -> Self {
        Self {
            operation: operation.to_string(),
            component: component.to_string(),
            timestamp: chrono::Utc::now(),
            additional_info: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_info(mut self, key: &str, value: &str) -> Self {
        self.additional_info.insert(key.to_string(), value.to_string());
        self
    }
}