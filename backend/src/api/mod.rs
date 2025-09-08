/// Enhanced API handlers with structured error handling
///
/// This module provides production-ready API handlers that use:
/// - Structured error handling with proper HTTP status codes
/// - Input validation and sanitization
/// - Comprehensive logging and metrics
/// - Circuit breaker pattern for resilience
/// - Rate limiting and security features

pub mod handlers;
pub mod responses;
pub mod validation;

pub use handlers::*;
pub use responses::*;
pub use validation::*;
