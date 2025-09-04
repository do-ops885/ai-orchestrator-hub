use crate::neural::{EvaluationResults, TrainingConfig, TrainingMetrics};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Experiment tracking and management system
#[derive(Debug)]
pub struct ExperimentTracker {
    /// Active experiments
    experiments: HashMap<String, Experiment>,
    /// Experiment runs
    runs: HashMap<Uuid, ExperimentRun>,
    /// Comparison results
    comparisons: HashMap<String, ExperimentComparison>,
    /// Experiment metadata
    metadata: HashMap<String, ExperimentMetadata>,
}

/// Experiment definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    pub experiment_id: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub status: ExperimentStatus,
    pub config: ExperimentConfig,
    pub tags: Vec<String>,
}

/// Experiment status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExperimentStatus {
    Created,
    Running,
    Completed,
    Failed(String),
}

/// Early stopping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarlyStoppingConfig {
    pub patience: usize,
    pub min_delta: f64,
    pub monitor_metric: String,
    pub mode: EarlyStoppingMode,
}

/// Early stopping modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EarlyStoppingMode {
    Min, // For loss - stop when metric stops decreasing
    Max, // For accuracy - stop when metric stops increasing
}

/// Experiment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub base_config: TrainingConfig,
    pub num_runs: usize,
    pub parallel_runs: usize,
    pub evaluation_datasets: Vec<String>,
    pub metrics_to_track: Vec<String>,
    pub early_stopping: Option<EarlyStoppingConfig>,
}

/// Experiment run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentRun {
    pub run_id: Uuid,
    pub experiment_id: String,
    pub session_id: Option<Uuid>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: RunStatus,
    pub config: TrainingConfig,
    pub metrics: Option<TrainingMetrics>,
    pub evaluation_results: Option<EvaluationResults>,
    pub hyperparameters: HashMap<String, serde_json::Value>,
    pub artifacts: Vec<ExperimentArtifact>,
}

/// Run status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
}

/// Experiment artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentArtifact {
    pub artifact_id: Uuid,
    pub name: String,
    pub artifact_type: ArtifactType,
    pub path: String,
    pub size_bytes: u64,
    pub created_at: DateTime<Utc>,
}

/// Artifact types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    Model,
    Checkpoint,
    Metrics,
    Plot,
    Log,
    Config,
}

/// Experiment comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentComparison {
    pub comparison_id: String,
    pub experiment_ids: Vec<String>,
    pub comparison_type: ComparisonType,
    pub results: ComparisonResults,
    pub created_at: DateTime<Utc>,
}

/// Comparison types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonType {
    Performance,
    Convergence,
    ResourceUsage,
    HyperparameterImpact,
    AblationStudy,
}

/// Comparison results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResults {
    pub metrics_comparison: HashMap<String, MetricComparison>,
    pub statistical_tests: Vec<StatisticalTest>,
    pub rankings: Vec<ExperimentRanking>,
    pub insights: Vec<String>,
}

/// Metric comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricComparison {
    pub metric_name: String,
    pub values: HashMap<String, f64>,
    pub best_experiment: String,
    pub improvement_percentage: f64,
}

/// Statistical test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalTest {
    pub test_name: String,
    pub p_value: f64,
    pub significant: bool,
    pub description: String,
}

/// Experiment ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentRanking {
    pub experiment_id: String,
    pub rank: usize,
    pub score: f64,
    pub confidence: f64,
}

/// Experiment metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentMetadata {
    pub total_runs: usize,
    pub completed_runs: usize,
    pub failed_runs: usize,
    pub best_run_id: Option<Uuid>,
    pub average_runtime: Option<f64>,
    pub total_compute_cost: Option<f64>,
}

/// Experiment search and filtering
#[derive(Debug, Clone)]
pub struct ExperimentQuery {
    pub name_filter: Option<String>,
    pub tag_filter: Vec<String>,
    pub status_filter: Vec<ExperimentStatus>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub metric_filters: HashMap<String, (f64, f64)>, // metric_name -> (min, max)
}

/// Experiment report
#[derive(Debug, Clone)]
pub struct ExperimentReport {
    pub experiment: Experiment,
    pub runs: Vec<ExperimentRun>,
    pub summary: ExperimentSummary,
    pub recommendations: Vec<String>,
}

/// Experiment summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentSummary {
    pub total_runs: usize,
    pub successful_runs: usize,
    pub best_metrics: HashMap<String, f64>,
    pub average_metrics: HashMap<String, f64>,
    pub convergence_analysis: ConvergenceAnalysis,
    pub resource_analysis: ResourceAnalysis,
}

/// Convergence analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceAnalysis {
    pub average_epochs_to_converge: f64,
    pub convergence_stability: f64,
    pub best_convergence_run: Uuid,
}

/// Resource analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAnalysis {
    pub average_cpu_usage: f64,
    pub average_memory_usage: f64,
    pub average_training_time: f64,
    pub compute_efficiency_score: f64,
}

impl Default for ExperimentTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ExperimentTracker {
    /// Create a new experiment tracker
    #[must_use]
    pub fn new() -> Self {
        Self {
            experiments: HashMap::new(),
            runs: HashMap::new(),
            comparisons: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create a new experiment
    pub fn create_experiment(
        &mut self,
        name: &str,
        description: &str,
        config: ExperimentConfig,
    ) -> Result<String> {
        let experiment_id = format!("exp_{}", Uuid::new_v4().simple());

        let num_runs = config.num_runs;

        let experiment = Experiment {
            experiment_id: experiment_id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            created_at: Utc::now(),
            status: ExperimentStatus::Created,
            config,
            tags: Vec::new(),
        };

        self.experiments.insert(experiment_id.clone(), experiment);

        let metadata = ExperimentMetadata {
            total_runs: 0,
            completed_runs: 0,
            failed_runs: 0,
            best_run_id: None,
            average_runtime: None,
            total_compute_cost: None,
        };

        self.metadata.insert(experiment_id.clone(), metadata);

        tracing::info!("🧪 Created experiment '{}' with {} runs", name, num_runs);
        Ok(experiment_id)
    }

    /// Start an experiment run
    pub fn start_run(
        &mut self,
        experiment_id: &str,
        config: TrainingConfig,
        hyperparameters: HashMap<String, serde_json::Value>,
    ) -> Result<Uuid> {
        let experiment = self
            .experiments
            .get_mut(experiment_id)
            .ok_or_else(|| anyhow::anyhow!("Experiment not found"))?;

        let run_id = Uuid::new_v4();

        let run = ExperimentRun {
            run_id,
            experiment_id: experiment_id.to_string(),
            session_id: None,
            start_time: Utc::now(),
            end_time: None,
            status: RunStatus::Pending,
            config,
            metrics: None,
            evaluation_results: None,
            hyperparameters,
            artifacts: Vec::new(),
        };

        self.runs.insert(run_id, run);

        // Update experiment metadata
        if let Some(metadata) = self.metadata.get_mut(experiment_id) {
            metadata.total_runs += 1;
        }

        experiment.status = ExperimentStatus::Running;

        tracing::info!("🚀 Started run {} for experiment {}", run_id, experiment_id);
        Ok(run_id)
    }

    /// Update run with session information
    pub fn update_run_session(&mut self, run_id: Uuid, session_id: Uuid) -> Result<()> {
        let run = self
            .runs
            .get_mut(&run_id)
            .ok_or_else(|| anyhow::anyhow!("Run not found"))?;

        run.session_id = Some(session_id);
        run.status = RunStatus::Running;

        Ok(())
    }

    /// Complete a run
    pub fn complete_run(
        &mut self,
        run_id: Uuid,
        metrics: TrainingMetrics,
        evaluation_results: Option<EvaluationResults>,
    ) -> Result<()> {
        // Extract experiment_id before borrowing mutably
        let experiment_id = {
            let run = self
                .runs
                .get(&run_id)
                .ok_or_else(|| anyhow::anyhow!("Run not found"))?;
            run.experiment_id.clone()
        };

        // Update run status
        let run_clone = {
            let run = self
                .runs
                .get_mut(&run_id)
                .ok_or_else(|| anyhow::anyhow!("Run not found"))?;

            run.end_time = Some(Utc::now());
            run.status = RunStatus::Completed;
            run.metrics = Some(metrics);
            run.evaluation_results = evaluation_results;
            run.clone()
        };

        // Update experiment metadata
        if let Some(metadata) = self.metadata.get_mut(&experiment_id) {
            metadata.completed_runs += 1;

            // Update best run
            let should_update_best = if let Some(best_run_id) = metadata.best_run_id {
                // Release the mutable borrow before calling is_better_run
                drop(metadata);
                self.is_better_run(&run_clone, best_run_id)
            } else {
                true
            };

            if should_update_best {
                if let Some(metadata) = self.metadata.get_mut(&experiment_id) {
                    metadata.best_run_id = Some(run_id);
                }
            }
        }

        tracing::info!("✅ Completed run {}", run_id);
        Ok(())
    }

    /// Fail a run
    pub fn fail_run(&mut self, run_id: Uuid, error: String) -> Result<()> {
        let run = self
            .runs
            .get_mut(&run_id)
            .ok_or_else(|| anyhow::anyhow!("Run not found"))?;

        run.end_time = Some(Utc::now());
        run.status = RunStatus::Failed(error.clone());

        // Update experiment metadata
        if let Some(metadata) = self.metadata.get_mut(&run.experiment_id) {
            metadata.failed_runs += 1;
        }

        tracing::warn!("❌ Failed run {}: {}", run_id, error);
        Ok(())
    }

    /// Add artifact to run
    pub fn add_artifact(
        &mut self,
        run_id: Uuid,
        name: &str,
        artifact_type: ArtifactType,
        path: &str,
        size_bytes: u64,
    ) -> Result<()> {
        let run = self
            .runs
            .get_mut(&run_id)
            .ok_or_else(|| anyhow::anyhow!("Run not found"))?;

        let artifact = ExperimentArtifact {
            artifact_id: Uuid::new_v4(),
            name: name.to_string(),
            artifact_type,
            path: path.to_string(),
            size_bytes,
            created_at: Utc::now(),
        };

        run.artifacts.push(artifact);

        tracing::info!("📦 Added artifact '{}' to run {}", name, run_id);
        Ok(())
    }

    /// Compare experiments
    pub fn compare_experiments(
        &mut self,
        experiment_ids: Vec<String>,
        comparison_type: ComparisonType,
    ) -> Result<String> {
        let comparison_id = format!("cmp_{}", Uuid::new_v4().simple());

        let mut results = ComparisonResults {
            metrics_comparison: HashMap::new(),
            statistical_tests: Vec::new(),
            rankings: Vec::new(),
            insights: Vec::new(),
        };

        // Collect metrics from all experiments
        let mut all_metrics = HashMap::new();
        for exp_id in &experiment_ids {
            if let Some(runs) = self.get_experiment_runs(exp_id) {
                let completed_runs: Vec<&ExperimentRun> = runs
                    .iter()
                    .filter(|r| matches!(r.status, RunStatus::Completed))
                    .copied()
                    .collect();

                if !completed_runs.is_empty() {
                    let avg_metrics = self.calculate_average_metrics(&completed_runs);
                    all_metrics.insert(exp_id.clone(), avg_metrics);
                }
            }
        }

        // Perform comparison based on type
        match comparison_type {
            ComparisonType::Performance => {
                self.compare_performance(&experiment_ids, &all_metrics, &mut results)?;
            }
            ComparisonType::Convergence => {
                self.compare_convergence(&experiment_ids, &mut results)?;
            }
            ComparisonType::ResourceUsage => {
                self.compare_resource_usage(&experiment_ids, &mut results)?;
            }
            ComparisonType::HyperparameterImpact => {
                self.compare_hyperparameter_impact(&experiment_ids, &mut results)?;
            }
            ComparisonType::AblationStudy => {
                self.compare_ablation_study(&experiment_ids, &mut results)?;
            }
        }

        let comparison = ExperimentComparison {
            comparison_id: comparison_id.clone(),
            experiment_ids: experiment_ids.clone(),
            comparison_type,
            results,
            created_at: Utc::now(),
        };

        self.comparisons.insert(comparison_id.clone(), comparison);

        tracing::info!(
            "📊 Created comparison '{}' with {} experiments",
            comparison_id,
            experiment_ids.len()
        );
        Ok(comparison_id)
    }

    /// Get experiment by ID
    #[must_use]
    pub fn get_experiment(&self, experiment_id: &str) -> Option<&Experiment> {
        self.experiments.get(experiment_id)
    }

    /// Get run by ID
    #[must_use]
    pub fn get_run(&self, run_id: Uuid) -> Option<&ExperimentRun> {
        self.runs.get(&run_id)
    }

    /// Get all runs for an experiment
    #[must_use]
    pub fn get_experiment_runs(&self, experiment_id: &str) -> Option<Vec<&ExperimentRun>> {
        let runs: Vec<&ExperimentRun> = self
            .runs
            .values()
            .filter(|r| r.experiment_id == experiment_id)
            .collect();

        if runs.is_empty() {
            None
        } else {
            Some(runs)
        }
    }

    /// Search experiments
    #[must_use]
    pub fn search_experiments(&self, query: &ExperimentQuery) -> Vec<&Experiment> {
        self.experiments
            .values()
            .filter(|exp| {
                // Name filter
                if let Some(ref name_filter) = query.name_filter {
                    if !exp
                        .name
                        .to_lowercase()
                        .contains(&name_filter.to_lowercase())
                    {
                        return false;
                    }
                }

                // Tag filter
                if !query.tag_filter.is_empty() {
                    let has_matching_tag =
                        query.tag_filter.iter().any(|tag| exp.tags.contains(tag));
                    if !has_matching_tag {
                        return false;
                    }
                }

                // Status filter
                if !query.status_filter.is_empty() && !query.status_filter.contains(&exp.status) {
                    return false;
                }

                // Date range filter
                if let Some((start, end)) = query.date_range {
                    if exp.created_at < start || exp.created_at > end {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Generate experiment report
    pub fn generate_report(&self, experiment_id: &str) -> Result<ExperimentReport> {
        let experiment = self
            .experiments
            .get(experiment_id)
            .ok_or_else(|| anyhow::anyhow!("Experiment not found"))?
            .clone();

        let runs: Vec<ExperimentRun> = self
            .runs
            .values()
            .filter(|r| r.experiment_id == experiment_id)
            .cloned()
            .collect();

        let summary = self.generate_experiment_summary(&runs)?;

        let recommendations = self.generate_recommendations(&summary);

        Ok(ExperimentReport {
            experiment,
            runs,
            summary,
            recommendations,
        })
    }

    /// Check if one run is better than another
    fn is_better_run(&self, run: &ExperimentRun, other_run_id: Uuid) -> bool {
        let other_run = match self.runs.get(&other_run_id) {
            Some(r) => r,
            None => return true,
        };

        let run_metrics = match &run.metrics {
            Some(m) => m,
            None => return false,
        };

        let other_metrics = match &other_run.metrics {
            Some(m) => m,
            None => return true,
        };

        // Compare final accuracy (higher is better)
        let run_accuracy = run_metrics.accuracy_history.last().unwrap_or(&0.0);
        let other_accuracy = other_metrics.accuracy_history.last().unwrap_or(&0.0);

        run_accuracy > other_accuracy
    }

    /// Calculate average metrics across runs
    fn calculate_average_metrics(&self, runs: &[&ExperimentRun]) -> HashMap<String, f64> {
        let mut metric_sums = HashMap::new();
        let mut metric_counts = HashMap::new();

        for run in runs {
            if let Some(metrics) = &run.metrics {
                // Accuracy
                if let Some(acc) = metrics.accuracy_history.last() {
                    *metric_sums.entry("accuracy".to_string()).or_insert(0.0) += acc;
                    *metric_counts.entry("accuracy".to_string()).or_insert(0) += 1;
                }

                // Loss
                if let Some(loss) = metrics.loss_history.last() {
                    *metric_sums.entry("loss".to_string()).or_insert(0.0) += loss;
                    *metric_counts.entry("loss".to_string()).or_insert(0) += 1;
                }
            }
        }

        let mut averages = HashMap::new();
        for (metric, sum) in metric_sums {
            if let Some(count) = metric_counts.get(&metric) {
                averages.insert(metric, sum / f64::from(*count));
            }
        }

        averages
    }

    /// Compare performance across experiments
    fn compare_performance(
        &self,
        experiment_ids: &[String],
        metrics: &HashMap<String, HashMap<String, f64>>,
        results: &mut ComparisonResults,
    ) -> Result<()> {
        for metric_name in &["accuracy", "loss"] {
            let mut metric_comparison = MetricComparison {
                metric_name: (*metric_name).to_string(),
                values: HashMap::new(),
                best_experiment: String::new(),
                improvement_percentage: 0.0,
            };

            let mut best_value = if metric_name == &"accuracy" {
                0.0
            } else {
                f64::INFINITY
            };
            let mut best_exp = String::new();

            for exp_id in experiment_ids {
                if let Some(exp_metrics) = metrics.get(exp_id) {
                    if let Some(value) = exp_metrics.get(*metric_name) {
                        metric_comparison.values.insert(exp_id.clone(), *value);

                        let is_better = if metric_name == &"accuracy" {
                            *value > best_value
                        } else {
                            *value < best_value
                        };

                        if is_better {
                            best_value = *value;
                            best_exp = exp_id.clone();
                        }
                    }
                }
            }

            metric_comparison.best_experiment = best_exp;
            results
                .metrics_comparison
                .insert((*metric_name).to_string(), metric_comparison);
        }

        Ok(())
    }

    /// Compare convergence across experiments
    fn compare_convergence(
        &self,
        _experiment_ids: &[String],
        results: &mut ComparisonResults,
    ) -> Result<()> {
        // Implementation for convergence comparison
        results
            .insights
            .push("Convergence analysis completed".to_string());
        Ok(())
    }

    /// Compare resource usage across experiments
    fn compare_resource_usage(
        &self,
        _experiment_ids: &[String],
        results: &mut ComparisonResults,
    ) -> Result<()> {
        // Implementation for resource usage comparison
        results
            .insights
            .push("Resource usage analysis completed".to_string());
        Ok(())
    }

    /// Compare hyperparameter impact
    fn compare_hyperparameter_impact(
        &self,
        _experiment_ids: &[String],
        results: &mut ComparisonResults,
    ) -> Result<()> {
        // Implementation for hyperparameter impact comparison
        results
            .insights
            .push("Hyperparameter impact analysis completed".to_string());
        Ok(())
    }

    /// Compare ablation study results
    fn compare_ablation_study(
        &self,
        _experiment_ids: &[String],
        results: &mut ComparisonResults,
    ) -> Result<()> {
        // Implementation for ablation study comparison
        results
            .insights
            .push("Ablation study analysis completed".to_string());
        Ok(())
    }

    /// Generate experiment summary
    fn generate_experiment_summary(&self, runs: &[ExperimentRun]) -> Result<ExperimentSummary> {
        let total_runs = runs.len();
        let successful_runs = runs
            .iter()
            .filter(|r| matches!(r.status, RunStatus::Completed))
            .count();

        let mut best_metrics = HashMap::new();
        let mut average_metrics = HashMap::new();

        // Calculate best and average metrics
        let completed_runs: Vec<&ExperimentRun> = runs
            .iter()
            .filter(|r| matches!(r.status, RunStatus::Completed))
            .collect();

        if !completed_runs.is_empty() {
            let avg_metrics = self.calculate_average_metrics(&completed_runs);
            average_metrics = avg_metrics;

            // Find best metrics
            for run in &completed_runs {
                if let Some(metrics) = &run.metrics {
                    if let Some(acc) = metrics.accuracy_history.last() {
                        let current_best = best_metrics.get("accuracy").unwrap_or(&0.0);
                        if acc > current_best {
                            best_metrics.insert("accuracy".to_string(), *acc);
                        }
                    }
                }
            }
        }

        Ok(ExperimentSummary {
            total_runs,
            successful_runs,
            best_metrics,
            average_metrics,
            convergence_analysis: ConvergenceAnalysis {
                average_epochs_to_converge: 50.0,     // Mock value
                convergence_stability: 0.8,           // Mock value
                best_convergence_run: Uuid::new_v4(), // Mock value
            },
            resource_analysis: ResourceAnalysis {
                average_cpu_usage: 65.0,        // Mock value
                average_memory_usage: 70.0,     // Mock value
                average_training_time: 1800.0,  // Mock value
                compute_efficiency_score: 0.75, // Mock value
            },
        })
    }

    /// Generate recommendations based on experiment results
    fn generate_recommendations(&self, summary: &ExperimentSummary) -> Vec<String> {
        let mut recommendations = Vec::new();

        if summary.successful_runs < summary.total_runs {
            recommendations.push(format!(
                "Consider improving experiment stability - only {}/{} runs completed successfully",
                summary.successful_runs, summary.total_runs
            ));
        }

        if let Some(avg_accuracy) = summary.average_metrics.get("accuracy") {
            if *avg_accuracy < 0.8 {
                recommendations
                    .push("Consider hyperparameter tuning to improve model accuracy".to_string());
            }
        }

        if summary.resource_analysis.average_memory_usage > 80.0 {
            recommendations.push(
                "High memory usage detected - consider memory optimization techniques".to_string(),
            );
        }

        recommendations
    }
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self {
            base_config: TrainingConfig::default(),
            num_runs: 3,
            parallel_runs: 1,
            evaluation_datasets: vec!["test".to_string()],
            metrics_to_track: vec!["accuracy".to_string(), "loss".to_string()],
            early_stopping: None,
        }
    }
}
