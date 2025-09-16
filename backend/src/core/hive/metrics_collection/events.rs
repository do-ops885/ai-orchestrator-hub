//! # Metrics Events Module
//!
//! This module handles event recording and counter management
//! for real-time metrics tracking.

use super::collector::MetricsCollector;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

impl MetricsCollector {
    /// Get event counters
    ///
    /// Returns a snapshot of all current event counters for monitoring
    /// system activity and event frequencies.
    ///
    /// ## Event Types
    ///
    /// - **Agent Events**: `agent_registered`, `agent_removed`
    /// - **Task Events**: `tasks_completed`, `tasks_successful`, `tasks_failed`
    /// - **System Events**: Various system-level event counters
    ///
    /// ## Performance
    ///
    /// O(1) time complexity - direct clone of counter map.
    /// Memory overhead from cloning the HashMap.
    ///
    /// ## Use Cases
    ///
    /// - Real-time monitoring dashboards
    /// - Alerting based on event rates
    /// - System activity analysis
    /// - Performance bottleneck detection
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::MetricsCollector;
    /// # async fn example(metrics_collector: &MetricsCollector) {
    /// let counters = metrics_collector.get_event_counters().await;
    ///
    /// if let Some(agent_regs) = counters.get("agent_registered") {
    ///     println!("Agents registered: {}", agent_regs);
    /// }
    ///
    /// if let Some(task_comps) = counters.get("tasks_completed") {
    ///     println!("Tasks completed: {}", task_comps);
    /// }
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a HashMap containing all event counters.
    pub async fn get_event_counters(&self) -> HashMap<String, u64> {
        self.event_counters.read().await.clone()
    }

    /// Get specific event counter
    ///
    /// Returns the count for a specific event type.
    ///
    /// ## Performance
    ///
    /// O(1) average case - HashMap lookup.
    ///
    /// # Parameters
    ///
    /// * `event_type` - The event type to query
    ///
    /// # Returns
    ///
    /// Returns the event count, or 0 if the event type doesn't exist.
    pub async fn get_event_count(&self, event_type: &str) -> u64 {
        let counters = self.event_counters.read().await;
        counters.get(event_type).copied().unwrap_or(0)
    }

    /// Increment event counter
    ///
    /// Manually increments an event counter. Useful for custom events
    /// not handled by the standard recording methods.
    ///
    /// ## Performance
    ///
    /// O(1) average case - HashMap insertion/update.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::MetricsCollector;
    /// # async fn example(metrics_collector: &MetricsCollector) {
    /// // Record a custom event
    /// metrics_collector.increment_event_counter("custom_event").await;
    /// # }
    /// ```
    ///
    /// # Parameters
    ///
    /// * `event_type` - The event type to increment
    pub async fn increment_event_counter(&self, event_type: String) {
        let mut counters = self.event_counters.write().await;
        *counters.entry(event_type).or_insert(0) += 1;
    }

    /// Reset event counter
    ///
    /// Resets a specific event counter to zero. Useful for testing
    /// or when starting fresh monitoring periods.
    ///
    /// ## Performance
    ///
    /// O(1) average case - HashMap update.
    ///
    /// # Parameters
    ///
    /// * `event_type` - The event type to reset
    pub async fn reset_event_counter(&self, event_type: &str) {
        let mut counters = self.event_counters.write().await;
        counters.insert(event_type.to_string(), 0);
    }

    /// Reset all event counters
    ///
    /// Resets all event counters to zero. Useful for testing or
    /// when starting fresh monitoring periods.
    ///
    /// ## Performance
    ///
    /// O(1) time complexity - HashMap clear operation.
    ///
    /// ## Warning
    ///
    /// This operation cannot be undone. All counter data will be lost.
    pub async fn reset_all_event_counters(&self) {
        let mut counters = self.event_counters.write().await;
        counters.clear();
    }

    /// Get event statistics
    ///
    /// Returns statistical information about event counters including
    /// totals, averages, and rate calculations.
    ///
    /// ## Statistics Calculated
    ///
    /// - **Total Events**: Sum of all event counters
    /// - **Unique Event Types**: Number of different event types
    /// - **Most Frequent Event**: Event type with highest count
    /// - **Event Rate**: Events per second since system start
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is number of event types.
    /// Involves iterating through all counters.
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::MetricsCollector;
    /// # async fn example(metrics_collector: &MetricsCollector) {
    /// let stats = metrics_collector.get_event_statistics().await;
    ///
    /// if let Some(total) = stats.get("total_events") {
    ///     println!("Total events: {}", total);
    /// }
    ///
    /// if let Some(rate) = stats.get("events_per_second") {
    ///     println!("Event rate: {:.2} events/sec", rate);
    /// }
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a JSON object with event statistics.
    pub async fn get_event_statistics(&self) -> serde_json::Value {
        let counters = self.event_counters.read().await;
        let uptime_seconds = self.start_time.elapsed().as_secs() as f64;

        let total_events: u64 = counters.values().sum();
        let unique_event_types = counters.len();

        let most_frequent =
            counters
                .iter()
                .max_by_key(|(_, &count)| count)
                .map(|(event_type, count)| {
                    serde_json::json!({
                        "event_type": event_type,
                        "count": count
                    })
                });

        let events_per_second = if uptime_seconds > 0.0 {
            total_events as f64 / uptime_seconds
        } else {
            0.0
        };

        serde_json::json!({
            "total_events": total_events,
            "unique_event_types": unique_event_types,
            "most_frequent_event": most_frequent,
            "events_per_second": events_per_second,
            "uptime_seconds": uptime_seconds
        })
    }
}
