/// Enhanced error handling with structured errors and context
pub mod error;
/// Advanced configuration management with validation
pub mod config;
/// Health checks and circuit breaker patterns
pub mod health_check;
/// Rate limiting and abuse prevention
pub mod rate_limiter;
/// Structured logging and observability
pub mod structured_logging;
/// Input validation utilities
pub mod validation;
/// Legacy testing utilities (deprecated - use testing_framework)
pub mod testing;
/// Comprehensive testing framework with benchmarking
pub mod testing_framework;

// Export commonly used types and traits
pub use error::*;
pub use config::*;
pub use validation::*;
pub use testing_framework::{TestHarness, TestConfig, TestReport};