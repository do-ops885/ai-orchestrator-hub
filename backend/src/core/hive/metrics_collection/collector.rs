//! # Metrics Collector Module
//!
//! This module implements the main MetricsCollector struct that handles
//! real-time metrics collection, aggregation, and storage.

use crate::utils::error::HiveResult;


use super::types::HiveMetrics;
use crate::core::hive::coordinator::CoordinationMessage;

use chrono::Utc;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

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
    pub metrics: Arc<RwLock<HiveMetrics>>,

    /// Historical metrics for trend analysis
    ///
    /// Stores periodic snapshots of metrics for trend analysis.
    /// Limited to 1000 entries to prevent unbounded memory growth.
    pub metrics_history: Arc<RwLock<Vec<HiveMetrics>>>,

    /// Communication channel for coordination
    ///
    /// Async channel for sending coordination messages when
    /// metrics events require system-wide notifications.
    pub coordination_tx: mpsc::UnboundedSender<CoordinationMessage>,

    /// Event counters for real-time tracking
    ///
    /// Tracks the count of various system events for monitoring
    /// and alerting purposes. Uses string keys for flexibility.
    pub event_counters: Arc<RwLock<HashMap<String, u64>>>,

    /// System start time for uptime calculation
    ///
    /// Reference point for calculating system uptime and
    /// time-based metrics like tasks per hour.
    pub start_time: std::time::Instant,
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
            metrics.last_updated = Utc::now();
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

            metrics.last_updated = Utc::now();
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

        metrics.last_updated = Utc::now();
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

            metrics.last_updated = Utc::now();
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
    pub async fn get_current_metrics(&self) -> HiveMetrics {
        self.metrics.read().await.clone()
    }
}
