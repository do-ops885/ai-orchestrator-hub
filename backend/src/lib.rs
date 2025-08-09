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

/// Core hive coordination and management
pub mod core;
/// Agent implementations and behaviors
pub mod agents;
/// Task management and distribution
pub mod tasks;
/// Neural processing and NLP capabilities
pub mod neural;
/// Communication protocols and WebSocket handling
pub mod communication;
/// Infrastructure components (metrics, caching, telemetry)
pub mod infrastructure;
/// Utility functions and configuration
pub mod utils;

pub use core::*;
pub use agents::*;
pub use tasks::*;
pub use neural::*;
pub use communication::*;
pub use infrastructure::*;
pub use utils::*;

/// Application state shared across the system
#[derive(Clone)]
pub struct AppState {
    /// The main hive coordinator managing all agents and tasks
    pub hive: std::sync::Arc<tokio::sync::RwLock<HiveCoordinator>>,
}