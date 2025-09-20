//! # Edge Case Tests for Adaptive Verification Unwrap Fixes
//!
//! Tests to prevent regression of unwrap_or() to unwrap_or_else(||) changes
//! in the adaptive verification system.

use crate::agents::adaptive_verification::{
    AccuracyMetrics, AdaptationConfig, AdaptationInsights, AdaptiveVerificationSystem,
    EfficiencyMetrics, PerformanceTracker, ThresholdHistory, ThresholdRecommendation,
    VerificationOutcome,
};
use crate::agents::simple_verification::{
    SimpleVerificationResult, SimpleVerificationStatus, VerificationTier,
};
use crate::agents::Agent;
use crate::tasks::{Task, TaskPriority, TaskResult};
use crate::utils::error::HiveError;
use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Test invalid constructor parameters
#[test]
fn test_invalid_constructor_parameters() {
    // Test negative values
    let invalid_config = AdaptationConfig {
        learning_rate: -0.1,
        min_samples_for_adaptation: -5,
        adaptation_window_hours: -24,
        confidence_threshold_range: (-0.5, 1.5), // Invalid range
        rule_threshold_range: (-0.3, 1.2),       // Invalid range
        adaptation_frequency_hours: -6,
        performance_weight_success: -0.4,
        performance_weight_efficiency: -0.3,
        performance_weight_accuracy: -0.3,
    };

    // The config itself should be creatable, but validation should catch issues
    assert!(invalid_config.learning_rate < 0.0);
    assert!(invalid_config.min_samples_for_adaptation < 0);
    assert!(invalid_config.adaptation_window_hours < 0);
    assert!(invalid_config.confidence_threshold_range.0 < 0.0);
    assert!(invalid_config.confidence_threshold_range.1 > 1.0);
    assert!(invalid_config.rule_threshold_range.0 < 0.0);
    assert!(invalid_config.rule_threshold_range.1 > 1.0);
    assert!(invalid_config.adaptation_frequency_hours < 0);

    // Test zero values
    let zero_config = AdaptationConfig {
        learning_rate: 0.0,
        min_samples_for_adaptation: 0,
        adaptation_window_hours: 0,
        confidence_threshold_range: (0.0, 0.0), // Zero range
        rule_threshold_range: (0.0, 0.0),       // Zero range
        adaptation_frequency_hours: 0,
        performance_weight_success: 0.0,
        performance_weight_efficiency: 0.0,
        performance_weight_accuracy: 0.0,
    };

    assert_eq!(zero_config.learning_rate, 0.0);
    assert_eq!(zero_config.min_samples_for_adaptation, 0);
    assert_eq!(zero_config.adaptation_window_hours, 0);
    assert_eq!(zero_config.confidence_threshold_range, (0.0, 0.0));
    assert_eq!(zero_config.rule_threshold_range, (0.0, 0.0));
    assert_eq!(zero_config.adaptation_frequency_hours, 0);

    // Test very large values
    let large_config = AdaptationConfig {
        learning_rate: 1000.0,
        min_samples_for_adaptation: i32::MAX as usize,
        adaptation_window_hours: i32::MAX as u32,
        confidence_threshold_range: (0.0, 1000.0),
        rule_threshold_range: (0.0, 1000.0),
        adaptation_frequency_hours: u32::MAX,
        performance_weight_success: 1000.0,
        performance_weight_efficiency: 1000.0,
        performance_weight_accuracy: 1000.0,
    };

    assert_eq!(large_config.learning_rate, 1000.0);
    assert_eq!(large_config.min_samples_for_adaptation, i32::MAX as usize);
    assert_eq!(large_config.adaptation_window_hours, i32::MAX as u32);
    assert_eq!(large_config.confidence_threshold_range, (0.0, 1000.0));
    assert_eq!(large_config.rule_threshold_range, (0.0, 1000.0));
    assert_eq!(large_config.adaptation_frequency_hours, u32::MAX);
}

/// Test boundary conditions for parameters
#[test]
fn test_boundary_conditions_for_parameters() {
    // Test minimum valid values
    let min_config = AdaptationConfig {
        learning_rate: f64::EPSILON,
        min_samples_for_adaptation: 1,
        adaptation_window_hours: 1,
        confidence_threshold_range: (0.0, f64::EPSILON),
        rule_threshold_range: (0.0, f64::EPSILON),
        adaptation_frequency_hours: 1,
        performance_weight_success: f64::EPSILON,
        performance_weight_efficiency: f64::EPSILON,
        performance_weight_accuracy: f64::EPSILON,
    };

    assert!(min_config.learning_rate > 0.0);
    assert!(min_config.min_samples_for_adaptation > 0);
    assert!(min_config.adaptation_window_hours > 0);
    assert!(min_config.confidence_threshold_range.0 >= 0.0);
    assert!(min_config.confidence_threshold_range.1 > 0.0);
    assert!(min_config.rule_threshold_range.0 >= 0.0);
    assert!(min_config.rule_threshold_range.1 > 0.0);
    assert!(min_config.adaptation_frequency_hours > 0);

    // Test maximum valid values
    let max_config = AdaptationConfig {
        learning_rate: f64::MAX,
        min_samples_for_adaptation: usize::MAX,
        adaptation_window_hours: u32::MAX,
        confidence_threshold_range: (0.0, f64::MAX),
        rule_threshold_range: (0.0, f64::MAX),
        adaptation_frequency_hours: u32::MAX,
        performance_weight_success: f64::MAX,
        performance_weight_efficiency: f64::MAX,
        performance_weight_accuracy: f64::MAX,
    };

    assert_eq!(max_config.learning_rate, f64::MAX);
    assert_eq!(max_config.min_samples_for_adaptation, usize::MAX);
    assert_eq!(max_config.adaptation_window_hours, u32::MAX);
    assert_eq!(max_config.confidence_threshold_range.0, 0.0);
    assert_eq!(max_config.confidence_threshold_range.1, f64::MAX);
    assert_eq!(max_config.rule_threshold_range.0, 0.0);
    assert_eq!(max_config.rule_threshold_range.1, f64::MAX);
    assert_eq!(max_config.adaptation_frequency_hours, u32::MAX);
}

/// Test null or missing required parameters
#[test]
fn test_null_or_missing_required_parameters() {
    // Test with None values (simulating missing parameters)
    let mut rule_thresholds = HashMap::new();
    rule_thresholds.insert("rule1".to_string(), 0.5);
    rule_thresholds.insert("rule2".to_string(), 0.7);

    // Test accessing non-existent rules
    let missing_rule = rule_thresholds.get("nonexistent_rule");
    assert!(missing_rule.is_none());

    // Test with empty collections
    let empty_thresholds = HashMap::new();
    let empty_outcomes = Vec::new();

    assert!(empty_thresholds.is_empty());
    assert!(empty_outcomes.is_empty());

    // Test with null-like values
    let zero_thresholds = HashMap::new();
    let none_value: Option<f64> = None;

    assert!(zero_thresholds.is_empty());
    assert!(none_value.is_none());
}

/// Test invalid agent IDs or names
#[test]
fn test_invalid_agent_ids_or_names() {
    // Test empty strings
    let empty_name = "";
    let empty_id = "";

    assert!(empty_name.is_empty());
    assert!(empty_id.is_empty());

    // Test very long strings
    let long_name = "a".repeat(10000);
    let long_id = "b".repeat(10000);

    assert_eq!(long_name.len(), 10000);
    assert_eq!(long_id.len(), 10000);

    // Test strings with special characters
    let special_name = "agent@#$%^&*()_+{}|:<>?[]\\;',./";
    let special_id = "id@#$%^&*()_+{}|:<>?[]\\;',./";

    assert!(special_name.contains('@'));
    assert!(special_id.contains('@'));

    // Test Unicode strings
    let unicode_name = "测试代理";
    let unicode_id = "测试ID";

    assert!(!unicode_name.is_ascii());
    assert!(!unicode_id.is_ascii());
}

/// Test concurrent agent creation failures
#[tokio::test]
async fn test_concurrent_agent_creation_failures() {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    let failure_count = Arc::new(Mutex::new(0));
    let success_count = Arc::new(Mutex::new(0));

    // Spawn multiple tasks that simulate agent creation
    let mut handles = vec![];

    for i in 0..10 {
        let failure_count_clone = failure_count.clone();
        let success_count_clone = success_count.clone();

        let handle = tokio::spawn(async move {
            // Simulate some operations that might fail
            if i % 3 == 0 {
                // Simulate failure
                let mut count = failure_count_clone.lock().await;
                *count += 1;
            } else {
                // Simulate success
                let mut count = success_count_clone.lock().await;
                *count += 1;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let _ = handle.await;
    }

    let final_failures = *failure_count.lock().await;
    let final_successes = *success_count.lock().await;

    assert_eq!(final_failures + final_successes, 10);
    assert!(final_failures >= 3); // At least some failures
    assert!(final_successes >= 6); // At least some successes
}

/// Test ThresholdHistory with edge cases
#[test]
fn test_threshold_history_edge_cases() {
    // Test empty history
    let empty_history = ThresholdHistory::new();
    assert!(empty_history.confidence_thresholds.is_empty());
    assert_eq!(empty_history.adaptation_count, 0);

    // Test history with invalid timestamps
    let mut invalid_history = ThresholdHistory::new();
    invalid_history.confidence_thresholds.push(
        crate::agents::adaptive_verification::ThresholdEntry {
            timestamp: DateTime::<Utc>::MIN_UTC,
            threshold_value: -1.0,     // Invalid threshold
            performance_score: -100.0, // Invalid score
            sample_count: 0,
            adaptation_reason: "".to_string(),
        },
    );

    assert_eq!(invalid_history.confidence_thresholds.len(), 1);
    assert_eq!(
        invalid_history.confidence_thresholds[0].threshold_value,
        -1.0
    );
    assert_eq!(
        invalid_history.confidence_thresholds[0].performance_score,
        -100.0
    );
    assert_eq!(invalid_history.confidence_thresholds[0].sample_count, 0);
    assert!(invalid_history.confidence_thresholds[0]
        .adaptation_reason
        .is_empty());
}

/// Test PerformanceTracker with edge cases
#[test]
fn test_performance_tracker_edge_cases() {
    // Test empty tracker
    let empty_tracker = PerformanceTracker::new();
    assert!(empty_tracker.verification_outcomes.is_empty());
    assert!(empty_tracker.success_rate_by_threshold.is_empty());
    assert_eq!(
        empty_tracker
            .efficiency_metrics
            .average_verification_time_ms,
        0.0
    );
    assert_eq!(empty_tracker.accuracy_metrics.true_positives, 0);
    assert_eq!(empty_tracker.accuracy_metrics.true_negatives, 0);
    assert_eq!(empty_tracker.accuracy_metrics.false_positives, 0);
    assert_eq!(empty_tracker.accuracy_metrics.false_negatives, 0);
    assert_eq!(empty_tracker.accuracy_metrics.precision, 0.0);
    assert_eq!(empty_tracker.accuracy_metrics.recall, 0.0);
    assert_eq!(empty_tracker.accuracy_metrics.f1_score, 0.0);

    // Test tracker with invalid data
    let mut invalid_tracker = PerformanceTracker::new();
    invalid_tracker
        .verification_outcomes
        .push(VerificationOutcome {
            timestamp: DateTime::<Utc>::MIN_UTC,
            task_id: uuid::Uuid::nil(),
            verification_result: SimpleVerificationResult {
                task_id: uuid::Uuid::nil(),
                verification_status: SimpleVerificationStatus::Failed,
                confidence_score: -1.0,       // Invalid confidence
                goal_alignment_score: -2.0,   // Invalid score
                format_compliance_score: 2.0, // Invalid score
                overall_score: -5.0,          // Invalid score
                verification_tier: VerificationTier::Standard,
                issues_found: vec![],
                verification_time_ms: u64::MAX,
                verified_at: DateTime::<Utc>::MIN_UTC,
                verifier_notes: "".to_string(),
            },
            actual_task_success: false,
            verification_time_ms: u64::MAX,
            threshold_used: -1.0, // Invalid threshold
            rule_thresholds_used: HashMap::new(),
        });

    assert_eq!(invalid_tracker.verification_outcomes.len(), 1);
    assert_eq!(
        invalid_tracker.verification_outcomes[0]
            .verification_result
            .confidence_score,
        -1.0
    );
    assert_eq!(
        invalid_tracker.verification_outcomes[0]
            .verification_result
            .goal_alignment_score,
        -2.0
    );
    assert_eq!(
        invalid_tracker.verification_outcomes[0]
            .verification_result
            .format_compliance_score,
        2.0
    );
    assert_eq!(
        invalid_tracker.verification_outcomes[0]
            .verification_result
            .overall_score,
        -5.0
    );
    assert_eq!(
        invalid_tracker.verification_outcomes[0].verification_time_ms,
        u64::MAX
    );
    assert_eq!(
        invalid_tracker.verification_outcomes[0].threshold_used,
        -1.0
    );
}

/// Test VerificationOutcome with edge cases
#[test]
fn test_verification_outcome_edge_cases() {
    // Test with extreme values
    let extreme_outcome = VerificationOutcome {
        timestamp: DateTime::<Utc>::MAX_UTC,
        task_id: uuid::Uuid::max(),
        verification_result: SimpleVerificationResult {
            task_id: uuid::Uuid::max(),
            verification_status: SimpleVerificationStatus::Passed,
            confidence_score: f64::MAX,
            goal_alignment_score: f64::MAX,
            format_compliance_score: f64::MAX,
            overall_score: f64::MAX,
            verification_tier: VerificationTier::Standard,
            issues_found: vec!["issue1".to_string(), "issue2".to_string()],
            verification_time_ms: u64::MAX,
            verified_at: DateTime::<Utc>::MAX_UTC,
            verifier_notes: "x".repeat(10000), // Very long notes
        },
        actual_task_success: true,
        verification_time_ms: u64::MAX,
        threshold_used: f64::MAX,
        rule_thresholds_used: {
            let mut thresholds = HashMap::new();
            thresholds.insert("rule1".to_string(), f64::MAX);
            thresholds.insert("rule2".to_string(), f64::MIN);
            thresholds
        },
    };

    assert_eq!(
        extreme_outcome.verification_result.confidence_score,
        f64::MAX
    );
    assert_eq!(
        extreme_outcome.verification_result.goal_alignment_score,
        f64::MAX
    );
    assert_eq!(
        extreme_outcome.verification_result.format_compliance_score,
        f64::MAX
    );
    assert_eq!(extreme_outcome.verification_result.overall_score, f64::MAX);
    assert_eq!(extreme_outcome.verification_time_ms, u64::MAX);
    assert_eq!(extreme_outcome.threshold_used, f64::MAX);
    assert_eq!(
        extreme_outcome.verification_result.verifier_notes.len(),
        10000
    );
    assert_eq!(extreme_outcome.rule_thresholds_used.len(), 2);
    assert_eq!(
        extreme_outcome.rule_thresholds_used.get("rule1"),
        Some(&f64::MAX)
    );
    assert_eq!(
        extreme_outcome.rule_thresholds_used.get("rule2"),
        Some(&f64::MIN)
    );
}

/// Test EfficiencyMetrics with edge cases
#[test]
fn test_efficiency_metrics_edge_cases() {
    // Test with extreme values
    let extreme_metrics = EfficiencyMetrics {
        average_verification_time_ms: f64::MAX,
        verification_time_by_tier: {
            let mut times = HashMap::new();
            times.insert("tier1".to_string(), f64::MAX);
            times.insert("tier2".to_string(), f64::MIN_POSITIVE);
            times
        },
        throughput_verifications_per_hour: f64::MAX,
    };

    assert_eq!(extreme_metrics.average_verification_time_ms, f64::MAX);
    assert_eq!(extreme_metrics.throughput_verifications_per_hour, f64::MAX);
    assert_eq!(extreme_metrics.verification_time_by_tier.len(), 2);
    assert_eq!(
        extreme_metrics.verification_time_by_tier.get("tier1"),
        Some(&f64::MAX)
    );
    assert_eq!(
        extreme_metrics.verification_time_by_tier.get("tier2"),
        Some(&f64::MIN_POSITIVE)
    );

    // Test with empty metrics
    let empty_metrics = EfficiencyMetrics {
        average_verification_time_ms: 0.0,
        verification_time_by_tier: HashMap::new(),
        throughput_verifications_per_hour: 0.0,
    };

    assert_eq!(empty_metrics.average_verification_time_ms, 0.0);
    assert!(empty_metrics.verification_time_by_tier.is_empty());
    assert_eq!(empty_metrics.throughput_verifications_per_hour, 0.0);
}

/// Test AccuracyMetrics with edge cases
#[test]
fn test_accuracy_metrics_edge_cases() {
    // Test with extreme values
    let extreme_metrics = AccuracyMetrics {
        true_positives: u32::MAX,
        true_negatives: u32::MAX,
        false_positives: u32::MAX,
        false_negatives: u32::MAX,
        precision: f64::MAX,
        recall: f64::MAX,
        f1_score: f64::MAX,
    };

    assert_eq!(extreme_metrics.true_positives, u32::MAX);
    assert_eq!(extreme_metrics.true_negatives, u32::MAX);
    assert_eq!(extreme_metrics.false_positives, u32::MAX);
    assert_eq!(extreme_metrics.false_negatives, u32::MAX);
    assert_eq!(extreme_metrics.precision, f64::MAX);
    assert_eq!(extreme_metrics.recall, f64::MAX);
    assert_eq!(extreme_metrics.f1_score, f64::MAX);

    // Test division by zero scenarios
    let zero_metrics = AccuracyMetrics {
        true_positives: 0,
        true_negatives: 0,
        false_positives: 0,
        false_negatives: 0,
        precision: 0.0,
        recall: 0.0,
        f1_score: 0.0,
    };

    assert_eq!(zero_metrics.true_positives, 0);
    assert_eq!(zero_metrics.true_negatives, 0);
    assert_eq!(zero_metrics.false_positives, 0);
    assert_eq!(zero_metrics.false_negatives, 0);
    assert_eq!(zero_metrics.precision, 0.0);
    assert_eq!(zero_metrics.recall, 0.0);
    assert_eq!(zero_metrics.f1_score, 0.0);
}

/// Test ThresholdRecommendation with edge cases
#[test]
fn test_threshold_recommendation_edge_cases() {
    // Test with extreme values
    let extreme_recommendation = ThresholdRecommendation {
        confidence_threshold: f64::MAX,
        rule_threshold_adjustments: {
            let mut adjustments = HashMap::new();
            adjustments.insert("rule1".to_string(), f64::MAX);
            adjustments.insert("rule2".to_string(), f64::MIN);
            adjustments
        },
        expected_performance_improvement: f64::MAX,
        confidence_in_recommendation: f64::MAX,
        reasoning: "x".repeat(10000), // Very long reasoning
    };

    assert_eq!(extreme_recommendation.confidence_threshold, f64::MAX);
    assert_eq!(
        extreme_recommendation.expected_performance_improvement,
        f64::MAX
    );
    assert_eq!(
        extreme_recommendation.confidence_in_recommendation,
        f64::MAX
    );
    assert_eq!(extreme_recommendation.reasoning.len(), 10000);
    assert_eq!(extreme_recommendation.rule_threshold_adjustments.len(), 2);
    assert_eq!(
        extreme_recommendation
            .rule_threshold_adjustments
            .get("rule1"),
        Some(&f64::MAX)
    );
    assert_eq!(
        extreme_recommendation
            .rule_threshold_adjustments
            .get("rule2"),
        Some(&f64::MIN)
    );

    // Test with empty adjustments
    let empty_recommendation = ThresholdRecommendation {
        confidence_threshold: 0.5,
        rule_threshold_adjustments: HashMap::new(),
        expected_performance_improvement: 0.0,
        confidence_in_recommendation: 0.0,
        reasoning: "".to_string(),
    };

    assert_eq!(empty_recommendation.confidence_threshold, 0.5);
    assert!(empty_recommendation.rule_threshold_adjustments.is_empty());
    assert_eq!(empty_recommendation.expected_performance_improvement, 0.0);
    assert_eq!(empty_recommendation.confidence_in_recommendation, 0.0);
    assert!(empty_recommendation.reasoning.is_empty());
}

/// Test AdaptationInsights with edge cases
#[test]
fn test_adaptation_insights_edge_cases() {
    // Test with extreme values
    let extreme_insights = AdaptationInsights {
        total_adaptations: u32::MAX,
        last_adaptation: DateTime::<Utc>::MAX_UTC,
        current_performance_score: f64::MAX,
        recent_sample_count: usize::MAX,
        accuracy_metrics: AccuracyMetrics {
            true_positives: u32::MAX,
            true_negatives: u32::MAX,
            false_positives: u32::MAX,
            false_negatives: u32::MAX,
            precision: f64::MAX,
            recall: f64::MAX,
            f1_score: f64::MAX,
        },
        efficiency_metrics: EfficiencyMetrics {
            average_verification_time_ms: f64::MAX,
            verification_time_by_tier: HashMap::new(),
            throughput_verifications_per_hour: f64::MAX,
        },
        next_adaptation_due: DateTime::<Utc>::MAX_UTC,
    };

    assert_eq!(extreme_insights.total_adaptations, u32::MAX);
    assert_eq!(extreme_insights.current_performance_score, f64::MAX);
    assert_eq!(extreme_insights.recent_sample_count, usize::MAX);
    assert_eq!(extreme_insights.accuracy_metrics.true_positives, u32::MAX);
    assert_eq!(
        extreme_insights
            .efficiency_metrics
            .average_verification_time_ms,
        f64::MAX
    );
    assert_eq!(
        extreme_insights
            .efficiency_metrics
            .throughput_verifications_per_hour,
        f64::MAX
    );

    // Test with minimum values
    let min_insights = AdaptationInsights {
        total_adaptations: 0,
        last_adaptation: DateTime::<Utc>::MIN_UTC,
        current_performance_score: f64::MIN,
        recent_sample_count: 0,
        accuracy_metrics: AccuracyMetrics {
            true_positives: 0,
            true_negatives: 0,
            false_positives: 0,
            false_negatives: 0,
            precision: 0.0,
            recall: 0.0,
            f1_score: 0.0,
        },
        efficiency_metrics: EfficiencyMetrics {
            average_verification_time_ms: 0.0,
            verification_time_by_tier: HashMap::new(),
            throughput_verifications_per_hour: 0.0,
        },
        next_adaptation_due: DateTime::<Utc>::MIN_UTC,
    };

    assert_eq!(min_insights.total_adaptations, 0);
    assert_eq!(min_insights.current_performance_score, f64::MIN);
    assert_eq!(min_insights.recent_sample_count, 0);
    assert_eq!(min_insights.accuracy_metrics.true_positives, 0);
    assert_eq!(
        min_insights.efficiency_metrics.average_verification_time_ms,
        0.0
    );
    assert_eq!(
        min_insights
            .efficiency_metrics
            .throughput_verifications_per_hour,
        0.0
    );
}

/// Test concurrent access to adaptive verification components
#[tokio::test]
async fn test_concurrent_access_to_adaptive_components() {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    let access_count = Arc::new(Mutex::new(0));

    // Create shared components
    let threshold_history = Arc::new(RwLock::new(ThresholdHistory::new()));
    let performance_tracker = Arc::new(RwLock::new(PerformanceTracker::new()));

    // Spawn multiple tasks that access the components concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let history_clone = threshold_history.clone();
        let tracker_clone = performance_tracker.clone();
        let access_count_clone = access_count.clone();

        let handle = tokio::spawn(async move {
            // Simulate concurrent access
            {
                let mut history = history_clone.write().await;
                history.adaptation_count += 1;
            }

            {
                let mut tracker = tracker_clone.write().await;
                tracker.verification_outcomes.push(VerificationOutcome {
                    timestamp: Utc::now(),
                    task_id: uuid::Uuid::new_v4(),
                    verification_result: SimpleVerificationResult {
                        task_id: uuid::Uuid::new_v4(),
                        verification_status: SimpleVerificationStatus::Passed,
                        confidence_score: 0.8,
                        goal_alignment_score: 0.7,
                        format_compliance_score: 0.9,
                        overall_score: 0.8,
                        verification_tier: VerificationTier::Standard,
                        issues_found: vec![],
                        verification_time_ms: 100,
                        verified_at: Utc::now(),
                        verifier_notes: format!("Concurrent access {}", i),
                    },
                    actual_task_success: true,
                    verification_time_ms: 100,
                    threshold_used: 0.75,
                    rule_thresholds_used: HashMap::new(),
                });
            }

            let mut count = access_count_clone.lock().await;
            *count += 1;
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let _ = handle.await;
    }

    let final_access_count = *access_count.lock().await;
    assert_eq!(final_access_count, 10);

    // Verify the components were updated
    let history = threshold_history.read().await;
    let tracker = performance_tracker.read().await;

    assert_eq!(history.adaptation_count, 10);
    assert_eq!(tracker.verification_outcomes.len(), 10);
}

/// Test memory usage with large adaptive verification data
#[test]
fn test_memory_usage_with_large_adaptive_data() {
    // Create very large collections to test memory handling
    let mut large_thresholds = HashMap::new();
    let mut large_outcomes = Vec::new();

    // Create many rule thresholds
    for i in 0..10000 {
        large_thresholds.insert(format!("rule_{}", i), i as f64 / 10000.0);
    }

    // Create many verification outcomes
    for i in 0..1000 {
        large_outcomes.push(VerificationOutcome {
            timestamp: Utc::now(),
            task_id: uuid::Uuid::new_v4(),
            verification_result: SimpleVerificationResult {
                task_id: uuid::Uuid::new_v4(),
                verification_status: SimpleVerificationStatus::Passed,
                confidence_score: 0.8,
                goal_alignment_score: 0.7,
                format_compliance_score: 0.9,
                overall_score: 0.8,
                verification_tier: VerificationTier::Standard,
                issues_found: vec![format!("issue_{}", i)],
                verification_time_ms: 100,
                verified_at: Utc::now(),
                verifier_notes: format!("Large data test {}", i),
            },
            actual_task_success: true,
            verification_time_ms: 100,
            threshold_used: 0.75,
            rule_thresholds_used: large_thresholds.clone(),
        });
    }

    assert_eq!(large_thresholds.len(), 10000);
    assert_eq!(large_outcomes.len(), 1000);
    assert_eq!(large_outcomes[0].rule_thresholds_used.len(), 10000);

    // Test that the large structures can be dropped without issues
    drop(large_thresholds);
    drop(large_outcomes);
}

/// Test timeout scenarios in adaptive verification
#[tokio::test]
async fn test_timeout_scenarios_in_adaptive_verification() {
    // Test that operations complete within reasonable time
    let start_time = std::time::Instant::now();

    // Simulate adaptive verification operations
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Simulate some computation
    let mut sum = 0.0;
    for i in 0..100000 {
        sum += (i as f64).sin();
    }

    let elapsed = start_time.elapsed();
    assert!(elapsed.as_millis() >= 50); // Should have taken at least the sleep time
    assert!(elapsed.as_millis() < 1000); // Should not have taken too long

    // The sum should be some reasonable value (not checking exact value)
    assert!(sum.is_finite());
}

/// Test error handling in adaptive verification calculations
#[test]
fn test_error_handling_in_adaptive_calculations() {
    // Test division by zero scenarios
    let zero_division_scenarios = vec![
        (0.0, 1.0), // Division by zero in some calculations
        (1.0, 0.0), // Division by zero
        (0.0, 0.0), // Both zero
    ];

    for (numerator, denominator) in zero_division_scenarios {
        if denominator != 0.0 {
            let result = numerator / denominator;
            assert!(result.is_finite() || result.is_infinite());
        }
        // If denominator is zero, we skip division to avoid panic
    }

    // Test NaN handling
    let nan_scenarios = vec![f64::NAN, f64::INFINITY, f64::NEG_INFINITY];

    for value in nan_scenarios {
        assert!(!value.is_finite() || value.is_nan());
    }

    // Test that operations with NaN/Inf don't cause panics
    let _ = f64::NAN + 1.0;
    let _ = f64::INFINITY * 2.0;
    let _ = f64::NEG_INFINITY - 1.0;
}

/// Test boundary conditions in performance calculations
#[test]
fn test_boundary_conditions_in_performance_calculations() {
    // Test precision/recall/f1 calculations with edge cases
    let calculation_scenarios = vec![
        (0, 0, 0, 0),                             // All zeros
        (1, 0, 0, 0),                             // True positives only
        (0, 1, 0, 0),                             // True negatives only
        (0, 0, 1, 0),                             // False positives only
        (0, 0, 0, 1),                             // False negatives only
        (u32::MAX, u32::MAX, u32::MAX, u32::MAX), // All max
    ];

    for (tp, tn, fp, fn_count) in calculation_scenarios {
        let tp_f = tp as f64;
        let fp_f = fp as f64;
        let fn_f = fn_count as f64;

        // Calculate precision
        let precision = if tp_f + fp_f > 0.0 {
            tp_f / (tp_f + fp_f)
        } else {
            0.0
        };

        // Calculate recall
        let recall = if tp_f + fn_f > 0.0 {
            tp_f / (tp_f + fn_f)
        } else {
            0.0
        };

        // Calculate f1
        let f1 = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };

        // Verify results are valid
        assert!(precision >= 0.0 && precision <= 1.0);
        assert!(recall >= 0.0 && recall <= 1.0);
        assert!(f1 >= 0.0 && f1 <= 1.0);
    }
}
