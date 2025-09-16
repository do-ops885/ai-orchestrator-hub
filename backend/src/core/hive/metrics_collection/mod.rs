//! # Metrics Collection Module
//!
//! This module provides comprehensive metrics collection, aggregation, and reporting
//! for the hive system, enabling performance monitoring and analytics.
//!
//! ## Architecture
//!
//! The metrics system uses a multi-layered approach:
//!
//! - **`types`**: Core data structures and types for metrics
//! - **`collector`**: Main metrics collection and aggregation logic
//! - **`export`**: Export functionality for different formats (JSON, Prometheus)
//! - **`history`**: Historical data tracking and trend analysis
//! - **`events`**: Event recording and counter management
//!
//! ## Key Features
//!
//! - **Multi-format Export**: JSON and Prometheus formats supported
//! - **Trend Analysis**: Historical data with configurable retention
//! - **Event Tracking**: Real-time counters for system events
//! - **Performance Scoring**: Agent efficiency and system health metrics
//! - **Resource Monitoring**: System resource usage tracking
//!
//! ## Usage
//!
//! ```rust,no_run
//! use hive::core::hive::metrics_collection::MetricsCollector;
//! use tokio::sync::mpsc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let (tx, _rx) = mpsc::unbounded_channel();
//!
//! let metrics_collector = MetricsCollector::new(tx).await?;
//!
//! // Record some events
//! let agent_id = uuid::Uuid::new_v4();
//! let task_id = uuid::Uuid::new_v4();
//!
//! metrics_collector.record_agent_event("registered", agent_id).await;
//! metrics_collector.record_task_completion(task_id, agent_id, true).await;
//!
//! // Get current metrics
//! let metrics = metrics_collector.get_current_metrics().await;
//! println!("Current metrics: {:?}", metrics);
//!
//! // Export in different formats
//! let json_export = metrics_collector.export_metrics("json").await?;
//! let prometheus_export = metrics_collector.export_metrics("prometheus").await?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Memory Usage**: O(n) where n is active agents + tasks + history size
//! - **CPU Overhead**: Minimal for event recording, periodic for aggregation
//! - **Storage**: Bounded history (1000 snapshots max)
//! - **Export Time**: O(1) for current metrics, O(n) for full export
//! - **Concurrency**: High concurrency with atomic operations

/// Core data structures and types for metrics
pub mod types;

/// Main metrics collection and aggregation logic
pub mod collector;

/// Export functionality for different formats
pub mod export;

/// Historical data tracking and trend analysis
pub mod history;

/// Event recording and counter management
pub mod events;

// Re-export the main types for convenience
pub use collector::MetricsCollector;
pub use types::{HiveMetrics, SwarmMetrics};
