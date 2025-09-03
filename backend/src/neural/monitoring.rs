use crate::neural::{TrainingMetrics, TrainingSession, DataBatch, Dataset};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};

/// Comprehensive monitoring and evaluation system for neural training
#[derive(Debug)]
pub struct TrainingMonitor {
    /// Training sessions being monitored
    sessions: HashMap<uuid::Uuid, SessionMonitor>,
    /// Performance benchmarks
    benchmarks: HashMap<String, BenchmarkResult>,
    /// Alert thresholds
    alert_thresholds: AlertThresholds,
    /// Monitoring history
    history: Vec<MonitoringEvent>,
}

/// Session-specific monitor
#[derive(Debug)]
pub struct SessionMonitor {
    pub session_id: uuid::Uuid,
    pub start_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub metrics_history: Vec<MetricsSnapshot>,
    pub alerts: Vec<Alert>,
    pub performance_indicators: PerformanceIndicators,
}

/// Metrics snapshot at a specific time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: DateTime<Utc>,
    pub epoch: usize,
    pub metrics: TrainingMetrics,
    pub system_resources: SystemResources,
}

/// System resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub gpu_usage: Option<f64>,
    pub disk_io: f64,
    pub network_io: f64,
}

/// Performance indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceIndicators {
    pub convergence_rate: f64,
    pub stability_score: f64,
    pub efficiency_score: f64,
    pub resource_efficiency: f64,
}

/// Alert system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub alert_id: uuid::Uuid,
    pub timestamp: DateTime<Utc>,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub resolved: bool,
}

/// Alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    PerformanceDegradation,
    MemoryLeak,
    ConvergenceStall,
    ResourceExhaustion,
    TrainingDivergence,
    HardwareFailure,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Alert thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub max_memory_usage: f64,
    pub max_cpu_usage: f64,
    pub min_convergence_rate: f64,
    pub max_loss_increase: f64,
    pub max_training_time: Duration,
}

/// Benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub benchmark_id: String,
    pub timestamp: DateTime<Utc>,
    pub operation: String,
    pub duration: Duration,
    pub throughput: f64,
    pub latency: Duration,
    pub resource_usage: SystemResources,
}

/// Monitoring event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringEvent {
    pub event_id: uuid::Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: MonitoringEventType,
    pub session_id: Option<uuid::Uuid>,
    pub details: serde_json::Value,
}

/// Monitoring event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringEventType {
    SessionStarted,
    SessionCompleted,
    EpochCompleted,
    AlertTriggered,
    BenchmarkCompleted,
    ResourceWarning,
}

/// Model evaluation system
#[derive(Debug)]
pub struct ModelEvaluator {
    /// Evaluation metrics
    metrics: HashMap<String, EvaluationMetric>,
    /// Confusion matrices
    confusion_matrices: HashMap<String, ConfusionMatrix>,
    /// ROC curves
    roc_curves: HashMap<String, ROCCurve>,
    /// Feature importance
    feature_importance: HashMap<String, Vec<FeatureImportance>>,
}

/// Evaluation metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationMetric {
    pub name: String,
    pub value: f64,
    pub confidence_interval: Option<(f64, f64)>,
    pub timestamp: DateTime<Utc>,
}

/// Confusion matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfusionMatrix {
    pub matrix: Vec<Vec<u32>>,
    pub labels: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// ROC curve data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ROCCurve {
    pub fpr: Vec<f64>,
    pub tpr: Vec<f64>,
    pub thresholds: Vec<f64>,
    pub auc: f64,
    pub timestamp: DateTime<Utc>,
}

/// Feature importance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureImportance {
    pub feature_name: String,
    pub importance_score: f64,
    pub rank: usize,
}

/// Visualization system
#[derive(Debug)]
pub struct TrainingVisualizer {
    /// Plot data
    plots: HashMap<String, PlotData>,
    /// Charts
    charts: HashMap<String, ChartData>,
}

/// Plot data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotData {
    pub plot_id: String,
    pub title: String,
    pub x_label: String,
    pub y_label: String,
    pub x_data: Vec<f64>,
    pub y_data: Vec<f64>,
    pub plot_type: PlotType,
}

/// Chart data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub chart_id: String,
    pub title: String,
    pub data: Vec<ChartSeries>,
}

/// Chart series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSeries {
    pub name: String,
    pub data: Vec<f64>,
    pub color: String,
}

/// Plot types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlotType {
    Line,
    Scatter,
    Bar,
    Histogram,
}

impl TrainingMonitor {
    /// Create a new training monitor
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            benchmarks: HashMap::new(),
            alert_thresholds: AlertThresholds::default(),
            history: Vec::new(),
        }
    }

    /// Start monitoring a training session
    pub fn start_monitoring(&mut self, session: &TrainingSession) {
        let monitor = SessionMonitor {
            session_id: session.session_id,
            start_time: session.start_time,
            last_update: Utc::now(),
            metrics_history: Vec::new(),
            alerts: Vec::new(),
            performance_indicators: PerformanceIndicators {
                convergence_rate: 0.0,
                stability_score: 1.0,
                efficiency_score: 1.0,
                resource_efficiency: 1.0,
            },
        };

        self.sessions.insert(session.session_id, monitor);

        self.log_event(MonitoringEventType::SessionStarted, Some(session.session_id), serde_json::json!({
            "config": format!("{:?}", session.config.model_type)
        }));

        tracing::info!("📊 Started monitoring session {}", session.session_id);
    }

    /// Update monitoring with new metrics
    pub async fn update_metrics(&mut self, session_id: uuid::Uuid, metrics: &TrainingMetrics, epoch: usize) -> Result<()> {
        let session_monitor = self.sessions.get_mut(&session_id)
            .ok_or_else(|| anyhow::anyhow!("Session monitor not found"))?;

        let system_resources = self.collect_system_resources().await?;
        let timestamp = Utc::now();

        let snapshot = MetricsSnapshot {
            timestamp,
            epoch,
            metrics: metrics.clone(),
            system_resources,
        };

        session_monitor.metrics_history.push(snapshot);
        session_monitor.last_update = timestamp;

        // Update performance indicators
        self.update_performance_indicators(session_monitor).await?;

        // Check for alerts
        self.check_alerts(session_monitor).await?;

        // Log epoch completion
        self.log_event(MonitoringEventType::EpochCompleted, Some(session_id), serde_json::json!({
            "epoch": epoch,
            "loss": metrics.loss_history.last().unwrap_or(&0.0),
            "accuracy": metrics.accuracy_history.last().unwrap_or(&0.0)
        }));

        Ok(())
    }

    /// Get session monitoring data
    pub fn get_session_monitor(&self, session_id: uuid::Uuid) -> Option<&SessionMonitor> {
        self.sessions.get(&session_id)
    }

    /// Get all active alerts
    pub fn get_active_alerts(&self) -> Vec<&Alert> {
        self.sessions.values()
            .flat_map(|session| session.alerts.iter())
            .filter(|alert| !alert.resolved)
            .collect()
    }

    /// Run performance benchmark
    pub async fn run_benchmark(&mut self, operation: &str) -> Result<String> {
        let benchmark_id = format!("benchmark_{}", uuid::Uuid::new_v4());
        let start_time = Instant::now();

        // Simulate benchmark operation
        tokio::time::sleep(Duration::from_millis(100)).await;

        let duration = start_time.elapsed();
        let resource_usage = self.collect_system_resources().await?;

        let benchmark = BenchmarkResult {
            benchmark_id: benchmark_id.clone(),
            timestamp: Utc::now(),
            operation: operation.to_string(),
            duration,
            throughput: 1000.0, // operations per second
            latency: Duration::from_millis(1),
            resource_usage,
        };

        self.benchmarks.insert(benchmark_id.clone(), benchmark);

        tracing::info!("🏃 Completed benchmark '{}' in {:?}", operation, duration);
        Ok(benchmark_id)
    }

    /// Generate monitoring report
    pub fn generate_report(&self, session_id: uuid::Uuid) -> Result<MonitoringReport> {
        let session_monitor = self.sessions.get(&session_id)
            .ok_or_else(|| anyhow::anyhow!("Session monitor not found"))?;

        let total_epochs = session_monitor.metrics_history.len();
        let avg_loss = session_monitor.metrics_history.iter()
            .map(|snapshot| snapshot.metrics.loss_history.last().unwrap_or(&0.0))
            .sum::<f64>() / total_epochs as f64;

        let avg_accuracy = session_monitor.metrics_history.iter()
            .map(|snapshot| snapshot.metrics.accuracy_history.last().unwrap_or(&0.0))
            .sum::<f64>() / total_epochs as f64;

        let total_training_time = session_monitor.metrics_history.last()
            .map(|last| last.timestamp.signed_duration_since(session_monitor.start_time))
            .unwrap_or(chrono::Duration::zero());

        Ok(MonitoringReport {
            session_id,
            total_epochs,
            avg_loss,
            avg_accuracy,
            total_training_time: total_training_time.to_std().unwrap_or(Duration::from_secs(0)),
            active_alerts: session_monitor.alerts.iter().filter(|a| !a.resolved).count(),
            performance_indicators: session_monitor.performance_indicators.clone(),
        })
    }

    /// Collect system resource usage
    async fn collect_system_resources(&self) -> Result<SystemResources> {
        // In a real implementation, this would collect actual system metrics
        // For now, return mock data
        Ok(SystemResources {
            cpu_usage: 45.0 + rand::random::<f64>() * 20.0,
            memory_usage: 60.0 + rand::random::<f64>() * 15.0,
            gpu_usage: Some(70.0 + rand::random::<f64>() * 20.0),
            disk_io: 25.0 + rand::random::<f64>() * 10.0,
            network_io: 15.0 + rand::random::<f64>() * 5.0,
        })
    }

    /// Update performance indicators
    async fn update_performance_indicators(&self, session_monitor: &mut SessionMonitor) -> Result<()> {
        let history = &session_monitor.metrics_history;

        if history.len() < 2 {
            return Ok(());
        }

        // Calculate convergence rate (improvement per epoch)
        let recent_losses: Vec<f64> = history.iter()
            .rev()
            .take(5)
            .map(|snapshot| snapshot.metrics.loss_history.last().unwrap_or(&0.0))
            .collect();

        let convergence_rate = if recent_losses.len() >= 2 {
            let first = recent_losses.last().unwrap();
            let last = recent_losses.first().unwrap();
            (first - last) / recent_losses.len() as f64
        } else {
            0.0
        };

        // Calculate stability score (inverse of loss variance)
        let loss_variance = if recent_losses.len() > 1 {
            let mean = recent_losses.iter().sum::<f64>() / recent_losses.len() as f64;
            recent_losses.iter()
                .map(|loss| (loss - mean).powi(2))
                .sum::<f64>() / recent_losses.len() as f64
        } else {
            0.0
        };

        let stability_score = if loss_variance > 0.0 {
            1.0 / (1.0 + loss_variance)
        } else {
            1.0
        };

        // Calculate efficiency score (performance per resource usage)
        let avg_cpu = history.iter()
            .map(|snapshot| snapshot.system_resources.cpu_usage)
            .sum::<f64>() / history.len() as f64;

        let avg_accuracy = history.iter()
            .map(|snapshot| snapshot.metrics.accuracy_history.last().unwrap_or(&0.0))
            .sum::<f64>() / history.len() as f64;

        let efficiency_score = if avg_cpu > 0.0 {
            avg_accuracy / avg_cpu
        } else {
            1.0
        };

        // Calculate resource efficiency
        let avg_memory = history.iter()
            .map(|snapshot| snapshot.system_resources.memory_usage)
            .sum::<f64>() / history.len() as f64;

        let resource_efficiency = if avg_memory > 0.0 {
            1.0 / avg_memory
        } else {
            1.0
        };

        session_monitor.performance_indicators = PerformanceIndicators {
            convergence_rate,
            stability_score,
            efficiency_score,
            resource_efficiency,
        };

        Ok(())
    }

    /// Check for alerts
    async fn check_alerts(&mut self, session_monitor: &mut SessionMonitor) -> Result<()> {
        let latest_snapshot = session_monitor.metrics_history.last()
            .ok_or_else(|| anyhow::anyhow!("No metrics snapshots available"))?;

        // Check memory usage
        if latest_snapshot.system_resources.memory_usage > self.alert_thresholds.max_memory_usage {
            self.create_alert(
                session_monitor,
                AlertType::ResourceExhaustion,
                AlertSeverity::High,
                format!("Memory usage exceeded threshold: {:.1}%", latest_snapshot.system_resources.memory_usage),
            ).await?;
        }

        // Check CPU usage
        if latest_snapshot.system_resources.cpu_usage > self.alert_thresholds.max_cpu_usage {
            self.create_alert(
                session_monitor,
                AlertType::ResourceExhaustion,
                AlertSeverity::Medium,
                format!("CPU usage exceeded threshold: {:.1}%", latest_snapshot.system_resources.cpu_usage),
            ).await?;
        }

        // Check convergence
        if session_monitor.performance_indicators.convergence_rate < self.alert_thresholds.min_convergence_rate {
            self.create_alert(
                session_monitor,
                AlertType::ConvergenceStall,
                AlertSeverity::Medium,
                format!("Convergence rate too low: {:.4}", session_monitor.performance_indicators.convergence_rate),
            ).await?;
        }

        // Check for loss increase
        if session_monitor.metrics_history.len() >= 2 {
            let current_loss = latest_snapshot.metrics.loss_history.last().unwrap_or(&0.0);
            let previous_snapshot = &session_monitor.metrics_history[session_monitor.metrics_history.len() - 2];
            let previous_loss = previous_snapshot.metrics.loss_history.last().unwrap_or(&0.0);

            if current_loss > previous_loss * (1.0 + self.alert_thresholds.max_loss_increase) {
                self.create_alert(
                    session_monitor,
                    AlertType::TrainingDivergence,
                    AlertSeverity::High,
                    format!("Loss increased significantly: {:.4} -> {:.4}", previous_loss, current_loss),
                ).await?;
            }
        }

        Ok(())
    }

    /// Create an alert
    async fn create_alert(
        &mut self,
        session_monitor: &mut SessionMonitor,
        alert_type: AlertType,
        severity: AlertSeverity,
        message: String,
    ) -> Result<()> {
        let alert = Alert {
            alert_id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            alert_type,
            severity,
            message,
            resolved: false,
        };

        session_monitor.alerts.push(alert.clone());

        self.log_event(MonitoringEventType::AlertTriggered, Some(session_monitor.session_id), serde_json::json!({
            "alert_type": format!("{:?}", alert_type),
            "severity": format!("{:?}", severity),
            "message": alert.message
        }));

        tracing::warn!("🚨 Alert triggered: {}", alert.message);
        Ok(())
    }

    /// Log monitoring event
    fn log_event(&mut self, event_type: MonitoringEventType, session_id: Option<uuid::Uuid>, details: serde_json::Value) {
        let event = MonitoringEvent {
            event_id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            session_id,
            details,
        };

        self.history.push(event);
    }
}

impl ModelEvaluator {
    /// Create a new model evaluator
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            confusion_matrices: HashMap::new(),
            roc_curves: HashMap::new(),
            feature_importance: HashMap::new(),
        }
    }

    /// Evaluate model on test dataset
    pub async fn evaluate_model(&mut self, model_id: &str, test_data: &Dataset) -> Result<EvaluationResults> {
        tracing::info!("🔍 Evaluating model '{}' on test dataset", model_id);

        // Simulate evaluation (in real implementation, this would run inference)
        let accuracy = 0.85 + rand::random::<f64>() * 0.1;
        let precision = 0.82 + rand::random::<f64>() * 0.1;
        let recall = 0.88 + rand::random::<f64>() * 0.1;
        let f1_score = 2.0 * precision * recall / (precision + recall);

        let metrics = vec![
            EvaluationMetric {
                name: "accuracy".to_string(),
                value: accuracy,
                confidence_interval: Some((accuracy - 0.05, accuracy + 0.05)),
                timestamp: Utc::now(),
            },
            EvaluationMetric {
                name: "precision".to_string(),
                value: precision,
                confidence_interval: Some((precision - 0.05, precision + 0.05)),
                timestamp: Utc::now(),
            },
            EvaluationMetric {
                name: "recall".to_string(),
                value: recall,
                confidence_interval: Some((recall - 0.05, recall + 0.05)),
                timestamp: Utc::now(),
            },
            EvaluationMetric {
                name: "f1_score".to_string(),
                value: f1_score,
                confidence_interval: Some((f1_score - 0.05, f1_score + 0.05)),
                timestamp: Utc::now(),
            },
        ];

        // Store metrics
        for metric in &metrics {
            self.metrics.insert(format!("{}_{}", model_id, metric.name), metric.clone());
        }

        // Generate confusion matrix
        let confusion_matrix = self.generate_confusion_matrix(model_id, test_data).await?;
        self.confusion_matrices.insert(model_id.to_string(), confusion_matrix);

        // Generate ROC curve
        let roc_curve = self.generate_roc_curve(model_id, test_data).await?;
        self.roc_curves.insert(model_id.to_string(), roc_curve);

        // Calculate feature importance
        let feature_importance = self.calculate_feature_importance(model_id, test_data).await?;
        self.feature_importance.insert(model_id.to_string(), feature_importance);

        Ok(EvaluationResults {
            model_id: model_id.to_string(),
            metrics,
            confusion_matrix,
            roc_curve,
            feature_importance,
        })
    }

    /// Generate confusion matrix
    async fn generate_confusion_matrix(&self, model_id: &str, test_data: &Dataset) -> Result<ConfusionMatrix> {
        // Simulate confusion matrix generation
        let num_classes = test_data.metadata.num_classes;
        let mut matrix = vec![vec![0u32; num_classes]; num_classes];

        // Fill with simulated values
        for i in 0..num_classes {
            for j in 0..num_classes {
                matrix[i][j] = if i == j {
                    80 + rand::random::<u32>() % 20 // High diagonal values
                } else {
                    rand::random::<u32>() % 10 // Low off-diagonal values
                };
            }
        }

        Ok(ConfusionMatrix {
            matrix,
            labels: test_data.metadata.class_names.clone(),
            timestamp: Utc::now(),
        })
    }

    /// Generate ROC curve
    async fn generate_roc_curve(&self, model_id: &str, test_data: &Dataset) -> Result<ROCCurve> {
        // Simulate ROC curve generation
        let mut fpr = Vec::new();
        let mut tpr = Vec::new();
        let mut thresholds = Vec::new();

        for i in 0..100 {
            let threshold = i as f64 / 100.0;
            thresholds.push(threshold);
            fpr.push(threshold * 0.3 + rand::random::<f64>() * 0.1);
            tpr.push(threshold * 0.8 + rand::random::<f64>() * 0.1);
        }

        // Calculate AUC (simplified)
        let auc = 0.85 + rand::random::<f64>() * 0.1;

        Ok(ROCCurve {
            fpr,
            tpr,
            thresholds,
            auc,
            timestamp: Utc::now(),
        })
    }

    /// Calculate feature importance
    async fn calculate_feature_importance(&self, model_id: &str, test_data: &Dataset) -> Result<Vec<FeatureImportance>> {
        let mut importance_scores = Vec::new();

        for (i, feature_name) in test_data.metadata.feature_names.iter().enumerate() {
            let importance = rand::random::<f64>() * 0.5 + 0.1; // Random importance score
            importance_scores.push((feature_name.clone(), importance));
        }

        // Sort by importance
        importance_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let feature_importance: Vec<FeatureImportance> = importance_scores.into_iter()
            .enumerate()
            .map(|(rank, (name, score))| FeatureImportance {
                feature_name: name,
                importance_score: score,
                rank: rank + 1,
            })
            .collect();

        Ok(feature_importance)
    }
}

impl TrainingVisualizer {
    /// Create a new training visualizer
    pub fn new() -> Self {
        Self {
            plots: HashMap::new(),
            charts: HashMap::new(),
        }
    }

    /// Create loss curve plot
    pub fn create_loss_curve(&mut self, session_id: uuid::Uuid, metrics: &TrainingMetrics) -> String {
        let plot_id = format!("loss_curve_{}", session_id);

        let plot = PlotData {
            plot_id: plot_id.clone(),
            title: "Training Loss Curve".to_string(),
            x_label: "Epoch".to_string(),
            y_label: "Loss".to_string(),
            x_data: (0..metrics.loss_history.len()).map(|x| x as f64).collect(),
            y_data: metrics.loss_history.clone(),
            plot_type: PlotType::Line,
        };

        self.plots.insert(plot_id.clone(), plot);
        plot_id
    }

    /// Create accuracy curve plot
    pub fn create_accuracy_curve(&mut self, session_id: uuid::Uuid, metrics: &TrainingMetrics) -> String {
        let plot_id = format!("accuracy_curve_{}", session_id);

        let plot = PlotData {
            plot_id: plot_id.clone(),
            title: "Training Accuracy Curve".to_string(),
            x_label: "Epoch".to_string(),
            y_label: "Accuracy".to_string(),
            x_data: (0..metrics.accuracy_history.len()).map(|x| x as f64).collect(),
            y_data: metrics.accuracy_history.clone(),
            plot_type: PlotType::Line,
        };

        self.plots.insert(plot_id.clone(), plot);
        plot_id
    }

    /// Create learning rate schedule plot
    pub fn create_lr_schedule_plot(&mut self, session_id: uuid::Uuid, metrics: &TrainingMetrics) -> String {
        let plot_id = format!("lr_schedule_{}", session_id);

        let plot = PlotData {
            plot_id: plot_id.clone(),
            title: "Learning Rate Schedule".to_string(),
            x_label: "Epoch".to_string(),
            y_label: "Learning Rate".to_string(),
            x_data: (0..metrics.learning_rate_history.len()).map(|x| x as f64).collect(),
            y_data: metrics.learning_rate_history.clone(),
            plot_type: PlotType::Line,
        };

        self.plots.insert(plot_id.clone(), plot);
        plot_id
    }

    /// Create resource usage chart
    pub fn create_resource_chart(&mut self, session_id: uuid::Uuid, snapshots: &[MetricsSnapshot]) -> String {
        let chart_id = format!("resources_{}", session_id);

        let cpu_data: Vec<f64> = snapshots.iter()
            .map(|snapshot| snapshot.system_resources.cpu_usage)
            .collect();

        let memory_data: Vec<f64> = snapshots.iter()
            .map(|snapshot| snapshot.system_resources.memory_usage)
            .collect();

        let chart = ChartData {
            chart_id: chart_id.clone(),
            title: "Resource Usage Over Time".to_string(),
            data: vec![
                ChartSeries {
                    name: "CPU Usage (%)".to_string(),
                    data: cpu_data,
                    color: "#FF6B6B".to_string(),
                },
                ChartSeries {
                    name: "Memory Usage (%)".to_string(),
                    data: memory_data,
                    color: "#4ECDC4".to_string(),
                },
            ],
        };

        self.charts.insert(chart_id.clone(), chart);
        chart_id
    }

    /// Get plot data
    pub fn get_plot(&self, plot_id: &str) -> Option<&PlotData> {
        self.plots.get(plot_id)
    }

    /// Get chart data
    pub fn get_chart(&self, chart_id: &str) -> Option<&ChartData> {
        self.charts.get(chart_id)
    }
}

/// Evaluation results
#[derive(Debug, Clone)]
pub struct EvaluationResults {
    pub model_id: String,
    pub metrics: Vec<EvaluationMetric>,
    pub confusion_matrix: ConfusionMatrix,
    pub roc_curve: ROCCurve,
    pub feature_importance: Vec<FeatureImportance>,
}

/// Monitoring report
#[derive(Debug, Clone)]
pub struct MonitoringReport {
    pub session_id: uuid::Uuid,
    pub total_epochs: usize,
    pub avg_loss: f64,
    pub avg_accuracy: f64,
    pub total_training_time: Duration,
    pub active_alerts: usize,
    pub performance_indicators: PerformanceIndicators,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_memory_usage: 90.0,
            max_cpu_usage: 95.0,
            min_convergence_rate: -0.001,
            max_loss_increase: 0.1,
            max_training_time: Duration::from_secs(3600), // 1 hour
        }
    }
}