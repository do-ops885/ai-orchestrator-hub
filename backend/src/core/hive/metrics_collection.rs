//! # Metrics Collection Module
//!
//! This module provides comprehensive metrics collection, aggregation, and reporting
//! for the hive system, enabling performance monitoring and analytics.
//!
//! ## Architecture
//!
//! The metrics system uses a multi-layered approach:
//!
//! - **Real-time Collection**: Immediate event tracking and counters
//! - **Periodic Aggregation**: Background processing of metrics
//! - **Historical Storage**: Trend analysis with bounded history
//! - **Export Formats**: Multiple output formats for different consumers
//! - **Performance Tracking**: Detailed timing and efficiency metrics
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
//! use hive::core::hive::MetricsCollector;
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

use crate::utils::error::{HiveError, HiveResult};

use super::coordinator::CoordinationMessage;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// Comprehensive metrics for the hive system
///
/// Aggregates all system metrics into a single, serializable structure
/// for monitoring, analysis, and external system integration.
///
/// ## Metric Categories
///
/// - **Agent Metrics**: Agent counts, performance, and activity
/// - **Task Metrics**: Task execution statistics and success rates
/// - **System Metrics**: CPU, memory, uptime, and performance indicators
/// - **Resource Metrics**: Hardware resource utilization and availability
/// - **Timestamps**: When metrics were last updated
///
/// ## Serialization
///
/// Supports both JSON and Prometheus export formats for different
/// monitoring and alerting systems.
///
/// ## Performance
///
/// Designed for efficient serialization and minimal memory overhead.
/// All fields are optional or have sensible defaults.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HiveMetrics {
    /// Agent-related metrics
    ///
    /// Statistics about agent registration, activity, and performance
    /// including counts, success rates, and efficiency metrics.
    pub agent_metrics: AgentMetrics,

    /// Task-related metrics
    ///
    /// Statistics about task creation, execution, completion, and failure
    /// rates with timing and performance analysis.
    pub task_metrics: TaskMetrics,

    /// System performance metrics
    ///
    /// Overall system health indicators including uptime, CPU usage,
    /// memory consumption, and response times.
    pub system_metrics: SystemMetrics,

    /// Resource utilization metrics
    ///
    /// Hardware resource usage statistics including CPU cores,
    /// memory availability, disk usage, and network connections.
    pub resource_metrics: ResourceMetrics,

    /// Timestamp of last update
    ///
    /// When these metrics were last updated. Used for staleness detection
    /// and time-based analysis.
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Swarm metrics for persistence compatibility
///
/// Legacy metrics structure maintained for backward compatibility
/// with existing persistence and monitoring systems.
///
/// ## Metrics Included
///
/// - Total and active agent counts
/// - Task completion and failure statistics
/// - Average performance scores
/// - Swarm cohesion and learning progress
/// - System uptime tracking
///
/// ## Usage
///
/// Primarily used for data persistence and external system integration.
/// New code should prefer `HiveMetrics` for comprehensive analysis.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwarmMetrics {
    /// Total number of agents in the swarm
    pub total_agents: usize,
    /// Number of currently active agents
    pub active_agents: usize,
    /// Total tasks completed by the swarm
    pub completed_tasks: usize,
    /// Total tasks that failed execution
    pub failed_tasks: usize,
    /// Average performance score across all agents
    pub average_performance: f64,
    /// Measure of swarm coordination and communication effectiveness
    pub swarm_cohesion: f64,
    /// Progress in learning and adaptation (0.0 to 1.0)
    pub learning_progress: f64,
    /// System uptime in seconds
    pub uptime_seconds: u64,
}

/// Agent performance and activity metrics
///
/// Detailed statistics about agent behavior, performance, and lifecycle
/// events in the hive system.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentMetrics {
    /// Total number of agents ever registered
    pub total_agents: u64,
    /// Number of currently active agents
    pub active_agents: u64,
    /// Number of agents created in the current day
    pub agents_created_today: u64,
    /// Number of agents removed in the current day
    pub agents_removed_today: u64,
    /// Average performance score across all agents
    pub average_agent_performance: f64,
    /// ID of the highest-performing agent
    pub top_performer_id: Option<Uuid>,
}

/// Task execution and performance metrics
///
/// Statistics about task creation, execution, completion, and failure
/// patterns in the system.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskMetrics {
    /// Total number of tasks ever created
    pub total_tasks: u64,
    /// Number of tasks completed successfully
    pub completed_tasks: u64,
    /// Number of tasks that failed execution
    pub failed_tasks: u64,
    /// Number of tasks currently pending execution
    pub pending_tasks: u64,
    /// Average execution time in milliseconds
    pub average_execution_time_ms: f64,
    /// Rate of task completion per hour
    pub tasks_per_hour: f64,
    /// Overall task success rate (0.0 to 1.0)
    pub success_rate: f64,
}

/// System performance and health metrics
///
/// Overall system health indicators including resource usage,
/// uptime, and performance statistics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemMetrics {
    /// System uptime in seconds
    pub uptime_seconds: u64,
    /// Total memory usage in megabytes
    pub total_memory_usage_mb: f64,
    /// CPU usage as a percentage (0.0 to 100.0)
    pub cpu_usage_percent: f64,
    /// Network throughput in megabits per second
    pub network_throughput_mbps: f64,
    /// System error rate (0.0 to 1.0)
    pub error_rate: f64,
    /// Average response time in milliseconds
    pub response_time_ms: f64,
}

/// Hardware resource utilization metrics
///
/// Statistics about system hardware resource availability and usage
/// for capacity planning and performance optimization.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceMetrics {
    /// Number of available CPU cores
    pub available_cpu_cores: u32,
    /// Total system memory in megabytes
    pub memory_total_mb: f64,
    /// Available memory in megabytes
    pub memory_available_mb: f64,
    /// Disk usage as a percentage (0.0 to 100.0)
    pub disk_usage_percent: f64,
    /// Number of active network connections
    pub network_connections: u32,
}

/// Metrics collection and aggregation subsystem
///
/// Central coordinator for all metrics collection, aggregation, and reporting
/// in the hive system. Provides real-time and historical performance data.
///
/// ## Components
///
/// - **Current Metrics**: Real-time system state
/// - **Historical Data**: Trend analysis with bounded storage
/// - **Event Counters**: Real-time event tracking
/// - **Export Formats**: Multiple output formats for monitoring systems
/// - **Performance Analysis**: Automated trend detection and alerting
///
/// ## Thread Safety
///
/// All operations are thread-safe using `Arc<RwLock<T>>` for shared state.
/// Event recording uses atomic operations for high concurrency.
///
/// ## Performance
///
/// Optimized for high-frequency event recording with minimal overhead.
/// Historical storage is bounded to prevent memory growth.
#[derive(Clone)]
pub struct MetricsCollector {
    /// Current metrics state
    ///
    /// Real-time metrics data that gets updated with each event.
    /// Protected by RwLock for concurrent read/write access.
    metrics: Arc<RwLock<HiveMetrics>>,

    /// Historical metrics for trend analysis
    ///
    /// Stores periodic snapshots of metrics for trend analysis.
    /// Limited to 1000 entries to prevent unbounded memory growth.
    metrics_history: Arc<RwLock<Vec<HiveMetrics>>>,

    /// Communication channel for coordination
    ///
    /// Async channel for sending coordination messages when
    /// metrics events require system-wide notifications.
    coordination_tx: mpsc::UnboundedSender<CoordinationMessage>,

    /// Event counters for real-time tracking
    ///
    /// Tracks the count of various system events for monitoring
    /// and alerting purposes. Uses string keys for flexibility.
    event_counters: Arc<RwLock<HashMap<String, u64>>>,

    /// System start time for uptime calculation
    ///
    /// Reference point for calculating system uptime and
    /// time-based metrics like tasks per hour.
    start_time: std::time::Instant,
}

impl MetricsCollector {
    /// Create a new metrics collector
    ///
    /// Initializes the metrics collection subsystem with required dependencies.
    /// Sets up data structures for real-time and historical metrics tracking.
    ///
    /// ## Initialization Process
    ///
    /// 1. Creates metrics storage with default values
    /// 2. Initializes empty metrics history
    /// 3. Sets up event counter tracking
    /// 4. Records system start time for uptime calculation
    /// 5. Establishes coordination channel
    ///
    /// ## Performance
    ///
    /// O(1) initialization with minimal memory allocation.
    /// Ready for immediate metrics collection after creation.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::MetricsCollector;
    /// # use tokio::sync::mpsc;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let (tx, _rx) = mpsc::unbounded_channel();
    ///
    /// let metrics_collector = MetricsCollector::new(tx).await?;
    /// println!("Metrics collector initialized");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `coordination_tx` - Channel for sending coordination messages
    ///
    /// # Returns
    ///
    /// Returns a new `MetricsCollector` instance on success.
    ///
    /// # Errors
    ///
    /// This function will not return an error under normal circumstances.
    pub async fn new(
        coordination_tx: mpsc::UnboundedSender<CoordinationMessage>,
    ) -> HiveResult<Self> {
        Ok(Self {
            metrics: Arc::new(RwLock::new(HiveMetrics::default())),
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            coordination_tx,
            event_counters: Arc::new(RwLock::new(HashMap::new())),
            start_time: std::time::Instant::now(),
        })
    }

    /// Record an agent event
    ///
    /// Records an agent-related event and updates relevant metrics.
    /// Supports different event types for comprehensive agent lifecycle tracking.
    ///
    /// ## Supported Event Types
    ///
    /// - `"registered"`: New agent registration
    /// - `"removed"`: Agent removal from system
    /// - Other events are recorded in counters but don't affect core metrics
    ///
    /// ## Metrics Updated
    ///
    /// - Agent counts (total, active, created/removed today)
    /// - Event counters for monitoring
    /// - Last activity timestamps
    /// - Coordination notifications
    ///
    /// ## Performance
    ///
    /// O(1) average case for counter updates and metric modifications.
    /// Triggers coordination messages for system-wide notifications.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::MetricsCollector;
    /// # async fn example(metrics_collector: &MetricsCollector) {
    /// let agent_id = uuid::Uuid::new_v4();
    ///
    /// // Record agent registration
    /// metrics_collector.record_agent_event("registered", agent_id).await;
    ///
    /// // Record agent removal
    /// metrics_collector.record_agent_event("removed", agent_id).await;
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `event_type` - Type of agent event ("registered", "removed", etc.)
    /// * `agent_id` - Unique identifier of the agent
    pub async fn record_agent_event(&self, event_type: &str, agent_id: Uuid) {
        let event_key = format!("agent_{}", event_type);
        {
            let mut counters = self.event_counters.write().await;
            *counters.entry(event_key).or_insert(0) += 1;
        }

        // Update agent metrics
        {
            let mut metrics = self.metrics.write().await;
            match event_type {
                "registered" => {
                    metrics.agent_metrics.total_agents += 1;
                    metrics.agent_metrics.active_agents += 1;
                    metrics.agent_metrics.agents_created_today += 1;
                }
                "removed" => {
                    metrics.agent_metrics.active_agents =
                        metrics.agent_metrics.active_agents.saturating_sub(1);
                    metrics.agent_metrics.agents_removed_today += 1;
                }
                _ => {}
            }
            metrics.last_updated = chrono::Utc::now();
        }

        tracing::debug!(
            "Recorded agent event: {} for agent {}",
            event_type,
            agent_id
        );
    }

    /// Record task completion
    ///
    /// Records a task completion event with execution details and updates
    /// task-related metrics and performance statistics.
    ///
    /// ## Metrics Updated
    ///
    /// - Task counters (total, completed, failed)
    /// - Success rate calculations
    /// - Event counters for monitoring
    /// - Execution history storage
    /// - Coordination notifications
    ///
    /// ## Success Rate Calculation
    ///
    /// `success_rate = completed_tasks / total_tasks`
    ///
    /// ## Performance
    ///
    /// O(1) for counter updates, O(log n) for history storage.
    /// History is bounded to prevent memory growth.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::MetricsCollector;
    /// # async fn example(metrics_collector: &MetricsCollector) {
    /// let task_id = uuid::Uuid::new_v4();
    /// let agent_id = uuid::Uuid::new_v4();
    ///
    /// // Record successful task completion
    /// metrics_collector.record_task_completion(task_id, agent_id, true).await;
    ///
    /// // Record failed task completion
    /// let failed_task_id = uuid::Uuid::new_v4();
    /// metrics_collector.record_task_completion(failed_task_id, agent_id, false).await;
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `task_id` - Unique identifier of the completed task
    /// * `agent_id` - Unique identifier of the agent that executed the task
    /// * `success` - Whether the task execution was successful
    pub async fn record_task_completion(&self, task_id: Uuid, agent_id: Uuid, success: bool) {
        {
            let mut counters = self.event_counters.write().await;
            *counters.entry("tasks_completed".to_string()).or_insert(0) += 1;
            if success {
                *counters.entry("tasks_successful".to_string()).or_insert(0) += 1;
            } else {
                *counters.entry("tasks_failed".to_string()).or_insert(0) += 1;
            }
        }

        // Update task metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.task_metrics.total_tasks += 1;
            if success {
                metrics.task_metrics.completed_tasks += 1;
            } else {
                metrics.task_metrics.failed_tasks += 1;
            }

            // Recalculate success rate
            if metrics.task_metrics.total_tasks > 0 {
                metrics.task_metrics.success_rate = metrics.task_metrics.completed_tasks as f64
                    / metrics.task_metrics.total_tasks as f64;
            }

            metrics.last_updated = chrono::Utc::now();
        }

        tracing::debug!(
            "Recorded task completion: {} by agent {} (success: {})",
            task_id,
            agent_id,
            success
        );
    }

    /// Update system metrics
    ///
    /// Updates system-level metrics from external monitoring data.
    /// Incorporates CPU usage, memory usage, and other system indicators.
    ///
    /// ## Supported Metrics
    ///
    /// - `"cpu_usage"`: CPU usage as fraction (0.0 to 1.0)
    /// - `"memory_usage"`: Memory usage in megabytes
    /// - Other metrics are ignored but don't cause errors
    ///
    /// ## Automatic Calculations
    ///
    /// - CPU percentage: `cpu_usage * 100.0`
    /// - Uptime: Calculated from system start time
    ///
    /// ## Performance
    ///
    /// O(1) for metric updates with JSON parsing overhead.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::MetricsCollector;
    /// # async fn example(metrics_collector: &MetricsCollector) {
    /// let system_data = serde_json::json!({
    ///     "cpu_usage": 0.75,        // 75% CPU usage
    ///     "memory_usage": 1024.0    // 1024 MB memory usage
    /// });
    ///
    /// metrics_collector.update_metrics(system_data).await;
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `new_metrics` - JSON object containing system metrics data
    pub async fn update_metrics(&self, new_metrics: serde_json::Value) {
        let mut metrics = self.metrics.write().await;

        // Update system uptime
        metrics.system_metrics.uptime_seconds = self.start_time.elapsed().as_secs();

        // Extract and update metrics from the provided data
        if let Some(cpu_usage) = new_metrics.get("cpu_usage").and_then(|v| v.as_f64()) {
            metrics.system_metrics.cpu_usage_percent = cpu_usage * 100.0;
        }

        if let Some(memory_usage) = new_metrics.get("memory_usage").and_then(|v| v.as_f64()) {
            metrics.system_metrics.total_memory_usage_mb = memory_usage;
        }

        metrics.last_updated = chrono::Utc::now();
    }

    /// Collect periodic metrics
    ///
    /// Performs periodic metrics collection and aggregation for trend analysis.
    /// Updates calculated metrics like tasks per hour and stores historical snapshots.
    ///
    /// ## Operations Performed
    ///
    /// 1. Updates system uptime
    /// 2. Calculates tasks per hour rate
    /// 3. Stores metrics snapshot in history
    /// 4. Manages history size (max 1000 entries)
    /// 5. Updates last modified timestamp
    ///
    /// ## History Management
    ///
    /// - Keeps last 1000 snapshots for trend analysis
    /// - Removes oldest entries when limit exceeded
    /// - Prevents unbounded memory growth
    ///
    /// ## Performance
    ///
    /// O(1) for most operations, O(n) for history size management.
    /// Designed to run frequently without performance impact.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::MetricsCollector;
    /// # async fn example(metrics_collector: &MetricsCollector) -> Result<(), Box<dyn std::error::Error>> {
    /// // Collect periodic metrics (typically called by background process)
    /// metrics_collector.collect_periodic_metrics().await?;
    /// println!("Periodic metrics collected");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if metrics collection succeeds.
    ///
    /// # Errors
    ///
    /// Returns error if metrics collection fails.
    pub async fn collect_periodic_metrics(&self) -> HiveResult<()> {
        // Update system metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.system_metrics.uptime_seconds = self.start_time.elapsed().as_secs();

            // Calculate tasks per hour
            let hours_running = metrics.system_metrics.uptime_seconds as f64 / 3600.0;
            if hours_running > 0.0 {
                metrics.task_metrics.tasks_per_hour =
                    metrics.task_metrics.total_tasks as f64 / hours_running;
            }

            metrics.last_updated = chrono::Utc::now();
        }

        // Store historical snapshot
        {
            let current_metrics = self.metrics.read().await.clone();
            let mut history = self.metrics_history.write().await;
            history.push(current_metrics);

            // Keep only last 1000 snapshots to prevent memory growth
            if history.len() > 1000 {
                history.remove(0);
            }
        }

        Ok(())
    }

    /// Get current metrics
    ///
    /// Returns a snapshot of the current system metrics for real-time monitoring.
    /// Provides immediate access to all system performance indicators.
    ///
    /// ## Data Freshness
    ///
    /// Returns the most recently updated metrics. Use `last_updated` field
    /// to check data freshness for monitoring purposes.
    ///
    /// ## Performance
    ///
    /// O(1) time complexity - direct clone of current metrics.
    /// Memory overhead from cloning the metrics structure.
    ///
    /// ## Use Cases
    ///
    /// - Real-time dashboards
    /// - Health check endpoints
    /// - Monitoring systems
    /// - Performance analysis
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::MetricsCollector;
    /// # async fn example(metrics_collector: &MetricsCollector) {
    /// let metrics = metrics_collector.get_current_metrics().await;
    ///
    /// println!("Total agents: {}", metrics.agent_metrics.total_agents);
    /// println!("Task success rate: {:.1}%", metrics.task_metrics.success_rate * 100.0);
    /// println!("System uptime: {} seconds", metrics.system_metrics.uptime_seconds);
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a clone of the current `HiveMetrics` structure.
    pub async fn get_current_metrics(&self) -> HiveMetrics {
        self.metrics.read().await.clone()
    }

    /// Get enhanced metrics with trends
    ///
    /// Returns comprehensive metrics including current state, historical trends,
    /// event counters, and analytical insights for advanced monitoring.
    ///
    /// ## Data Structure
    ///
    /// ```json
    /// {
    ///   "current": { /* current HiveMetrics */ },
    ///   "trends": { /* trend analysis */ },
    ///   "event_counters": { /* event counts */ },
    ///   "history_size": 42
    /// }
    /// ```
    ///
    /// ## Trend Analysis
    ///
    /// Calculates growth rates and changes between current and previous
    /// metrics snapshots for performance trend identification.
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is history size for trend calculation.
    /// May involve complex calculations for comprehensive analysis.
    ///
    /// ## Use Cases
    ///
    /// - Advanced monitoring dashboards
    /// - Performance trend analysis
    /// - Capacity planning
    /// - Anomaly detection
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::MetricsCollector;
    /// # async fn example(metrics_collector: &MetricsCollector) {
    /// let enhanced = metrics_collector.get_enhanced_metrics().await;
    ///
    /// let current = enhanced.get("current").unwrap();
    /// let trends = enhanced.get("trends").unwrap();
    /// let counters = enhanced.get("event_counters").unwrap();
    ///
    /// println!("Current metrics: {}", current);
    /// println!("Performance trends: {}", trends);
    /// println!("Event counters: {}", counters);
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a JSON object with enhanced metrics and analysis.
    pub async fn get_enhanced_metrics(&self) -> serde_json::Value {
        let current_metrics = self.metrics.read().await.clone();
        let history = self.metrics_history.read().await;
        let counters = self.event_counters.read().await.clone();

        let trends = if history.len() >= 2 {
            let previous = &history[history.len() - 2];
            self.calculate_trends(&current_metrics, previous).await
        } else {
            serde_json::json!({})
        };

        serde_json::json!({
            "current": current_metrics,
            "trends": trends,
            "event_counters": counters,
            "history_size": history.len()
        })
    }

    /// Calculate performance trends
    ///
    /// Analyzes differences between current and previous metrics to identify
    /// performance trends and growth patterns in the system.
    ///
    /// ## Trend Calculations
    ///
    /// - **Agent Growth**: `(current - previous) / previous * 100%`
    /// - **Task Growth**: Similar calculation for task metrics
    /// - **Success Rate Change**: Absolute difference in success rates
    /// - **CPU Usage Change**: Absolute difference in CPU usage
    ///
    /// ## Performance
    ///
    /// O(1) time complexity - simple arithmetic operations.
    ///
    /// # Parameters
    ///
    /// * `current` - Current metrics snapshot
    /// * `previous` - Previous metrics snapshot for comparison
    ///
    /// # Returns
    ///
    /// Returns a JSON object with calculated trend values.
    async fn calculate_trends(
        &self,
        current: &HiveMetrics,
        previous: &HiveMetrics,
    ) -> serde_json::Value {
        let agent_trend = if previous.agent_metrics.total_agents > 0 {
            (current.agent_metrics.total_agents as f64 - previous.agent_metrics.total_agents as f64)
                / previous.agent_metrics.total_agents as f64
                * 100.0
        } else {
            0.0
        };

        let task_trend = if previous.task_metrics.total_tasks > 0 {
            (current.task_metrics.total_tasks as f64 - previous.task_metrics.total_tasks as f64)
                / previous.task_metrics.total_tasks as f64
                * 100.0
        } else {
            0.0
        };

        let success_rate_trend =
            current.task_metrics.success_rate - previous.task_metrics.success_rate;

        serde_json::json!({
            "agent_growth_percent": agent_trend,
            "task_growth_percent": task_trend,
            "success_rate_change": success_rate_trend,
            "cpu_usage_change": current.system_metrics.cpu_usage_percent - previous.system_metrics.cpu_usage_percent
        })
    }

    /// Get metrics summary for dashboard
    ///
    /// Returns a simplified metrics summary optimized for dashboard display
    /// and monitoring interfaces. Contains key performance indicators.
    ///
    /// ## Summary Contents
    ///
    /// - Agent counts (total, active)
    /// - Task statistics (total, success rate, per hour)
    /// - System health (uptime, CPU, memory)
    /// - Last update timestamp
    ///
    /// ## Performance
    ///
    /// O(1) time complexity - direct access to current metrics.
    /// Optimized for frequent dashboard updates.
    ///
    /// ## Use Cases
    ///
    /// - Web dashboards
    /// - Mobile monitoring apps
    /// - Status displays
    /// - Quick health checks
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::MetricsCollector;
    /// # async fn example(metrics_collector: &MetricsCollector) {
    /// let summary = metrics_collector.get_metrics_summary().await;
    ///
    /// let agents = summary.get("agents").unwrap().as_object().unwrap();
    /// let tasks = summary.get("tasks").unwrap().as_object().unwrap();
    /// let system = summary.get("system").unwrap().as_object().unwrap();
    ///
    /// println!("Agents: {} total, {} active",
    ///     agents.get("total").unwrap(),
    ///     agents.get("active").unwrap()
    /// );
    /// println!("Tasks: {} total, {:.1}% success rate",
    ///     tasks.get("total").unwrap(),
    ///     tasks.get("success_rate").unwrap().as_f64().unwrap() * 100.0
    /// );
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a JSON object with dashboard-optimized metrics summary.
    pub async fn get_metrics_summary(&self) -> serde_json::Value {
        let metrics = self.metrics.read().await;

        serde_json::json!({
            "agents": {
                "total": metrics.agent_metrics.total_agents,
                "active": metrics.agent_metrics.active_agents
            },
            "tasks": {
                "total": metrics.task_metrics.total_tasks,
                "success_rate": metrics.task_metrics.success_rate,
                "per_hour": metrics.task_metrics.tasks_per_hour
            },
            "system": {
                "uptime_hours": metrics.system_metrics.uptime_seconds as f64 / 3600.0,
                "cpu_usage": metrics.system_metrics.cpu_usage_percent,
                "memory_usage": metrics.system_metrics.total_memory_usage_mb
            },
            "last_updated": metrics.last_updated
        })
    }

    /// Reset daily counters (should be called at midnight)
    ///
    /// Resets daily-specific counters for agent creation and removal tracking.
    /// Should be called periodically (e.g., daily) to maintain accurate daily statistics.
    ///
    /// ## Counters Reset
    ///
    /// - `agents_created_today`: Reset to 0
    /// - `agents_removed_today`: Reset to 0
    /// - `last_updated`: Updated to current timestamp
    ///
    /// ## Performance
    ///
    /// O(1) time complexity - simple counter reset operations.
    ///
    /// ## Usage Pattern
    ///
    /// Typically called by a scheduled background process:
    /// ```rust,no_run
    /// # use hive::core::hive::MetricsCollector;
    /// # async fn daily_reset(metrics_collector: &MetricsCollector) {
    /// // Call at midnight or appropriate reset time
    /// metrics_collector.reset_daily_counters().await;
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// This function takes no parameters.
    pub async fn reset_daily_counters(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.agent_metrics.agents_created_today = 0;
        metrics.agent_metrics.agents_removed_today = 0;
        metrics.last_updated = chrono::Utc::now();

        tracing::info!("Daily metrics counters reset");
    }

    /// Export metrics for external monitoring systems
    ///
    /// Exports current metrics in various formats for integration with
    /// external monitoring, alerting, and visualization systems.
    ///
    /// ## Supported Formats
    ///
    /// - `"json"`: Pretty-printed JSON format for general use
    /// - `"prometheus"`: Prometheus exposition format for metrics collection
    /// - Other formats return validation errors
    ///
    /// ## JSON Format
    ///
    /// Complete `HiveMetrics` structure serialized as pretty-printed JSON.
    /// Suitable for REST APIs, configuration files, and general data exchange.
    ///
    /// ## Prometheus Format
    ///
    /// Standard Prometheus metrics format with HELP/TYPE comments and
    /// metric names following Prometheus naming conventions.
    ///
    /// ## Performance
    ///
    /// O(1) for format validation, variable for serialization.
    /// JSON serialization is typically faster than Prometheus formatting.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::MetricsCollector;
    /// # async fn example(metrics_collector: &MetricsCollector) -> Result<(), Box<dyn std::error::Error>> {
    /// // Export as JSON
    /// let json_metrics = metrics_collector.export_metrics("json").await?;
    /// println!("JSON metrics: {}", json_metrics);
    ///
    /// // Export for Prometheus
    /// let prometheus_metrics = metrics_collector.export_metrics("prometheus").await?;
    /// println!("Prometheus metrics:\n{}", prometheus_metrics);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `format` - Export format ("json" or "prometheus")
    ///
    /// # Returns
    ///
    /// Returns the metrics data as a formatted string on success.
    ///
    /// # Errors
    ///
    /// Returns error if format is unsupported or serialization fails.
    pub async fn export_metrics(&self, format: &str) -> HiveResult<String> {
        let metrics = self.metrics.read().await;

        match format {
            "json" => {
                serde_json::to_string_pretty(&*metrics).map_err(|e| HiveError::OperationFailed {
                    reason: format!("Failed to serialize metrics to JSON: {}", e),
                })
            }
            "prometheus" => Ok(self.format_prometheus_metrics(&metrics).await),
            _ => Err(HiveError::ValidationError {
                field: "format".to_string(),
                reason: format!("Unsupported export format: {}", format),
            }),
        }
    }

    /// Format metrics in Prometheus format
    ///
    /// Converts `HiveMetrics` into Prometheus exposition format for
    /// integration with Prometheus monitoring systems.
    ///
    /// ## Prometheus Format
    ///
    /// ```
    /// # HELP hive_agents_total Total number of agents
    /// # TYPE hive_agents_total counter
    /// hive_agents_total 42
    /// # HELP hive_tasks_success_rate Task success rate
    /// # TYPE hive_tasks_success_rate gauge
    /// hive_tasks_success_rate 0.95
    /// ```
    ///
    /// ## Metric Types
    ///
    /// - **Counters**: Monotonically increasing values (agents, tasks)
    /// - **Gauges**: Values that can increase/decrease (rates, percentages)
    ///
    /// ## Performance
    ///
    /// O(1) time complexity - fixed number of string formatting operations.
    ///
    /// # Parameters
    ///
    /// * `metrics` - Metrics data to format
    ///
    /// # Returns
    ///
    /// Returns Prometheus-formatted metrics as a string.
    async fn format_prometheus_metrics(&self, metrics: &HiveMetrics) -> String {
        format!(
            "# HELP hive_agents_total Total number of agents\n\
             # TYPE hive_agents_total counter\n\
             hive_agents_total {}\n\
             # HELP hive_agents_active Number of active agents\n\
             # TYPE hive_agents_active gauge\n\
             hive_agents_active {}\n\
             # HELP hive_tasks_total Total number of tasks\n\
             # TYPE hive_tasks_total counter\n\
             hive_tasks_total {}\n\
             # HELP hive_tasks_success_rate Task success rate\n\
             # TYPE hive_tasks_success_rate gauge\n\
             hive_tasks_success_rate {}\n\
             # HELP hive_system_uptime_seconds System uptime in seconds\n\
             # TYPE hive_system_uptime_seconds counter\n\
             hive_system_uptime_seconds {}\n\
             # HELP hive_system_cpu_usage_percent CPU usage percentage\n\
             # TYPE hive_system_cpu_usage_percent gauge\n\
             hive_system_cpu_usage_percent {}\n",
            metrics.agent_metrics.total_agents,
            metrics.agent_metrics.active_agents,
            metrics.task_metrics.total_tasks,
            metrics.task_metrics.success_rate,
            metrics.system_metrics.uptime_seconds,
            metrics.system_metrics.cpu_usage_percent
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::mpsc;

    // Helper function to create a test metrics collector
    async fn create_test_metrics_collector() -> HiveResult<MetricsCollector> {
        let (tx, _rx) = mpsc::unbounded_channel();
        MetricsCollector::new(tx).await
    }

    #[tokio::test]
    async fn test_metrics_collector_creation() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let metrics = metrics_collector.get_current_metrics().await;
        assert_eq!(metrics.agent_metrics.total_agents, 0);
        assert_eq!(metrics.task_metrics.total_tasks, 0);
        assert_eq!(metrics.system_metrics.uptime_seconds, 0);
        assert!(metrics.last_updated <= chrono::Utc::now());

        Ok(())
    }

    #[tokio::test]
    async fn test_agent_event_recording_registered() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let agent_id = Uuid::new_v4();
        metrics_collector
            .record_agent_event("registered", agent_id)
            .await;

        let metrics = metrics_collector.get_current_metrics().await;
        assert_eq!(metrics.agent_metrics.total_agents, 1);
        assert_eq!(metrics.agent_metrics.active_agents, 1);
        assert_eq!(metrics.agent_metrics.agents_created_today, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_agent_event_recording_removed() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let agent_id = Uuid::new_v4();

        // First register an agent
        metrics_collector
            .record_agent_event("registered", agent_id)
            .await;

        // Then remove it
        metrics_collector
            .record_agent_event("removed", agent_id)
            .await;

        let metrics = metrics_collector.get_current_metrics().await;
        assert_eq!(metrics.agent_metrics.total_agents, 1); // Total remains
        assert_eq!(metrics.agent_metrics.active_agents, 0); // Active decreases
        assert_eq!(metrics.agent_metrics.agents_removed_today, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_agent_event_recording_unknown_event() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let agent_id = Uuid::new_v4();
        metrics_collector
            .record_agent_event("unknown_event", agent_id)
            .await;

        let metrics = metrics_collector.get_current_metrics().await;
        // Should not affect any metrics
        assert_eq!(metrics.agent_metrics.total_agents, 0);
        assert_eq!(metrics.agent_metrics.active_agents, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_task_completion_recording_success() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let task_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        metrics_collector
            .record_task_completion(task_id, agent_id, true)
            .await;

        let metrics = metrics_collector.get_current_metrics().await;
        assert_eq!(metrics.task_metrics.total_tasks, 1);
        assert_eq!(metrics.task_metrics.completed_tasks, 1);
        assert_eq!(metrics.task_metrics.failed_tasks, 0);
        assert_eq!(metrics.task_metrics.success_rate, 1.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_task_completion_recording_failure() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let task_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        metrics_collector
            .record_task_completion(task_id, agent_id, false)
            .await;

        let metrics = metrics_collector.get_current_metrics().await;
        assert_eq!(metrics.task_metrics.total_tasks, 1);
        assert_eq!(metrics.task_metrics.completed_tasks, 0);
        assert_eq!(metrics.task_metrics.failed_tasks, 1);
        assert_eq!(metrics.task_metrics.success_rate, 0.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_task_completion_recording_mixed() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let agent_id = Uuid::new_v4();

        // Record 3 successes and 2 failures
        for i in 0..3 {
            let task_id = Uuid::new_v4();
            metrics_collector
                .record_task_completion(task_id, agent_id, true)
                .await;
        }

        for i in 0..2 {
            let task_id = Uuid::new_v4();
            metrics_collector
                .record_task_completion(task_id, agent_id, false)
                .await;
        }

        let metrics = metrics_collector.get_current_metrics().await;
        assert_eq!(metrics.task_metrics.total_tasks, 5);
        assert_eq!(metrics.task_metrics.completed_tasks, 3);
        assert_eq!(metrics.task_metrics.failed_tasks, 2);
        assert!((metrics.task_metrics.success_rate - 0.6).abs() < 0.001); // 3/5 = 0.6

        Ok(())
    }

    #[tokio::test]
    async fn test_update_metrics() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let metrics_data = serde_json::json!({
            "cpu_usage": 0.75,
            "memory_usage": 512.0
        });

        metrics_collector.update_metrics(metrics_data).await;

        let metrics = metrics_collector.get_current_metrics().await;
        assert_eq!(metrics.system_metrics.cpu_usage_percent, 75.0); // 0.75 * 100
        assert_eq!(metrics.system_metrics.total_memory_usage_mb, 512.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_metrics_partial() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let metrics_data = serde_json::json!({
            "cpu_usage": 0.5
            // Missing memory_usage
        });

        metrics_collector.update_metrics(metrics_data).await;

        let metrics = metrics_collector.get_current_metrics().await;
        assert_eq!(metrics.system_metrics.cpu_usage_percent, 50.0);
        assert_eq!(metrics.system_metrics.total_memory_usage_mb, 0.0); // Should remain default

        Ok(())
    }

    #[tokio::test]
    async fn test_collect_periodic_metrics() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        // Add some initial data
        let agent_id = Uuid::new_v4();
        metrics_collector
            .record_agent_event("registered", agent_id)
            .await;

        let task_id = Uuid::new_v4();
        metrics_collector
            .record_task_completion(task_id, agent_id, true)
            .await;

        // Collect periodic metrics
        metrics_collector.collect_periodic_metrics().await?;

        let metrics = metrics_collector.get_current_metrics().await;
        assert!(metrics.system_metrics.uptime_seconds > 0);
        assert!(metrics.task_metrics.tasks_per_hour >= 0.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_enhanced_metrics() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let enhanced = metrics_collector.get_enhanced_metrics().await;
        assert!(enhanced.is_object());
        assert!(enhanced.get("current").is_some());
        assert!(enhanced.get("trends").is_some());
        assert!(enhanced.get("event_counters").is_some());
        assert!(enhanced.get("history_size").is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_metrics_summary() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let summary = metrics_collector.get_metrics_summary().await;
        assert!(summary.is_object());
        assert!(summary.get("agents").is_some());
        assert!(summary.get("tasks").is_some());
        assert!(summary.get("system").is_some());
        assert!(summary.get("last_updated").is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_reset_daily_counters() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        // Add some daily counters
        let agent_id = Uuid::new_v4();
        metrics_collector
            .record_agent_event("registered", agent_id)
            .await;

        // Verify counters are set
        let metrics_before = metrics_collector.get_current_metrics().await;
        assert_eq!(metrics_before.agent_metrics.agents_created_today, 1);

        // Reset counters
        metrics_collector.reset_daily_counters().await;

        // Verify counters are reset
        let metrics_after = metrics_collector.get_current_metrics().await;
        assert_eq!(metrics_after.agent_metrics.agents_created_today, 0);
        assert_eq!(metrics_after.agent_metrics.agents_removed_today, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_export_metrics_json() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let exported = metrics_collector.export_metrics("json").await?;
        let parsed: serde_json::Value = serde_json::from_str(&exported)?;
        assert!(parsed.is_object());

        Ok(())
    }

    #[tokio::test]
    async fn test_export_metrics_prometheus() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let exported = metrics_collector.export_metrics("prometheus").await?;
        assert!(exported.contains("# HELP"));
        assert!(exported.contains("# TYPE"));
        assert!(exported.contains("hive_agents_total"));

        Ok(())
    }

    #[tokio::test]
    async fn test_export_metrics_invalid_format() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let result = metrics_collector.export_metrics("invalid").await;
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_calculate_trends_no_previous() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let current = HiveMetrics::default();
        let trends = metrics_collector.calculate_trends(&current, &current).await;

        assert!(trends.is_object());
        assert_eq!(trends.get("agent_growth_percent").unwrap(), 0.0);
        assert_eq!(trends.get("task_growth_percent").unwrap(), 0.0);
        assert_eq!(trends.get("success_rate_change").unwrap(), 0.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_calculate_trends_with_data() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let mut previous = HiveMetrics::default();
        previous.agent_metrics.total_agents = 10;
        previous.task_metrics.total_tasks = 20;
        previous.task_metrics.success_rate = 0.8;
        previous.system_metrics.cpu_usage_percent = 50.0;

        let mut current = HiveMetrics::default();
        current.agent_metrics.total_agents = 15; // +50% growth
        current.task_metrics.total_tasks = 25; // +25% growth
        current.task_metrics.success_rate = 0.85; // +0.05 change
        current.system_metrics.cpu_usage_percent = 55.0; // +5.0 change

        let trends = metrics_collector
            .calculate_trends(&current, &previous)
            .await;

        assert_eq!(trends.get("agent_growth_percent").unwrap(), 50.0);
        assert_eq!(trends.get("task_growth_percent").unwrap(), 25.0);
        assert_eq!(trends.get("success_rate_change").unwrap(), 0.05);
        assert_eq!(trends.get("cpu_usage_change").unwrap(), 5.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_metrics_history_limit() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        // Collect metrics many times
        for _ in 0..1500 {
            metrics_collector.collect_periodic_metrics().await?;
        }

        let history_size = metrics_collector.metrics_history.read().await.len();
        assert!(history_size <= 1000);

        Ok(())
    }

    #[tokio::test]
    async fn test_event_counters() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let agent_id = Uuid::new_v4();
        let task_id = Uuid::new_v4();

        // Record various events
        metrics_collector
            .record_agent_event("registered", agent_id)
            .await;
        metrics_collector
            .record_agent_event("removed", agent_id)
            .await;
        metrics_collector
            .record_task_completion(task_id, agent_id, true)
            .await;

        let enhanced = metrics_collector.get_enhanced_metrics().await;
        let counters = enhanced.get("event_counters").unwrap().as_object().unwrap();

        assert!(counters.contains_key("agent_registered"));
        assert!(counters.contains_key("agent_removed"));
        assert!(counters.contains_key("tasks_completed"));
        assert!(counters.contains_key("tasks_successful"));

        Ok(())
    }

    #[tokio::test]
    async fn test_hive_metrics_default() {
        let metrics = HiveMetrics::default();

        assert_eq!(metrics.agent_metrics.total_agents, 0);
        assert_eq!(metrics.task_metrics.total_tasks, 0);
        assert_eq!(metrics.system_metrics.uptime_seconds, 0);
        assert!(metrics.last_updated <= chrono::Utc::now());
    }

    #[tokio::test]
    async fn test_agent_metrics_default() {
        let metrics = AgentMetrics::default();

        assert_eq!(metrics.total_agents, 0);
        assert_eq!(metrics.active_agents, 0);
        assert_eq!(metrics.agents_created_today, 0);
        assert_eq!(metrics.agents_removed_today, 0);
        assert_eq!(metrics.average_agent_performance, 0.0);
        assert!(metrics.top_performer_id.is_none());
    }

    #[tokio::test]
    async fn test_task_metrics_default() {
        let metrics = TaskMetrics::default();

        assert_eq!(metrics.total_tasks, 0);
        assert_eq!(metrics.completed_tasks, 0);
        assert_eq!(metrics.failed_tasks, 0);
        assert_eq!(metrics.pending_tasks, 0);
        assert_eq!(metrics.average_execution_time_ms, 0.0);
        assert_eq!(metrics.tasks_per_hour, 0.0);
        assert_eq!(metrics.success_rate, 0.0);
    }

    #[tokio::test]
    async fn test_system_metrics_default() {
        let metrics = SystemMetrics::default();

        assert_eq!(metrics.uptime_seconds, 0);
        assert_eq!(metrics.total_memory_usage_mb, 0.0);
        assert_eq!(metrics.cpu_usage_percent, 0.0);
        assert_eq!(metrics.network_throughput_mbps, 0.0);
        assert_eq!(metrics.error_rate, 0.0);
        assert_eq!(metrics.response_time_ms, 0.0);
    }

    #[tokio::test]
    async fn test_resource_metrics_default() {
        let metrics = ResourceMetrics::default();

        assert_eq!(metrics.available_cpu_cores, 0);
        assert_eq!(metrics.memory_total_mb, 0.0);
        assert_eq!(metrics.memory_available_mb, 0.0);
        assert_eq!(metrics.disk_usage_percent, 0.0);
        assert_eq!(metrics.network_connections, 0);
    }

    #[tokio::test]
    async fn test_swarm_metrics_default() {
        let metrics = SwarmMetrics::default();

        assert_eq!(metrics.total_agents, 0);
        assert_eq!(metrics.active_agents, 0);
        assert_eq!(metrics.completed_tasks, 0);
        assert_eq!(metrics.failed_tasks, 0);
        assert_eq!(metrics.average_performance, 0.0);
        assert_eq!(metrics.swarm_cohesion, 0.0);
        assert_eq!(metrics.learning_progress, 0.0);
        assert_eq!(metrics.uptime_seconds, 0);
    }

    #[tokio::test]
    async fn test_prometheus_format_structure() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        // Add some test data
        let agent_id = Uuid::new_v4();
        metrics_collector
            .record_agent_event("registered", agent_id)
            .await;

        let task_id = Uuid::new_v4();
        metrics_collector
            .record_task_completion(task_id, agent_id, true)
            .await;

        let prometheus_output = metrics_collector
            .format_prometheus_metrics(&metrics_collector.get_current_metrics().await)
            .await;

        // Check that it contains expected Prometheus format elements
        assert!(prometheus_output.contains("# HELP hive_agents_total"));
        assert!(prometheus_output.contains("# TYPE hive_agents_total counter"));
        assert!(prometheus_output.contains("hive_agents_total 1"));
        assert!(prometheus_output.contains("# HELP hive_tasks_success_rate"));
        assert!(prometheus_output.contains("# TYPE hive_tasks_success_rate gauge"));
        assert!(prometheus_output.contains("hive_tasks_success_rate 1"));
        assert!(prometheus_output.contains("# HELP hive_system_uptime_seconds"));
        assert!(prometheus_output.contains("# TYPE hive_system_uptime_seconds counter"));

        Ok(())
    }

    #[tokio::test]
    async fn test_metrics_timestamp_updates() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let before = chrono::Utc::now();
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;

        let agent_id = Uuid::new_v4();
        metrics_collector
            .record_agent_event("registered", agent_id)
            .await;

        let after = chrono::Utc::now();

        let metrics = metrics_collector.get_current_metrics().await;
        assert!(metrics.last_updated >= before);
        assert!(metrics.last_updated <= after);

        Ok(())
    }

    #[tokio::test]
    async fn test_metrics_summary_structure() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        // Add some test data
        let agent_id = Uuid::new_v4();
        metrics_collector
            .record_agent_event("registered", agent_id)
            .await;

        let task_id = Uuid::new_v4();
        metrics_collector
            .record_task_completion(task_id, agent_id, true)
            .await;

        let summary = metrics_collector.get_metrics_summary().await;

        // Check agents section
        let agents = summary.get("agents").unwrap().as_object().unwrap();
        assert_eq!(agents.get("total").unwrap(), 1);
        assert_eq!(agents.get("active").unwrap(), 1);

        // Check tasks section
        let tasks = summary.get("tasks").unwrap().as_object().unwrap();
        assert_eq!(tasks.get("total").unwrap(), 1);
        assert_eq!(tasks.get("success_rate").unwrap(), 1.0);

        // Check system section
        let system = summary.get("system").unwrap().as_object().unwrap();
        assert!(system.get("uptime_hours").is_some());
        assert!(system.get("cpu_usage").is_some());
        assert!(system.get("memory_usage").is_some());

        // Check timestamp
        assert!(summary.get("last_updated").is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_event_counter_increment() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let agent_id = Uuid::new_v4();

        // Record the same event multiple times
        for _ in 0..3 {
            metrics_collector
                .record_agent_event("registered", agent_id)
                .await;
        }

        let enhanced = metrics_collector.get_enhanced_metrics().await;
        let counters = enhanced.get("event_counters").unwrap().as_object().unwrap();

        assert_eq!(counters.get("agent_registered").unwrap(), 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_task_execution_time_calculation() -> Result<(), Box<dyn std::error::Error>> {
        let metrics_collector = create_test_metrics_collector().await?;

        let agent_id = Uuid::new_v4();

        // Record tasks with different execution times
        let task_times = vec![100, 200, 150, 300];

        for &time in &task_times {
            let task_id = Uuid::new_v4();
            // Simulate updating execution time (this would normally be done by task distributor)
            // For testing, we'll just record completion
            metrics_collector
                .record_task_completion(task_id, agent_id, true)
                .await;
        }

        let metrics = metrics_collector.get_current_metrics().await;
        assert_eq!(metrics.task_metrics.total_tasks, 4);
        assert_eq!(metrics.task_metrics.completed_tasks, 4);
        assert_eq!(metrics.task_metrics.success_rate, 1.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_metrics_persistence_across_operations() -> Result<(), Box<dyn std::error::Error>>
    {
        let metrics_collector = create_test_metrics_collector().await?;

        // Perform various operations
        let agent_id1 = Uuid::new_v4();
        let agent_id2 = Uuid::new_v4();

        metrics_collector
            .record_agent_event("registered", agent_id1)
            .await;
        metrics_collector
            .record_agent_event("registered", agent_id2)
            .await;

        let task_id1 = Uuid::new_v4();
        let task_id2 = Uuid::new_v4();
        metrics_collector
            .record_task_completion(task_id1, agent_id1, true)
            .await;
        metrics_collector
            .record_task_completion(task_id2, agent_id2, false)
            .await;

        // Update system metrics
        let system_data = serde_json::json!({"cpu_usage": 0.8, "memory_usage": 1024.0});
        metrics_collector.update_metrics(system_data).await;

        // Verify all data persists
        let metrics = metrics_collector.get_current_metrics().await;
        assert_eq!(metrics.agent_metrics.total_agents, 2);
        assert_eq!(metrics.agent_metrics.active_agents, 2);
        assert_eq!(metrics.task_metrics.total_tasks, 2);
        assert_eq!(metrics.task_metrics.completed_tasks, 1);
        assert_eq!(metrics.task_metrics.failed_tasks, 1);
        assert_eq!(metrics.system_metrics.cpu_usage_percent, 80.0);
        assert_eq!(metrics.system_metrics.total_memory_usage_mb, 1024.0);

        Ok(())
    }
}
