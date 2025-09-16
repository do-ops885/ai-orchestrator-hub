//! # Agent Management Module
//!
//! This module provides comprehensive agent lifecycle management including
//! registration, monitoring, performance tracking, and resource allocation.
//!
//! ## Architecture
//!
//! The agent management system uses a modular approach:
//!
//! - **`types`**: Core data structures and types
//! - **`registry`**: Agent storage and basic CRUD operations
//! - **`lifecycle`**: Lifecycle management and monitoring
//! - **`metrics`**: Performance tracking and analytics
//! - **`manager`**: Main coordinator providing unified API
//!
//! ## Key Features
//!
//! - **Concurrent Agent Registry**: Thread-safe agent storage and retrieval
//! - **Performance Monitoring**: Detailed metrics collection per agent
//! - **Resource-Aware Registration**: Capacity checking before agent creation
//! - **Lifecycle Management**: Complete agent creation to removal workflow
//! - **Analytics Integration**: Performance trends and efficiency analysis
//!
//! ## Usage
//!
//! ```rust,no_run
//! use hive::core::hive::agent_management::AgentManager;
//! use std::sync::Arc;
//! use tokio::sync::mpsc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let resource_manager = Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);
//! let (tx, _rx) = mpsc::unbounded_channel();
//!
//! let agent_manager = AgentManager::new(resource_manager, tx).await?;
//!
//! // Create an agent
//! let config = serde_json::json!({"type": "worker", "name": "example_agent"});
//! let agent_id = agent_manager.create_agent(config).await?;
//!
//! // Get agent information
//! if let Some(agent) = agent_manager.get_agent(agent_id).await {
//!     println!("Agent: {}", agent.name);
//! }
//!
//! // Get system status
//! let status = agent_manager.get_status().await;
//! println!("Agent status: {}", status);
//! # Ok(())
//! # }
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Agent Creation**: O(1) with resource checking overhead
//! - **Agent Lookup**: O(1) average case with hash map access
//! - **Status Retrieval**: O(n) where n is active agents
//! - **Memory Usage**: O(n) where n is agents + metrics
//! - **Concurrency**: High concurrency with DashMap and async operations

/// Core data structures and types
pub mod types;

/// Agent storage and basic CRUD operations
pub mod registry;

/// Lifecycle management and monitoring
pub mod lifecycle;

/// Performance tracking and analytics
pub mod metrics;

/// Main coordinator providing unified API
pub mod manager;

// Re-export the main types for convenience
pub use manager::AgentManager;
pub use types::{AgentMetrics, AgentRegistrationResult};
