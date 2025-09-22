//! Phase 3: Logging Standardization
//!
//! Implements structured logging with tracing, request tracing with correlation IDs,
//! and configurable log levels for operational excellence.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, Level, Span};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};
use uuid::Uuid;

/// Configuration for structured logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Log format (json, text)
    pub format: String,
    /// Enable request tracing
    pub request_tracing: bool,
    /// Enable correlation IDs
    pub correlation_ids: bool,
    /// Log file path (optional)
    pub file_path: Option<String>,
    /// Maximum log file size in MB
    pub max_file_size_mb: usize,
    /// Maximum number of log files to keep
    pub max_files: usize,
    /// Custom log fields
    pub custom_fields: HashMap<String, String>,
}

/// Structured logger with configurable output
#[derive(Clone)]
pub struct StructuredLogger {
    config: Arc<RwLock<LoggingConfig>>,
    request_tracer: Arc<RequestTracer>,
}

impl StructuredLogger {
    /// Create a new structured logger
    pub fn new(config: LoggingConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let logger = Self {
            config: Arc::new(RwLock::new(config)),
            request_tracer: Arc::new(RequestTracer::new()),
        };

        logger.init_tracing_subscriber()?;
        Ok(logger)
    }

    /// Initialize the tracing subscriber based on configuration
    fn init_tracing_subscriber(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let config = self.config.try_read().expect("replaced unwrap");

        // Add environment filter
        let filter = EnvFilter::try_from_env("RUST_LOG")
            .unwrap_or_else(|_| EnvFilter::new(config.level.clone()));

        // Configure the formatter based on format and create subscriber
        let subscriber: Box<dyn tracing::Subscriber + Send + Sync> = if config.format == "json" {
            let formatter = fmt::layer()
                .json()
                .with_target(true)
                .with_current_span(true)
                .with_span_list(true);

            Box::new(Registry::default().with(filter).with(formatter))
        } else {
            let formatter = fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true);

            Box::new(Registry::default().with(filter).with(formatter))
        };

        // Add file logging if configured
        if let Some(file_path) = &config.file_path {
            // Note: In production, you'd want to use a proper file appender
            // For now, we'll just log to stdout with file indication
            info!("File logging configured for: {}", file_path);
        }

        // Set as global default
        tracing::subscriber::set_global_default(subscriber)
            .map_err(|e| format!("Failed to set global subscriber: {}", e))?;

        info!(
            "Structured logging initialized with level: {}",
            config.level
        );
        Ok(())
    }

    /// Update logging configuration (hot-reload)
    pub async fn update_config(
        &self,
        new_config: LoggingConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        *self.config.write().await = new_config.clone();
        self.init_tracing_subscriber()?;
        info!("Logging configuration updated");
        Ok(())
    }

    /// Get current configuration
    pub async fn get_config(&self) -> LoggingConfig {
        self.config.read().await.clone()
    }

    /// Log a structured message with custom fields
    pub fn log_structured(
        &self,
        level: Level,
        message: &str,
        fields: HashMap<String, serde_json::Value>,
    ) {
        let mut log_fields = fields.clone();

        // Add custom fields from config
        let config = self.config.try_read().expect("replaced unwrap");
        for (key, value) in &config.custom_fields {
            log_fields.insert(key.clone(), serde_json::Value::String(value.clone()));
        }

        match level {
            Level::TRACE => tracing::trace!(message, fields = ?log_fields),
            Level::DEBUG => tracing::debug!(message, fields = ?log_fields),
            Level::INFO => tracing::info!(message, fields = ?log_fields),
            Level::WARN => tracing::warn!(message, fields = ?log_fields),
            Level::ERROR => tracing::error!(message, fields = ?log_fields),
        }
    }

    /// Get the request tracer
    pub fn request_tracer(&self) -> Arc<RequestTracer> {
        Arc::clone(&self.request_tracer)
    }
}

/// Request tracer for correlation IDs and distributed tracing
#[derive(Clone)]
pub struct RequestTracer {
    active_requests: Arc<RwLock<HashMap<String, RequestContext>>>,
}

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub correlation_id: String,
    pub start_time: std::time::Instant,
    pub method: String,
    pub path: String,
    pub user_id: Option<String>,
    pub custom_fields: HashMap<String, String>,
}

impl RequestTracer {
    /// Create a new request tracer
    pub fn new() -> Self {
        Self {
            active_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start tracing a request
    pub async fn start_request(&self, method: &str, path: &str, user_id: Option<String>) -> String {
        let correlation_id = Uuid::new_v4().to_string();
        let context = RequestContext {
            correlation_id: correlation_id.clone(),
            start_time: std::time::Instant::now(),
            method: method.to_string(),
            path: path.to_string(),
            user_id,
            custom_fields: HashMap::new(),
        };

        self.active_requests
            .write()
            .await
            .insert(correlation_id.clone(), context);

        info!(
            correlation_id = %correlation_id,
            method = %method,
            path = %path,
            "Request started"
        );

        correlation_id
    }

    /// End tracing a request
    pub async fn end_request(
        &self,
        correlation_id: &str,
        status_code: u16,
        response_size: Option<usize>,
    ) {
        if let Some(context) = self.active_requests.write().await.remove(correlation_id) {
            let duration = context.start_time.elapsed();

            info!(
                correlation_id = %correlation_id,
                method = %context.method,
                path = %context.path,
                status = %status_code,
                duration_ms = %duration.as_millis(),
                response_size = ?response_size,
                "Request completed"
            );
        }
    }

    /// Add custom field to request context
    pub async fn add_request_field(&self, correlation_id: &str, key: String, value: String) {
        if let Some(context) = self.active_requests.write().await.get_mut(correlation_id) {
            context.custom_fields.insert(key, value);
        }
    }

    /// Get request context
    pub async fn get_request_context(&self, correlation_id: &str) -> Option<RequestContext> {
        self.active_requests
            .read()
            .await
            .get(correlation_id)
            .cloned()
    }

    /// Get all active requests
    pub async fn get_active_requests(&self) -> HashMap<String, RequestContext> {
        self.active_requests.read().await.clone()
    }

    /// Clean up old completed requests (garbage collection)
    pub async fn cleanup_old_requests(&self, max_age_secs: u64) {
        let mut active_requests = self.active_requests.write().await;
        let now = std::time::Instant::now();

        active_requests
            .retain(|_, context| now.duration_since(context.start_time).as_secs() < max_age_secs);
    }
}

/// Tracing span helpers for instrumenting code
pub struct TracingSpan;

impl TracingSpan {
    /// Create a span for tool execution
    pub fn tool_execution(tool_name: &str, tool_id: Option<&str>) -> Span {
        tracing::info_span!(
            "tool_execution",
            tool_name = %tool_name,
            tool_id = ?tool_id,
        )
    }

    /// Create a span for cache operations
    pub fn cache_operation(operation: &str, cache_type: &str, key: Option<&str>) -> Span {
        tracing::debug_span!(
            "cache_operation",
            operation = %operation,
            cache_type = %cache_type,
            key = ?key,
        )
    }

    /// Create a span for database operations
    pub fn database_operation(operation: &str, table: &str, query: Option<&str>) -> Span {
        tracing::debug_span!(
            "database_operation",
            operation = %operation,
            table = %table,
            query = ?query,
        )
    }

    /// Create a span for external API calls
    pub fn external_api_call(method: &str, url: &str, service: Option<&str>) -> Span {
        tracing::info_span!(
            "external_api_call",
            method = %method,
            url = %url,
            service = ?service,
        )
    }

    /// Create a span for agent operations
    pub fn agent_operation(agent_id: &str, operation: &str, task_id: Option<&str>) -> Span {
        tracing::info_span!(
            "agent_operation",
            agent_id = %agent_id,
            operation = %operation,
            task_id = ?task_id,
        )
    }
}

/// Log level utilities
pub struct LogLevel;

impl LogLevel {
    /// Parse log level from string
    pub fn from_str(level: &str) -> Result<Level, Box<dyn std::error::Error + Send + Sync>> {
        match level.to_lowercase().as_str() {
            "trace" => Ok(Level::TRACE),
            "debug" => Ok(Level::DEBUG),
            "info" => Ok(Level::INFO),
            "warn" => Ok(Level::WARN),
            "error" => Ok(Level::ERROR),
            _ => Err(format!("Invalid log level: {}", level).into()),
        }
    }

    /// Convert log level to string
    pub fn to_string(level: Level) -> &'static str {
        match level {
            Level::TRACE => "trace",
            Level::DEBUG => "debug",
            Level::INFO => "info",
            Level::WARN => "warn",
            Level::ERROR => "error",
        }
    }
}

/// Macros for structured logging
#[macro_export]
macro_rules! log_request {
    ($tracer:expr, $method:expr, $path:expr) => {
        $tracer.start_request($method, $path, None).await
    };
    ($tracer:expr, $method:expr, $path:expr, $user_id:expr) => {
        $tracer.start_request($method, $path, Some($user_id)).await
    };
}

#[macro_export]
macro_rules! log_request_end {
    ($tracer:expr, $correlation_id:expr, $status:expr) => {
        $tracer.end_request($correlation_id, $status, None).await
    };
    ($tracer:expr, $correlation_id:expr, $status:expr, $response_size:expr) => {
        $tracer
            .end_request($correlation_id, $status, Some($response_size))
            .await
    };
}

#[macro_export]
macro_rules! log_structured {
    ($logger:expr, $level:expr, $message:expr) => {
        $logger.log_structured($level, $message, std::collections::HashMap::new())
    };
    ($logger:expr, $level:expr, $message:expr, $($key:expr => $value:expr),*) => {{
        let mut fields = std::collections::HashMap::new();
        $(
            fields.insert($key.to_string(), serde_json::json!($value));
        )*
        $logger.log_structured($level, $message, fields)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_request_tracer() {
        let tracer = RequestTracer::new();

        // Start a request
        let correlation_id = tracer
            .start_request("GET", "/api/test", Some("user123"))
            .await;

        // Verify it's tracked
        let context = tracer
            .get_request_context(&correlation_id)
            .await
            .expect("replaced unwrap");
        assert_eq!(context.method, "GET");
        assert_eq!(context.path, "/api/test");
        assert_eq!(context.user_id, Some("user123".to_string()));

        // Add a custom field
        tracer
            .add_request_field(
                &correlation_id,
                "custom_field".to_string(),
                "custom_value".to_string(),
            )
            .await;

        let context = tracer
            .get_request_context(&correlation_id)
            .await
            .expect("replaced unwrap");
        assert_eq!(
            context.custom_fields.get("custom_field"),
            Some(&"custom_value".to_string())
        );

        // End the request
        tracer.end_request(&correlation_id, 200, Some(1024)).await;

        // Verify it's removed from active requests
        assert!(tracer.get_request_context(&correlation_id).await.is_none());
    }

    #[tokio::test]
    async fn test_cleanup_old_requests() {
        let tracer = RequestTracer::new();

        // Start a request
        let correlation_id = tracer.start_request("GET", "/api/test", None).await;

        // Wait a bit
        sleep(Duration::from_millis(10)).await;

        // Clean up requests older than 1 millisecond
        tracer.cleanup_old_requests(1).await;

        // Request should be cleaned up
        assert!(tracer.get_request_context(&correlation_id).await.is_none());
    }

    #[test]
    fn test_log_level_parsing() {
        assert_eq!(
            LogLevel::from_str("info").expect("replaced unwrap"),
            Level::INFO
        );
        assert_eq!(
            LogLevel::from_str("DEBUG").expect("replaced unwrap"),
            Level::DEBUG
        );
        assert_eq!(LogLevel::from_str("invalid").is_err(), true);
    }

    #[test]
    fn test_log_level_to_string() {
        assert_eq!(LogLevel::to_string(Level::WARN), "warn");
        assert_eq!(LogLevel::to_string(Level::ERROR), "error");
    }
}
