/// Performance benchmarking and testing utilities
pub mod benchmarks;
/// Intelligent caching system with TTL and size management
pub mod cache;
/// Circuit breaker pattern implementation for fault tolerance
pub mod circuit_breaker;
/// Intelligent alerting system with predictive analytics
pub mod intelligent_alerting;
/// Memory pool management for efficient allocations
pub mod memory_pool;
/// Comprehensive metrics collection and analysis
pub mod metrics;
/// HTTP middleware for request processing
pub mod middleware;
/// System monitoring and health checks
pub mod monitoring;
/// Performance optimization and auto-tuning
pub mod performance_optimizer;
/// Data persistence and state management
pub mod persistence;
/// Resource allocation and management
pub mod resource_manager;
/// Security middleware and request validation
pub mod security_middleware;
/// Telemetry collection and reporting
pub mod telemetry;

pub use benchmarks::{
    create_default_benchmark_suite, BenchmarkSuite,
    PerformanceMonitor as BenchmarkPerformanceMonitor,
};
pub use cache::*;
pub use circuit_breaker::*;
pub use intelligent_alerting::*;
pub use memory_pool::*;
pub use metrics::{MetricsCollector, PerformanceMetrics, SystemMetrics};
pub use middleware::*;
pub use monitoring::{AgentMonitor, PerformanceMonitor as MonitoringPerformanceMonitor};
pub use performance_optimizer::PerformanceOptimizer;
pub use persistence::*;
pub use resource_manager::*;
pub use security_middleware::*;
pub use telemetry::*;
