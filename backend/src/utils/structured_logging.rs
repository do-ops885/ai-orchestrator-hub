use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{Level, event};
use uuid::Uuid;

/// Structured logging utilities for better observability and debugging
pub struct StructuredLogger;

impl StructuredLogger {
    /// Log agent lifecycle events with structured data
    pub fn log_agent_event(event_type: AgentEventType, agent_id: Uuid, details: AgentEventDetails) {
        event!(
            Level::INFO,
            agent_id = %agent_id,
            event_type = ?event_type,
            agent_name = %details.agent_name,
            agent_type = %details.agent_type,
            capabilities_count = details.capabilities.len(),
            energy_level = details.energy_level,
            "Agent lifecycle event"
        );
    }

    /// Log task execution events with performance metrics
    pub fn log_task_event(event_type: TaskEventType, task_id: Uuid, details: TaskEventDetails) {
        event!(
            Level::INFO,
            task_id = %task_id,
            event_type = ?event_type,
            task_description = %details.description,
            priority = ?details.priority,
            assigned_agent = ?details.assigned_agent,
            execution_time_ms = details.execution_time_ms,
            success = details.success,
            "Task execution event"
        );
    }

    /// Log system performance metrics
    pub fn log_performance_metrics(metrics: PerformanceMetrics) {
        event!(
            Level::INFO,
            cpu_usage = metrics.cpu_usage,
            memory_usage = metrics.memory_usage,
            active_agents = metrics.active_agents,
            pending_tasks = metrics.pending_tasks,
            completed_tasks = metrics.completed_tasks,
            failed_tasks = metrics.failed_tasks,
            average_response_time_ms = metrics.average_response_time_ms,
            "System performance metrics"
        );
    }

    /// Log security events for audit trails
    pub fn log_security_event(event_type: SecurityEventType, details: SecurityEventDetails) {
        match event_type {
            SecurityEventType::RateLimitExceeded | SecurityEventType::InvalidInput => {
                event!(
                    Level::WARN,
                    event_type = ?event_type,
                    client_id = %details.client_id,
                    endpoint = %details.endpoint,
                    user_agent = ?details.user_agent,
                    ip_address = ?details.ip_address,
                    timestamp = %details.timestamp,
                    "Security event"
                );
            }
            SecurityEventType::UnauthorizedAccess | SecurityEventType::SuspiciousActivity => {
                event!(
                    Level::ERROR,
                    event_type = ?event_type,
                    client_id = %details.client_id,
                    endpoint = %details.endpoint,
                    user_agent = ?details.user_agent,
                    ip_address = ?details.ip_address,
                    timestamp = %details.timestamp,
                    "Security event"
                );
            }
            SecurityEventType::AuthenticationSuccess => {
                event!(
                    Level::INFO,
                    event_type = ?event_type,
                    client_id = %details.client_id,
                    endpoint = %details.endpoint,
                    user_agent = ?details.user_agent,
                    ip_address = ?details.ip_address,
                    timestamp = %details.timestamp,
                    "Security event"
                );
            }
        }
    }

    /// Log API request/response for debugging
    pub fn log_api_request(
        method: &str,
        path: &str,
        status_code: u16,
        duration_ms: u64,
        client_id: &str,
    ) {
        match status_code {
            200..=299 => {
                event!(
                    Level::INFO,
                    method = method,
                    path = path,
                    status_code = status_code,
                    duration_ms = duration_ms,
                    client_id = client_id,
                    "API request completed"
                );
            }
            400..=499 => {
                event!(
                    Level::WARN,
                    method = method,
                    path = path,
                    status_code = status_code,
                    duration_ms = duration_ms,
                    client_id = client_id,
                    "API request completed"
                );
            }
            500..=599 => {
                event!(
                    Level::ERROR,
                    method = method,
                    path = path,
                    status_code = status_code,
                    duration_ms = duration_ms,
                    client_id = client_id,
                    "API request completed"
                );
            }
            _ => {
                event!(
                    Level::DEBUG,
                    method = method,
                    path = path,
                    status_code = status_code,
                    duration_ms = duration_ms,
                    client_id = client_id,
                    "API request completed"
                );
            }
        }
    }

    /// Log neural network training events
    pub fn log_neural_event(event_type: NeuralEventType, details: NeuralEventDetails) {
        event!(
            Level::INFO,
            event_type = ?event_type,
            model_type = %details.model_type,
            training_samples = details.training_samples,
            accuracy = details.accuracy,
            loss = details.loss,
            epoch = details.epoch,
            learning_rate = details.learning_rate,
            "Neural network event"
        );
    }

    /// Log errors with context for better debugging
    pub fn log_error_with_context(error: &dyn std::error::Error, context: ErrorContext) {
        event!(
            Level::ERROR,
            error = %error,
            operation = %context.operation,
            component = %context.component,
            agent_id = ?context.agent_id,
            task_id = ?context.task_id,
            additional_data = ?context.additional_data,
            "Error occurred with context"
        );
    }
}

/// Agent lifecycle event types
#[derive(Debug, Serialize, Deserialize)]
pub enum AgentEventType {
    Created,
    Started,
    TaskAssigned,
    TaskCompleted,
    TaskFailed,
    LearningUpdate,
    EnergyChanged,
    StateChanged,
    Destroyed,
}

/// Agent event details for structured logging
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentEventDetails {
    pub agent_name: String,
    pub agent_type: String,
    pub capabilities: Vec<String>,
    pub energy_level: f64,
    pub state: String,
    pub position: Option<(f64, f64)>,
}

/// Task execution event types
#[derive(Debug, Serialize, Deserialize)]
pub enum TaskEventType {
    Created,
    Queued,
    Assigned,
    Started,
    Completed,
    Failed,
    Cancelled,
    Retried,
}

/// Task event details for structured logging
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskEventDetails {
    pub description: String,
    pub priority: String,
    pub assigned_agent: Option<Uuid>,
    pub execution_time_ms: Option<u64>,
    pub success: bool,
    pub error_message: Option<String>,
    pub retry_count: u32,
}

/// System performance metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub active_agents: usize,
    pub pending_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub average_response_time_ms: f64,
    pub uptime_seconds: u64,
}

/// Security event types for audit logging
#[derive(Debug, Serialize, Deserialize)]
pub enum SecurityEventType {
    AuthenticationSuccess,
    UnauthorizedAccess,
    RateLimitExceeded,
    InvalidInput,
    SuspiciousActivity,
}

/// Security event details
#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityEventDetails {
    pub client_id: String,
    pub endpoint: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub additional_info: HashMap<String, String>,
}

/// Neural network event types
#[derive(Debug, Serialize, Deserialize)]
pub enum NeuralEventType {
    TrainingStarted,
    TrainingCompleted,
    ModelUpdated,
    PredictionMade,
    PerformanceEvaluated,
}

/// Neural network event details
#[derive(Debug, Serialize, Deserialize)]
pub struct NeuralEventDetails {
    pub model_type: String,
    pub training_samples: usize,
    pub accuracy: Option<f64>,
    pub loss: Option<f64>,
    pub epoch: Option<u32>,
    pub learning_rate: Option<f64>,
}

/// Error context for better debugging
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorContext {
    pub operation: String,
    pub component: String,
    pub agent_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub additional_data: HashMap<String, String>,
}

impl ErrorContext {
    /// Create new error context
    pub fn new(operation: &str, component: &str) -> Self {
        Self {
            operation: operation.to_string(),
            component: component.to_string(),
            agent_id: None,
            task_id: None,
            additional_data: HashMap::new(),
        }
    }

    /// Add agent context
    pub fn with_agent(mut self, agent_id: Uuid) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    /// Add task context
    pub fn with_task(mut self, task_id: Uuid) -> Self {
        self.task_id = Some(task_id);
        self
    }

    /// Add additional data
    pub fn with_data(mut self, key: &str, value: &str) -> Self {
        self.additional_data
            .insert(key.to_string(), value.to_string());
        self
    }
}

/// Macro for easy structured logging
#[macro_export]
macro_rules! log_agent_event {
    ($event_type:expr, $agent_id:expr, $details:expr) => {
        $crate::utils::structured_logging::StructuredLogger::log_agent_event(
            $event_type,
            $agent_id,
            $details,
        );
    };
}

#[macro_export]
macro_rules! log_task_event {
    ($event_type:expr, $task_id:expr, $details:expr) => {
        $crate::utils::structured_logging::StructuredLogger::log_task_event(
            $event_type,
            $task_id,
            $details,
        );
    };
}

#[macro_export]
macro_rules! log_security_event {
    ($event_type:expr, $details:expr) => {
        $crate::utils::structured_logging::StructuredLogger::log_security_event(
            $event_type,
            $details,
        );
    };
}
