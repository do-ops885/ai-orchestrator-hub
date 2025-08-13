//! # Verification Engine
//!
//! Core engine that implements different verification strategies and coordinates
//! the verification process for agent pairs.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use anyhow::Result;
use chrono::{DateTime, Utc};
use tracing::{info, debug, warn};

use crate::agents::verification::{
    VerifiableTask, VerificationResult, VerificationStatus, VerificationStrategy,
    VerificationMethod, Discrepancy, DiscrepancySeverity, VerificationDetails
};
use crate::tasks::TaskResult;
use crate::neural::NLPProcessor;

/// Main verification engine that coordinates different verification strategies
pub struct VerificationEngine {
    pub strategies: HashMap<VerificationMethod, Box<dyn VerificationStrategy>>,
    pub nlp_processor: Arc<NLPProcessor>,
    pub goal_alignment_analyzer: GoalAlignmentAnalyzer,
    pub quality_assessor: QualityAssessor,
    pub bias_detector: BiasDetector,
}

/// Analyzes how well results align with original goals
pub struct GoalAlignmentAnalyzer {
    nlp_processor: Arc<NLPProcessor>,
    alignment_threshold: f64,
}

/// Assesses quality of task results using independent metrics
pub struct QualityAssessor {
    quality_metrics: Vec<QualityMetric>,
    min_quality_threshold: f64,
}

/// Detects potential bias in verification process
pub struct BiasDetector {
    bias_patterns: Vec<BiasPattern>,
    detection_threshold: f64,
}

/// Represents a quality metric for assessment
#[derive(Debug, Clone)]
pub struct QualityMetric {
    pub name: String,
    pub weight: f64,
    pub evaluator: QualityEvaluator,
}

/// Different types of quality evaluators
#[derive(Debug, Clone)]
pub enum QualityEvaluator {
    Completeness,
    Accuracy,
    Efficiency,
    Clarity,
    Consistency,
}

/// Patterns that indicate potential bias
#[derive(Debug, Clone)]
pub struct BiasPattern {
    pub pattern_name: String,
    pub description: String,
    pub detection_criteria: Vec<String>,
    pub severity: DiscrepancySeverity,
}

impl VerificationEngine {
    pub async fn new(nlp_processor: Arc<NLPProcessor>) -> Result<Self> {
        let mut strategies: HashMap<VerificationMethod, Box<dyn VerificationStrategy>> = HashMap::new();
        
        // Initialize verification strategies
        strategies.insert(
            VerificationMethod::GoalAlignment,
            Box::new(GoalAlignmentVerifier::new(nlp_processor.clone())),
        );
        
        strategies.insert(
            VerificationMethod::QualityAssessment,
            Box::new(QualityAssessmentVerifier::new()),
        );
        
        strategies.insert(
            VerificationMethod::OutputAnalysis,
            Box::new(OutputAnalysisVerifier::new()),
        );
        
        strategies.insert(
            VerificationMethod::ProcessValidation,
            Box::new(ProcessValidationVerifier::new()),
        );

        let goal_alignment_analyzer = GoalAlignmentAnalyzer::new(nlp_processor.clone());
        let quality_assessor = QualityAssessor::new();
        let bias_detector = BiasDetector::new();

        Ok(Self {
            strategies,
            nlp_processor: nlp_processor.clone(),
            goal_alignment_analyzer,
            quality_assessor,
            bias_detector,
        })
    }

    /// Perform comprehensive verification of a task result
    pub async fn verify_task_result(
        &self,
        task: &VerifiableTask,
        result: &TaskResult,
        verification_method: VerificationMethod,
    ) -> Result<VerificationResult> {
        debug!("Starting verification for task {} using method {:?}", 
               task.base_task.id, verification_method);

        // Get the appropriate verification strategy
        let strategy = self.strategies.get(&verification_method)
            .ok_or_else(|| anyhow::anyhow!("Verification method {:?} not available", verification_method))?;

        // Perform verification
        let mut verification_result = strategy.verify(task, result, &task.original_goal).await?;

        // Detect potential bias in verification
        let bias_score = self.bias_detector.detect_bias(&verification_result, task, result).await?;
        if bias_score > self.bias_detector.detection_threshold {
            warn!("Potential bias detected in verification (score: {:.3})", bias_score);
            verification_result.verification_confidence *= (1.0 - bias_score * 0.5);
        }

        // Final validation
        self.validate_verification_result(&verification_result, task)?;

        info!("Verification completed for task {} with status {:?} and confidence {:.3}",
              task.base_task.id, verification_result.verification_status, verification_result.verification_confidence);

        Ok(verification_result)
    }

    /// Validate that verification result meets requirements
    fn validate_verification_result(
        &self,
        result: &VerificationResult,
        task: &VerifiableTask,
    ) -> Result<()> {
        // Check if confidence meets minimum threshold
        if result.verification_confidence < task.verification_requirements.min_confidence {
            return Err(anyhow::anyhow!(
                "Verification confidence {:.3} below required threshold {:.3}",
                result.verification_confidence,
                task.verification_requirements.min_confidence
            ));
        }

        // Check for critical discrepancies
        let critical_discrepancies = result.discrepancies_found.iter()
            .filter(|d| matches!(d.severity, DiscrepancySeverity::Critical))
            .count();

        if critical_discrepancies > 0 {
            warn!("Found {} critical discrepancies in verification", critical_discrepancies);
        }

        Ok(())
    }

    /// Get available verification methods
    pub fn get_available_methods(&self) -> Vec<VerificationMethod> {
        self.strategies.keys().cloned().collect()
    }
}

impl GoalAlignmentAnalyzer {
    pub fn new(nlp_processor: Arc<NLPProcessor>) -> Self {
        Self {
            nlp_processor,
            alignment_threshold: 0.7,
        }
    }

    /// Analyze how well the result aligns with the original goal
    pub async fn analyze_alignment(
        &self,
        original_goal: &str,
        result_output: &str,
    ) -> Result<f64> {
        // Tokenize both goal and result
        let goal_tokens: Vec<String> = original_goal.split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();
        
        let result_tokens: Vec<String> = result_output.split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();

        // Analyze sentiment alignment
        let goal_sentiment = self.nlp_processor.analyze_sentiment(&goal_tokens);
        let result_sentiment = self.nlp_processor.analyze_sentiment(&result_tokens);
        
        let sentiment_alignment = 1.0 - (goal_sentiment - result_sentiment).abs();

        // Calculate keyword overlap
        let goal_keywords: std::collections::HashSet<_> = goal_tokens.iter().collect();
        let result_keywords: std::collections::HashSet<_> = result_tokens.iter().collect();
        
        let intersection_size = goal_keywords.intersection(&result_keywords).count();
        let union_size = goal_keywords.union(&result_keywords).count();
        
        let keyword_overlap = if union_size > 0 {
            intersection_size as f64 / union_size as f64
        } else {
            0.0
        };

        // Combine metrics
        let alignment_score = (sentiment_alignment * 0.4) + (keyword_overlap * 0.6);
        
        debug!("Goal alignment analysis: sentiment={:.3}, keywords={:.3}, total={:.3}",
               sentiment_alignment, keyword_overlap, alignment_score);

        Ok(alignment_score.clamp(0.0, 1.0))
    }
}

impl QualityAssessor {
    pub fn new() -> Self {
        let quality_metrics = vec![
            QualityMetric {
                name: "Completeness".to_string(),
                weight: 0.3,
                evaluator: QualityEvaluator::Completeness,
            },
            QualityMetric {
                name: "Accuracy".to_string(),
                weight: 0.3,
                evaluator: QualityEvaluator::Accuracy,
            },
            QualityMetric {
                name: "Clarity".to_string(),
                weight: 0.2,
                evaluator: QualityEvaluator::Clarity,
            },
            QualityMetric {
                name: "Efficiency".to_string(),
                weight: 0.2,
                evaluator: QualityEvaluator::Efficiency,
            },
        ];

        Self {
            quality_metrics,
            min_quality_threshold: 0.6,
        }
    }

    /// Assess the quality of task output
    pub async fn assess_quality(
        &self,
        task: &VerifiableTask,
        result: &TaskResult,
    ) -> Result<f64> {
        let mut total_score = 0.0;
        let mut total_weight = 0.0;

        for metric in &self.quality_metrics {
            let score = self.evaluate_metric(&metric.evaluator, task, result).await?;
            total_score += score * metric.weight;
            total_weight += metric.weight;
        }

        let quality_score = if total_weight > 0.0 {
            total_score / total_weight
        } else {
            0.0
        };

        debug!("Quality assessment completed with score: {:.3}", quality_score);
        Ok(quality_score.clamp(0.0, 1.0))
    }

    async fn evaluate_metric(
        &self,
        evaluator: &QualityEvaluator,
        task: &VerifiableTask,
        result: &TaskResult,
    ) -> Result<f64> {
        match evaluator {
            QualityEvaluator::Completeness => {
                // Check if all required elements are present
                let output_length = result.output.len();
                let expected_min_length = task.base_task.description.len() / 2; // Heuristic
                
                if output_length >= expected_min_length {
                    Ok(0.9)
                } else {
                    Ok((output_length as f64 / expected_min_length as f64).min(1.0))
                }
            }
            QualityEvaluator::Accuracy => {
                // Use success flag and quality score from result
                let base_score = if result.success { 0.8 } else { 0.2 };
                let quality_bonus = result.quality_score.unwrap_or(0.0) * 0.2;
                Ok((base_score + quality_bonus).min(1.0))
            }
            QualityEvaluator::Clarity => {
                // Simple heuristic based on output structure
                let has_structure = result.output.contains('\n') || result.output.contains('.');
                let reasonable_length = result.output.len() > 10 && result.output.len() < 10000;
                
                if has_structure && reasonable_length {
                    Ok(0.8)
                } else if reasonable_length {
                    Ok(0.6)
                } else {
                    Ok(0.3)
                }
            }
            QualityEvaluator::Efficiency => {
                // Based on execution time relative to estimated duration
                if let Some(estimated) = task.base_task.estimated_duration {
                    let actual_seconds = result.execution_time / 1000; // Convert ms to seconds
                    let efficiency = (estimated as f64 / actual_seconds as f64).min(2.0) / 2.0;
                    Ok(efficiency.clamp(0.1, 1.0))
                } else {
                    Ok(0.7) // Default if no estimate available
                }
            }
            QualityEvaluator::Consistency => {
                // This would require comparison with similar tasks
                Ok(0.7) // Placeholder
            }
        }
    }
}

impl BiasDetector {
    pub fn new() -> Self {
        let bias_patterns = vec![
            BiasPattern {
                pattern_name: "Confirmation Bias".to_string(),
                description: "Verification only looks for confirming evidence".to_string(),
                detection_criteria: vec![
                    "no_negative_checks".to_string(),
                    "only_positive_indicators".to_string(),
                ],
                severity: DiscrepancySeverity::Major,
            },
            BiasPattern {
                pattern_name: "Anchoring Bias".to_string(),
                description: "Verification heavily influenced by initial success indicator".to_string(),
                detection_criteria: vec![
                    "early_success_fixation".to_string(),
                    "insufficient_deep_analysis".to_string(),
                ],
                severity: DiscrepancySeverity::Minor,
            },
        ];

        Self {
            bias_patterns,
            detection_threshold: 0.6,
        }
    }

    /// Detect potential bias in verification process
    pub async fn detect_bias(
        &self,
        verification_result: &VerificationResult,
        task: &VerifiableTask,
        result: &TaskResult,
    ) -> Result<f64> {
        let mut bias_score: f64 = 0.0;

        // Check for confirmation bias
        if verification_result.discrepancies_found.is_empty() && 
           verification_result.verification_confidence > 0.95 &&
           result.quality_score.unwrap_or(1.0) < 0.8 {
            bias_score += 0.3;
            debug!("Potential confirmation bias detected");
        }

        // Check for anchoring bias
        if verification_result.verification_confidence > 0.9 &&
           verification_result.independent_assessment.len() < 50 {
            bias_score += 0.2;
            debug!("Potential anchoring bias detected");
        }

        // Check for insufficient analysis
        if verification_result.verification_details.reasoning.len() < 20 {
            bias_score += 0.1;
            debug!("Insufficient analysis depth detected");
        }

        Ok(bias_score.clamp(0.0, 1.0))
    }
}