//! # Metrics History Module
//!
//! This module handles historical metrics data tracking and trend analysis.

use super::collector::MetricsCollector;
use super::types::HiveMetrics;

impl MetricsCollector {
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
    ///   "trends": {
    ///     "agent_growth_rate": 0.05,
    ///     "task_success_trend": "improving"
    ///   },
    ///   "event_counters": {
    ///     "agent_registered": 42,
    ///     "tasks_completed": 1000
    ///   },
    ///   "history_size": 150
    /// }
    /// ```
    ///
    /// ## Trend Analysis
    ///
    /// - **Agent Growth Rate**: Rate of agent registration over time
    /// - **Task Success Trend**: Direction of success rate changes
    /// - **Performance Trends**: Historical performance pattern analysis
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is history size.
    /// Suitable for periodic analysis, not real-time operations.
    ///
    /// ## Use Cases
    ///
    /// - Trend monitoring and alerting
    /// - Capacity planning
    /// - Performance analysis
    /// - System health assessment
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// # use hive::core::hive::MetricsCollector;
    /// # async fn example(metrics_collector: &MetricsCollector) {
    /// let enhanced = metrics_collector.get_enhanced_metrics().await;
    ///
    /// if let Some(current) = enhanced.get("current") {
    ///     println!("Current metrics: {}", current);
    /// }
    ///
    /// if let Some(trends) = enhanced.get("trends") {
    ///     println!("System trends: {}", trends);
    /// }
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a JSON object with comprehensive metrics and trend analysis.
    pub async fn get_enhanced_metrics(&self) -> serde_json::Value {
        let current_metrics = self.metrics.read().await.clone();
        let history = self.metrics_history.read().await.clone();
        let event_counters = self.event_counters.read().await.clone();

        let trends = self.calculate_trends(&history).await;
        let history_size = history.len();

        serde_json::json!({
            "current": current_metrics,
            "trends": trends,
            "event_counters": event_counters,
            "history_size": history_size
        })
    }

    /// Calculate trends from historical data
    ///
    /// Analyzes historical metrics to identify trends and patterns
    /// for predictive monitoring and capacity planning.
    ///
    /// ## Trend Calculations
    ///
    /// - **Agent Growth Rate**: Linear regression on agent counts
    /// - **Task Success Trend**: Direction of success rate changes
    /// - **Performance Trends**: Average performance over time windows
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is history size.
    /// Uses simple statistical methods for trend detection.
    ///
    /// # Parameters
    ///
    /// * `history` - Historical metrics snapshots
    ///
    /// # Returns
    ///
    /// Returns a JSON object with calculated trends.
    async fn calculate_trends(&self, history: &[HiveMetrics]) -> serde_json::Value {
        if history.is_empty() {
            return serde_json::json!({
                "agent_growth_rate": 0.0,
                "task_success_trend": "insufficient_data",
                "performance_trend": "insufficient_data"
            });
        }

        // Calculate agent growth rate (simple linear trend)
        let agent_growth_rate = if history.len() >= 2 {
            if let (Some(first_metrics), Some(last_metrics)) = (history.first(), history.last()) {
                let first = first_metrics.agent_metrics.total_agents as f64;
                let last = last_metrics.agent_metrics.total_agents as f64;
                let periods = (history.len() - 1) as f64;

                if periods > 0.0 {
                    (last - first) / periods
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Calculate task success trend
        let task_success_trend = self.calculate_success_trend(history);

        // Calculate performance trend
        let performance_trend = self.calculate_performance_trend(history);

        serde_json::json!({
            "agent_growth_rate": agent_growth_rate,
            "task_success_trend": task_success_trend,
            "performance_trend": performance_trend
        })
    }

    /// Calculate task success trend
    ///
    /// Determines if task success rates are improving, declining, or stable
    /// based on historical data analysis.
    ///
    /// ## Trend Determination
    ///
    /// - **"improving"**: Success rate increasing over time
    /// - **"declining"**: Success rate decreasing over time
    /// - **"stable"**: Success rate relatively constant
    /// - **"insufficient_data"**: Not enough data for analysis
    ///
    /// # Parameters
    ///
    /// * `history` - Historical metrics snapshots
    ///
    /// # Returns
    ///
    /// Returns trend description as a string.
    fn calculate_success_trend(&self, history: &[HiveMetrics]) -> String {
        if history.len() < 3 {
            return "insufficient_data".to_string();
        }

        let recent_success_rates: Vec<f64> = history
            .iter()
            .rev()
            .take(10) // Last 10 snapshots
            .map(|m| m.task_metrics.success_rate)
            .collect();

        if recent_success_rates.len() < 2 {
            return "insufficient_data".to_string();
        }

        let first_half: Vec<f64> = recent_success_rates
            .iter()
            .take(recent_success_rates.len() / 2)
            .cloned()
            .collect();
        let second_half: Vec<f64> = recent_success_rates
            .iter()
            .skip(recent_success_rates.len() / 2)
            .cloned()
            .collect();

        let first_avg = first_half.iter().sum::<f64>() / first_half.len() as f64;
        let second_avg = second_half.iter().sum::<f64>() / second_half.len() as f64;

        let threshold = 0.02; // 2% change threshold

        if (second_avg - first_avg) > threshold {
            "improving".to_string()
        } else if (first_avg - second_avg) > threshold {
            "declining".to_string()
        } else {
            "stable".to_string()
        }
    }

    /// Calculate performance trend
    ///
    /// Analyzes overall system performance trends based on
    /// agent performance scores and system metrics.
    ///
    /// ## Performance Indicators
    ///
    /// - Agent performance scores
    /// - Task completion rates
    /// - System resource usage
    /// - Error rates
    ///
    /// # Parameters
    ///
    /// * `history` - Historical metrics snapshots
    ///
    /// # Returns
    ///
    /// Returns performance trend description.
    fn calculate_performance_trend(&self, history: &[HiveMetrics]) -> String {
        if history.len() < 3 {
            return "insufficient_data".to_string();
        }

        // Simple performance score based on multiple factors
        let performance_scores: Vec<f64> = history
            .iter()
            .map(|m| {
                let agent_score = m.agent_metrics.average_agent_performance;
                let task_score = m.task_metrics.success_rate;
                let system_score = 1.0 - (m.system_metrics.error_rate / 100.0); // Convert to 0-1 scale

                (agent_score + task_score + system_score) / 3.0
            })
            .collect();

        if performance_scores.len() < 2 {
            return "insufficient_data".to_string();
        }

        let first_half: Vec<f64> = performance_scores
            .iter()
            .take(performance_scores.len() / 2)
            .cloned()
            .collect();
        let second_half: Vec<f64> = performance_scores
            .iter()
            .skip(performance_scores.len() / 2)
            .cloned()
            .collect();

        let first_avg = first_half.iter().sum::<f64>() / first_half.len() as f64;
        let second_avg = second_half.iter().sum::<f64>() / second_half.len() as f64;

        let threshold = 0.05; // 5% change threshold

        if (second_avg - first_avg) > threshold {
            "improving".to_string()
        } else if (first_avg - second_avg) > threshold {
            "declining".to_string()
        } else {
            "stable".to_string()
        }
    }

    /// Get metrics history
    ///
    /// Returns the complete historical metrics data for detailed analysis.
    /// Useful for external analysis tools and long-term trend monitoring.
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is history size.
    /// Memory overhead from cloning historical data.
    ///
    /// ## Use Cases
    ///
    /// - External analysis and reporting
    /// - Long-term trend analysis
    /// - Data export for ML training
    /// - Historical performance reviews
    ///
    /// # Returns
    ///
    /// Returns a vector of historical `HiveMetrics` snapshots.
    pub async fn get_metrics_history(&self) -> Vec<HiveMetrics> {
        self.metrics_history.read().await.clone()
    }

    /// Clear metrics history
    ///
    /// Removes all historical metrics data. Useful for testing or
    /// when starting fresh after system changes.
    ///
    /// ## Performance
    ///
    /// O(1) time complexity - simple vector clear operation.
    ///
    /// ## Use Cases
    ///
    /// - Testing and development
    /// - System maintenance
    /// - Memory management
    ///
    /// ## Warning
    ///
    /// This operation cannot be undone. Historical data will be lost.
    pub async fn clear_metrics_history(&self) {
        self.metrics_history.write().await.clear();
    }
}
