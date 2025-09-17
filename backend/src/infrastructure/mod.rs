pub mod async_optimizer;
/// Performance benchmarking and testing utilities
pub mod benchmarks;
/// Intelligent caching system with TTL and size management
pub mod cache;
/// Cache invalidation manager with dependency tracking
pub mod cache_invalidation;
/// Cache performance monitoring and metrics
pub mod cache_monitoring;
/// Cache optimization strategies and benchmarks
pub mod cache_optimization;
/// Cache warming and prefetching system
pub mod cache_warming;
/// Cached database query wrapper with intelligent caching
pub mod cached_query;
/// Circuit breaker pattern implementation for fault tolerance
pub mod circuit_breaker;
/// Intelligent alerting system with predictive analytics
pub mod intelligent_alerting;
pub mod intelligent_cache;
/// Memory pool management for efficient allocations
pub mod memory_pool;
/// Comprehensive metrics collection and analysis
pub mod metrics;
/// HTTP middleware for request processing
pub mod middleware;
/// System monitoring and health checks
pub mod monitoring;
pub mod performance_integration;
/// Performance optimization and auto-tuning
pub mod performance_optimizer;
/// Data persistence and state management
pub mod persistence;
/// Resource allocation and management
pub mod resource_manager;
/// Security middleware and request validation
pub mod security_middleware;
pub mod streaming;
pub use streaming::{StreamConfig, StreamProcessor};
/// Telemetry collection and reporting
pub mod telemetry;

pub use benchmarks::{
    create_default_benchmark_suite, BenchmarkSuite,
    PerformanceMonitor as BenchmarkPerformanceMonitor,
};
pub use cache::*;
pub use cache_invalidation::*;
pub use cache_monitoring::*;
pub use cache_optimization::*;
pub use cache_warming::*;
pub use cached_query::*;
pub use circuit_breaker::*;
pub use intelligent_alerting::*;
pub use intelligent_cache::*;
pub use memory_pool::*;
pub use metrics::{MetricsCollector, PerformanceMetrics, SystemMetrics};
pub use middleware::*;
pub use monitoring::{
    AgentMonitor, PerformanceMonitor as MonitoringPerformanceMonitor, ProductionMonitoringConfig,
    ProductionMonitoringSystem,
};
pub use performance_optimizer::PerformanceOptimizer;
pub use persistence::*;
pub use resource_manager::*;
pub use security_middleware::*;
pub use telemetry::*;
