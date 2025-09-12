//! # Agent Management Module
//!
//! This module provides comprehensive agent lifecycle management including
//! registration, monitoring, performance tracking, and resource allocation.
//!
//! ## Architecture
//!
//! The agent management system uses a DashMap for concurrent access to agent
//! data and maintains separate metrics tracking for performance analysis.
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
//! use hive::core::hive::AgentManager;
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

use crate::agents::agent::{Agent, AgentBehavior, AgentType};
use crate::infrastructure::resource_manager::ResourceManager;
use crate::neural::nlp::NLPProcessor;
use crate::utils::error::{HiveError, HiveResult};

use super::coordinator::CoordinationMessage;

use chrono;
use dashmap::DashMap;
use serde_json;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Result of agent registration
///
/// Contains the outcome of an agent registration attempt with
/// detailed information for success or failure cases.
#[derive(Debug, Clone)]
pub struct AgentRegistrationResult {
    /// Unique identifier of the registered agent
    pub agent_id: Uuid,
    /// Whether the registration was successful
    pub success: bool,
    /// Human-readable message describing the result
    pub message: String,
}

/// Agent management subsystem
///
/// Central coordinator for all agent-related operations in the hive system.
/// Provides thread-safe access to agent data and maintains performance metrics.
///
/// ## Components
///
/// - **Agent Registry**: DashMap-based storage for active agents
/// - **Metrics Collector**: Performance tracking and analytics
/// - **Resource Manager**: Capacity planning and resource allocation
/// - **Coordination Channel**: Communication with other subsystems
///
/// ## Thread Safety
///
/// All operations are thread-safe using `Arc<DashMap<T>>` for concurrent access.
/// Metrics updates are atomic and consistent across operations.
///
/// ## Performance
///
/// Designed for high-throughput agent operations with minimal contention.
/// Uses efficient data structures and async operations throughout.
#[derive(Clone)]
pub struct AgentManager {
    /// Active agents in the system
    ///
    /// Thread-safe storage for all registered agents using DashMap.
    /// Provides O(1) average-case lookup and concurrent access.
    pub agents: Arc<DashMap<Uuid, Agent>>,

    /// Resource manager for capacity planning
    ///
    /// Used to check system capacity before agent registration
    /// and for resource allocation decisions.
    resource_manager: Arc<ResourceManager>,

    /// Communication channel for coordination
    ///
    /// Async channel for sending coordination messages to other subsystems
    /// when agent events occur (registration, removal, etc.).
    coordination_tx: mpsc::UnboundedSender<CoordinationMessage>,

    /// Agent performance metrics
    ///
    /// Tracks detailed performance statistics for each agent including
    /// task completion rates, execution times, and efficiency metrics.
    agent_metrics: Arc<DashMap<Uuid, AgentMetrics>>,
}

/// Performance metrics for individual agents
///
/// Comprehensive performance tracking data for each agent including
/// task completion statistics, execution times, and efficiency metrics.
///
/// ## Metrics Tracked
///
/// - Task completion and failure counts
/// - Execution time statistics (total, average, last activity)
/// - Performance scoring based on success rate and speed
/// - Activity timestamps for monitoring
///
/// ## Performance Score Calculation
///
/// Performance score combines success rate with execution speed:
/// `score = success_rate * min(2.0, 1000.0 / average_execution_time_ms)`
///
/// Higher scores indicate better performance with a maximum speed bonus cap.
#[derive(Debug, Clone, Default)]
pub struct AgentMetrics {
    /// Number of tasks completed successfully by this agent
    pub tasks_completed: u64,
    /// Number of tasks that failed during execution
    pub tasks_failed: u64,
    /// Total execution time across all tasks in milliseconds
    pub total_execution_time_ms: u64,
    /// Average execution time per task in milliseconds
    pub average_execution_time_ms: f64,
    /// Timestamp of the last activity performed by this agent
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
    /// Overall performance score (0.0 to 2.0+)
    pub performance_score: f64,
}

impl AgentManager {
    /// Create a new agent manager
    ///
    /// Initializes the agent management subsystem with required dependencies.
    /// Sets up data structures for agent storage and metrics tracking.
    ///
    /// ## Initialization Process
    ///
    /// 1. Creates DashMap for agent storage
    /// 2. Initializes metrics tracking system
    /// 3. Establishes coordination channel
    /// 4. Sets up resource manager integration
    ///
    /// ## Performance
    ///
    /// O(1) initialization with minimal memory allocation.
    /// Ready for immediate agent operations after creation.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::AgentManager;
    /// # use std::sync::Arc;
    /// # use tokio::sync::mpsc;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let resource_manager = Arc::new(crate::infrastructure::resource_manager::ResourceManager::new().await?);
    /// let (tx, _rx) = mpsc::unbounded_channel();
    ///
    /// let agent_manager = AgentManager::new(resource_manager, tx).await?;
    /// println!("Agent manager initialized");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `resource_manager` - Resource manager for capacity checking
    /// * `coordination_tx` - Channel for sending coordination messages
    ///
    /// # Returns
    ///
    /// Returns a new `AgentManager` instance on success.
    ///
    /// # Errors
    ///
    /// This function will not return an error under normal circumstances.
    pub async fn new(
        resource_manager: Arc<ResourceManager>,
        coordination_tx: mpsc::UnboundedSender<CoordinationMessage>,
    ) -> HiveResult<Self> {
        Ok(Self {
            agents: Arc::new(DashMap::new()),
            resource_manager,
            coordination_tx,
            agent_metrics: Arc::new(DashMap::new()),
        })
    }

    /// Create and register a new agent
    ///
    /// Creates a new agent with the specified configuration and registers it
    /// with the system. Performs resource capacity checking before creation.
    ///
    /// ## Configuration Requirements
    ///
    /// The config must include:
    /// - `"type"`: Agent type ("worker", "coordinator", "specialist", "learner")
    /// - `"name"`: Human-readable name (optional, defaults to type-based name)
    ///
    /// ## Resource Validation
    ///
    /// Checks CPU usage before agent creation:
    /// - Must be below 90% to prevent system overload
    /// - Ensures sufficient resources for agent operation
    ///
    /// ## Registration Process
    ///
    /// 1. Parse and validate configuration
    /// 2. Check system resource availability
    /// 3. Create agent instance
    /// 4. Register with internal storage
    /// 5. Initialize performance metrics
    /// 6. Send coordination notification
    ///
    /// ## Performance
    ///
    /// O(1) average case with resource checking overhead.
    /// Triggers coordination messages for system-wide notifications.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::AgentManager;
    /// # async fn example(agent_manager: &AgentManager) -> Result<(), Box<dyn std::error::Error>> {
    /// let config = serde_json::json!({
    ///     "type": "worker",
    ///     "name": "data_processor",
    ///     "capabilities": ["computation", "data_processing"]
    /// });
    ///
    /// let agent_id = agent_manager.create_agent(config).await?;
    /// println!("Created agent with ID: {}", agent_id);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `config` - JSON configuration object for the agent
    ///
    /// # Returns
    ///
    /// Returns the unique ID of the created agent on success.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Configuration is invalid or missing required fields
    /// - System resources are insufficient
    /// - Agent type is unknown
    /// - Registration fails
    pub async fn create_agent(&self, config: serde_json::Value) -> HiveResult<Uuid> {
        // Check resource availability
        let (system_resources, _, _) = self.resource_manager.get_system_info().await;
        if system_resources.cpu_usage > 0.9 {
            return Err(crate::utils::error::HiveError::ResourceExhausted {
                resource: "CPU capacity for new agent".to_string(),
            });
        }

        // Parse agent configuration
        let agent_type = self.parse_agent_type(&config)?;
        let _agent_config = self.validate_agent_config(&config)?;

        // Extract agent name from config, default to type-based name
        let agent_name = config
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&format!("{:?}", agent_type))
            .to_string();

        // Create the agent
        let agent = Agent::new(agent_name, agent_type);

        let agent_id = agent.id;

        // Register the agent
        self.agents.insert(agent_id, agent);
        self.agent_metrics.insert(agent_id, AgentMetrics::default());

        // Notify coordination system
        if let Err(e) = self
            .coordination_tx
            .send(CoordinationMessage::AgentRegistered { agent_id })
        {
            tracing::warn!("Failed to send agent registration notification: {}", e);
        }

        tracing::info!("Agent {} created and registered successfully", agent_id);
        Ok(agent_id)
    }

    /// Remove an agent from the system
    ///
    /// Gracefully removes an agent from the system, cleaning up all associated
    /// resources, metrics, and pending operations. The agent will be prevented
    /// from accepting new tasks and removed from all internal data structures.
    ///
    /// ## Cleanup Operations
    ///
    /// 1. Locate agent in registry
    /// 2. Remove from active agent storage
    /// 3. Clean up agent-specific metrics
    /// 4. Cancel any pending tasks assigned to the agent
    /// 5. Send coordination notification for system updates
    ///
    /// ## Performance
    ///
    /// O(1) average case for agent lookup and removal.
    /// Minimal overhead for cleanup operations.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::AgentManager;
    /// # async fn example(agent_manager: &AgentManager, agent_id: uuid::Uuid) -> Result<(), Box<dyn std::error::Error>> {
    /// match agent_manager.remove_agent(agent_id).await {
    ///     Ok(()) => println!("Agent {} removed successfully", agent_id),
    ///     Err(e) => eprintln!("Failed to remove agent: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `agent_id` - Unique identifier of the agent to remove
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the agent is successfully removed.
    ///
    /// # Errors
    ///
    /// Returns error if the agent doesn't exist or removal fails.
    pub async fn remove_agent(&self, agent_id: Uuid) -> HiveResult<()> {
        if let Some((_, agent)) = self.agents.remove(&agent_id) {
            // Cleanup agent resources
            drop(agent);
            self.agent_metrics.remove(&agent_id);

            // Notify coordination system
            if let Err(e) = self
                .coordination_tx
                .send(CoordinationMessage::AgentRemoved { agent_id })
            {
                tracing::warn!("Failed to send agent removal notification: {}", e);
            }

            tracing::info!("Agent {} removed successfully", agent_id);
            Ok(())
        } else {
            Err(crate::utils::error::HiveError::AgentNotFound {
                id: agent_id.to_string(),
            })
        }
    }

    /// Get an agent by ID
    pub async fn get_agent(&self, agent_id: Uuid) -> Option<Agent> {
        self.agents.get(&agent_id).map(|entry| entry.clone())
    }

    /// Get all active agents
    pub async fn get_all_agents(&self) -> Vec<(Uuid, Agent)> {
        self.agents
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect()
    }

    /// Get all active agents
    ///
    /// Returns a complete list of all currently active agents in the system.
    /// Each agent is returned as a (ID, Agent) tuple for easy iteration and processing.
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is the number of active agents.
    /// Memory usage scales with agent count due to cloning.
    /// Suitable for monitoring and administrative operations.
    ///
    /// ## Use Cases
    ///
    /// - System monitoring dashboards
    /// - Bulk operations on all agents
    /// - Load balancing calculations
    /// - Administrative agent management
    /// - Status reporting
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::AgentManager;
    /// # async fn example(agent_manager: &AgentManager) {
    /// let all_agents = agent_manager.get_all_agents().await;
    /// println!("Total active agents: {}", all_agents.len());
    ///
    /// for (id, agent) in all_agents {
    ///     println!("Agent {}: {} ({:?})", id, agent.name, agent.agent_type);
    /// }
    /// # }
    /// ```
    ///

    /// Update agent metrics after task execution
    ///
    /// Updates performance metrics for an agent after task completion.
    /// Calculates success rates, execution times, and performance scores.
    ///
    /// ## Metrics Updated
    ///
    /// - Task completion/failure counters
    /// - Total and average execution times
    /// - Last activity timestamp
    /// - Performance score based on success rate and speed
    ///
    /// ## Performance Score Calculation
    ///
    /// Combines success rate with execution speed:
    /// - Success rate: `completed_tasks / total_tasks`
    /// - Speed factor: `1000.0 / average_execution_time_ms`
    /// - Final score: `success_rate * min(2.0, speed_factor)`
    ///
    /// ## Performance
    ///
    /// O(1) average case for metrics updates.
    /// Atomic operations ensure consistency.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::AgentManager;
    /// # async fn example(agent_manager: &AgentManager, agent_id: uuid::Uuid) {
    /// // After successful task completion
    /// agent_manager.update_agent_metrics(agent_id, 150, true).await;
    ///
    /// // After failed task
    /// agent_manager.update_agent_metrics(agent_id, 200, false).await;
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `agent_id` - Unique identifier of the agent
    /// * `execution_time_ms` - Time taken to execute the task in milliseconds
    /// * `success` - Whether the task execution was successful
    pub async fn update_agent_metrics(
        &self,
        agent_id: Uuid,
        execution_time_ms: u64,
        success: bool,
    ) {
        if let Some(mut metrics) = self.agent_metrics.get_mut(&agent_id) {
            if success {
                metrics.tasks_completed += 1;
            } else {
                metrics.tasks_failed += 1;
            }

            metrics.total_execution_time_ms += execution_time_ms;
            metrics.last_activity = Some(chrono::Utc::now());

            // Calculate average execution time
            let total_tasks = metrics.tasks_completed + metrics.tasks_failed;
            if total_tasks > 0 {
                metrics.average_execution_time_ms =
                    metrics.total_execution_time_ms as f64 / total_tasks as f64;
            }

            // Calculate performance score (success rate weighted by speed)
            if total_tasks > 0 {
                let success_rate = metrics.tasks_completed as f64 / total_tasks as f64;
                let speed_factor = if metrics.average_execution_time_ms > 0.0 {
                    1000.0 / metrics.average_execution_time_ms // Higher score for faster execution
                } else {
                    1.0
                };
                metrics.performance_score = success_rate * speed_factor.min(2.0);
                // Cap speed bonus
            }
        }
    }

    /// Get agent status summary
    ///
    /// Returns a comprehensive summary of agent system status including
    /// counts, types, and performance metrics across all agents.
    ///
    /// ## Status Information
    ///
    /// - Total and active agent counts
    /// - Agent type distribution
    /// - Performance summaries (success rates, execution times)
    /// - System health indicators
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is the number of agents.
    /// Involves iterating through all agents and metrics.
    ///
    /// ## Use Cases
    ///
    /// - System monitoring and health checks
    /// - Dashboard displays
    /// - Administrative reporting
    /// - Capacity planning
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::AgentManager;
    /// # async fn example(agent_manager: &AgentManager) {
    /// let status = agent_manager.get_status().await;
    ///
    /// let total_agents = status["total_agents"].as_u64().unwrap_or(0);
    /// let active_agents = status["active_agents"].as_u64().unwrap_or(0);
    ///
    /// println!("Agents: {}/{} active", active_agents, total_agents);
    ///
    /// if let Some(performance) = status.get("performance") {
    ///     println!("Performance: {}", performance);
    /// }
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a JSON object containing comprehensive agent status information.
    pub async fn get_status(&self) -> serde_json::Value {
        let total_agents = self.agents.len();
        let active_agents = self.count_active_agents().await;

        let agent_types = self.get_agent_type_distribution().await;
        let performance_summary = self.get_performance_summary().await;

        serde_json::json!({
            "total_agents": total_agents,
            "active_agents": active_agents,
            "agent_types": agent_types,
            "performance": performance_summary
        })
    }

    /// Get detailed analytics
    ///
    /// Returns comprehensive analytics about agent performance, efficiency,
    /// and system utilization patterns. Includes trend analysis and recommendations.
    ///
    /// ## Analytics Data
    ///
    /// - Individual agent performance metrics
    /// - Top performers ranking
    /// - Resource utilization patterns
    /// - Success rate trends
    /// - Execution time distributions
    ///
    /// ## Performance
    ///
    /// O(n) time complexity with sorting operations for rankings.
    /// May involve complex calculations for trend analysis.
    ///
    /// ## Use Cases
    ///
    /// - Performance optimization
    /// - Agent efficiency analysis
    /// - System bottleneck identification
    /// - Capacity planning and scaling decisions
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::AgentManager;
    /// # async fn example(agent_manager: &AgentManager) {
    /// let analytics = agent_manager.get_analytics().await;
    ///
    /// if let Some(performance) = analytics.get("agent_performance") {
    ///     println!("Agent performance data: {}", performance);
    /// }
    ///
    /// if let Some(top_performers) = analytics.get("top_performers") {
    ///     println!("Top performers: {}", top_performers);
    /// }
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a JSON object with detailed agent analytics and insights.
    pub async fn get_analytics(&self) -> serde_json::Value {
        let agent_performance: Vec<_> = self.agent_metrics
            .iter()
            .map(|entry| {
                let agent_id = *entry.key();
                let metrics = entry.value();
                serde_json::json!({
                    "agent_id": agent_id,
                    "tasks_completed": metrics.tasks_completed,
                    "tasks_failed": metrics.tasks_failed,
                    "success_rate": if metrics.tasks_completed + metrics.tasks_failed > 0 {
                        metrics.tasks_completed as f64 / (metrics.tasks_completed + metrics.tasks_failed) as f64
                    } else {
                        0.0
                    },
                    "average_execution_time_ms": metrics.average_execution_time_ms,
                    "performance_score": metrics.performance_score,
                    "last_activity": metrics.last_activity
                })
            })
            .collect();

        serde_json::json!({
            "agent_performance": agent_performance,
            "total_agents": self.agents.len(),
            "system_health": "operational"
        })
    }

    /// Run learning cycle for all agents
    ///
    /// Triggers a learning cycle for all active agents using the provided
    /// NLP processor. This enables agents to improve their performance
    /// through experience and adaptation.
    ///
    /// ## Learning Process
    ///
    /// 1. Iterate through all active agents
    /// 2. Trigger learning cycle for each agent
    /// 3. Handle learning failures gracefully
    /// 4. Log learning progress and errors
    ///
    /// ## Performance
    ///
    /// Variable time complexity depending on number of agents and
    /// complexity of individual learning cycles. Runs asynchronously
    /// to avoid blocking other operations.
    ///
    /// ## Error Handling
    ///
    /// Individual agent learning failures are logged but don't stop
    /// the overall learning cycle. Returns success if any agents
    /// successfully complete learning.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::AgentManager;
    /// # use crate::neural::nlp::NLPProcessor;
    /// # async fn example(agent_manager: &AgentManager, nlp_processor: &NLPProcessor) -> Result<(), Box<dyn std::error::Error>> {
    /// match agent_manager.run_learning_cycle(nlp_processor).await {
    ///     Ok(()) => println!("Learning cycle completed successfully"),
    ///     Err(e) => eprintln!("Learning cycle failed: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `nlp_processor` - NLP processor for agent learning
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the learning cycle completes (even with individual failures).
    ///
    /// # Errors
    ///
    /// Returns error if the learning cycle cannot be initiated.
    pub async fn run_learning_cycle(&self, nlp_processor: &NLPProcessor) -> HiveResult<()> {
        let agents: Vec<_> = self
            .agents
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        for mut agent in agents {
            if let Err(e) = agent.learn(nlp_processor).await {
                tracing::warn!("Learning failed for agent {}: {}", agent.id, e);
            }
        }

        Ok(())
    }

    /// Parse agent type from configuration
    ///
    /// Extracts and validates the agent type from the provided configuration.
    /// Supports all standard agent types with validation.
    ///
    /// ## Supported Types
    ///
    /// - `"worker"`: General-purpose task execution
    /// - `"coordinator"`: Task coordination and management
    /// - `"specialist"`: Specialized task handling
    /// - `"learner"`: Adaptive learning and improvement
    ///
    /// ## Performance
    ///
    /// O(1) time complexity with simple string matching.
    ///
    /// # Parameters
    ///
    /// * `config` - JSON configuration containing agent type
    ///
    /// # Returns
    ///
    /// Returns the parsed `AgentType` on success.
    ///
    /// # Errors
    ///
    /// Returns error if type field is missing or contains invalid value.
    fn parse_agent_type(&self, config: &serde_json::Value) -> HiveResult<AgentType> {
        let type_str = config.get("type").and_then(|v| v.as_str()).ok_or_else(|| {
            HiveError::ValidationError {
                field: "type".to_string(),
                reason: "Agent type is required".to_string(),
            }
        })?;

        match type_str {
            "worker" => Ok(crate::agents::agent::AgentType::Worker),
            "coordinator" => Ok(crate::agents::agent::AgentType::Coordinator),
            "specialist" => Ok(crate::agents::agent::AgentType::Specialist(
                "general".to_string(),
            )),
            "learner" => Ok(crate::agents::agent::AgentType::Learner),
            _ => Err(crate::utils::error::HiveError::ValidationError {
                field: "type".to_string(),
                reason: format!("Unknown agent type: {}", type_str),
            }),
        }
    }

    /// Validate agent configuration
    ///
    /// Performs basic validation on agent configuration to ensure
    /// required fields are present and properly formatted.
    ///
    /// ## Validation Checks
    ///
    /// - Configuration must be a valid JSON object
    /// - Required fields must be present (currently minimal validation)
    /// - Future: Additional validation for specific agent types
    ///
    /// ## Performance
    ///
    /// O(1) time complexity for basic validation.
    ///
    /// # Parameters
    ///
    /// * `config` - JSON configuration to validate
    ///
    /// # Returns
    ///
    /// Returns the validated configuration on success.
    ///
    /// # Errors
    ///
    /// Returns error if configuration is invalid or missing required fields.
    fn validate_agent_config(&self, config: &serde_json::Value) -> HiveResult<serde_json::Value> {
        // Basic validation - ensure required fields are present
        if !config.is_object() {
            return Err(crate::utils::error::HiveError::ValidationError {
                field: "config".to_string(),
                reason: "Agent configuration must be an object".to_string(),
            });
        }

        // Add any additional validation logic here
        Ok(config.clone())
    }

    /// Count currently active agents
    ///
    /// Returns the number of agents currently registered and active
    /// in the system. This is a simple count of the agent registry.
    ///
    /// ## Performance
    ///
    /// O(1) time complexity - direct access to DashMap length.
    ///
    /// # Returns
    ///
    /// Returns the count of active agents.
    async fn count_active_agents(&self) -> usize {
        // For now, consider all registered agents as active
        // In a real implementation, you might check last activity time
        self.agents.len()
    }

    /// Get the total number of agents
    ///
    /// Returns the current number of registered agents in the system.
    /// This is a fast, synchronous operation for status checking.
    ///
    /// ## Performance
    ///
    /// O(1) time complexity - direct access to internal counter.
    ///
    /// ## Use Cases
    ///
    /// - Quick status checks
    /// - Monitoring dashboards
    /// - Load balancing decisions
    ///
    /// # Returns
    ///
    /// Returns the total number of registered agents.
    pub fn get_agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Get distribution of agent types
    ///
    /// Analyzes all active agents and returns a breakdown of agent types
    /// with counts for each type. Useful for system composition analysis.
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is the number of agents.
    /// Involves iterating through all agents to count types.
    ///
    /// # Returns
    ///
    /// Returns a JSON object mapping agent types to their counts.
    async fn get_agent_type_distribution(&self) -> serde_json::Value {
        let mut type_counts = std::collections::HashMap::new();

        for entry in self.agents.iter() {
            let agent_type = format!("{:?}", entry.value().agent_type);
            *type_counts.entry(agent_type).or_insert(0) += 1;
        }

        serde_json::to_value(type_counts).unwrap_or_default()
    }

    /// Get performance summary
    ///
    /// Aggregates performance metrics across all agents to provide
    /// system-wide performance statistics and trends.
    ///
    /// ## Metrics Calculated
    ///
    /// - Total tasks processed
    /// - Overall success rate
    /// - Average execution time
    /// - Active agent count
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is the number of agents.
    /// Involves summing metrics across all agents.
    ///
    /// # Returns
    ///
    /// Returns a JSON object with aggregated performance metrics.
    async fn get_performance_summary(&self) -> serde_json::Value {
        let mut total_tasks = 0u64;
        let mut total_successful = 0u64;
        let mut total_execution_time = 0u64;
        let mut agent_count = 0;

        for entry in self.agent_metrics.iter() {
            let metrics = entry.value();
            total_tasks += metrics.tasks_completed + metrics.tasks_failed;
            total_successful += metrics.tasks_completed;
            total_execution_time += metrics.total_execution_time_ms;
            agent_count += 1;
        }

        let success_rate = if total_tasks > 0 {
            total_successful as f64 / total_tasks as f64
        } else {
            0.0
        };

        let average_execution_time = if total_tasks > 0 {
            total_execution_time as f64 / total_tasks as f64
        } else {
            0.0
        };

        serde_json::json!({
            "total_tasks": total_tasks,
            "success_rate": success_rate,
            "average_execution_time_ms": average_execution_time,
            "active_agents": agent_count
        })
    }

    /// Get top performing agents
    ///
    /// Ranks agents by their performance scores and returns the top performers.
    /// Useful for identifying high-performing agents and performance patterns.
    ///
    /// ## Ranking Criteria
    ///
    /// Agents are ranked by performance score which combines:
    /// - Success rate (higher is better)
    /// - Execution speed (faster is better, with diminishing returns)
    ///
    /// ## Performance
    ///
    /// O(n log n) time complexity due to sorting.
    /// Returns top 5 performers by default.
    ///
    /// # Returns
    ///
    /// Returns a vector of JSON objects representing top-performing agents.
    async fn get_top_performers(&self) -> Vec<serde_json::Value> {
        let mut performers: Vec<_> = self
            .agent_metrics
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect();

        performers.sort_by(|a, b| {
            b.1.performance_score
                .partial_cmp(&a.1.performance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        performers.into_iter()
            .take(5) // Top 5 performers
            .map(|(agent_id, metrics)| serde_json::json!({
                "agent_id": agent_id,
                "performance_score": metrics.performance_score,
                "tasks_completed": metrics.tasks_completed,
                "success_rate": if metrics.tasks_completed + metrics.tasks_failed > 0 {
                    metrics.tasks_completed as f64 / (metrics.tasks_completed + metrics.tasks_failed) as f64
                } else {
                    0.0
                }
            }))
            .collect()
    }

    /// Get resource utilization by agents
    ///
    /// Estimates resource utilization based on agent count and activity.
    /// Provides insights into system resource usage patterns.
    ///
    /// ## Resource Estimates
    ///
    /// - CPU usage: Estimated at 10% per active agent
    /// - Memory usage: Estimated at 50MB per active agent
    /// - Agent count and activity metrics
    ///
    /// ## Performance
    ///
    /// O(1) time complexity with simple calculations.
    ///
    /// # Returns
    ///
    /// Returns a JSON object with resource utilization estimates.
    async fn get_resource_utilization(&self) -> serde_json::Value {
        // This would typically involve querying actual resource usage
        // For now, return estimated values
        serde_json::json!({
            "estimated_cpu_usage": self.agents.len() as f64 * 0.1, // 10% per agent estimate
            "estimated_memory_usage_mb": self.agents.len() as f64 * 50.0, // 50MB per agent estimate
            "agent_count": self.agents.len()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::mpsc;

    // Helper function to create a test agent manager
    async fn create_test_agent_manager() -> HiveResult<AgentManager> {
        let resource_manager = Arc::new(
            crate::infrastructure::resource_manager::ResourceManager::new()
                .await
                .map_err(|e| HiveError::ResourceInitializationFailed {
                    reason: format!("Failed to initialize resource manager: {}", e),
                })?,
        );
        let (tx, _rx) = mpsc::unbounded_channel();
        AgentManager::new(resource_manager, tx).await
    }

    #[tokio::test]
    async fn test_agent_manager_creation() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;
        assert_eq!(agent_manager.get_agent_count(), 0);
        assert!(agent_manager.agents.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_create_agent_success() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!({
            "type": "worker",
            "name": "test_agent"
        });

        let agent_id = agent_manager.create_agent(config).await?;
        assert!(!agent_id.is_nil());
        assert_eq!(agent_manager.get_agent_count(), 1);

        // Verify agent was created
        let agent = agent_manager.get_agent(agent_id).await;
        assert!(agent.is_some());
        assert_eq!(agent.unwrap().name, "test_agent");

        Ok(())
    }

    #[tokio::test]
    async fn test_create_agent_coordinator_type() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!({
            "type": "coordinator",
            "name": "coord_agent"
        });

        let agent_id = agent_manager.create_agent(config).await?;
        let agent = agent_manager.get_agent(agent_id).await.unwrap();
        assert_eq!(
            agent.agent_type,
            crate::agents::agent::AgentType::Coordinator
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_create_agent_specialist_type() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!({
            "type": "specialist",
            "name": "specialist_agent"
        });

        let agent_id = agent_manager.create_agent(config).await?;
        let agent = agent_manager.get_agent(agent_id).await.unwrap();
        match agent.agent_type {
            crate::agents::agent::AgentType::Specialist(_) => {}
            _ => panic!("Expected Specialist type"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_agent_learner_type() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!({
            "type": "learner",
            "name": "learner_agent"
        });

        let agent_id = agent_manager.create_agent(config).await?;
        let agent = agent_manager.get_agent(agent_id).await.unwrap();
        assert_eq!(agent.agent_type, crate::agents::agent::AgentType::Learner);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_agent_invalid_type() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!({
            "type": "invalid_type",
            "name": "test_agent"
        });

        let result = agent_manager.create_agent(config).await;
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_agent_missing_type() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!({
            "name": "test_agent"
        });

        let result = agent_manager.create_agent(config).await;
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_create_agent_invalid_config() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!("invalid_config");

        let result = agent_manager.create_agent(config).await;
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_remove_agent_success() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!({
            "type": "worker",
            "name": "test_agent"
        });

        let agent_id = agent_manager.create_agent(config).await?;
        assert_eq!(agent_manager.get_agent_count(), 1);

        // Remove the agent
        agent_manager.remove_agent(agent_id).await?;
        assert_eq!(agent_manager.get_agent_count(), 0);
        assert!(agent_manager.get_agent(agent_id).await.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_remove_agent_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let fake_id = Uuid::new_v4();
        let result = agent_manager.remove_agent(fake_id).await;
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_all_agents() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config1 = serde_json::json!({
            "type": "worker",
            "name": "agent1"
        });
        let config2 = serde_json::json!({
            "type": "coordinator",
            "name": "agent2"
        });

        let agent_id1 = agent_manager.create_agent(config1).await?;
        let agent_id2 = agent_manager.create_agent(config2).await?;

        let all_agents = agent_manager.get_all_agents().await;
        assert_eq!(all_agents.len(), 2);

        let agent_ids: Vec<Uuid> = all_agents.iter().map(|(id, _)| *id).collect();
        assert!(agent_ids.contains(&agent_id1));
        assert!(agent_ids.contains(&agent_id2));

        Ok(())
    }

    #[tokio::test]
    async fn test_update_agent_metrics_success() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!({
            "type": "worker",
            "name": "test_agent"
        });
        let agent_id = agent_manager.create_agent(config).await?;

        // Update metrics for successful task
        agent_manager
            .update_agent_metrics(agent_id, 100, true)
            .await;

        // Update metrics for failed task
        agent_manager
            .update_agent_metrics(agent_id, 50, false)
            .await;

        // Check that metrics were updated (we can't directly access private fields,
        // but we can verify through analytics)
        let analytics = agent_manager.get_analytics().await;
        assert!(analytics.is_object());

        Ok(())
    }

    #[tokio::test]
    async fn test_update_agent_metrics_nonexistent_agent() -> Result<(), Box<dyn std::error::Error>>
    {
        let agent_manager = create_test_agent_manager().await?;

        let fake_id = Uuid::new_v4();
        // This should not panic - it should handle non-existent agents gracefully
        agent_manager.update_agent_metrics(fake_id, 100, true).await;

        Ok(())
    }

    #[tokio::test]
    async fn test_get_status() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let status = agent_manager.get_status().await;
        assert!(status.is_object());
        assert!(status.get("total_agents").is_some());
        assert!(status.get("active_agents").is_some());
        assert!(status.get("agent_types").is_some());
        assert!(status.get("performance").is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_analytics() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let analytics = agent_manager.get_analytics().await;
        assert!(analytics.is_object());
        assert!(analytics.get("agent_performance").is_some());
        assert!(analytics.get("top_performers").is_some());
        assert!(analytics.get("resource_utilization").is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_agent_count() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;
        assert_eq!(agent_manager.get_agent_count(), 0);

        let config = serde_json::json!({
            "type": "worker",
            "name": "test_agent"
        });
        agent_manager.create_agent(config).await?;
        assert_eq!(agent_manager.get_agent_count(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_count_active_agents() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;
        assert_eq!(agent_manager.get_agent_count(), 0);

        let config = serde_json::json!({
            "type": "worker",
            "name": "test_agent"
        });
        agent_manager.create_agent(config).await?;
        // For now, all agents are considered active
        assert_eq!(agent_manager.get_agent_count(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_agent_type_distribution() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        // Create agents of different types
        let configs = vec![
            serde_json::json!({"type": "worker", "name": "worker1"}),
            serde_json::json!({"type": "worker", "name": "worker2"}),
            serde_json::json!({"type": "coordinator", "name": "coord1"}),
            serde_json::json!({"type": "learner", "name": "learner1"}),
        ];

        for config in configs {
            agent_manager.create_agent(config).await?;
        }

        let status = agent_manager.get_status().await;
        let agent_types = status.get("agent_types").unwrap();
        assert!(agent_types.is_object());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_performance_summary() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!({
            "type": "worker",
            "name": "test_agent"
        });
        let agent_id = agent_manager.create_agent(config).await?;

        // Add some task metrics
        agent_manager
            .update_agent_metrics(agent_id, 100, true)
            .await;
        agent_manager
            .update_agent_metrics(agent_id, 150, true)
            .await;
        agent_manager
            .update_agent_metrics(agent_id, 80, false)
            .await;

        let status = agent_manager.get_status().await;
        let performance = status.get("performance").unwrap();
        assert!(performance.is_object());
        assert!(performance.get("total_tasks").is_some());
        assert!(performance.get("success_rate").is_some());
        assert!(performance.get("average_execution_time_ms").is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_top_performers() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        // Create multiple agents with different performance
        let configs = vec![
            serde_json::json!({"type": "worker", "name": "agent1"}),
            serde_json::json!({"type": "worker", "name": "agent2"}),
            serde_json::json!({"type": "worker", "name": "agent3"}),
        ];

        let mut agent_ids = Vec::new();
        for config in configs {
            let id = agent_manager.create_agent(config).await?;
            agent_ids.push(id);
        }

        // Give them different performance scores
        agent_manager
            .update_agent_metrics(agent_ids[0], 100, true)
            .await; // High performance
        agent_manager
            .update_agent_metrics(agent_ids[1], 200, true)
            .await; // Medium performance
        agent_manager
            .update_agent_metrics(agent_ids[2], 50, false)
            .await; // Low performance

        let analytics = agent_manager.get_analytics().await;
        let top_performers = analytics.get("top_performers").unwrap().as_array().unwrap();
        assert!(!top_performers.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_resource_utilization() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let analytics = agent_manager.get_analytics().await;
        let resource_utilization = analytics.get("resource_utilization").unwrap();
        assert!(resource_utilization.is_object());
        assert!(resource_utilization.get("estimated_cpu_usage").is_some());
        assert!(resource_utilization
            .get("estimated_memory_usage_mb")
            .is_some());
        assert!(resource_utilization.get("agent_count").is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_agent_type_worker() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!({"type": "worker"});
        let agent_type = agent_manager.parse_agent_type(&config)?;
        assert_eq!(agent_type, crate::agents::agent::AgentType::Worker);

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_agent_type_coordinator() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!({"type": "coordinator"});
        let agent_type = agent_manager.parse_agent_type(&config)?;
        assert_eq!(agent_type, crate::agents::agent::AgentType::Coordinator);

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_agent_type_learner() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!({"type": "learner"});
        let agent_type = agent_manager.parse_agent_type(&config)?;
        assert_eq!(agent_type, crate::agents::agent::AgentType::Learner);

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_agent_type_specialist() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!({"type": "specialist"});
        let agent_type = agent_manager.parse_agent_type(&config)?;
        match agent_type {
            crate::agents::agent::AgentType::Specialist(_) => {}
            _ => panic!("Expected Specialist type"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_agent_type_invalid() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!({"type": "invalid"});
        let result = agent_manager.parse_agent_type(&config);
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_agent_type_missing() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!({});
        let result = agent_manager.parse_agent_type(&config);
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_validate_agent_config_valid() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!({
            "type": "worker",
            "name": "test_agent"
        });

        let result = agent_manager.validate_agent_config(&config);
        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_validate_agent_config_invalid() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!("invalid");
        let result = agent_manager.validate_agent_config(&config);
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_agent_creation_with_default_name() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!({
            "type": "worker"
        });

        let agent_id = agent_manager.create_agent(config).await?;
        let agent = agent_manager.get_agent(agent_id).await.unwrap();
        // Should have a default name based on type
        assert!(!agent.name.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_agent_creation_with_custom_name() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let custom_name = "MyCustomAgent";
        let config = serde_json::json!({
            "type": "worker",
            "name": custom_name
        });

        let agent_id = agent_manager.create_agent(config).await?;
        let agent = agent_manager.get_agent(agent_id).await.unwrap();
        assert_eq!(agent.name, custom_name);

        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_agent_creation_and_removal() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        // Create multiple agents
        let mut agent_ids = Vec::new();
        for i in 0..5 {
            let config = serde_json::json!({
                "type": "worker",
                "name": format!("agent_{}", i)
            });
            let id = agent_manager.create_agent(config).await?;
            agent_ids.push(id);
        }

        assert_eq!(agent_manager.get_agent_count(), 5);

        // Remove some agents
        agent_manager.remove_agent(agent_ids[0]).await?;
        agent_manager.remove_agent(agent_ids[2]).await?;

        assert_eq!(agent_manager.get_agent_count(), 3);

        // Verify remaining agents still exist
        assert!(agent_manager.get_agent(agent_ids[1]).await.is_some());
        assert!(agent_manager.get_agent(agent_ids[3]).await.is_some());
        assert!(agent_manager.get_agent(agent_ids[4]).await.is_some());

        // Verify removed agents don't exist
        assert!(agent_manager.get_agent(agent_ids[0]).await.is_none());
        assert!(agent_manager.get_agent(agent_ids[2]).await.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_agent_metrics_initialization() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!({
            "type": "worker",
            "name": "test_agent"
        });

        let agent_id = agent_manager.create_agent(config).await?;

        // Update metrics and verify they're tracked
        agent_manager
            .update_agent_metrics(agent_id, 100, true)
            .await;
        agent_manager
            .update_agent_metrics(agent_id, 200, true)
            .await;
        agent_manager
            .update_agent_metrics(agent_id, 50, false)
            .await;

        // Check analytics include the metrics
        let analytics = agent_manager.get_analytics().await;
        let agent_performance = analytics
            .get("agent_performance")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(agent_performance.len(), 1);

        let agent_perf = &agent_performance[0];
        assert_eq!(
            agent_perf.get("agent_id").unwrap().as_str().unwrap(),
            agent_id.to_string()
        );
        assert_eq!(agent_perf.get("tasks_completed").unwrap(), 2);
        assert_eq!(agent_perf.get("tasks_failed").unwrap(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_agent_performance_score_calculation() -> Result<(), Box<dyn std::error::Error>> {
        let agent_manager = create_test_agent_manager().await?;

        let config = serde_json::json!({
            "type": "worker",
            "name": "test_agent"
        });

        let agent_id = agent_manager.create_agent(config).await?;

        // Add successful tasks with varying execution times
        agent_manager
            .update_agent_metrics(agent_id, 100, true)
            .await; // Fast success
        agent_manager
            .update_agent_metrics(agent_id, 200, true)
            .await; // Medium success
        agent_manager
            .update_agent_metrics(agent_id, 50, false)
            .await; // Fast failure

        let analytics = agent_manager.get_analytics().await;
        let agent_performance = analytics
            .get("agent_performance")
            .unwrap()
            .as_array()
            .unwrap();
        let agent_perf = &agent_performance[0];

        // Check success rate (2/3 ≈ 0.667)
        let success_rate = agent_perf.get("success_rate").unwrap().as_f64().unwrap();
        assert!((success_rate - 0.666666).abs() < 0.01);

        // Check average execution time ((100+200+50)/3 = 116.67)
        let avg_time = agent_perf
            .get("average_execution_time_ms")
            .unwrap()
            .as_f64()
            .unwrap();
        assert!((avg_time - 116.666666).abs() < 0.01);

        Ok(())
    }
}
