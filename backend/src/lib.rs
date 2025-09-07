#![warn(missing_docs)]
#![warn(unused)]
#![warn(dead_code)]
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::unwrap_used)]
#![warn(unused_comparisons)]
#![warn(clippy::uninlined_format_args)]
#![warn(clippy::cast_lossless)]
#![warn(clippy::useless_vec)]
#![warn(unused_imports)]
#![warn(clippy::single_component_path_imports)]
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

/// Server setup and routing
pub mod server;

/// System initialization
pub mod init;

/// Comprehensive unit and integration tests
pub mod tests;

// Re-export core types with explicit imports to avoid ambiguous glob re-exports
pub use agents::{
    Agent, AgentBehavior, AgentCapability, AgentMemory, AgentRecoveryManager, AgentState,
    AgentType, Discrepancy, DiscrepancySeverity, OverallTaskStatus, VerificationDetails,
    VerificationLevel, VerificationMethod, VerificationResult, VerificationStatus,
    VerifiedTaskResult,
};
pub use communication::handle_websocket;
pub use communication::mcp::{HiveMCPServer, MCPToolHandler};
pub use core::{HiveCoordinator, SwarmIntelligenceEngine};
pub use infrastructure::{
    CircuitBreaker, IntelligentAlertingSystem, MetricsCollector, PerformanceOptimizer,
    PersistenceManager, ResourceManager, TelemetryCollector,
};
pub use neural::{AdaptiveLearningSystem, HybridNeuralProcessor, NLPProcessor};
pub use tasks::{Task, TaskPriority, TaskQueue, TaskResult, TaskStatus, WorkStealingQueue};
pub use utils::{rate_limiter::RateLimiter, SecurityAuditor};
pub use utils::{HiveConfig, HiveError, HiveResult, InputValidator};

// Server functions
pub use crate::init::initialize_system;
pub use crate::server::{create_router, start_background_tasks};

/// Application state shared across the system
///
/// This struct contains the comprehensive state that needs to be shared across
/// different parts of the application, particularly the HTTP handlers
/// and background tasks.
#[derive(Clone)]
pub struct AppState {
    /// The main hive coordinator managing all agents and tasks
    pub hive: std::sync::Arc<tokio::sync::RwLock<HiveCoordinator>>,
    /// System configuration
    pub config: std::sync::Arc<HiveConfig>,
    /// Enhanced metrics collection system with alerting and trend analysis
    pub metrics: std::sync::Arc<MetricsCollector>,
    /// Advanced metrics collector with predictive analytics
    pub advanced_metrics: std::sync::Arc<MetricsCollector>,
    /// Intelligent alerting system with adaptive thresholds
    pub intelligent_alerting: std::sync::Arc<IntelligentAlertingSystem>,
    /// Circuit breaker for resilience
    pub circuit_breaker: std::sync::Arc<CircuitBreaker>,
    /// Agent recovery manager for error handling
    pub recovery_manager: std::sync::Arc<AgentRecoveryManager>,
    /// Swarm intelligence engine for formation optimization
    pub swarm_intelligence: std::sync::Arc<tokio::sync::RwLock<SwarmIntelligenceEngine>>,
    /// Persistence manager for state recovery and checkpointing
    pub persistence_manager: std::sync::Arc<PersistenceManager>,
    /// Adaptive learning system for continuous improvement
    pub adaptive_learning: std::sync::Arc<tokio::sync::RwLock<AdaptiveLearningSystem>>,
    /// Rate limiter for API protection
    pub rate_limiter: std::sync::Arc<RateLimiter>,
    /// Performance optimizer for system optimization
    pub performance_optimizer: std::sync::Arc<PerformanceOptimizer>,
    /// Security auditor for security logging
    pub security_auditor: std::sync::Arc<SecurityAuditor>,
}
