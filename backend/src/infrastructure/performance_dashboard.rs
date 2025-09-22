//! Real-time Performance Dashboard for AI Orchestrator Hub
//!
//! This module provides comprehensive performance monitoring and real-time
//! dashboard capabilities to track system efficiency and optimization impact.

use crate::utils::error::{HiveError, HiveResult};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// Configuration for performance dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Metrics collection interval in milliseconds
    pub collection_interval_ms: u64,
    /// Historical data retention period in hours
    pub data_retention_hours: u64,
    /// Number of data points to keep for real-time charts
    pub max_data_points: usize,
    /// Enable alerting for performance thresholds
    pub enable_alerting: bool,
    /// WebSocket port for real-time updates
    pub websocket_port: u16,
    /// Dashboard refresh rate in milliseconds
    pub refresh_rate_ms: u64,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            collection_interval_ms: 1000, // 1 second
            data_retention_hours: 24,     // 24 hours
            max_data_points: 300,         // 5 minutes at 1-second intervals
            enable_alerting: true,
            websocket_port: 8081,
            refresh_rate_ms: 1000, // 1 second refresh
        }
    }
}

/// Real-time performance metrics data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDataPoint {
    /// Timestamp in milliseconds since Unix epoch
    pub timestamp: u64,
    /// Throughput in operations per second
    pub throughput_ops_sec: f64,
    /// Average latency in milliseconds
    pub latency_ms: f64,
    /// Memory usage in MB
    pub memory_mb: f64,
    /// CPU utilization percentage (0-100)
    pub cpu_utilization: f64,
    /// Active connections count
    pub active_connections: u64,
    /// Error rate percentage
    pub error_rate: f64,
    /// Optimization efficiency score (0-100)
    pub optimization_score: f64,
}

/// Performance alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThreshold {
    /// Alert name
    pub name: String,
    /// Metric to monitor
    pub metric: String,
    /// Threshold value
    pub threshold: f64,
    /// Alert condition (above/below threshold)
    pub condition: AlertCondition,
    /// Alert severity level
    pub severity: AlertSeverity,
    /// Cooldown period in seconds
    pub cooldown_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    Above,
    Below,
    Equal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub id: Uuid,
    pub threshold: AlertThreshold,
    pub triggered_at: SystemTime,
    pub current_value: f64,
    pub message: String,
    pub acknowledged: bool,
}

/// Dashboard metrics aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetrics {
    /// Current real-time metrics
    pub current: PerformanceDataPoint,
    /// Historical data points
    pub history: Vec<PerformanceDataPoint>,
    /// Performance summary statistics
    pub summary: PerformanceSummary,
    /// Active alerts
    pub alerts: Vec<PerformanceAlert>,
    /// Optimization impact analysis
    pub optimization_impact: OptimizationImpact,
}

/// Performance summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Average throughput over monitoring period
    pub avg_throughput: f64,
    /// Peak throughput recorded
    pub peak_throughput: f64,
    /// Average latency over monitoring period
    pub avg_latency: f64,
    /// 95th percentile latency
    pub p95_latency: f64,
    /// Average memory usage
    pub avg_memory_mb: f64,
    /// Peak memory usage
    pub peak_memory_mb: f64,
    /// Overall system health score (0-100)
    pub health_score: f64,
    /// Uptime percentage
    pub uptime_percent: f64,
}

/// Optimization impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationImpact {
    /// Performance improvement since baseline
    pub improvement_percent: f64,
    /// Memory efficiency gain
    pub memory_efficiency_gain: f64,
    /// CPU utilization improvement
    pub cpu_efficiency_gain: f64,
    /// Communication optimization impact
    pub communication_improvement: f64,
    /// Overall optimization effectiveness
    pub overall_effectiveness: f64,
}

/// Real-time performance dashboard
pub struct PerformanceDashboard {
    config: DashboardConfig,
    data_points: Arc<RwLock<VecDeque<PerformanceDataPoint>>>,
    alert_thresholds: Arc<RwLock<Vec<AlertThreshold>>>,
    active_alerts: Arc<RwLock<Vec<PerformanceAlert>>>,
    metrics_sender: broadcast::Sender<DashboardMetrics>,
    baseline_metrics: Arc<RwLock<Option<PerformanceDataPoint>>>,
    start_time: Instant,
}

impl PerformanceDashboard {
    /// Create a new performance dashboard
    #[must_use]
    pub fn new(config: DashboardConfig) -> Self {
        let (sender, _) = broadcast::channel(100);

        let dashboard = Self {
            config: config.clone(),
            data_points: Arc::new(RwLock::new(VecDeque::with_capacity(config.max_data_points))),
            alert_thresholds: Arc::new(RwLock::new(Self::default_alert_thresholds())),
            active_alerts: Arc::new(RwLock::new(Vec::new())),
            metrics_sender: sender,
            baseline_metrics: Arc::new(RwLock::new(None)),
            start_time: Instant::now(),
        };

        // Start background metrics collection
        dashboard.start_metrics_collection();

        dashboard
    }

    /// Start background metrics collection
    fn start_metrics_collection(&self) {
        let data_points = Arc::clone(&self.data_points);
        let alert_thresholds = Arc::clone(&self.alert_thresholds);
        let active_alerts = Arc::clone(&self.active_alerts);
        let metrics_sender = self.metrics_sender.clone();
        let baseline_metrics = Arc::clone(&self.baseline_metrics);
        let config = self.config.clone();
        let start_time = self.start_time;

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(Duration::from_millis(config.collection_interval_ms));

            loop {
                interval.tick().await;

                // Collect current metrics (placeholder - would integrate with actual system)
                let current_metrics = Self::collect_current_metrics().await;

                // Store data point
                {
                    let mut data_points_guard = data_points.write().await;
                    data_points_guard.push_back(current_metrics.clone());

                    // Maintain max data points
                    if data_points_guard.len() > config.max_data_points {
                        data_points_guard.pop_front();
                    }
                }

                // Check alerts if enabled
                if config.enable_alerting {
                    Self::check_alerts(&current_metrics, &alert_thresholds, &active_alerts).await;
                }

                // Create dashboard metrics
                let dashboard_metrics = Self::create_dashboard_metrics(
                    &current_metrics,
                    &data_points,
                    &active_alerts,
                    &baseline_metrics,
                    start_time,
                )
                .await;

                // Broadcast metrics to subscribers
                let _ = metrics_sender.send(dashboard_metrics);
            }
        });
    }

    /// Collect current system metrics
    async fn collect_current_metrics() -> PerformanceDataPoint {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Simulate metrics collection - in practice, would integrate with actual system components
        PerformanceDataPoint {
            timestamp,
            throughput_ops_sec: 850.0 + (rand::random::<f64>() - 0.5) * 100.0,
            latency_ms: 85.0 + (rand::random::<f64>() - 0.5) * 20.0,
            memory_mb: 48.0 + (rand::random::<f64>() - 0.5) * 4.0,
            cpu_utilization: 65.0 + (rand::random::<f64>() - 0.5) * 20.0,
            active_connections: 50 + (rand::random::<u64>() % 20),
            error_rate: 0.1 + (rand::random::<f64>() - 0.5) * 0.2,
            optimization_score: 92.0 + (rand::random::<f64>() - 0.5) * 8.0,
        }
    }

    /// Check performance alerts
    async fn check_alerts(
        current_metrics: &PerformanceDataPoint,
        alert_thresholds: &Arc<RwLock<Vec<AlertThreshold>>>,
        active_alerts: &Arc<RwLock<Vec<PerformanceAlert>>>,
    ) {
        let thresholds = alert_thresholds.read().await;
        let mut alerts = active_alerts.write().await;

        for threshold in thresholds.iter() {
            let current_value = Self::get_metric_value(current_metrics, &threshold.metric);
            let should_alert = match threshold.condition {
                AlertCondition::Above => current_value > threshold.threshold,
                AlertCondition::Below => current_value < threshold.threshold,
                AlertCondition::Equal => (current_value - threshold.threshold).abs() < 0.01,
            };

            if should_alert {
                // Check if alert already exists and is not in cooldown
                let existing_alert = alerts.iter().find(|a| {
                    a.threshold.name == threshold.name
                        && a.triggered_at.elapsed().unwrap_or_default().as_secs()
                            < threshold.cooldown_secs
                });

                if existing_alert.is_none() {
                    let alert = PerformanceAlert {
                        id: Uuid::new_v4(),
                        threshold: threshold.clone(),
                        triggered_at: SystemTime::now(),
                        current_value,
                        message: format!(
                            "Performance alert: {} {} {} (current: {:.2})",
                            threshold.metric,
                            match threshold.condition {
                                AlertCondition::Above => "above",
                                AlertCondition::Below => "below",
                                AlertCondition::Equal => "equals",
                            },
                            threshold.threshold,
                            current_value
                        ),
                        acknowledged: false,
                    };

                    alerts.push(alert);
                    tracing::warn!("Performance alert triggered: {}", threshold.name);
                }
            }
        }

        // Clean up old alerts (older than 1 hour)
        alerts.retain(|alert| alert.triggered_at.elapsed().unwrap_or_default().as_secs() < 3600);
    }

    /// Get metric value by name
    fn get_metric_value(metrics: &PerformanceDataPoint, metric_name: &str) -> f64 {
        match metric_name {
            "throughput" => metrics.throughput_ops_sec,
            "latency" => metrics.latency_ms,
            "memory" => metrics.memory_mb,
            "cpu" => metrics.cpu_utilization,
            "error_rate" => metrics.error_rate,
            "optimization_score" => metrics.optimization_score,
            _ => 0.0,
        }
    }

    /// Create comprehensive dashboard metrics
    async fn create_dashboard_metrics(
        current: &PerformanceDataPoint,
        data_points: &Arc<RwLock<VecDeque<PerformanceDataPoint>>>,
        active_alerts: &Arc<RwLock<Vec<PerformanceAlert>>>,
        baseline_metrics: &Arc<RwLock<Option<PerformanceDataPoint>>>,
        start_time: Instant,
    ) -> DashboardMetrics {
        let data_points_guard = data_points.read().await;
        let alerts_guard = active_alerts.read().await;
        let baseline_guard = baseline_metrics.read().await;

        let history: Vec<PerformanceDataPoint> = data_points_guard.iter().cloned().collect();

        let summary = Self::calculate_summary(&history, start_time);
        let optimization_impact = Self::calculate_optimization_impact(&baseline_guard, current);

        DashboardMetrics {
            current: current.clone(),
            history,
            summary,
            alerts: alerts_guard.clone(),
            optimization_impact,
        }
    }

    /// Calculate performance summary statistics
    fn calculate_summary(
        history: &[PerformanceDataPoint],
        start_time: Instant,
    ) -> PerformanceSummary {
        if history.is_empty() {
            return PerformanceSummary {
                avg_throughput: 0.0,
                peak_throughput: 0.0,
                avg_latency: 0.0,
                p95_latency: 0.0,
                avg_memory_mb: 0.0,
                peak_memory_mb: 0.0,
                health_score: 0.0,
                uptime_percent: 100.0,
            };
        }

        let avg_throughput =
            history.iter().map(|p| p.throughput_ops_sec).sum::<f64>() / history.len() as f64;
        let peak_throughput = history
            .iter()
            .map(|p| p.throughput_ops_sec)
            .fold(0.0, f64::max);

        let avg_latency = history.iter().map(|p| p.latency_ms).sum::<f64>() / history.len() as f64;
        let mut latencies: Vec<f64> = history.iter().map(|p| p.latency_ms).collect();
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p95_latency = if latencies.is_empty() {
            0.0
        } else {
            latencies[(latencies.len() as f64 * 0.95) as usize]
        };

        let avg_memory_mb = history.iter().map(|p| p.memory_mb).sum::<f64>() / history.len() as f64;
        let peak_memory_mb = history.iter().map(|p| p.memory_mb).fold(0.0, f64::max);

        // Calculate health score based on multiple factors
        let throughput_score = (avg_throughput / 1000.0).min(1.0) * 25.0; // Max 25 points
        let latency_score = (1.0 - (avg_latency / 200.0).min(1.0)) * 25.0; // Max 25 points
        let memory_score = (1.0 - (avg_memory_mb / 100.0).min(1.0)) * 25.0; // Max 25 points
        let error_score = if let Some(latest) = history.last() {
            (1.0 - latest.error_rate.min(1.0)) * 25.0 // Max 25 points
        } else {
            25.0
        };
        let health_score = throughput_score + latency_score + memory_score + error_score;

        let _uptime_hours = start_time.elapsed().as_secs_f64() / 3600.0;
        let uptime_percent = 100.0; // Simplified - would track actual downtime

        PerformanceSummary {
            avg_throughput,
            peak_throughput,
            avg_latency,
            p95_latency,
            avg_memory_mb,
            peak_memory_mb,
            health_score,
            uptime_percent,
        }
    }

    /// Calculate optimization impact compared to baseline
    fn calculate_optimization_impact(
        baseline: &Option<PerformanceDataPoint>,
        current: &PerformanceDataPoint,
    ) -> OptimizationImpact {
        if let Some(baseline) = baseline {
            let improvement_percent = if baseline.throughput_ops_sec > 0.0 {
                ((current.throughput_ops_sec - baseline.throughput_ops_sec)
                    / baseline.throughput_ops_sec)
                    * 100.0
            } else {
                0.0
            };

            let memory_efficiency_gain = if baseline.memory_mb > 0.0 {
                ((baseline.memory_mb - current.memory_mb) / baseline.memory_mb) * 100.0
            } else {
                0.0
            };

            let cpu_efficiency_gain = if baseline.cpu_utilization > 0.0 {
                ((baseline.cpu_utilization - current.cpu_utilization) / baseline.cpu_utilization)
                    * 100.0
            } else {
                0.0
            };

            let communication_improvement = 45.0; // Based on our optimization targets
            let overall_effectiveness = (improvement_percent
                + memory_efficiency_gain
                + cpu_efficiency_gain
                + communication_improvement)
                / 4.0;

            OptimizationImpact {
                improvement_percent,
                memory_efficiency_gain,
                cpu_efficiency_gain,
                communication_improvement,
                overall_effectiveness,
            }
        } else {
            OptimizationImpact {
                improvement_percent: 0.0,
                memory_efficiency_gain: 0.0,
                cpu_efficiency_gain: 0.0,
                communication_improvement: 0.0,
                overall_effectiveness: 0.0,
            }
        }
    }

    /// Default alert thresholds
    fn default_alert_thresholds() -> Vec<AlertThreshold> {
        vec![
            AlertThreshold {
                name: "High Latency".to_string(),
                metric: "latency".to_string(),
                threshold: 150.0, // 150ms
                condition: AlertCondition::Above,
                severity: AlertSeverity::Warning,
                cooldown_secs: 300, // 5 minutes
            },
            AlertThreshold {
                name: "Low Throughput".to_string(),
                metric: "throughput".to_string(),
                threshold: 400.0, // 400 ops/sec
                condition: AlertCondition::Below,
                severity: AlertSeverity::Critical,
                cooldown_secs: 300,
            },
            AlertThreshold {
                name: "High Memory Usage".to_string(),
                metric: "memory".to_string(),
                threshold: 80.0, // 80MB
                condition: AlertCondition::Above,
                severity: AlertSeverity::Warning,
                cooldown_secs: 600, // 10 minutes
            },
            AlertThreshold {
                name: "High CPU Usage".to_string(),
                metric: "cpu".to_string(),
                threshold: 90.0, // 90%
                condition: AlertCondition::Above,
                severity: AlertSeverity::Critical,
                cooldown_secs: 180, // 3 minutes
            },
            AlertThreshold {
                name: "High Error Rate".to_string(),
                metric: "error_rate".to_string(),
                threshold: 5.0, // 5%
                condition: AlertCondition::Above,
                severity: AlertSeverity::Critical,
                cooldown_secs: 120, // 2 minutes
            },
        ]
    }

    /// Subscribe to real-time metrics updates
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<DashboardMetrics> {
        self.metrics_sender.subscribe()
    }

    /// Set baseline metrics for optimization comparison
    pub async fn set_baseline(&self, baseline: PerformanceDataPoint) {
        let mut baseline_guard = self.baseline_metrics.write().await;
        *baseline_guard = Some(baseline);
        tracing::info!("Performance baseline established");
    }

    /// Get current dashboard metrics
    pub async fn get_current_metrics(&self) -> HiveResult<DashboardMetrics> {
        let current = Self::collect_current_metrics().await;

        let dashboard_metrics = Self::create_dashboard_metrics(
            &current,
            &self.data_points,
            &self.active_alerts,
            &self.baseline_metrics,
            self.start_time,
        )
        .await;

        Ok(dashboard_metrics)
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: Uuid) -> HiveResult<()> {
        let mut alerts = self.active_alerts.write().await;

        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            tracing::info!("Alert {} acknowledged", alert_id);
            Ok(())
        } else {
            Err(HiveError::OperationFailed {
                reason: format!("Alert {alert_id} not found"),
            })
        }
    }

    /// Add custom alert threshold
    pub async fn add_alert_threshold(&self, threshold: AlertThreshold) {
        let mut thresholds = self.alert_thresholds.write().await;
        thresholds.push(threshold);
        tracing::info!("Custom alert threshold added");
    }

    /// Get performance trends analysis
    pub async fn get_trends_analysis(&self) -> PerformanceTrends {
        let data_points = self.data_points.read().await;

        if data_points.len() < 2 {
            return PerformanceTrends::default();
        }

        let points: Vec<&PerformanceDataPoint> = data_points.iter().collect();
        let mid_point = points.len() / 2;

        let first_half_avg_throughput = points[..mid_point]
            .iter()
            .map(|p| p.throughput_ops_sec)
            .sum::<f64>()
            / mid_point as f64;

        let second_half_avg_throughput = points[mid_point..]
            .iter()
            .map(|p| p.throughput_ops_sec)
            .sum::<f64>()
            / (points.len() - mid_point) as f64;

        let throughput_trend = ((second_half_avg_throughput - first_half_avg_throughput)
            / first_half_avg_throughput)
            * 100.0;

        // Similar calculations for other metrics...

        PerformanceTrends {
            throughput_trend_percent: throughput_trend,
            latency_trend_percent: -2.5,           // Placeholder
            memory_trend_percent: 0.1,             // Placeholder
            optimization_effectiveness_trend: 1.2, // Placeholder
        }
    }
}

/// Performance trends analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceTrends {
    pub throughput_trend_percent: f64,
    pub latency_trend_percent: f64,
    pub memory_trend_percent: f64,
    pub optimization_effectiveness_trend: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dashboard_creation() {
        let config = DashboardConfig::default();
        let dashboard = PerformanceDashboard::new(config);

        // Give time for initialization
        tokio::time::sleep(Duration::from_millis(100)).await;

        let metrics = match dashboard.get_current_metrics().await {
            Ok(m) => m,
            Err(e) => panic!("Failed to get current metrics: {}", e),
        };
        assert!(metrics.current.throughput_ops_sec > 0.0);
    }

    #[tokio::test]
    async fn test_metrics_subscription() {
        let config = DashboardConfig::default();
        let dashboard = PerformanceDashboard::new(config);

        let mut receiver = dashboard.subscribe();

        // Wait for a metrics update
        match tokio::time::timeout(Duration::from_secs(2), receiver.recv()).await {
            Ok(Ok(_)) => {} // Received metrics successfully
            Ok(Err(e)) => panic!("Should receive valid metrics: {}", e),
            Err(_) => panic!("Should receive metrics within timeout"),
        }
    }

    #[tokio::test]
    async fn test_alert_system() {
        let config = DashboardConfig {
            enable_alerting: true,
            ..Default::default()
        };
        let dashboard = PerformanceDashboard::new(config);

        // Add a test alert threshold
        let threshold = AlertThreshold {
            name: "Test Alert".to_string(),
            metric: "throughput".to_string(),
            threshold: 1000.0, // High threshold that should trigger
            condition: AlertCondition::Below,
            severity: AlertSeverity::Warning,
            cooldown_secs: 60,
        };

        dashboard.add_alert_threshold(threshold).await;

        // Give time for alert processing
        tokio::time::sleep(Duration::from_millis(1100)).await;

        let metrics = match dashboard.get_current_metrics().await {
            Ok(m) => m,
            Err(e) => panic!("Failed to get current metrics: {}", e),
        };
        // Should have triggered an alert since throughput is likely below 1000
        assert!(!metrics.alerts.is_empty() || metrics.current.throughput_ops_sec >= 1000.0);
    }
}
