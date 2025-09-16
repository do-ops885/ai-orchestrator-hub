//! # Task Management Legacy Module
//!
//! This module provides comprehensive task lifecycle management including
//! creation, distribution, execution tracking, and performance monitoring.
//!
//! ## Architecture
//!
//! The task management system uses a dual-queue approach:
//!
//! - **`types`**: Core data structures and types for tasks and execution
//! - **`distributor`**: Main task distributor with queue management
//! - **`execution`**: Task execution logic and result processing
//! - **`analytics`**: Analytics and reporting functionality
//!
//! ## Key Features
//!
//! - **Dual Queue System**: Work-stealing + legacy queue for reliability
//! - **Task Verification**: Comprehensive execution validation
//! - **Performance Tracking**: Detailed execution metrics and analytics
//! - **Priority Handling**: Task prioritization and scheduling
//! - **Error Recovery**: Robust error handling and retry mechanisms
//!
//! ## Usage
//!
//! ```rust,no_run
//! use hive::core::hive::task_management_legacy::TaskDistributor;
//! use std::sync::Arc;
//! use tokio::sync::mpsc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let resource_manager = Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);
//! let (tx, _rx) = mpsc::unbounded_channel();
//!
//! let task_distributor = TaskDistributor::new(resource_manager, tx).await?;
//!
//! // Create a task
//! let config = serde_json::json!({
//!     "type": "computation",
//!     "title": "Example Task",
//!     "description": "A sample task"
//! });
//! let task_id = task_distributor.create_task(config).await?;
//!
//! // Get system status
//! let status = task_distributor.get_status().await;
//! println!("Task status: {}", status);
//!
//! // Get analytics
//! let analytics = task_distributor.get_analytics().await;
//! println!("Task analytics: {}", analytics);
//! # Ok(())
//! # }
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Task Creation**: O(log n) for queue insertion
//! - **Task Execution**: Variable based on task complexity
//! - **Status Retrieval**: O(n) for comprehensive status
//! - **Memory Usage**: O(n) where n is queued + active tasks
//! - **Concurrency**: High concurrency with async operations

/// Core data structures and types for tasks
pub mod types;

/// Main task distributor with queue management
pub mod distributor;

/// Task execution logic and result processing
pub mod execution;

/// Analytics and reporting functionality
pub mod analytics;

// Re-export the main types for convenience
pub use distributor::TaskDistributor;
pub use types::{TaskExecutionResult, TaskMetrics, TaskStatus};
