/// Authentication and authorization system
pub mod auth;
/// Centralized error handling tests
#[cfg(test)]
mod centralized_error_handling_tests;
/// Advanced configuration management with validation
pub mod config;
/// Enhanced error handling with structured errors and context
pub mod error;
/// Safe error handling utilities and helpers
pub mod error_handling;
pub mod error_recovery;
/// Health checks and circuit breaker patterns
pub mod health_check;
/// Mathematical utilities
pub mod math;
/// Rate limiting and abuse prevention
pub mod rate_limiter;
/// Security utilities and audit logging
pub mod security;
/// Structured logging and observability
pub mod structured_logging;
/// Legacy testing utilities (deprecated - use `testing_framework`)
pub mod testing;
/// Comprehensive testing framework with benchmarking
pub mod testing_framework;
/// Centralized error handling validation
mod validate_centralized_error_handling;
/// Input validation utilities
pub mod validation;

// Export commonly used types and traits
// pub use auth::*; // Commented out to avoid unused import warnings
pub use config::*;
pub use error::*;
pub use security::*;
pub use validation::*;
// pub use testing_framework::TestHarness;
