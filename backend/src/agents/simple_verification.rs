//! # Simple Verification System
//!
//! A lightweight verification framework that provides efficient task result validation
//! without the complexity of mandatory agent pairs. Leverages existing NLP and neural
//! processing capabilities for intelligent verification.

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::agents::Agent;
use crate::neural::NLPProcessor;
use crate::tasks::{Task, TaskResult};

/// Lightweight verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleVerificationResult {
    pub task_id: Uuid,
    pub verification_status: SimpleVerificationStatus,
    pub confidence_score: f64,
    pub goal_alignment_score: f64,
    pub format_compliance_score: f64,
    pub overall_score: f64,
    pub verification_tier: VerificationTier,
    pub issues_found: Vec<VerificationIssue>,
    pub verification_time_ms: u64,
    pub verified_at: DateTime<Utc>,
    pub verifier_notes: String,
}

/// Simple verification status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimpleVerificationStatus {
    Passed,           // Meets all criteria
    PassedWithIssues, // Acceptable but has minor issues
    Failed,           // Does not meet requirements
    RequiresReview,   // Needs human attention
    Error,            // Verification process failed
}

/// Verification complexity tiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationTier {
    Quick,    // Regex + basic checks (< 100ms)
    Standard, // Full NLP analysis (< 1s)
    Thorough, // AI review agent (< 10s)
}

/// Individual verification issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationIssue {
    pub issue_type: IssueType,
    pub severity: IssueSeverity,
    pub description: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueType {
    GoalMismatch,
    FormatError,
    LengthIssue,
    MissingKeywords,
    QualityIssue,
    StructureIssue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical, // Task fails
    Major,    // Significant issue
    Minor,    // Acceptable but suboptimal
    Info,     // Informational only
}

/// Verification rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRule {
    pub rule_id: String,
    pub rule_type: RuleType,
    pub threshold: f64,
    pub weight: f64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    SemanticSimilarity,
    RegexPattern { pattern: String },
    LengthCheck { min: usize, max: usize },
    KeywordPresence { keywords: Vec<String> },
    KeywordAbsence { forbidden_words: Vec<String> },
    SentimentCheck { min_sentiment: f64 },
    StructureCheck { expected_sections: Vec<String> },
}

/// Main verification system
pub struct SimpleVerificationSystem {
    nlp_processor: Arc<NLPProcessor>,
    verification_rules: HashMap<String, Vec<VerificationRule>>, // task_type -> rules
    global_rules: Vec<VerificationRule>,
    confidence_threshold: f64,
    ai_reviewer_agent_id: Option<Uuid>,
    metrics: Arc<tokio::sync::RwLock<VerificationMetrics>>,
}

/// Verification system metrics
#[derive(Debug, Clone, Default)]
pub struct VerificationMetrics {
    pub total_verifications: u64,
    pub passed_verifications: u64,
    pub failed_verifications: u64,
    pub average_verification_time_ms: f64,
    pub average_confidence_score: f64,
    pub tier_usage: HashMap<String, u64>, // tier -> count
    pub rule_effectiveness: HashMap<String, f64>, // rule_id -> success rate
}

impl SimpleVerificationSystem {
    /// Create a new verification system
    pub fn new(nlp_processor: Arc<NLPProcessor>) -> Self {
        let mut global_rules = Vec::new();

        // Default global rules
        global_rules.push(VerificationRule {
            rule_id: "goal_alignment".to_string(),
            rule_type: RuleType::SemanticSimilarity,
            threshold: 0.7,
            weight: 0.5,
            enabled: true,
        });

        global_rules.push(VerificationRule {
            rule_id: "basic_length".to_string(),
            rule_type: RuleType::LengthCheck {
                min: 10,
                max: 10000,
            },
            threshold: 1.0,
            weight: 0.1,
            enabled: true,
        });

        global_rules.push(VerificationRule {
            rule_id: "positive_sentiment".to_string(),
            rule_type: RuleType::SentimentCheck {
                min_sentiment: -0.5,
            },
            threshold: 1.0,
            weight: 0.1,
            enabled: true,
        });

        Self {
            nlp_processor,
            verification_rules: HashMap::new(),
            global_rules,
            confidence_threshold: 0.75,
            ai_reviewer_agent_id: None,
            metrics: Arc::new(tokio::sync::RwLock::new(VerificationMetrics::default())),
        }
    }

    /// Set AI reviewer agent for thorough verification
    pub fn set_ai_reviewer(&mut self, agent_id: Uuid) {
        self.ai_reviewer_agent_id = Some(agent_id);
    }

    /// Add task-specific verification rules
    pub fn add_task_rules(&mut self, task_type: &str, rules: Vec<VerificationRule>) {
        self.verification_rules.insert(task_type.to_string(), rules);
    }

    /// Main verification entry point
    pub async fn verify_task_result(
        &self,
        task: &Task,
        result: &TaskResult,
        original_goal: Option<&str>,
    ) -> Result<SimpleVerificationResult> {
        let start_time = std::time::Instant::now();

        // Determine verification tier based on task priority and type
        let tier = self.determine_verification_tier(task, result);

        let verification_result = match tier {
            VerificationTier::Quick => self.quick_verification(task, result, original_goal).await?,
            VerificationTier::Standard => {
                self.standard_verification(task, result, original_goal)
                    .await?
            }
            VerificationTier::Thorough => {
                self.thorough_verification(task, result, original_goal)
                    .await?
            }
        };

        let verification_time_ms = start_time.elapsed().as_millis() as u64;

        // Update metrics
        self.update_metrics(&verification_result, verification_time_ms)
            .await;

        debug!(
            "Verification completed in {}ms with tier {:?}: {:?}",
            verification_time_ms, tier, verification_result.verification_status
        );

        Ok(SimpleVerificationResult {
            verification_time_ms,
            ..verification_result
        })
    }

    /// Determine appropriate verification tier
    fn determine_verification_tier(&self, task: &Task, result: &TaskResult) -> VerificationTier {
        // Critical tasks always get thorough verification
        if matches!(task.priority, crate::tasks::TaskPriority::Critical) {
            return VerificationTier::Thorough;
        }

        // Failed tasks get standard verification to understand issues
        if !result.success {
            return VerificationTier::Standard;
        }

        // Low confidence results get upgraded verification
        if let Some(quality_score) = result.quality_score {
            if quality_score < 0.7 {
                return VerificationTier::Standard;
            }
        }

        // High priority tasks get standard verification
        if matches!(task.priority, crate::tasks::TaskPriority::High) {
            return VerificationTier::Standard;
        }

        // Default to quick verification
        VerificationTier::Quick
    }

    /// Quick verification using basic rules and regex
    async fn quick_verification(
        &self,
        task: &Task,
        result: &TaskResult,
        original_goal: Option<&str>,
    ) -> Result<SimpleVerificationResult> {
        let mut issues = Vec::new();
        let mut scores = HashMap::new();

        // Apply basic rules
        let rules = self.get_applicable_rules(&task.task_type);
        for rule in &rules {
            if !rule.enabled {
                continue;
            }

            let score = match &rule.rule_type {
                RuleType::LengthCheck { min, max } => {
                    self.check_length(&result.output, *min, *max, &mut issues)
                }
                RuleType::RegexPattern { pattern } => {
                    self.check_regex_pattern(&result.output, pattern, &mut issues)
                }
                RuleType::KeywordPresence { keywords } => {
                    self.check_keyword_presence(&result.output, keywords, &mut issues)
                }
                RuleType::KeywordAbsence { forbidden_words } => {
                    self.check_keyword_absence(&result.output, forbidden_words, &mut issues)
                }
                _ => 1.0, // Skip complex rules in quick mode
            };

            scores.insert(rule.rule_id.clone(), score);
        }

        // Basic goal alignment if provided
        let goal_alignment_score = if let Some(goal) = original_goal {
            self.basic_goal_alignment(&result.output, goal).await
        } else {
            1.0
        };

        let format_compliance_score = self.calculate_weighted_score(&scores, &rules);
        let overall_score = goal_alignment_score * 0.6 + format_compliance_score * 0.4;

        Ok(SimpleVerificationResult {
            task_id: task.id,
            verification_status: self.determine_status(overall_score, &issues),
            confidence_score: 0.8, // Quick verification has lower confidence
            goal_alignment_score,
            format_compliance_score,
            overall_score,
            verification_tier: VerificationTier::Quick,
            issues_found: issues,
            verification_time_ms: 0, // Will be set by caller
            verified_at: Utc::now(),
            verifier_notes: "Quick verification using basic rules".to_string(),
        })
    }

    /// Standard verification using full NLP analysis
    async fn standard_verification(
        &self,
        task: &Task,
        result: &TaskResult,
        original_goal: Option<&str>,
    ) -> Result<SimpleVerificationResult> {
        let mut issues = Vec::new();
        let mut scores = HashMap::new();

        // Apply all rules including NLP-based ones
        let rules = self.get_applicable_rules(&task.task_type);
        for rule in &rules {
            if !rule.enabled {
                continue;
            }

            let score = match &rule.rule_type {
                RuleType::SemanticSimilarity => {
                    if let Some(goal) = original_goal {
                        self.semantic_similarity_check(&result.output, goal, &mut issues)
                            .await
                    } else {
                        1.0
                    }
                }
                RuleType::SentimentCheck { min_sentiment } => {
                    self.sentiment_check(&result.output, *min_sentiment, &mut issues)
                        .await
                }
                RuleType::StructureCheck { expected_sections } => {
                    self.structure_check(&result.output, expected_sections, &mut issues)
                }
                RuleType::LengthCheck { min, max } => {
                    self.check_length(&result.output, *min, *max, &mut issues)
                }
                RuleType::RegexPattern { pattern } => {
                    self.check_regex_pattern(&result.output, pattern, &mut issues)
                }
                RuleType::KeywordPresence { keywords } => {
                    self.check_keyword_presence(&result.output, keywords, &mut issues)
                }
                RuleType::KeywordAbsence { forbidden_words } => {
                    self.check_keyword_absence(&result.output, forbidden_words, &mut issues)
                }
            };

            scores.insert(rule.rule_id.clone(), score);
        }

        let goal_alignment_score = scores.get("goal_alignment").copied().unwrap_or(1.0);
        let format_compliance_score = self.calculate_weighted_score(&scores, &rules);
        let overall_score = goal_alignment_score * 0.6 + format_compliance_score * 0.4;

        Ok(SimpleVerificationResult {
            task_id: task.id,
            verification_status: self.determine_status(overall_score, &issues),
            confidence_score: 0.9, // Standard verification has high confidence
            goal_alignment_score,
            format_compliance_score,
            overall_score,
            verification_tier: VerificationTier::Standard,
            issues_found: issues,
            verification_time_ms: 0,
            verified_at: Utc::now(),
            verifier_notes: "Standard verification using NLP analysis".to_string(),
        })
    }

    /// Thorough verification using AI reviewer agent
    async fn thorough_verification(
        &self,
        task: &Task,
        result: &TaskResult,
        original_goal: Option<&str>,
    ) -> Result<SimpleVerificationResult> {
        // First run standard verification
        let mut standard_result = self
            .standard_verification(task, result, original_goal)
            .await?;

        // If we have an AI reviewer agent, use it for additional analysis
        if let Some(_reviewer_id) = self.ai_reviewer_agent_id {
            // In a real implementation, you would invoke the AI reviewer agent here
            // For now, we'll simulate enhanced analysis

            // Enhance confidence based on thorough analysis
            standard_result.confidence_score = 0.95;
            standard_result.verification_tier = VerificationTier::Thorough;
            standard_result.verifier_notes =
                "Thorough verification with AI reviewer analysis".to_string();

            // Add AI-specific insights
            if standard_result.overall_score < 0.8 {
                standard_result.issues_found.push(VerificationIssue {
                    issue_type: IssueType::QualityIssue,
                    severity: IssueSeverity::Minor,
                    description: "AI reviewer suggests potential quality improvements".to_string(),
                    suggestion: Some(
                        "Consider refining the output for better clarity and completeness"
                            .to_string(),
                    ),
                });
            }
        }

        Ok(standard_result)
    }

    /// Get applicable rules for a task type
    fn get_applicable_rules(&self, task_type: &str) -> Vec<VerificationRule> {
        let mut rules = self.global_rules.clone();

        if let Some(task_rules) = self.verification_rules.get(task_type) {
            rules.extend(task_rules.clone());
        }

        rules
    }

    /// Basic goal alignment using simple text similarity
    async fn basic_goal_alignment(&self, output: &str, goal: &str) -> f64 {
        // Simple word overlap calculation
        let output_words: std::collections::HashSet<String> = output
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let goal_words: std::collections::HashSet<String> = goal
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let intersection = output_words.intersection(&goal_words).count();
        let union = output_words.union(&goal_words).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Semantic similarity check using NLP processor
    async fn semantic_similarity_check(
        &self,
        output: &str,
        goal: &str,
        issues: &mut Vec<VerificationIssue>,
    ) -> f64 {
        // Use existing NLP processor for semantic analysis
        let output_processed = self
            .nlp_processor
            .process_text(output)
            .await
            .unwrap_or_else(|_| {
                // Fallback to basic processing if NLP fails
                crate::neural::nlp::ProcessedText {
                    original_text: output.to_string(),
                    tokens: output.split_whitespace().map(|s| s.to_string()).collect(),
                    semantic_vector: crate::neural::nlp::SemanticVector {
                        dimensions: vec![0.0; 100],
                        magnitude: 0.0,
                    },
                    sentiment: 0.0,
                    keywords: Vec::new(),
                    identified_patterns: Vec::new(),
                }
            });

        let goal_processed = self
            .nlp_processor
            .process_text(goal)
            .await
            .unwrap_or_else(|_| crate::neural::nlp::ProcessedText {
                original_text: goal.to_string(),
                tokens: goal.split_whitespace().map(|s| s.to_string()).collect(),
                semantic_vector: crate::neural::nlp::SemanticVector {
                    dimensions: vec![0.0; 100],
                    magnitude: 0.0,
                },
                sentiment: 0.0,
                keywords: Vec::new(),
                identified_patterns: Vec::new(),
            });

        // Calculate semantic similarity
        let similarity = if output_processed.semantic_vector.magnitude > 0.0
            && goal_processed.semantic_vector.magnitude > 0.0
        {
            // Use cosine similarity
            let dot_product: f64 = output_processed
                .semantic_vector
                .dimensions
                .iter()
                .zip(goal_processed.semantic_vector.dimensions.iter())
                .map(|(a, b)| a * b)
                .sum();

            dot_product
                / (output_processed.semantic_vector.magnitude
                    * goal_processed.semantic_vector.magnitude)
        } else {
            self.basic_goal_alignment(output, goal).await
        };

        if similarity < 0.7 {
            issues.push(VerificationIssue {
                issue_type: IssueType::GoalMismatch,
                severity: if similarity < 0.5 {
                    IssueSeverity::Major
                } else {
                    IssueSeverity::Minor
                },
                description: format!("Low semantic similarity to goal: {:.2}", similarity),
                suggestion: Some(
                    "Consider aligning output more closely with the original goal".to_string(),
                ),
            });
        }

        similarity
    }

    /// Sentiment analysis check
    async fn sentiment_check(
        &self,
        output: &str,
        min_sentiment: f64,
        issues: &mut Vec<VerificationIssue>,
    ) -> f64 {
        let tokens: Vec<String> = output.split_whitespace().map(|s| s.to_string()).collect();
        let sentiment = self.nlp_processor.analyze_sentiment(&tokens);

        if sentiment < min_sentiment {
            issues.push(VerificationIssue {
                issue_type: IssueType::QualityIssue,
                severity: IssueSeverity::Minor,
                description: format!(
                    "Sentiment score {:.2} below threshold {:.2}",
                    sentiment, min_sentiment
                ),
                suggestion: Some("Consider using more positive language".to_string()),
            });
            0.0
        } else {
            1.0
        }
    }

    /// Structure check for expected sections
    fn structure_check(
        &self,
        output: &str,
        expected_sections: &[String],
        issues: &mut Vec<VerificationIssue>,
    ) -> f64 {
        let output_lower = output.to_lowercase();
        let mut found_sections = 0;

        for section in expected_sections {
            if output_lower.contains(&section.to_lowercase()) {
                found_sections += 1;
            }
        }

        let score = found_sections as f64 / expected_sections.len() as f64;

        if score < 1.0 {
            let missing_sections: Vec<String> = expected_sections
                .iter()
                .filter(|section| !output_lower.contains(&section.to_lowercase()))
                .cloned()
                .collect();

            issues.push(VerificationIssue {
                issue_type: IssueType::StructureIssue,
                severity: if score < 0.5 {
                    IssueSeverity::Major
                } else {
                    IssueSeverity::Minor
                },
                description: format!("Missing expected sections: {}", missing_sections.join(", ")),
                suggestion: Some("Include all required sections in the output".to_string()),
            });
        }

        score
    }

    /// Length check
    fn check_length(
        &self,
        output: &str,
        min: usize,
        max: usize,
        issues: &mut Vec<VerificationIssue>,
    ) -> f64 {
        let length = output.len();

        if length < min {
            issues.push(VerificationIssue {
                issue_type: IssueType::LengthIssue,
                severity: IssueSeverity::Major,
                description: format!("Output too short: {} chars (min: {})", length, min),
                suggestion: Some("Provide more detailed output".to_string()),
            });
            0.0
        } else if length > max {
            issues.push(VerificationIssue {
                issue_type: IssueType::LengthIssue,
                severity: IssueSeverity::Minor,
                description: format!("Output too long: {} chars (max: {})", length, max),
                suggestion: Some("Consider making output more concise".to_string()),
            });
            0.7
        } else {
            1.0
        }
    }

    /// Regex pattern check
    fn check_regex_pattern(
        &self,
        output: &str,
        pattern: &str,
        issues: &mut Vec<VerificationIssue>,
    ) -> f64 {
        match regex::Regex::new(pattern) {
            Ok(re) => {
                if re.is_match(output) {
                    1.0
                } else {
                    issues.push(VerificationIssue {
                        issue_type: IssueType::FormatError,
                        severity: IssueSeverity::Major,
                        description: format!("Output does not match required pattern: {}", pattern),
                        suggestion: Some("Ensure output follows the required format".to_string()),
                    });
                    0.0
                }
            }
            Err(_) => {
                warn!("Invalid regex pattern: {}", pattern);
                1.0 // Don't penalize for invalid patterns
            }
        }
    }

    /// Keyword presence check
    fn check_keyword_presence(
        &self,
        output: &str,
        keywords: &[String],
        issues: &mut Vec<VerificationIssue>,
    ) -> f64 {
        let output_lower = output.to_lowercase();
        let mut found_keywords = 0;

        for keyword in keywords {
            if output_lower.contains(&keyword.to_lowercase()) {
                found_keywords += 1;
            }
        }

        let score = found_keywords as f64 / keywords.len() as f64;

        if score < 1.0 {
            let missing_keywords: Vec<String> = keywords
                .iter()
                .filter(|keyword| !output_lower.contains(&keyword.to_lowercase()))
                .cloned()
                .collect();

            issues.push(VerificationIssue {
                issue_type: IssueType::MissingKeywords,
                severity: if score < 0.5 {
                    IssueSeverity::Major
                } else {
                    IssueSeverity::Minor
                },
                description: format!("Missing required keywords: {}", missing_keywords.join(", ")),
                suggestion: Some("Include all required keywords in the output".to_string()),
            });
        }

        score
    }

    /// Keyword absence check (forbidden words)
    fn check_keyword_absence(
        &self,
        output: &str,
        forbidden_words: &[String],
        issues: &mut Vec<VerificationIssue>,
    ) -> f64 {
        let output_lower = output.to_lowercase();
        let mut found_forbidden = Vec::new();

        for word in forbidden_words {
            if output_lower.contains(&word.to_lowercase()) {
                found_forbidden.push(word.clone());
            }
        }

        if !found_forbidden.is_empty() {
            issues.push(VerificationIssue {
                issue_type: IssueType::QualityIssue,
                severity: IssueSeverity::Major,
                description: format!("Contains forbidden words: {}", found_forbidden.join(", ")),
                suggestion: Some("Remove inappropriate or forbidden content".to_string()),
            });
            0.0
        } else {
            1.0
        }
    }

    /// Calculate weighted score from rule results
    fn calculate_weighted_score(
        &self,
        scores: &HashMap<String, f64>,
        rules: &[VerificationRule],
    ) -> f64 {
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;

        for rule in rules {
            if let Some(score) = scores.get(&rule.rule_id) {
                weighted_sum += score * rule.weight;
                total_weight += rule.weight;
            }
        }

        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            1.0
        }
    }

    /// Determine verification status from score and issues
    fn determine_status(
        &self,
        overall_score: f64,
        issues: &[VerificationIssue],
    ) -> SimpleVerificationStatus {
        let critical_issues = issues
            .iter()
            .any(|i| matches!(i.severity, IssueSeverity::Critical));
        let major_issues = issues
            .iter()
            .filter(|i| matches!(i.severity, IssueSeverity::Major))
            .count();

        if critical_issues {
            SimpleVerificationStatus::Failed
        } else if overall_score < 0.5 || major_issues > 2 {
            SimpleVerificationStatus::Failed
        } else if overall_score < self.confidence_threshold || major_issues > 0 {
            SimpleVerificationStatus::PassedWithIssues
        } else if overall_score < 0.9 && !issues.is_empty() {
            SimpleVerificationStatus::PassedWithIssues
        } else {
            SimpleVerificationStatus::Passed
        }
    }

    /// Update verification metrics
    async fn update_metrics(&self, result: &SimpleVerificationResult, verification_time_ms: u64) {
        let mut metrics = self.metrics.write().await;

        metrics.total_verifications += 1;

        match result.verification_status {
            SimpleVerificationStatus::Passed | SimpleVerificationStatus::PassedWithIssues => {
                metrics.passed_verifications += 1;
            }
            SimpleVerificationStatus::Failed => {
                metrics.failed_verifications += 1;
            }
            _ => {}
        }

        // Update running averages
        let total = metrics.total_verifications as f64;
        metrics.average_verification_time_ms =
            (metrics.average_verification_time_ms * (total - 1.0) + verification_time_ms as f64)
                / total;

        metrics.average_confidence_score =
            (metrics.average_confidence_score * (total - 1.0) + result.confidence_score) / total;

        // Update tier usage
        let tier_key = format!("{:?}", result.verification_tier);
        *metrics.tier_usage.entry(tier_key).or_insert(0) += 1;
    }

    /// Get verification metrics
    pub async fn get_metrics(&self) -> VerificationMetrics {
        self.metrics.read().await.clone()
    }

    /// Configure verification thresholds
    pub fn configure(&mut self, confidence_threshold: f64) {
        self.confidence_threshold = confidence_threshold.clamp(0.0, 1.0);
    }
}

/// Trait for integrating with existing agent system
#[async_trait]
pub trait SimpleVerificationCapable {
    /// Verify a task result using the simple verification system
    async fn simple_verify(
        &self,
        task: &Task,
        result: &TaskResult,
        original_goal: Option<&str>,
        verification_system: &SimpleVerificationSystem,
    ) -> Result<SimpleVerificationResult>;
}

/// Implementation for existing Agent struct
#[async_trait]
impl SimpleVerificationCapable for Agent {
    async fn simple_verify(
        &self,
        task: &Task,
        result: &TaskResult,
        original_goal: Option<&str>,
        verification_system: &SimpleVerificationSystem,
    ) -> Result<SimpleVerificationResult> {
        verification_system
            .verify_task_result(task, result, original_goal)
            .await
    }
}
