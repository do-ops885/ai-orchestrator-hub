/// Standardized API response types with enhanced error handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use crate::utils::error::HiveError;

/// Standard API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Whether the operation was successful
    pub success: bool,
    /// Response data (present on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// Error information (present on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Request ID for tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Standardized error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    /// Error code for programmatic handling
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// Field-specific validation errors
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub field_errors: Vec<FieldError>,
}

/// Field-specific validation error
#[derive(Debug, Serialize, Deserialize)]
pub struct FieldError {
    /// Field name that failed validation
    pub field: String,
    /// Validation error message
    pub message: String,
    /// Invalid value (if safe to include)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Create a successful response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
            request_id: None,
        }
    }
    
    /// Create a successful response with request ID
    pub fn success_with_id(data: T, request_id: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
            request_id: Some(request_id),
        }
    }
    
    /// Create an error response
    pub fn error(error: ApiError) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(error),
            timestamp: chrono::Utc::now(),
            request_id: None,
        }
    }
    
    /// Create an error response with request ID
    pub fn error_with_id(error: ApiError, request_id: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(error),
            timestamp: chrono::Utc::now(),
            request_id: Some(request_id),
        }
    }
}

impl ApiError {
    /// Create a new API error
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: None,
            field_errors: Vec::new(),
        }
    }
    
    /// Add details to the error
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
    
    /// Add field errors
    pub fn with_field_errors(mut self, field_errors: Vec<FieldError>) -> Self {
        self.field_errors = field_errors;
        self
    }
}

impl FieldError {
    /// Create a new field error
    pub fn new(field: &str, message: &str) -> Self {
        Self {
            field: field.to_string(),
            message: message.to_string(),
            value: None,
        }
    }
    
    /// Add the invalid value
    pub fn with_value(mut self, value: &str) -> Self {
        self.value = Some(value.to_string());
        self
    }
}

/// Convert HiveError to ApiError with appropriate HTTP status
impl From<HiveError> for (StatusCode, Json<ApiResponse<()>>) {
    fn from(error: HiveError) -> Self {
        let (status_code, api_error) = match error {
            HiveError::AgentNotFound { id } => (
                StatusCode::NOT_FOUND,
                ApiError::new("AGENT_NOT_FOUND", &format!("Agent not found: {}", id))
            ),
            HiveError::AgentCreationFailed { reason } => (
                StatusCode::BAD_REQUEST,
                ApiError::new("AGENT_CREATION_FAILED", &reason)
            ),
            HiveError::TaskNotFound { id } => (
                StatusCode::NOT_FOUND,
                ApiError::new("TASK_NOT_FOUND", &format!("Task not found: {}", id))
            ),
            HiveError::TaskCreationFailed { reason } => (
                StatusCode::BAD_REQUEST,
                ApiError::new("TASK_CREATION_FAILED", &reason)
            ),
            HiveError::ValidationError { field, reason } => (
                StatusCode::BAD_REQUEST,
                ApiError::new("VALIDATION_ERROR", "Input validation failed")
                    .with_field_errors(vec![FieldError::new(&field, &reason)])
            ),
            HiveError::ResourceExhausted { resource } => (
                StatusCode::SERVICE_UNAVAILABLE,
                ApiError::new("RESOURCE_EXHAUSTED", &format!("Resource exhausted: {}", resource))
            ),
            HiveError::SystemOverloaded { reason } => (
                StatusCode::SERVICE_UNAVAILABLE,
                ApiError::new("SYSTEM_OVERLOADED", &reason)
            ),
            HiveError::ConfigurationError { reason } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiError::new("CONFIGURATION_ERROR", &reason)
            ),
            HiveError::TimeoutError { operation, duration_ms } => (
                StatusCode::REQUEST_TIMEOUT,
                ApiError::new("TIMEOUT_ERROR", &format!("Operation '{}' timed out after {}ms", operation, duration_ms))
            ),
            HiveError::CircuitBreakerOpen { reason } => (
                StatusCode::SERVICE_UNAVAILABLE,
                ApiError::new("CIRCUIT_BREAKER_OPEN", &reason)
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiError::new("INTERNAL_ERROR", "An internal error occurred")
            ),
        };
        
        (status_code, Json(ApiResponse::error(api_error)))
    }
}

/// Result type for API handlers
pub type ApiResult<T> = Result<Json<ApiResponse<T>>, (StatusCode, Json<ApiResponse<()>>)>;

/// Helper function to create success responses
pub fn success<T>(data: T) -> ApiResult<T> {
    Ok(Json(ApiResponse::success(data)))
}

/// Helper function to create error responses
pub fn error(status: StatusCode, code: &str, message: &str) -> (StatusCode, Json<ApiResponse<()>>) {
    (status, Json(ApiResponse::error(ApiError::new(code, message))))
}

/// Helper function to create validation error responses
pub fn validation_error(field_errors: Vec<FieldError>) -> (StatusCode, Json<ApiResponse<()>>) {
    let api_error = ApiError::new("VALIDATION_ERROR", "Input validation failed")
        .with_field_errors(field_errors);
    (StatusCode::BAD_REQUEST, Json(ApiResponse::error(api_error)))
}