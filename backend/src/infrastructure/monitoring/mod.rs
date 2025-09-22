//! Modular Monitoring Infrastructure
//!
//! This module provides comprehensive monitoring capabilities broken down into
//! focused, maintainable components following the single responsibility principle.

pub mod agent_discovery;
pub mod agent_monitor;
pub mod automation;
pub mod behavior_analyzer;
pub mod dashboard;
pub mod diagnostics;
pub mod health_checks;
pub mod health_monitor;
pub mod integration;
pub mod logging;
pub mod performance_monitor;
pub mod phase3_metrics;
pub mod production_monitoring;
pub mod prometheus_exporter;
pub mod reporting;
pub mod types;

// Re-export main types for backward compatibility
pub use agent_monitor::AgentMonitor;
pub use types::*;

// Re-export key components
pub use agent_discovery::AgentDiscovery;
pub use automation::Automation;
pub use behavior_analyzer::BehaviorAnalyzer;
pub use dashboard::{Dashboard, EnhancedDashboard};
pub use diagnostics::Diagnostics;
pub use health_checks::{HealthChecker, HealthReport, HealthStatus};
pub use health_monitor::HealthMonitor;
pub use integration::Integration;
pub use logging::{LoggingConfig, RequestTracer, StructuredLogger};
pub use performance_monitor::PerformanceMonitor;
pub use phase3_metrics::Phase3MetricsCollector;
pub use production_monitoring::{ProductionMonitoringConfig, ProductionMonitoringSystem};
pub use prometheus_exporter::PrometheusExporter;
pub use reporting::Reporting;
