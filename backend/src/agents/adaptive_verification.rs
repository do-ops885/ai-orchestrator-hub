//! # Adaptive Verification Enhancement
//!
//! Enhances the existing Simple Verification System with machine learning-based
//! threshold optimization. Integrates with the existing AdaptiveLearningSystem
//! to automatically adjust verification thresholds based on historical performance.

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

use crate::agents::Agent;
use crate::agents::simple_verification::{
    SimpleVerificationResult, SimpleVerificationStatus, SimpleVerificationSystem,
};
use crate::neural::adaptive_learning::AdaptiveLearningSystem;
use crate::tasks::{Task, TaskResult};

/// Enhanced verification system with adaptive threshold learning
pub struct AdaptiveVerificationSystem {
    base_verification: SimpleVerificationSystem,
    learning_system: Arc<tokio::sync::RwLock<AdaptiveLearningSystem>>,
    threshold_history: Arc<tokio::sync::RwLock<ThresholdHistory>>,
    adaptation_config: AdaptationConfig,
    performance_tracker: Arc<tokio::sync::RwLock<PerformanceTracker>>,
}

/// Configuration for adaptive threshold learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationConfig {
    pub learning_rate: f64,
    pub min_samples_for_adaptation: usize,
    pub adaptation_window_hours: u32,
    pub confidence_threshold_range: (f64, f64), // (min, max)
    pub rule_threshold_range: (f64, f64),
    pub adaptation_frequency_hours: u32,
    pub performance_weight_success: f64,
    pub performance_weight_efficiency: f64,
    pub performance_weight_accuracy: f64,
}

impl Default for AdaptationConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.05,
            min_samples_for_adaptation: 10,
            adaptation_window_hours: 24,
            confidence_threshold_range: (0.5, 0.95),
            rule_threshold_range: (0.3, 0.9),
            adaptation_frequency_hours: 6,
            performance_weight_success: 0.4,
            performance_weight_efficiency: 0.3,
            performance_weight_accuracy: 0.3,
        }
    }
}

/// Historical threshold data for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdHistory {
    pub confidence_threshold_history: Vec<ThresholdEntry>,
    pub rule_threshold_history: HashMap<String, Vec<ThresholdEntry>>,
    pub last_adaptation: DateTime<Utc>,
    pub adaptation_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdEntry {
    pub timestamp: DateTime<Utc>,
    pub threshold_value: f64,
    pub performance_score: f64,
    pub sample_count: usize,
    pub adaptation_reason: String,
}

/// Performance tracking for threshold optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTracker {
    pub verification_outcomes: Vec<VerificationOutcome>,
    pub success_rate_by_threshold: HashMap<String, f64>, // threshold_range -> success_rate
    pub efficiency_metrics: EfficiencyMetrics,
    pub accuracy_metrics: AccuracyMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationOutcome {
    pub timestamp: DateTime<Utc>,
    pub task_id: Uuid,
    pub verification_result: SimpleVerificationResult,
    pub actual_task_success: bool, // Ground truth from task execution
    pub verification_time_ms: u64,
    pub threshold_used: f64,
    pub rule_thresholds_used: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    pub average_verification_time_ms: f64,
    pub verification_time_by_tier: HashMap<String, f64>,
    pub throughput_verifications_per_hour: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyMetrics {
    pub true_positives: u32,  // Correctly identified successful tasks
    pub true_negatives: u32,  // Correctly identified failed tasks
    pub false_positives: u32, // Incorrectly passed failed tasks
    pub false_negatives: u32, // Incorrectly failed successful tasks
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
}

/// Adaptive threshold recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdRecommendation {
    pub confidence_threshold: f64,
    pub rule_threshold_adjustments: HashMap<String, f64>,
    pub expected_performance_improvement: f64,
    pub confidence_in_recommendation: f64,
    pub reasoning: String,
}

impl AdaptiveVerificationSystem {
    /// Create a new adaptive verification system
    pub async fn new(
        base_verification: SimpleVerificationSystem,
        learning_system: Arc<tokio::sync::RwLock<AdaptiveLearningSystem>>,
        config: AdaptationConfig,
    ) -> Self {
        Self {
            base_verification,
            learning_system,
            threshold_history: Arc::new(tokio::sync::RwLock::new(ThresholdHistory::new())),
            adaptation_config: config,
            performance_tracker: Arc::new(tokio::sync::RwLock::new(PerformanceTracker::new())),
        }
    }

    /// Enhanced verification with adaptive learning
    pub async fn adaptive_verify_task_result(
        &mut self,
        task: &Task,
        result: &TaskResult,
        original_goal: Option<&str>,
        agent: &Agent,
    ) -> Result<SimpleVerificationResult> {
        // Get current optimal thresholds
        let recommendation = self.get_current_threshold_recommendation().await?;

        // Apply adaptive thresholds to base verification system
        self.apply_threshold_recommendation(&recommendation).await;

        // Perform verification with adaptive thresholds
        let verification_result = self
            .base_verification
            .verify_task_result(task, result, original_goal)
            .await?;

        // Record outcome for learning
        self.record_verification_outcome(
            task,
            &verification_result,
            result.success,
            recommendation.confidence_threshold,
            &recommendation.rule_threshold_adjustments,
        )
        .await?;

        // Learn from this interaction
        let outcome_score = self.calculate_outcome_score(&verification_result, result.success);
        self.learning_system
            .write()
            .await
            .learn_from_interaction(agent, &task.description, outcome_score)
            .await?;

        // Check if it's time for threshold adaptation
        if self.should_adapt_thresholds().await {
            self.adapt_thresholds().await?;
        }

        Ok(verification_result)
    }

    /// Get current threshold recommendation based on learning
    async fn get_current_threshold_recommendation(&self) -> Result<ThresholdRecommendation> {
        let performance = self.performance_tracker.read().await;
        let _history = self.threshold_history.read().await;

        // Analyze recent performance data
        let recent_outcomes = self.get_recent_outcomes(
            &performance,
            Duration::hours(self.adaptation_config.adaptation_window_hours as i64),
        );

        if recent_outcomes.len() < self.adaptation_config.min_samples_for_adaptation {
            // Not enough data, use current thresholds
            return Ok(ThresholdRecommendation {
                confidence_threshold: 0.75, // Default from SimpleVerificationSystem
                rule_threshold_adjustments: HashMap::new(),
                expected_performance_improvement: 0.0,
                confidence_in_recommendation: 0.5,
                reasoning: "Insufficient data for adaptation".to_string(),
            });
        }

        // Calculate optimal confidence threshold
        let optimal_confidence_threshold = self
            .calculate_optimal_confidence_threshold(&recent_outcomes)
            .await;

        // Calculate optimal rule thresholds
        let optimal_rule_thresholds = self
            .calculate_optimal_rule_thresholds(&recent_outcomes)
            .await;

        // Estimate performance improvement
        let current_performance = self.calculate_current_performance_score(&recent_outcomes);
        let expected_performance = self
            .estimate_performance_with_thresholds(
                &recent_outcomes,
                optimal_confidence_threshold,
                &optimal_rule_thresholds,
            )
            .await;

        let performance_improvement = expected_performance - current_performance;

        // Calculate confidence in recommendation
        let recommendation_confidence = self
            .calculate_recommendation_confidence(recent_outcomes.len(), performance_improvement);

        Ok(ThresholdRecommendation {
            confidence_threshold: optimal_confidence_threshold,
            rule_threshold_adjustments: optimal_rule_thresholds,
            expected_performance_improvement: performance_improvement,
            confidence_in_recommendation: recommendation_confidence,
            reasoning: format!(
                "Based on {} recent samples, optimal thresholds should improve performance by {:.2}%",
                recent_outcomes.len(),
                performance_improvement * 100.0
            ),
        })
    }

    /// Calculate optimal confidence threshold using performance data
    async fn calculate_optimal_confidence_threshold(
        &self,
        outcomes: &[&VerificationOutcome],
    ) -> f64 {
        let mut best_threshold = 0.75;
        let mut best_score = 0.0;

        // Test different threshold values
        for threshold_test in (50..=95).step_by(5) {
            let threshold = threshold_test as f64 / 100.0;
            let score = self
                .evaluate_threshold_performance(outcomes, threshold)
                .await;

            if score > best_score {
                best_score = score;
                best_threshold = threshold;
            }
        }

        // Ensure within configured range
        best_threshold.clamp(
            self.adaptation_config.confidence_threshold_range.0,
            self.adaptation_config.confidence_threshold_range.1,
        )
    }

    /// Calculate optimal rule thresholds
    async fn calculate_optimal_rule_thresholds(
        &self,
        outcomes: &[&VerificationOutcome],
    ) -> HashMap<String, f64> {
        let mut optimal_thresholds = HashMap::new();

        // Get all unique rule IDs from outcomes
        let rule_ids: std::collections::HashSet<String> = outcomes
            .iter()
            .flat_map(|outcome| outcome.rule_thresholds_used.keys())
            .cloned()
            .collect();

        for rule_id in rule_ids {
            let optimal_threshold = self
                .calculate_optimal_rule_threshold(outcomes, &rule_id)
                .await;
            optimal_thresholds.insert(rule_id, optimal_threshold);
        }

        optimal_thresholds
    }

    /// Calculate optimal threshold for a specific rule
    async fn calculate_optimal_rule_threshold(
        &self,
        outcomes: &[&VerificationOutcome],
        rule_id: &str,
    ) -> f64 {
        let mut best_threshold = 0.7;
        let mut best_score = 0.0;

        // Test different threshold values for this rule
        for threshold_test in (30..=90).step_by(10) {
            let threshold = threshold_test as f64 / 100.0;
            let score = self
                .evaluate_rule_threshold_performance(outcomes, rule_id, threshold)
                .await;

            if score > best_score {
                best_score = score;
                best_threshold = threshold;
            }
        }

        // Ensure within configured range
        best_threshold.clamp(
            self.adaptation_config.rule_threshold_range.0,
            self.adaptation_config.rule_threshold_range.1,
        )
    }

    /// Evaluate performance of a confidence threshold
    async fn evaluate_threshold_performance(
        &self,
        outcomes: &[&VerificationOutcome],
        threshold: f64,
    ) -> f64 {
        let mut correct_predictions = 0;
        let mut total_predictions = 0;
        let mut total_efficiency = 0.0;

        for outcome in outcomes {
            total_predictions += 1;

            // Simulate what would happen with this threshold
            let would_pass = outcome.verification_result.overall_score >= threshold;
            let actually_succeeded = outcome.actual_task_success;

            if would_pass == actually_succeeded {
                correct_predictions += 1;
            }

            // Factor in efficiency (verification time)
            let efficiency_score = 1.0 - (outcome.verification_time_ms as f64 / 10000.0).min(1.0);
            total_efficiency += efficiency_score;
        }

        if total_predictions == 0 {
            return 0.0;
        }

        let accuracy = correct_predictions as f64 / total_predictions as f64;
        let avg_efficiency = total_efficiency / total_predictions as f64;

        // Weighted performance score
        accuracy * self.adaptation_config.performance_weight_accuracy
            + avg_efficiency * self.adaptation_config.performance_weight_efficiency
    }

    /// Evaluate performance of a rule threshold
    async fn evaluate_rule_threshold_performance(
        &self,
        outcomes: &[&VerificationOutcome],
        rule_id: &str,
        threshold: f64,
    ) -> f64 {
        let relevant_outcomes: Vec<_> = outcomes
            .iter()
            .filter(|outcome| outcome.rule_thresholds_used.contains_key(rule_id))
            .collect();

        if relevant_outcomes.is_empty() {
            return 0.5; // Neutral score if no data
        }

        // Similar evaluation logic as confidence threshold but for specific rule
        self.evaluate_threshold_performance(
            &relevant_outcomes.into_iter().copied().collect::<Vec<_>>(),
            threshold,
        )
        .await
    }

    /// Apply threshold recommendation to base verification system
    async fn apply_threshold_recommendation(&mut self, recommendation: &ThresholdRecommendation) {
        // Configure base verification system with new thresholds
        self.base_verification
            .configure(recommendation.confidence_threshold);

        // Note: In a full implementation, you would also apply rule threshold adjustments
        // This would require extending the SimpleVerificationSystem to support dynamic rule threshold updates
        debug!(
            "Applied adaptive thresholds: confidence={:.3}, rules={:?}",
            recommendation.confidence_threshold, recommendation.rule_threshold_adjustments
        );
    }

    /// Record verification outcome for learning
    async fn record_verification_outcome(
        &self,
        task: &Task,
        verification_result: &SimpleVerificationResult,
        actual_success: bool,
        threshold_used: f64,
        rule_thresholds_used: &HashMap<String, f64>,
    ) -> Result<()> {
        let outcome = VerificationOutcome {
            timestamp: Utc::now(),
            task_id: task.id,
            verification_result: verification_result.clone(),
            actual_task_success: actual_success,
            verification_time_ms: verification_result.verification_time_ms,
            threshold_used,
            rule_thresholds_used: rule_thresholds_used.clone(),
        };

        let mut tracker = self.performance_tracker.write().await;
        tracker.verification_outcomes.push(outcome);

        // Maintain reasonable history size
        if tracker.verification_outcomes.len() > 10000 {
            tracker.verification_outcomes.drain(0..1000);
        }

        // Update accuracy metrics
        self.update_accuracy_metrics(&mut tracker, verification_result, actual_success)
            .await;

        Ok(())
    }

    /// Update accuracy metrics based on verification outcome
    async fn update_accuracy_metrics(
        &self,
        tracker: &mut PerformanceTracker,
        verification_result: &SimpleVerificationResult,
        actual_success: bool,
    ) {
        let verification_passed = matches!(
            verification_result.verification_status,
            SimpleVerificationStatus::Passed | SimpleVerificationStatus::PassedWithIssues
        );

        match (verification_passed, actual_success) {
            (true, true) => tracker.accuracy_metrics.true_positives += 1,
            (false, false) => tracker.accuracy_metrics.true_negatives += 1,
            (true, false) => tracker.accuracy_metrics.false_positives += 1,
            (false, true) => tracker.accuracy_metrics.false_negatives += 1,
        }

        // Recalculate derived metrics
        let tp = tracker.accuracy_metrics.true_positives as f64;
        let _tn = tracker.accuracy_metrics.true_negatives as f64;
        let fp = tracker.accuracy_metrics.false_positives as f64;
        let fn_count = tracker.accuracy_metrics.false_negatives as f64;

        tracker.accuracy_metrics.precision = if tp + fp > 0.0 { tp / (tp + fp) } else { 0.0 };
        tracker.accuracy_metrics.recall = if tp + fn_count > 0.0 {
            tp / (tp + fn_count)
        } else {
            0.0
        };

        let precision = tracker.accuracy_metrics.precision;
        let recall = tracker.accuracy_metrics.recall;
        tracker.accuracy_metrics.f1_score = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };
    }

    /// Check if it's time to adapt thresholds
    async fn should_adapt_thresholds(&self) -> bool {
        let history = self.threshold_history.read().await;
        let now = Utc::now();
        let time_since_last_adaptation = now - history.last_adaptation;

        time_since_last_adaptation
            >= Duration::hours(self.adaptation_config.adaptation_frequency_hours as i64)
    }

    /// Perform threshold adaptation
    async fn adapt_thresholds(&self) -> Result<()> {
        let recommendation = self.get_current_threshold_recommendation().await?;

        // Only adapt if we have sufficient confidence in the recommendation
        if recommendation.confidence_in_recommendation >= 0.7
            && recommendation.expected_performance_improvement > 0.01
        {
            // Record the adaptation
            let mut history = self.threshold_history.write().await;

            history.confidence_threshold_history.push(ThresholdEntry {
                timestamp: Utc::now(),
                threshold_value: recommendation.confidence_threshold,
                performance_score: recommendation.expected_performance_improvement,
                sample_count: self.get_recent_sample_count().await,
                adaptation_reason: recommendation.reasoning.clone(),
            });

            history.last_adaptation = Utc::now();
            history.adaptation_count += 1;

            info!(
                "Adapted verification thresholds: confidence={:.3}, expected_improvement={:.2}%, reason={}",
                recommendation.confidence_threshold,
                recommendation.expected_performance_improvement * 100.0,
                recommendation.reasoning
            );
        } else {
            debug!(
                "Skipped threshold adaptation: insufficient confidence ({:.2}) or improvement ({:.2}%)",
                recommendation.confidence_in_recommendation,
                recommendation.expected_performance_improvement * 100.0
            );
        }

        Ok(())
    }

    /// Helper methods
    fn get_recent_outcomes<'a>(
        &self,
        performance: &'a PerformanceTracker,
        window: Duration,
    ) -> Vec<&'a VerificationOutcome> {
        let cutoff = Utc::now() - window;
        performance
            .verification_outcomes
            .iter()
            .filter(|outcome| outcome.timestamp >= cutoff)
            .collect()
    }

    fn calculate_current_performance_score(&self, outcomes: &[&VerificationOutcome]) -> f64 {
        if outcomes.is_empty() {
            return 0.5;
        }

        let accuracy = outcomes
            .iter()
            .map(|outcome| {
                let verification_passed = matches!(
                    outcome.verification_result.verification_status,
                    SimpleVerificationStatus::Passed | SimpleVerificationStatus::PassedWithIssues
                );
                if verification_passed == outcome.actual_task_success {
                    1.0
                } else {
                    0.0
                }
            })
            .sum::<f64>()
            / outcomes.len() as f64;

        let efficiency = outcomes
            .iter()
            .map(|outcome| 1.0 - (outcome.verification_time_ms as f64 / 10000.0).min(1.0))
            .sum::<f64>()
            / outcomes.len() as f64;

        accuracy * self.adaptation_config.performance_weight_accuracy
            + efficiency * self.adaptation_config.performance_weight_efficiency
    }

    async fn estimate_performance_with_thresholds(
        &self,
        outcomes: &[&VerificationOutcome],
        confidence_threshold: f64,
        _rule_thresholds: &HashMap<String, f64>,
    ) -> f64 {
        // Simplified estimation - in practice, you'd simulate the full verification process
        self.evaluate_threshold_performance(outcomes, confidence_threshold)
            .await
    }

    fn calculate_recommendation_confidence(
        &self,
        sample_count: usize,
        performance_improvement: f64,
    ) -> f64 {
        let sample_confidence = (sample_count as f64 / 100.0).min(1.0);
        let improvement_confidence = (performance_improvement * 10.0).min(1.0);

        (sample_confidence + improvement_confidence) / 2.0
    }

    fn calculate_outcome_score(
        &self,
        verification_result: &SimpleVerificationResult,
        actual_success: bool,
    ) -> f64 {
        let verification_passed = matches!(
            verification_result.verification_status,
            SimpleVerificationStatus::Passed | SimpleVerificationStatus::PassedWithIssues
        );

        // Score based on correctness and confidence
        let correctness_score = if verification_passed == actual_success {
            1.0
        } else {
            0.0
        };
        let confidence_score = verification_result.confidence_score;

        (correctness_score * 0.7) + (confidence_score * 0.3)
    }

    async fn get_recent_sample_count(&self) -> usize {
        let performance = self.performance_tracker.read().await;
        let window = Duration::hours(self.adaptation_config.adaptation_window_hours as i64);
        self.get_recent_outcomes(&performance, window).len()
    }

    /// Get adaptive verification insights
    pub async fn get_adaptation_insights(&self) -> AdaptationInsights {
        let history = self.threshold_history.read().await;
        let performance = self.performance_tracker.read().await;

        let recent_outcomes = self.get_recent_outcomes(&performance, Duration::hours(24));
        let current_performance = self.calculate_current_performance_score(&recent_outcomes);

        AdaptationInsights {
            total_adaptations: history.adaptation_count,
            last_adaptation: history.last_adaptation,
            current_performance_score: current_performance,
            recent_sample_count: recent_outcomes.len(),
            accuracy_metrics: performance.accuracy_metrics.clone(),
            efficiency_metrics: performance.efficiency_metrics.clone(),
            next_adaptation_due: history.last_adaptation
                + Duration::hours(self.adaptation_config.adaptation_frequency_hours as i64),
        }
    }
}

impl ThresholdHistory {
    fn new() -> Self {
        Self {
            confidence_threshold_history: Vec::new(),
            rule_threshold_history: HashMap::new(),
            last_adaptation: Utc::now() - Duration::hours(24), // Allow immediate first adaptation
            adaptation_count: 0,
        }
    }
}

impl PerformanceTracker {
    fn new() -> Self {
        Self {
            verification_outcomes: Vec::new(),
            success_rate_by_threshold: HashMap::new(),
            efficiency_metrics: EfficiencyMetrics {
                average_verification_time_ms: 0.0,
                verification_time_by_tier: HashMap::new(),
                throughput_verifications_per_hour: 0.0,
            },
            accuracy_metrics: AccuracyMetrics {
                true_positives: 0,
                true_negatives: 0,
                false_positives: 0,
                false_negatives: 0,
                precision: 0.0,
                recall: 0.0,
                f1_score: 0.0,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationInsights {
    pub total_adaptations: u32,
    pub last_adaptation: DateTime<Utc>,
    pub current_performance_score: f64,
    pub recent_sample_count: usize,
    pub accuracy_metrics: AccuracyMetrics,
    pub efficiency_metrics: EfficiencyMetrics,
    pub next_adaptation_due: DateTime<Utc>,
}

/// Trait for integrating adaptive verification with existing systems
#[async_trait]
pub trait AdaptiveVerificationCapable {
    /// Perform adaptive verification with learning
    async fn adaptive_verify(
        &self,
        task: &Task,
        result: &TaskResult,
        original_goal: Option<&str>,
        adaptive_system: &mut AdaptiveVerificationSystem,
    ) -> Result<SimpleVerificationResult>;
}

#[async_trait]
impl AdaptiveVerificationCapable for Agent {
    async fn adaptive_verify(
        &self,
        task: &Task,
        result: &TaskResult,
        original_goal: Option<&str>,
        adaptive_system: &mut AdaptiveVerificationSystem,
    ) -> Result<SimpleVerificationResult> {
        adaptive_system
            .adaptive_verify_task_result(task, result, original_goal, self)
            .await
    }
}
