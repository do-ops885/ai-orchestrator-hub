//! # Task Analytics Module
//!
//! This module provides analytics and reporting functionality
//! for task execution patterns, performance trends, and system insights.

use crate::utils::error::HiveResult;

use super::distributor::TaskDistributor;
use super::types::{TaskExecutionResult, TaskStatus};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

impl TaskDistributor {
    /// Get detailed analytics
    ///
    /// Returns comprehensive analytics about task performance, efficiency,
    /// and system utilization patterns. Includes trend analysis and recommendations.
    ///
    /// ## Analytics Data
    ///
    /// - Task execution patterns and timing
    /// - Success/failure rates and trends
    /// - Agent performance breakdown
    /// - Queue efficiency metrics
    /// - System throughput analysis
    ///
    /// ## Performance
    ///
    /// O(n) time complexity with sorting operations for rankings.
    /// May involve complex calculations for trend analysis.
    ///
    /// ## Use Cases
    ///
    /// - Performance optimization
    /// - System bottleneck identification
    /// - Capacity planning and scaling decisions
    /// - Agent efficiency analysis
    pub async fn get_analytics(&self) -> serde_json::Value {
        let execution_stats = self.get_execution_statistics().await;
        let performance_summary = self.get_system_performance_summary().await;
        let queue_efficiency = self.calculate_queue_efficiency().await;
        let agent_workload = self.get_agent_workload_distribution().await;

        serde_json::json!({
            "execution_statistics": execution_stats,
            "performance_summary": performance_summary,
            "queue_efficiency": queue_efficiency,
            "agent_workload_distribution": agent_workload,
            "system_health": self.assess_system_health().await
        })
    }

    /// Calculate queue efficiency
    ///
    /// Analyzes the efficiency of task queuing and distribution
    /// across both legacy and work-stealing queues.
    ///
    /// ## Efficiency Metrics
    ///
    /// - Queue utilization rates
    /// - Task distribution balance
    /// - Queue processing throughput
    /// - Bottleneck identification
    ///
    /// ## Performance
    ///
    /// O(1) for basic queue size checks.
    /// More complex analysis may involve O(n) operations.
    async fn calculate_queue_efficiency(&self) -> serde_json::Value {
        let legacy_queue_size = self.task_queue.read().await.len();
        let work_stealing_queue_size = self.work_stealing_queue.len().await;
        let total_queued = legacy_queue_size + work_stealing_queue_size;

        // Calculate distribution balance
        let distribution_balance = if total_queued > 0 {
            let legacy_ratio = legacy_queue_size as f64 / total_queued as f64;
            let work_stealing_ratio = work_stealing_queue_size as f64 / total_queued as f64;

            // Ideal balance is when both queues are utilized proportionally
            // Lower imbalance score is better (0.0 = perfect balance, 1.0 = complete imbalance)
            (legacy_ratio - work_stealing_ratio).abs()
        } else {
            0.0
        };

        serde_json::json!({
            "legacy_queue_size": legacy_queue_size,
            "work_stealing_queue_size": work_stealing_queue_size,
            "total_queued_tasks": total_queued,
            "distribution_balance": distribution_balance,
            "efficiency_score": 1.0 - distribution_balance.min(1.0)
        })
    }

    /// Get agent workload distribution
    ///
    /// Analyzes how tasks are distributed across different agents
    /// to identify workload imbalances and optimization opportunities.
    ///
    /// ## Workload Analysis
    ///
    /// - Tasks per agent ratios
    /// - Workload balance scores
    /// - Agent utilization rates
    /// - Bottleneck identification
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is the number of tasks.
    /// Involves analyzing execution history.
    async fn get_agent_workload_distribution(&self) -> serde_json::Value {
        let history = self.execution_history.read().await;
        let mut agent_task_counts = HashMap::new();

        // Count tasks per agent
        for result in history.iter() {
            let agent_id = result.agent_id.to_string();
            *agent_task_counts.entry(agent_id).or_insert(0) += 1;
        }

        if agent_task_counts.is_empty() {
            return serde_json::json!({
                "total_agents": 0,
                "average_tasks_per_agent": 0.0,
                "workload_balance_score": 0.0,
                "agent_distribution": {}
            });
        }

        let total_agents = agent_task_counts.len();
        let total_tasks: usize = agent_task_counts.values().sum();
        let average_tasks_per_agent = total_tasks as f64 / total_agents as f64;

        // Calculate workload balance score (0.0 = perfect balance, 1.0 = complete imbalance)
        let mut balance_score = 0.0;
        for &count in agent_task_counts.values() {
            let deviation =
                (count as f64 - average_tasks_per_agent).abs() / average_tasks_per_agent;
            balance_score += deviation;
        }
        balance_score /= total_agents as f64;

        serde_json::json!({
            "total_agents": total_agents,
            "average_tasks_per_agent": average_tasks_per_agent,
            "workload_balance_score": balance_score,
            "agent_distribution": agent_task_counts
        })
    }

    /// Assess system health
    ///
    /// Provides an overall assessment of system health based on
    /// task execution patterns, queue efficiency, and performance metrics.
    ///
    /// ## Health Indicators
    ///
    /// - Task success rates
    /// - Queue processing efficiency
    /// - Agent workload balance
    /// - System throughput
    /// - Error rates and patterns
    ///
    /// ## Health Levels
    ///
    /// - **"excellent"**: All metrics within optimal ranges
    /// - **"good"**: Minor issues but overall healthy
    /// - **"fair"**: Some performance concerns
    /// - **"poor"**: Significant issues requiring attention
    /// - **"critical"**: System experiencing major problems
    ///
    /// ## Performance
    ///
    /// O(n) time complexity for comprehensive analysis.
    /// Involves multiple metric calculations and aggregations.
    async fn assess_system_health(&self) -> serde_json::Value {
        let execution_stats = self.get_execution_statistics().await;
        let queue_efficiency = self.calculate_queue_efficiency().await;
        let workload_balance = self.get_agent_workload_distribution().await;

        // Extract key metrics
        let success_rate = execution_stats
            .get("success_rate")
            .and_then(|v| v.as_f64())
            .unwrap_or_default();

        let efficiency_score = queue_efficiency
            .get("efficiency_score")
            .and_then(|v| v.as_f64())
            .unwrap_or_default();

        let balance_score = workload_balance
            .get("workload_balance_score")
            .and_then(|v| v.as_f64())
            .unwrap_or_default();

        // Calculate overall health score (0.0 to 1.0, higher is better)
        let health_score = (success_rate + efficiency_score + (1.0 - balance_score)) / 3.0;

        // Determine health level
        let health_level = if health_score >= 0.9 {
            "excellent"
        } else if health_score >= 0.75 {
            "good"
        } else if health_score >= 0.6 {
            "fair"
        } else if health_score >= 0.4 {
            "poor"
        } else {
            "critical"
        };

        // Generate recommendations
        let recommendations = self
            .generate_health_recommendations(success_rate, efficiency_score, balance_score)
            .await;

        serde_json::json!({
            "health_level": health_level,
            "health_score": health_score,
            "success_rate": success_rate,
            "efficiency_score": efficiency_score,
            "workload_balance_score": balance_score,
            "recommendations": recommendations
        })
    }

    /// Generate health recommendations
    ///
    /// Provides specific recommendations for improving system health
    /// based on current performance metrics and identified issues.
    ///
    /// ## Recommendation Types
    ///
    /// - Task success rate improvements
    /// - Queue optimization suggestions
    /// - Workload balancing recommendations
    /// - Agent performance enhancements
    /// - System configuration advice
    ///
    /// ## Performance
    ///
    /// O(1) time complexity - simple threshold-based recommendations.
    async fn generate_health_recommendations(
        &self,
        success_rate: f64,
        efficiency_score: f64,
        balance_score: f64,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if success_rate < 0.8 {
            recommendations.push(
                "Task success rate is below 80%. Consider reviewing agent capabilities and task requirements.".to_string()
            );
        }

        if efficiency_score < 0.7 {
            recommendations.push(
                "Queue efficiency is suboptimal. Consider optimizing work-stealing queue configuration.".to_string()
            );
        }

        if balance_score > 0.3 {
            recommendations.push(
                "Agent workload is imbalanced. Consider redistributing tasks or adding more agents.".to_string()
            );
        }

        if recommendations.is_empty() {
            recommendations.push(
                "System is performing well. Continue monitoring for optimal performance."
                    .to_string(),
            );
        }

        recommendations
    }

    /// Get performance trends
    ///
    /// Analyzes historical data to identify performance trends
    /// and predict future system behavior.
    ///
    /// ## Trend Analysis
    ///
    /// - Success rate trends over time
    /// - Execution time patterns
    /// - Queue size fluctuations
    /// - Agent performance changes
    ///
    /// ## Performance
    ///
    /// O(n) time complexity where n is history size.
    /// Involves time-series analysis of historical data.
    pub async fn get_performance_trends(&self) -> serde_json::Value {
        let history = self.execution_history.read().await;

        if history.len() < 5 {
            return serde_json::json!({
                "data_points": history.len(),
                "trend": "insufficient_data",
                "recommendation": "Need more execution history for trend analysis"
            });
        }

        // Simple trend analysis based on recent executions
        let recent_executions: Vec<&TaskExecutionResult> = history.iter().rev().take(20).collect();

        let recent_success_rate = recent_executions.iter().filter(|r| r.success).count() as f64
            / recent_executions.len() as f64;

        let older_executions: Vec<&TaskExecutionResult> =
            history.iter().rev().skip(20).take(20).collect();

        let older_success_rate = if !older_executions.is_empty() {
            older_executions.iter().filter(|r| r.success).count() as f64
                / older_executions.len() as f64
        } else {
            recent_success_rate
        };

        let trend = if (recent_success_rate - older_success_rate).abs() < 0.05 {
            "stable"
        } else if recent_success_rate > older_success_rate {
            "improving"
        } else {
            "declining"
        };

        serde_json::json!({
            "data_points": history.len(),
            "recent_success_rate": recent_success_rate,
            "older_success_rate": older_success_rate,
            "trend": trend,
            "recommendation": self.generate_trend_recommendation(trend)
        })
    }

    /// Generate trend recommendation
    ///
    /// Provides recommendations based on performance trend analysis.
    ///
    /// # Parameters
    ///
    /// * `trend` - The identified performance trend
    ///
    /// # Returns
    ///
    /// Returns a recommendation string based on the trend.
    fn generate_trend_recommendation(&self, trend: &str) -> String {
        match trend {
            "improving" => {
                "Performance is improving. Continue current optimization efforts.".to_string()
            }
            "declining" => {
                "Performance is declining. Investigate recent changes and system bottlenecks."
                    .to_string()
            }
            "stable" => {
                "Performance is stable. Monitor for opportunities to further optimize.".to_string()
            }
            _ => "Insufficient data for trend analysis. Continue collecting performance metrics."
                .to_string(),
        }
    }
}
