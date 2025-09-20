//! # Adaptive Verification Enhancement
//!
//! Enhances the existing Simple Verification System with machine learning-based
//! threshold optimization. Integrates with the existing AdaptiveLearningSystem
//! to automatically adjust verification thresholds based on historical performance.

#![allow(clippy::unused_async)]

use crate::utils::error::HiveResult;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task;
use tracing::{debug, info};
use uuid::Uuid;

use crate::agents::simple_verification::{
    SimpleVerificationResult, SimpleVerificationStatus, SimpleVerificationSystem,
};
use crate::agents::{Agent, AgentBehavior, CommunicationComplexity};
use crate::communication::patterns::CommunicationConfig;
use crate::communication::protocols::{MessageEnvelope, MessagePayload, MessageType};
use crate::neural::adaptive_learning::AdaptiveLearningSystem;
use crate::neural::NLPProcessor;
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
    pub confidence_thresholds: Vec<ThresholdEntry>,
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
    pub efficiency_metrics: EfficiencyMetrics,
    pub accuracy_metrics: AccuracyMetrics,
    pub success_rate_by_threshold: HashMap<String, f64>,
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
    pub fn new(
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
    ) -> HiveResult<SimpleVerificationResult> {
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
    async fn get_current_threshold_recommendation(&self) -> HiveResult<ThresholdRecommendation> {
        let performance = self.performance_tracker.read().await;
        let _history = self.threshold_history.read().await;

        // Analyze recent performance data
        let recent_outcomes = self.get_recent_outcomes(
            &performance,
            Duration::hours(i64::from(self.adaptation_config.adaptation_window_hours)),
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
            .await?;

        // Calculate optimal rule thresholds
        let optimal_rule_thresholds = self
            .calculate_optimal_rule_thresholds(&recent_outcomes)
            .await?;

        // Estimate performance improvement
        let current_performance = self.calculate_current_performance_score(&recent_outcomes);
        let expected_performance = self
            .estimate_performance_with_thresholds(
                &recent_outcomes,
                optimal_confidence_threshold,
                &optimal_rule_thresholds,
            )
            .await?;

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
    ) -> HiveResult<f64> {
        let mut best_threshold = 0.75;
        let mut best_score = 0.0;

        // Test different threshold values - optimized to reduce async overhead
        let threshold_tests: Vec<f64> =
            (50..=95).step_by(5).map(|t| f64::from(t) / 100.0).collect();

        // Process thresholds sequentially but with optimized evaluation
        for threshold in threshold_tests {
            let score = self
                .evaluate_threshold_performance(outcomes, threshold)
                .await?;

            if score > best_score {
                best_score = score;
                best_threshold = threshold;
            }
        }

        // Ensure within configured range
        Ok(best_threshold.clamp(
            self.adaptation_config.confidence_threshold_range.0,
            self.adaptation_config.confidence_threshold_range.1,
        ))
    }

    /// Calculate optimal rule thresholds
    async fn calculate_optimal_rule_thresholds(
        &self,
        outcomes: &[&VerificationOutcome],
    ) -> HiveResult<HashMap<String, f64>> {
        // Move computation to blocking thread
        let outcomes_vec: Vec<VerificationOutcome> =
            outcomes.iter().map(|&outcome| outcome.clone()).collect();
        let config = self.adaptation_config.clone();

        let result = task::spawn_blocking(move || {
            let outcomes_refs: Vec<&VerificationOutcome> = outcomes_vec.iter().collect();
            Self::calculate_optimal_rule_thresholds_sync(&outcomes_refs, &config)
        })
        .await
        .map_err(|e| {
            tracing::warn!(
                "Failed to calculate optimal rule thresholds asynchronously: {}, using defaults",
                e
            );
            crate::utils::error::HiveError::TaskExecutionFailed {
                reason: "Failed to spawn blocking task for rule threshold calculation".to_string(),
            }
        })?;

        Ok(result)
    }

    /// Synchronous calculation of optimal rule thresholds
    fn calculate_optimal_rule_thresholds_sync(
        outcomes: &[&VerificationOutcome],
        config: &AdaptationConfig,
    ) -> HashMap<String, f64> {
        let mut optimal_thresholds = HashMap::new();

        // Get all unique rule IDs from outcomes
        let rule_ids: std::collections::HashSet<String> = outcomes
            .iter()
            .flat_map(|outcome| outcome.rule_thresholds_used.keys())
            .cloned()
            .collect();

        for rule_id in rule_ids {
            let optimal_threshold =
                Self::calculate_optimal_rule_threshold_sync(outcomes, &rule_id, config);
            optimal_thresholds.insert(rule_id, optimal_threshold);
        }

        optimal_thresholds
    }

    /// Synchronous calculation of optimal rule threshold
    fn calculate_optimal_rule_threshold_sync(
        outcomes: &[&VerificationOutcome],
        rule_id: &str,
        config: &AdaptationConfig,
    ) -> f64 {
        let mut best_threshold = 0.7;
        let mut best_score = 0.0;

        // Test different threshold values for this rule
        for threshold_test in (30..=90).step_by(10) {
            let threshold = f64::from(threshold_test) / 100.0;

            // Evaluate synchronously
            let score = Self::evaluate_rule_threshold_performance_sync(
                outcomes, rule_id, threshold, config,
            );

            if score > best_score {
                best_score = score;
                best_threshold = threshold;
            }
        }

        // Ensure within configured range
        best_threshold.clamp(config.rule_threshold_range.0, config.rule_threshold_range.1)
    }

    /// Synchronous evaluation of rule threshold performance
    fn evaluate_rule_threshold_performance_sync(
        outcomes: &[&VerificationOutcome],
        rule_id: &str,
        threshold: f64,
        config: &AdaptationConfig,
    ) -> f64 {
        let relevant_outcomes: Vec<&VerificationOutcome> = outcomes
            .iter()
            .filter(|outcome| outcome.rule_thresholds_used.contains_key(rule_id))
            .copied()
            .collect();

        if relevant_outcomes.is_empty() {
            return 0.5; // Neutral score if no data
        }

        // Evaluate synchronously
        Self::evaluate_threshold_performance_sync(&relevant_outcomes, threshold, config)
    }

    /// Evaluate performance of a confidence threshold
    async fn evaluate_threshold_performance(
        &self,
        outcomes: &[&VerificationOutcome],
        threshold: f64,
    ) -> HiveResult<f64> {
        // Move CPU-intensive computation to blocking thread to avoid blocking async runtime
        let outcomes_vec: Vec<VerificationOutcome> =
            outcomes.iter().map(|&outcome| outcome.clone()).collect();
        let config = self.adaptation_config.clone();

        let result = task::spawn_blocking(move || {
            let outcomes_refs: Vec<&VerificationOutcome> = outcomes_vec.iter().collect();
            Self::evaluate_threshold_performance_sync(&outcomes_refs, threshold, &config)
        })
        .await
        .map_err(|e| {
            tracing::warn!(
                "Failed to evaluate threshold performance asynchronously: {}, using fallback",
                e
            );
            crate::utils::error::HiveError::TaskExecutionFailed {
                reason: "Failed to spawn blocking task for threshold performance evaluation"
                    .to_string(),
            }
        })?;

        Ok(result)
    }

    /// Synchronous version of threshold performance evaluation
    fn evaluate_threshold_performance_sync(
        outcomes: &[&VerificationOutcome],
        threshold: f64,
        config: &AdaptationConfig,
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

        let accuracy = f64::from(correct_predictions) / f64::from(total_predictions);
        let avg_efficiency = total_efficiency / f64::from(total_predictions);

        // Weighted performance score
        accuracy * config.performance_weight_accuracy
            + avg_efficiency * config.performance_weight_efficiency
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
    ) -> HiveResult<()> {
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
        let tp = f64::from(tracker.accuracy_metrics.true_positives);
        let fp = f64::from(tracker.accuracy_metrics.false_positives);
        let fn_count = f64::from(tracker.accuracy_metrics.false_negatives);

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
            >= Duration::hours(i64::from(self.adaptation_config.adaptation_frequency_hours))
    }

    /// Perform threshold adaptation
    async fn adapt_thresholds(&self) -> HiveResult<()> {
        let recommendation = self.get_current_threshold_recommendation().await?;

        // Only adapt if we have sufficient confidence in the recommendation
        if recommendation.confidence_in_recommendation >= 0.7
            && recommendation.expected_performance_improvement > 0.01
        {
            // Record the adaptation
            let mut history = self.threshold_history.write().await;

            history.confidence_thresholds.push(ThresholdEntry {
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
    #[allow(clippy::unused_self)]
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
    ) -> HiveResult<f64> {
        // Simplified estimation - in practice, you'd simulate the full verification process
        self.evaluate_threshold_performance(outcomes, confidence_threshold)
            .await
    }

    #[allow(clippy::unused_self)]
    fn calculate_recommendation_confidence(
        &self,
        sample_count: usize,
        performance_improvement: f64,
    ) -> f64 {
        let sample_confidence = (sample_count as f64 / 100.0).min(1.0);
        let improvement_confidence = (performance_improvement * 10.0).min(1.0);

        f64::midpoint(sample_confidence, improvement_confidence)
    }

    #[allow(clippy::unused_self)]
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
        let window = Duration::hours(i64::from(self.adaptation_config.adaptation_window_hours));
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
                + Duration::hours(i64::from(self.adaptation_config.adaptation_frequency_hours)),
        }
    }
}

impl ThresholdHistory {
    fn new() -> Self {
        Self {
            confidence_thresholds: Vec::new(),
            last_adaptation: Utc::now() - Duration::hours(24), // Allow immediate first adaptation
            adaptation_count: 0,
        }
    }
}

impl PerformanceTracker {
    fn new() -> Self {
        Self {
            verification_outcomes: Vec::new(),
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
            success_rate_by_threshold: HashMap::new(),
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
    ) -> HiveResult<SimpleVerificationResult>;
}

#[async_trait]
impl AgentBehavior for AdaptiveVerificationSystem {
    async fn execute_task(&mut self, _task: Task) -> HiveResult<TaskResult> {
        // Adaptive verification agents don't execute tasks directly
        // They enhance verification of task results
        Err(crate::utils::error::HiveError::AgentExecutionFailed {
            reason: "AdaptiveVerificationSystem does not execute tasks directly".to_string(),
        })
    }

    async fn communicate(
        &mut self,
        envelope: MessageEnvelope,
    ) -> HiveResult<Option<MessageEnvelope>> {
        // Standardized communication pattern for adaptive verification
        let complexity = match envelope.priority {
            crate::communication::patterns::MessagePriority::Low => CommunicationComplexity::Simple,
            crate::communication::patterns::MessagePriority::Normal => {
                CommunicationComplexity::Standard
            }
            crate::communication::patterns::MessagePriority::High => {
                CommunicationComplexity::Complex
            }
            crate::communication::patterns::MessagePriority::Critical => {
                CommunicationComplexity::Heavy
            }
        };

        // Use standardized delay based on complexity
        let delay_ms = match complexity {
            CommunicationComplexity::Simple => 50,
            CommunicationComplexity::Standard => 100,
            CommunicationComplexity::Complex => 200,
            CommunicationComplexity::Heavy => 500,
        };

        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

        match envelope.message_type {
            MessageType::Request => {
                let response_payload = match &envelope.payload {
                    MessagePayload::Text(text) => {
                        MessagePayload::Text(format!(
                            "Adaptive verification system acknowledging: {} - Ready to adapt thresholds",
                            text
                        ))
                    }
                    MessagePayload::Json(json) => {
                        MessagePayload::Json(serde_json::json!({
                            "response": "Adaptive verification system ready",
                            "current_adaptation_insights": self.get_adaptation_insights().await,
                            "original_request": json
                        }))
                    }
                    _ => MessagePayload::Text(
                        "Adaptive verification system acknowledged message".to_string()
                    ),
                };

                let response =
                    MessageEnvelope::new_response(&envelope, Uuid::new_v4(), response_payload);
                Ok(Some(response))
            }
            MessageType::Broadcast => {
                tracing::info!(
                    "Adaptive verification system received broadcast: {:?}",
                    envelope.payload
                );
                Ok(None)
            }
            MessageType::CoordinationRequest => {
                // Handle coordination for threshold adaptation
                if let MessagePayload::CoordinationData {
                    performance_metrics,
                    ..
                } = &envelope.payload
                {
                    tracing::info!(
                        "Received coordination data for threshold adaptation: {:?}",
                        performance_metrics
                    );
                }
                Ok(None)
            }
            _ => {
                let response = MessageEnvelope::new_response(
                    &envelope,
                    Uuid::new_v4(),
                    MessagePayload::Text(format!(
                        "Adaptive verification system processed message of type {:?}",
                        envelope.message_type
                    )),
                );
                Ok(Some(response))
            }
        }
    }

    async fn request_response(
        &mut self,
        request: MessageEnvelope,
        timeout: std::time::Duration,
    ) -> HiveResult<MessageEnvelope> {
        // Simulate processing time for adaptive verification
        tokio::time::sleep(timeout / 4).await;

        let response = MessageEnvelope::new_response(
            &request,
            Uuid::new_v4(),
            MessagePayload::Json(serde_json::json!({
                "response": "Adaptive verification system processed request",
                "adaptation_insights": self.get_adaptation_insights().await,
                "processing_timeout": timeout.as_millis()
            })),
        );

        Ok(response)
    }

    async fn learn(&mut self, _nlp_processor: &NLPProcessor) -> HiveResult<()> {
        // Adaptive verification learning is handled through task verification
        // This could trigger threshold adaptation based on learning patterns
        debug!("Adaptive verification system learning triggered");
        Ok(())
    }

    async fn update_position(
        &mut self,
        _swarm_center: (f64, f64),
        _neighbors: &[Agent],
    ) -> HiveResult<()> {
        // Adaptive verification systems don't participate in swarm positioning
        Ok(())
    }

    fn get_communication_config(&self) -> CommunicationConfig {
        CommunicationConfig {
            default_timeout: std::time::Duration::from_secs(30),
            max_retries: 3,
            retry_delay: std::time::Duration::from_millis(200),
            max_concurrent_messages: 50,
            buffer_size: 4096,
            enable_compression: true,
            delivery_guarantee: crate::communication::patterns::DeliveryGuarantee::AtLeastOnce,
        }
    }
}

#[async_trait]
impl AdaptiveVerificationCapable for Agent {
    async fn adaptive_verify(
        &self,
        task: &Task,
        result: &TaskResult,
        original_goal: Option<&str>,
        adaptive_system: &mut AdaptiveVerificationSystem,
    ) -> HiveResult<SimpleVerificationResult> {
        adaptive_system
            .adaptive_verify_task_result(task, result, original_goal, self)
            .await
    }
}

    #[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::simple_verification::{SimpleVerificationSystem, VerificationTier};
    use crate::neural::adaptive_learning::{AdaptiveLearningConfig, AdaptiveLearningSystem};
    use crate::tasks::{TaskPriority, TaskResult};
    use crate::tests::test_utils::{create_test_agent, create_test_task};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    async fn create_test_adaptive_system() -> AdaptiveVerificationSystem {
        let nlp_processor_result = NLPProcessor::new().await;
        let nlp_processor = Arc::new(match nlp_processor_result {
            Ok(proc) => proc,
            Err(e) => panic!("Failed to create NLP processor: {:?}", e),
        });
        let base_verification = SimpleVerificationSystem::new(nlp_processor);
        let learning_config = AdaptiveLearningConfig::default();
        let learning_system_result = AdaptiveLearningSystem::new(learning_config).await;
        let learning_system = Arc::new(RwLock::new(match learning_system_result {
            Ok(sys) => sys,
            Err(e) => panic!("Failed to create learning system: {:?}", e),
        }));
        let config = AdaptationConfig::default();
        AdaptiveVerificationSystem::new(base_verification, learning_system, config)
    }

    fn create_test_task_result(success: bool) -> TaskResult {
        TaskResult {
            task_id: uuid::Uuid::new_v4(),
            agent_id: uuid::Uuid::new_v4(),
            success,
            output: if success {
                "Task completed successfully".to_string()
            } else {
                "Task failed".to_string()
            },
            execution_time: 1000,
            error_message: if success {
                None
            } else {
                Some("Execution error".to_string())
            },
            completed_at: Utc::now(),
            quality_score: Some(if success { 0.9 } else { 0.3 }),
            learned_insights: vec![],
        }
    }

    #[tokio::test]
    async fn test_adaptive_verification_system_creation() {
        let system = create_test_adaptive_system().await;

        // Check that the system is initialized correctly
        assert_eq!(system.adaptation_config.learning_rate, 0.05);
        assert_eq!(system.adaptation_config.min_samples_for_adaptation, 10);
    }

    #[tokio::test]
    async fn test_adaptation_config_default() {
        let config = AdaptationConfig::default();

        assert_eq!(config.learning_rate, 0.05);
        assert_eq!(config.min_samples_for_adaptation, 10);
        assert_eq!(config.adaptation_window_hours, 24);
        assert_eq!(config.confidence_threshold_range, (0.5, 0.95));
        assert_eq!(config.rule_threshold_range, (0.3, 0.9));
        assert_eq!(config.adaptation_frequency_hours, 6);
        assert_eq!(config.performance_weight_success, 0.4);
        assert_eq!(config.performance_weight_efficiency, 0.3);
        assert_eq!(config.performance_weight_accuracy, 0.3);
    }

    #[tokio::test]
    async fn test_threshold_history_new() {
        let history = ThresholdHistory::new();

        assert!(history.confidence_thresholds.is_empty());
        assert_eq!(history.adaptation_count, 0);
        // last_adaptation should be set to allow immediate adaptation
        assert!(Utc::now() - history.last_adaptation < chrono::Duration::hours(1));
    }

    #[tokio::test]
    async fn test_performance_tracker_new() {
        let tracker = PerformanceTracker::new();

        assert!(tracker.verification_outcomes.is_empty());
        assert!(tracker.success_rate_by_threshold.is_empty());
        assert_eq!(tracker.efficiency_metrics.average_verification_time_ms, 0.0);
        assert_eq!(tracker.accuracy_metrics.true_positives, 0);
        assert_eq!(tracker.accuracy_metrics.true_negatives, 0);
        assert_eq!(tracker.accuracy_metrics.false_positives, 0);
        assert_eq!(tracker.accuracy_metrics.false_negatives, 0);
        assert_eq!(tracker.accuracy_metrics.precision, 0.0);
        assert_eq!(tracker.accuracy_metrics.recall, 0.0);
        assert_eq!(tracker.accuracy_metrics.f1_score, 0.0);
    }

    #[tokio::test]
    async fn test_adaptive_verify_task_result_success() -> Result<(), Box<dyn std::error::Error>> {
        let mut system = create_test_adaptive_system().await;
        let agent = create_test_agent("TestAgent", crate::agents::AgentType::Worker);
        let task = create_test_task("Test task", "general", TaskPriority::Medium);
        let result = create_test_task_result(true);

        let verification_result = system
            .adaptive_verify_task_result(&task, &result, None, &agent)
            .await?;

        assert!(verification_result.verification_time_ms > 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_adaptive_verify_task_result_failure() -> Result<(), Box<dyn std::error::Error>> {
        let mut system = create_test_adaptive_system().await;
        let agent = create_test_agent("TestAgent", crate::agents::AgentType::Worker);
        let task = create_test_task("Test task", "general", TaskPriority::Medium);
        let result = create_test_task_result(false);

        let verification_result = system
            .adaptive_verify_task_result(&task, &result, None, &agent)
            .await?;

        assert!(verification_result.verification_time_ms > 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_current_threshold_recommendation() -> Result<(), Box<dyn std::error::Error>> {
        let system = create_test_adaptive_system().await;

        let rec = system.get_current_threshold_recommendation().await?;

        assert!(rec.confidence_threshold >= 0.5 && rec.confidence_threshold <= 0.95);
        assert!(rec.expected_performance_improvement >= 0.0);
        assert!(rec.confidence_in_recommendation >= 0.0 && rec.confidence_in_recommendation <= 1.0);
        assert!(!rec.reasoning.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_record_verification_outcome() {
        let system = create_test_adaptive_system().await;
        let task = create_test_task("Test task", "general", TaskPriority::Medium);
        let verification_result = SimpleVerificationResult {
            task_id: uuid::Uuid::new_v4(),
            verification_status: SimpleVerificationStatus::Passed,
            confidence_score: 0.9,
            goal_alignment_score: 0.85,
            format_compliance_score: 0.88,
            overall_score: 0.85,
            verification_tier: VerificationTier::Standard,
            issues_found: vec![],
            verification_time_ms: 500,
            verified_at: Utc::now(),
            verifier_notes: "Test verification".to_string(),
        };
        let threshold_used = 0.8;
        let rule_thresholds_used = std::collections::HashMap::new();

        let result = system
            .record_verification_outcome(
                &task,
                &verification_result,
                true,
                threshold_used,
                &rule_thresholds_used,
            )
            .await;

        assert!(result.is_ok());

        // Check that outcome was recorded
        let tracker = system.performance_tracker.read().await;
        assert_eq!(tracker.verification_outcomes.len(), 1);
        assert_eq!(tracker.accuracy_metrics.true_positives, 1);
    }

    #[tokio::test]
    async fn test_update_accuracy_metrics() {
        let system = create_test_adaptive_system().await;
        let mut tracker = PerformanceTracker::new();

        // Test true positive
        let verification_result_tp = SimpleVerificationResult {
            task_id: uuid::Uuid::new_v4(),
            verification_status: SimpleVerificationStatus::Passed,
            confidence_score: 0.9,
            goal_alignment_score: 0.85,
            format_compliance_score: 0.88,
            overall_score: 0.85,
            verification_tier: VerificationTier::Standard,
            issues_found: vec![],
            verification_time_ms: 500,
            verified_at: Utc::now(),
            verifier_notes: "Test verification TP".to_string(),
        };
        system
            .update_accuracy_metrics(&mut tracker, &verification_result_tp, true)
            .await;
        assert_eq!(tracker.accuracy_metrics.true_positives, 1);

        // Test false positive
        let verification_result_fp = SimpleVerificationResult {
            task_id: uuid::Uuid::new_v4(),
            verification_status: SimpleVerificationStatus::Passed,
            confidence_score: 0.9,
            goal_alignment_score: 0.85,
            format_compliance_score: 0.88,
            overall_score: 0.85,
            verification_tier: VerificationTier::Standard,
            issues_found: vec![],
            verification_time_ms: 500,
            verified_at: Utc::now(),
            verifier_notes: "Test verification FP".to_string(),
        };
        system
            .update_accuracy_metrics(&mut tracker, &verification_result_fp, false)
            .await;
        assert_eq!(tracker.accuracy_metrics.false_positives, 1);

        // Test true negative
        let verification_result_tn = SimpleVerificationResult {
            task_id: uuid::Uuid::new_v4(),
            verification_status: SimpleVerificationStatus::Failed,
            confidence_score: 0.8,
            goal_alignment_score: 0.3,
            format_compliance_score: 0.4,
            overall_score: 0.3,
            verification_tier: VerificationTier::Standard,
            issues_found: vec![],
            verification_time_ms: 500,
            verified_at: Utc::now(),
            verifier_notes: "Test verification TN".to_string(),
        };
        system
            .update_accuracy_metrics(&mut tracker, &verification_result_tn, false)
            .await;
        assert_eq!(tracker.accuracy_metrics.true_negatives, 1);

        // Test false negative
        let verification_result_fn = SimpleVerificationResult {
            task_id: uuid::Uuid::new_v4(),
            verification_status: SimpleVerificationStatus::Failed,
            confidence_score: 0.8,
            goal_alignment_score: 0.3,
            format_compliance_score: 0.4,
            overall_score: 0.3,
            verification_tier: VerificationTier::Standard,
            issues_found: vec![],
            verification_time_ms: 500,
            verified_at: Utc::now(),
            verifier_notes: "Test verification FN".to_string(),
        };
        system
            .update_accuracy_metrics(&mut tracker, &verification_result_fn, true)
            .await;
        assert_eq!(tracker.accuracy_metrics.false_negatives, 1);

        // Check precision, recall, f1
        assert_eq!(tracker.accuracy_metrics.precision, 0.5); // 1 TP / (1 TP + 1 FP)
        assert_eq!(tracker.accuracy_metrics.recall, 0.5); // 1 TP / (1 TP + 1 FN)
        assert_eq!(tracker.accuracy_metrics.f1_score, 0.5); // 2 * (0.5 * 0.5) / (0.5 + 0.5)
    }

    #[tokio::test]
    async fn test_should_adapt_thresholds() {
        let mut system = create_test_adaptive_system().await;

        // Initially should not adapt (just created)
        assert!(!system.should_adapt_thresholds().await);

        // Manually set last adaptation to past
        {
            let mut history = system.threshold_history.write().await;
            history.last_adaptation = Utc::now() - chrono::Duration::hours(7); // More than 6 hours
        }

        // Now should adapt
        assert!(system.should_adapt_thresholds().await);
    }

    #[tokio::test]
    async fn test_calculate_outcome_score() {
        let system = create_test_adaptive_system().await;

        // Test correct verification
        let correct_result = SimpleVerificationResult {
            task_id: uuid::Uuid::new_v4(),
            verification_status: SimpleVerificationStatus::Passed,
            confidence_score: 0.9,
            goal_alignment_score: 0.85,
            format_compliance_score: 0.88,
            overall_score: 0.85,
            verification_tier: VerificationTier::Standard,
            issues_found: vec![],
            verification_time_ms: 500,
            verified_at: Utc::now(),
            verifier_notes: "Test correct result".to_string(),
        };
        let score_correct = system.calculate_outcome_score(&correct_result, true);
        assert!(score_correct > 0.7); // High score for correct verification

        // Test incorrect verification
        let incorrect_result = SimpleVerificationResult {
            task_id: uuid::Uuid::new_v4(),
            verification_status: SimpleVerificationStatus::Passed,
            confidence_score: 0.9,
            goal_alignment_score: 0.85,
            format_compliance_score: 0.88,
            overall_score: 0.85,
            verification_tier: VerificationTier::Standard,
            issues_found: vec![],
            verification_time_ms: 500,
            verified_at: Utc::now(),
            verifier_notes: "Test incorrect result".to_string(),
        };
        let score_incorrect = system.calculate_outcome_score(&incorrect_result, false);
        assert!(score_incorrect < 0.4); // Low score for incorrect verification
    }

    #[tokio::test]
    async fn test_get_adaptation_insights() {
        let system = create_test_adaptive_system().await;

        let insights = system.get_adaptation_insights().await;

        assert_eq!(insights.total_adaptations, 0);
        assert!(
            insights.current_performance_score >= 0.0 && insights.current_performance_score <= 1.0
        );
        assert!(insights.recent_sample_count >= 0);
        assert!(insights.next_adaptation_due > Utc::now());
    }

    #[tokio::test]
    async fn test_evaluate_threshold_performance_sync() {
        let config = AdaptationConfig::default();

        // Create mock outcomes
        let outcomes = vec![
            &VerificationOutcome {
                timestamp: Utc::now(),
                task_id: uuid::Uuid::new_v4(),
                verification_result: SimpleVerificationResult {
                    task_id: uuid::Uuid::new_v4(),
                    verification_status: SimpleVerificationStatus::Passed,
                    confidence_score: 0.85,
                    goal_alignment_score: 0.9,
                    format_compliance_score: 0.92,
                    overall_score: 0.9,
                    verification_tier: VerificationTier::Standard,
                    issues_found: vec![],
                    verification_time_ms: 1000,
                    verified_at: Utc::now(),
                    verifier_notes: "Test evaluation result".to_string(),
                },
                actual_task_success: true,
                verification_time_ms: 1000,
                threshold_used: 0.8,
                rule_thresholds_used: std::collections::HashMap::new(),
            },
            &VerificationOutcome {
                timestamp: Utc::now(),
                task_id: uuid::Uuid::new_v4(),
                verification_result: SimpleVerificationResult {
                    task_id: uuid::Uuid::new_v4(),
                    verification_status: SimpleVerificationStatus::Failed,
                    confidence_score: 0.7,
                    goal_alignment_score: 0.3,
                    format_compliance_score: 0.4,
                    overall_score: 0.3,
                    verification_tier: VerificationTier::Standard,
                    issues_found: vec![],
                    verification_time_ms: 1500,
                    verified_at: Utc::now(),
                    verifier_notes: "Test failed result".to_string(),
                },
                actual_task_success: false,
                verification_time_ms: 1500,
                threshold_used: 0.8,
                rule_thresholds_used: std::collections::HashMap::new(),
            },
        ];

        let score = AdaptiveVerificationSystem::evaluate_threshold_performance_sync(
            &outcomes, 0.8, &config,
        );

        assert!(score >= 0.0 && score <= 1.0);
    }

    #[tokio::test]
    async fn test_calculate_recommendation_confidence() {
        let system = create_test_adaptive_system().await;

        // Test with high sample count and improvement
        let confidence_high = system.calculate_recommendation_confidence(100, 0.1);
        assert!(confidence_high > 0.8);

        // Test with low sample count
        let confidence_low = system.calculate_recommendation_confidence(5, 0.1);
        assert!(confidence_low < 0.6);
    }

    #[tokio::test]
    async fn test_get_recent_sample_count() -> Result<(), Box<dyn std::error::Error>> {
        let system = create_test_adaptive_system().await;

        // Initially should be 0
        let count = system.get_recent_sample_count().await;
        assert_eq!(count, 0);

        // Add some outcomes
        let task = create_test_task("Test task", "general", TaskPriority::Medium);
        let verification_result = SimpleVerificationResult {
            task_id: uuid::Uuid::new_v4(),
            verification_status: SimpleVerificationStatus::Passed,
            confidence_score: 0.9,
            goal_alignment_score: 0.85,
            format_compliance_score: 0.88,
            overall_score: 0.85,
            verification_tier: VerificationTier::Standard,
            issues_found: vec![],
            verification_time_ms: 500,
            verified_at: Utc::now(),
            verifier_notes: "Test record result".to_string(),
        };

        system
            .record_verification_outcome(
                &task,
                &verification_result,
                true,
                0.8,
                &std::collections::HashMap::new(),
            )
            .await?;

        let count_after = system.get_recent_sample_count().await;
        assert_eq!(count_after, 1);
        Ok(())
    }

    // Test edge cases

    #[tokio::test]
    async fn test_adaptive_verify_with_empty_goal() {
        let mut system = create_test_adaptive_system().await;
        let agent = create_test_agent("TestAgent", crate::agents::AgentType::Worker);
        let task = create_test_task("Test task", "general", TaskPriority::Medium);
        let result = create_test_task_result(true);

        // Test with None goal
        let verification_result = system
            .adaptive_verify_task_result(&task, &result, None, &agent)
            .await;
        assert!(verification_result.is_ok());
    }

    #[tokio::test]
    async fn test_threshold_bounds() {
        let config = AdaptationConfig {
            confidence_threshold_range: (0.6, 0.9),
            ..Default::default()
        };

        // Ensure ranges are valid
        assert!(config.confidence_threshold_range.0 < config.confidence_threshold_range.1);
        assert!(config.rule_threshold_range.0 < config.rule_threshold_range.1);
    }

    #[tokio::test]
    async fn test_performance_tracker_with_no_outcomes() {
        let tracker = PerformanceTracker::new();

        // Should handle empty outcomes gracefully
        assert_eq!(tracker.verification_outcomes.len(), 0);
        assert_eq!(tracker.accuracy_metrics.true_positives, 0);
    }
}
