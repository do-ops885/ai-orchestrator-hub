//! Modular Task Management System
//!
//! This module provides comprehensive task lifecycle management broken down into
//! focused, maintainable components following the single responsibility principle.

pub mod task_creation;
pub mod task_distribution;
pub mod task_distributor;
pub mod task_executor;
pub mod task_maintenance;
pub mod task_metrics;
pub mod task_queue;
pub mod task_status;
pub mod task_types;

// Re-export main types for backward compatibility
pub use task_distributor::TaskDistributor;
pub use task_types::*;

// Re-export key components
pub use task_creation::TaskCreator;
pub use task_distribution::TaskDistributor as TaskDistributionManager;
pub use task_executor::TaskExecutor as TaskExecutionManager;
pub use task_maintenance::TaskMaintenanceManager;
pub use task_metrics::TaskMetricsCollector;
pub use task_queue::TaskQueueManager;
pub use task_status::TaskStatusReporter;
