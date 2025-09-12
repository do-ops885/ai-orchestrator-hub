//! # Agent Pair Programming Verification System
//!
//! Implements a sophisticated verification system where every task is executed by a primary agent
//! and independently verified by a verification agent. This ensures that results are validated
//! against original goals rather than just execution criteria.

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::agents::Agent;
use crate::neural::NLPProcessor;
use crate::tasks::{Task, TaskResult};

/// Enhanced task structure that includes verification requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiableTask {
    /// Base task information
    pub base_task: Task,
    /// Immutable original objective that verification focuses on
    pub original_goal: String,
    /// Specific criteria that define success
    pub success_criteria: Vec<SuccessCriterion>,
    /// Requirements for verification process
    pub verification_requirements: VerificationRequirements,
    /// History of verification attempts
    pub verification_history: Vec<VerificationAttempt>,
    /// Verification level required for this task
    pub verification_level: VerificationLevel,
}

/// Defines what constitutes success for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriterion {
    pub criterion_id: Uuid,
    pub description: String,
    pub measurable_outcome: String,
    pub verification_method: VerificationMethod,
    pub weight: f64,    // Importance weight (0.0 to 1.0)
    pub threshold: f64, // Minimum score to pass this criterion
}

/// Different methods available for verification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum VerificationMethod {
    /// Analyze the output against expected patterns
    OutputAnalysis,
    /// Validate the process used was appropriate
    ProcessValidation,
    /// Check alignment with original goals
    GoalAlignment,
    /// Assess quality using independent metrics
    QualityAssessment,
    /// Re-execute critical parts independently
    IndependentExecution,
    /// Semantic analysis of results
    SemanticValidation,
}

/// Requirements for verification process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequirements {
    /// Minimum confidence score required
    pub min_confidence: f64,
    /// Maximum time allowed for verification
    pub max_verification_time: chrono::Duration,
    /// Whether multiple verification methods are required
    pub require_multiple_methods: bool,
    /// Whether human review is required for conflicts
    pub require_human_review_on_conflict: bool,
    /// Specific verification methods that must be used
    pub required_methods: Vec<VerificationMethod>,
}

/// Level of verification required
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationLevel {
    /// No verification required (trusted agents, low-risk tasks)
    None,
    /// Simple goal alignment check
    Basic,
    /// Full independent verification
    Standard,
    /// Multiple verification strategies with cross-validation
    Comprehensive,
}

/// Record of a verification attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationAttempt {
    pub attempt_id: Uuid,
    pub verifier_agent: Uuid,
    pub method_used: VerificationMethod,
    pub result: VerificationResult,
    pub timestamp: DateTime<Utc>,
    pub duration: chrono::Duration,
}

/// Result of verification process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub verification_id: Uuid,
    pub task_id: Uuid,
    pub verifier_agent: Uuid,
    pub verification_status: VerificationStatus,
    pub goal_alignment_score: f64,
    pub quality_score: f64,
    pub independent_assessment: String,
    pub discrepancies_found: Vec<Discrepancy>,
    pub verification_confidence: f64,
    pub method_used: VerificationMethod,
    pub timestamp: DateTime<Utc>,
    pub verification_details: VerificationDetails,
}

/// Status of verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// Fully meets original goals
    Verified,
    /// Meets some but not all goals
    PartiallyVerified,
    /// Does not meet original goals
    Failed,
    /// Needs human or coordinator review
    RequiresReview,
    /// Cannot determine due to insufficient information
    Inconclusive,
    /// Verification process encountered an error
    VerificationError,
}

/// Specific discrepancy found during verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discrepancy {
    pub id: Uuid,
    pub criterion_id: Uuid,
    pub expected: String,
    pub actual: String,
    pub severity: DiscrepancySeverity,
    pub description: String,
}

/// Severity of a discrepancy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscrepancySeverity {
    Critical, // Task fails completely
    Major,    // Significant issue but task might still be acceptable
    Minor,    // Small issue that doesn't affect overall success
    Cosmetic, // Aesthetic or style issue
}

/// Detailed verification information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationDetails {
    pub criteria_scores: HashMap<Uuid, f64>,
    pub method_specific_data: HashMap<String, serde_json::Value>,
    pub reasoning: String,
    pub alternative_approaches_considered: Vec<String>,
    pub confidence_factors: Vec<String>,
}

/// Manages agent pairs for task execution and verification
#[derive(Debug, Clone)]
pub struct AgentPair {
    pub pair_id: Uuid,
    pub primary_agent: Uuid,
    pub verification_agent: Uuid,
    pub specialization: String,
    pub trust_score: f64,
    pub verification_history: Vec<VerificationResult>,
    pub created_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub performance_metrics: PairMetrics,
}

/// Performance metrics for agent pairs
#[derive(Debug, Clone, Default)]
pub struct PairMetrics {
    pub total_tasks: u32,
    pub successful_verifications: u32,
    pub failed_verifications: u32,
    pub average_verification_time: f64,
    pub average_confidence: f64,
    pub discrepancy_detection_rate: f64,
    pub false_positive_rate: f64,
    pub false_negative_rate: f64,
}

/// Coordinates agent pairs and manages verification process
pub struct PairCoordinator {
    pub active_pairs: HashMap<Uuid, AgentPair>,
    pub pair_performance_metrics: HashMap<Uuid, PairMetrics>,
    pub verification_standards: VerificationStandards,
    pub nlp_processor: std::sync::Arc<NLPProcessor>,
}

/// Standards and thresholds for verification
#[derive(Debug, Clone)]
pub struct VerificationStandards {
    pub min_goal_alignment_score: f64,
    pub min_quality_score: f64,
    pub min_verification_confidence: f64,
    pub max_acceptable_discrepancies: usize,
    pub critical_discrepancy_threshold: f64,
}

impl Default for VerificationStandards {
    fn default() -> Self {
        Self {
            min_goal_alignment_score: 0.7,
            min_quality_score: 0.6,
            min_verification_confidence: 0.8,
            max_acceptable_discrepancies: 3,
            critical_discrepancy_threshold: 0.9,
        }
    }
}

/// Trait for verification strategies
#[async_trait]
pub trait VerificationStrategy: Send + Sync {
    /// Perform verification of task result
    async fn verify(
        &self,
        task: &VerifiableTask,
        result: &TaskResult,
        original_goal: &str,
    ) -> Result<VerificationResult>;

    /// Get the criteria this strategy uses for verification
    fn get_verification_criteria(&self) -> Vec<String>;

    /// Confirm this strategy is independent from execution
    fn is_independent_from_execution(&self) -> bool;

    /// Get the verification method this strategy implements
    fn get_method(&self) -> VerificationMethod;
}

impl PairCoordinator {
    pub fn new(nlp_processor: std::sync::Arc<NLPProcessor>) -> Self {
        Self {
            active_pairs: HashMap::new(),
            pair_performance_metrics: HashMap::new(),
            verification_standards: VerificationStandards::default(),
            nlp_processor,
        }
    }

    /// Create a new agent pair for task execution and verification
    pub fn create_agent_pair(
        &mut self,
        primary_agent: Uuid,
        verification_agent: Uuid,
        specialization: String,
    ) -> Result<Uuid> {
        // Ensure agents are different
        if primary_agent == verification_agent {
            return Err(anyhow::anyhow!(
                "Primary and verification agents must be different"
            ));
        }

        let pair_id = Uuid::new_v4();
        let pair = AgentPair {
            pair_id,
            primary_agent,
            verification_agent,
            specialization,
            trust_score: 0.5, // Start with neutral trust
            verification_history: Vec::new(),
            created_at: Utc::now(),
            last_used: Utc::now(),
            performance_metrics: PairMetrics::default(),
        };

        self.active_pairs.insert(pair_id, pair);
        self.pair_performance_metrics
            .insert(pair_id, PairMetrics::default());

        info!(
            "Created agent pair {} with primary {} and verifier {}",
            pair_id, primary_agent, verification_agent
        );

        Ok(pair_id)
    }

    /// Select the optimal agent pair for a given task
    pub fn select_optimal_pair(
        &self,
        task: &VerifiableTask,
        available_agents: &[Agent],
    ) -> Result<Option<Uuid>> {
        let mut best_pair_id = None;
        let mut best_score = 0.0;

        for (pair_id, pair) in &self.active_pairs {
            // Check if both agents are available and capable
            let primary_agent = available_agents.iter().find(|a| a.id == pair.primary_agent);
            let verification_agent = available_agents
                .iter()
                .find(|a| a.id == pair.verification_agent);

            if let (Some(primary), Some(verifier)) = (primary_agent, verification_agent) {
                // Calculate suitability score
                let primary_fitness = primary.calculate_task_fitness(&task.base_task);
                let verifier_capability =
                    PairCoordinator::calculate_verification_capability(verifier, task);
                let pair_trust = pair.trust_score;
                let specialization_match =
                    if task.base_task.task_type.contains(&pair.specialization) {
                        0.2
                    } else {
                        0.0
                    };

                let total_score = primary_fitness * 0.4
                    + verifier_capability * 0.3
                    + pair_trust * 0.2
                    + specialization_match;

                if total_score > best_score {
                    best_score = total_score;
                    best_pair_id = Some(*pair_id);
                }
            }
        }

        Ok(best_pair_id)
    }

    /// Calculate an agent's capability for verification tasks
    fn calculate_verification_capability(agent: &Agent, _task: &VerifiableTask) -> f64 {
        let mut score = 0.0;

        // Base capability from agent's general proficiency
        let avg_proficiency = if agent.capabilities.is_empty() {
            0.5
        } else {
            agent
                .capabilities
                .iter()
                .map(|c| c.proficiency)
                .sum::<f64>()
                / agent.capabilities.len() as f64
        };
        score += avg_proficiency * 0.4;

        // Experience bonus
        score += (agent.memory.experiences.len() as f64 / 100.0).min(0.3);

        // Energy level
        score += (agent.energy / 100.0) * 0.2;

        // Specialization match for verification
        match &agent.agent_type {
            crate::agents::AgentType::Coordinator => score += 0.1, // Good at oversight
            crate::agents::AgentType::Learner => score += 0.15,    // Good at analysis
            _ => {}
        }

        score.min(1.0)
    }

    /// Update pair performance metrics after task completion
    pub fn update_pair_metrics(
        &mut self,
        pair_id: Uuid,
        verification_result: &VerificationResult,
        verification_duration: chrono::Duration,
    ) -> Result<()> {
        if let Some(metrics) = self.pair_performance_metrics.get_mut(&pair_id) {
            metrics.total_tasks += 1;

            match verification_result.verification_status {
                VerificationStatus::Verified | VerificationStatus::PartiallyVerified => {
                    metrics.successful_verifications += 1;
                }
                VerificationStatus::Failed | VerificationStatus::VerificationError => {
                    metrics.failed_verifications += 1;
                }
                _ => {} // Inconclusive cases don't count as success or failure
            }

            // Update running averages
            let task_count = f64::from(metrics.total_tasks);
            metrics.average_verification_time = (metrics.average_verification_time
                * (task_count - 1.0)
                + verification_duration.num_milliseconds() as f64)
                / task_count;

            metrics.average_confidence = (metrics.average_confidence * (task_count - 1.0)
                + verification_result.verification_confidence)
                / task_count;

            // Update trust score for the pair
            if let Some(pair) = self.active_pairs.get_mut(&pair_id) {
                let success_rate =
                    f64::from(metrics.successful_verifications) / f64::from(metrics.total_tasks);
                pair.trust_score = (pair.trust_score * 0.8) + (success_rate * 0.2);
                pair.last_used = Utc::now();
                pair.verification_history.push(verification_result.clone());

                // Limit history size
                if pair.verification_history.len() > 100 {
                    pair.verification_history.remove(0);
                }
            }

            debug!(
                "Updated metrics for pair {}: success rate {:.2}%",
                pair_id,
                (f64::from(metrics.successful_verifications) / f64::from(metrics.total_tasks))
                    * 100.0
            );
        }

        Ok(())
    }

    /// Get performance statistics for all pairs
    #[must_use]
    pub fn get_pair_statistics(&self) -> HashMap<Uuid, PairMetrics> {
        self.pair_performance_metrics.clone()
    }

    /// Remove underperforming pairs
    pub fn cleanup_underperforming_pairs(&mut self, min_trust_threshold: f64) -> Result<usize> {
        let mut removed_count = 0;
        let mut pairs_to_remove = Vec::new();

        for (pair_id, pair) in &self.active_pairs {
            if pair.trust_score < min_trust_threshold && pair.verification_history.len() >= 10 {
                pairs_to_remove.push(*pair_id);
            }
        }

        for pair_id in pairs_to_remove {
            self.active_pairs.remove(&pair_id);
            self.pair_performance_metrics.remove(&pair_id);
            removed_count += 1;
            warn!("Removed underperforming pair: {}", pair_id);
        }

        if removed_count > 0 {
            info!("Cleaned up {} underperforming agent pairs", removed_count);
        }

        Ok(removed_count)
    }
}

impl VerifiableTask {
    /// Create a new verifiable task from a base task
    #[must_use]
    pub fn from_task(
        base_task: Task,
        original_goal: String,
        verification_level: VerificationLevel,
    ) -> Self {
        let success_criteria = Self::generate_default_criteria(&base_task, &original_goal);
        let verification_requirements = Self::generate_requirements(&verification_level);

        Self {
            base_task,
            original_goal,
            success_criteria,
            verification_requirements,
            verification_history: Vec::new(),
            verification_level,
        }
    }

    /// Generate default success criteria based on task and goal
    fn generate_default_criteria(_task: &Task, original_goal: &str) -> Vec<SuccessCriterion> {
        let mut criteria = Vec::new();

        // Basic completion criterion
        criteria.push(SuccessCriterion {
            criterion_id: Uuid::new_v4(),
            description: "Task completion".to_string(),
            measurable_outcome: "Task is marked as completed successfully".to_string(),
            verification_method: VerificationMethod::OutputAnalysis,
            weight: 0.3,
            threshold: 0.8,
        });

        // Goal alignment criterion
        criteria.push(SuccessCriterion {
            criterion_id: Uuid::new_v4(),
            description: "Goal alignment".to_string(),
            measurable_outcome: format!("Result aligns with original goal: {original_goal}"),
            verification_method: VerificationMethod::GoalAlignment,
            weight: 0.5,
            threshold: 0.7,
        });

        // Quality criterion
        criteria.push(SuccessCriterion {
            criterion_id: Uuid::new_v4(),
            description: "Output quality".to_string(),
            measurable_outcome: "Output meets quality standards".to_string(),
            verification_method: VerificationMethod::QualityAssessment,
            weight: 0.2,
            threshold: 0.6,
        });

        criteria
    }

    /// Generate verification requirements based on verification level
    fn generate_requirements(level: &VerificationLevel) -> VerificationRequirements {
        match level {
            VerificationLevel::None => VerificationRequirements {
                min_confidence: 0.0,
                max_verification_time: chrono::Duration::seconds(1),
                require_multiple_methods: false,
                require_human_review_on_conflict: false,
                required_methods: vec![],
            },
            VerificationLevel::Basic => VerificationRequirements {
                min_confidence: 0.6,
                max_verification_time: chrono::Duration::seconds(30),
                require_multiple_methods: false,
                require_human_review_on_conflict: false,
                required_methods: vec![VerificationMethod::GoalAlignment],
            },
            VerificationLevel::Standard => VerificationRequirements {
                min_confidence: 0.8,
                max_verification_time: chrono::Duration::minutes(5),
                require_multiple_methods: true,
                require_human_review_on_conflict: false,
                required_methods: vec![
                    VerificationMethod::GoalAlignment,
                    VerificationMethod::QualityAssessment,
                ],
            },
            VerificationLevel::Comprehensive => VerificationRequirements {
                min_confidence: 0.9,
                max_verification_time: chrono::Duration::minutes(15),
                require_multiple_methods: true,
                require_human_review_on_conflict: true,
                required_methods: vec![
                    VerificationMethod::GoalAlignment,
                    VerificationMethod::QualityAssessment,
                    VerificationMethod::IndependentExecution,
                ],
            },
        }
    }

    /// Check if verification requirements are met
    #[must_use]
    pub fn meets_verification_requirements(&self, result: &VerificationResult) -> bool {
        // Check confidence threshold
        if result.verification_confidence < self.verification_requirements.min_confidence {
            return false;
        }

        // Check if required methods were used
        if self
            .verification_requirements
            .required_methods
            .contains(&result.method_used)
        {
            return true;
        }

        // For multiple methods requirement, this would need additional logic
        // to track multiple verification results
        true
    }
}

/// Combined result of task execution and verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedTaskResult {
    /// The original task execution result
    pub execution_result: crate::tasks::TaskResult,
    /// The verification result from the verification agent
    pub verification_result: VerificationResult,
    /// Overall status combining execution and verification
    pub overall_status: OverallTaskStatus,
    /// Final confidence score combining execution and verification
    pub final_confidence: f64,
    /// Whether the task meets all requirements
    pub meets_requirements: bool,
    /// Timestamp when verification was completed
    pub verified_at: DateTime<Utc>,
}

/// Overall status of a verified task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverallTaskStatus {
    /// Task executed successfully and verification passed
    FullyVerified,
    /// Task executed successfully but verification found issues
    ExecutedButUnverified,
    /// Task failed execution
    ExecutionFailed,
    /// Task executed but verification failed
    VerificationFailed,
    /// Verification process encountered errors
    VerificationError,
    /// Requires human review to resolve conflicts
    RequiresReview,
}

impl Default for VerificationRequirements {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            max_verification_time: chrono::Duration::minutes(2),
            require_multiple_methods: false,
            require_human_review_on_conflict: false,
            required_methods: vec![VerificationMethod::GoalAlignment],
        }
    }
}

impl VerifiedTaskResult {
    /// Create a new verified task result
    #[must_use]
    pub fn new(
        execution_result: crate::tasks::TaskResult,
        verification_result: VerificationResult,
    ) -> Self {
        let overall_status =
            Self::determine_overall_status(&execution_result, &verification_result);
        let final_confidence =
            Self::calculate_final_confidence(&execution_result, &verification_result);
        let meets_requirements = Self::check_requirements(&execution_result, &verification_result);

        Self {
            execution_result,
            verification_result,
            overall_status,
            final_confidence,
            meets_requirements,
            verified_at: Utc::now(),
        }
    }

    fn determine_overall_status(
        execution_result: &crate::tasks::TaskResult,
        verification_result: &VerificationResult,
    ) -> OverallTaskStatus {
        if !execution_result.success {
            return OverallTaskStatus::ExecutionFailed;
        }

        match verification_result.verification_status {
            VerificationStatus::Verified => OverallTaskStatus::FullyVerified,
            VerificationStatus::PartiallyVerified | VerificationStatus::Inconclusive => {
                OverallTaskStatus::ExecutedButUnverified
            }
            VerificationStatus::Failed => OverallTaskStatus::VerificationFailed,
            VerificationStatus::RequiresReview => OverallTaskStatus::RequiresReview,
            VerificationStatus::VerificationError => OverallTaskStatus::VerificationError,
        }
    }

    fn calculate_final_confidence(
        execution_result: &crate::tasks::TaskResult,
        verification_result: &VerificationResult,
    ) -> f64 {
        let execution_confidence = if execution_result.success {
            execution_result.quality_score.unwrap_or(0.8)
        } else {
            0.2
        };

        // Combine execution and verification confidence
        (execution_confidence * 0.4 + verification_result.verification_confidence * 0.6)
            .clamp(0.0, 1.0)
    }

    fn check_requirements(
        execution_result: &crate::tasks::TaskResult,
        verification_result: &VerificationResult,
    ) -> bool {
        execution_result.success
            && matches!(
                verification_result.verification_status,
                VerificationStatus::Verified | VerificationStatus::PartiallyVerified
            )
    }
}
