/// Authentication and authorization system
pub mod auth;
/// Advanced configuration management with validation
pub mod config;
/// Enhanced error handling with structured errors and context
pub mod error;
/// Health checks and circuit breaker patterns
pub mod health_check;
/// Rate limiting and abuse prevention
pub mod rate_limiter;
/// Security utilities and audit logging
pub mod security;
/// Structured logging and observability
pub mod structured_logging;
/// Legacy testing utilities (deprecated - use testing_framework)
pub mod testing;
/// Comprehensive testing framework with benchmarking
pub mod testing_framework;
/// Input validation utilities
pub mod validation;

// Export commonly used types and traits
pub use auth::*;
pub use config::*;
pub use error::*;
pub use security::*;
pub use validation::*;
// pub use testing_framework::TestHarness;
