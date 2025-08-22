#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![allow(clippy::cargo)]
#![allow(missing_docs)]
#![allow(unused)]
#![allow(dead_code)]
#![allow(clippy::clone_on_ref_ptr)]
#![allow(clippy::unwrap_used)]
#![allow(unused_comparisons)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::useless_vec)]
#![allow(unused_imports)]
#![allow(clippy::single_component_path_imports)]
//! # Multiagent Hive System
//!
//! A sophisticated hybrid neural multiagent hive system implementing swarm intelligence
//! with NLP self-learning capabilities. Built with a "CPU-native, GPU-optional" philosophy
//! for maximum intelligence on minimal hardware.
//!
//! ## Architecture Overview
//!
//! The system is organized into several core modules:
//!
//! - [`core`]: Central hive coordination and management
//! - [`agents`]: Individual agent implementations and behaviors
//! - [`tasks`]: Task queue management and distribution
//! - [`neural`]: Neural processing and NLP capabilities
//! - [`communication`]: WebSocket and MCP protocol handling
//! - [`infrastructure`]: Metrics, caching, and resource management
//! - [`utils`]: Configuration, validation, and error handling
//!
//! ## Quick Start
//!
//! ```rust
//! use multiagent_hive::{HiveCoordinator, Agent, AgentType};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize the hive coordinator
//!     let mut hive = HiveCoordinator::new().await?;
//!     
//!     // Create a worker agent
//!     let agent = Agent::new("Worker-1".to_string(), AgentType::Worker);
//!     hive.add_agent(agent).await?;
//!     
//!     // Start the hive
//!     hive.start().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Feature Flags
//!
//! - `basic-nlp` (default): Lightweight NLP processing
//! - `advanced-neural`: Enables ruv-FANN neural networks
//! - `gpu-acceleration`: GPU support for neural processing

//! # Multiagent Hive System
//!
//! A sophisticated hybrid neural multiagent hive system implementing swarm intelligence
//! with NLP self-learning capabilities. CPU-native, GPU-optional - built for the GPU-poor.

/// Agent implementations and behaviors
pub mod agents;
/// Communication protocols and WebSocket handling
pub mod communication;
/// Core hive coordination and management
pub mod core;
/// Infrastructure components (metrics, caching, telemetry)
pub mod infrastructure;
/// Neural processing and NLP capabilities
pub mod neural;
/// Task management and distribution
pub mod tasks;
/// Utility functions and configuration
pub mod utils;

/// Comprehensive unit and integration tests
pub mod tests;

// Re-export core types with specific imports to avoid ambiguity
pub use agents::{Agent, AgentCapability, AgentMemory, AgentState, AgentType};
pub use communication::{mcp, WebSocketMessage};
pub use core::{HiveCoordinator, SwarmMetrics};
pub use infrastructure::{MetricsCollector, ResourceManager};
pub use neural::{HybridNeuralProcessor, NLPProcessor};
pub use tasks::*;
pub use utils::*;

/// Application state shared across the system
#[derive(Clone)]
pub struct AppState {
    /// The main hive coordinator managing all agents and tasks
    pub hive: std::sync::Arc<tokio::sync::RwLock<HiveCoordinator>>,
}
