use crate::neural::{CpuOptimizer, VectorizedOps};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLPInsight {
    pub pattern: String,
    pub confidence: f64,
    pub context: String,
    pub learned_from: Vec<Uuid>, // agent IDs that contributed to this insight
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticVector {
    pub dimensions: Vec<f64>,
    pub magnitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguagePattern {
    pub pattern_id: Uuid,
    pub pattern_type: PatternType,
    pub keywords: Vec<String>,
    pub semantic_vector: SemanticVector,
    pub success_rate: f64,
    pub usage_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    TaskDescription,
    SuccessIndicator,
    FailureIndicator,
    CommunicationStyle,
    LearningCue,
}

#[derive(Debug)]
pub struct NLPProcessor {
    pub learned_patterns: RwLock<HashMap<String, LanguagePattern>>,
    pub insights: RwLock<Vec<NLPInsight>>,
    pub vocabulary: RwLock<HashMap<String, f64>>, // word -> importance score
    pub semantic_cache: RwLock<HashMap<String, SemanticVector>>,
    pub cpu_optimizer: CpuOptimizer,
}

impl NLPProcessor {
    pub async fn new() -> Result<Self> {
        let optimizer = CpuOptimizer::new();
        tracing::info!(
            "🚀 CPU Optimizer initialized with SIMD support: AVX2={}, NEON={}",
            optimizer.simd_support.avx2,
            optimizer.simd_support.neon
        );

        Ok(Self {
            learned_patterns: RwLock::new(HashMap::new()),
            insights: RwLock::new(Vec::new()),
            vocabulary: RwLock::new(HashMap::new()),
            semantic_cache: RwLock::new(HashMap::new()),
            cpu_optimizer: optimizer,
        })
    }

    pub async fn process_text(&self, text: &str) -> Result<ProcessedText> {
        let tokens = self.tokenize(text);
        let semantic_vector = self.generate_semantic_vector(&tokens).await;
        let sentiment = self.analyze_sentiment(&tokens);
        let keywords = self.extract_keywords(text, 5).await;
        let patterns = self.identify_patterns(&tokens).await;

        Ok(ProcessedText {
            original_text: text.to_string(),
            tokens,
            semantic_vector,
            sentiment,
            keywords,
            identified_patterns: patterns,
        })
    }

    pub async fn learn_from_interaction(
        &self,
        input: &str,
        output: &str,
        success: bool,
        agent_id: Uuid,
    ) -> Result<()> {
        let processed_input = self.process_text(input).await?;
        let processed_output = self.process_text(output).await?;

        // Update vocabulary based on success/failure
        self.update_vocabulary(&processed_input.tokens, success)
            .await;
        self.update_vocabulary(&processed_output.tokens, success)
            .await;

        // Learn patterns from successful interactions
        if success {
            self.learn_success_pattern(&processed_input, &processed_output, agent_id)
                .await?;
        } else {
            self.learn_failure_pattern(&processed_input, &processed_output, agent_id)
                .await?;
        }

        // Generate insights
        self.generate_insights(&processed_input, &processed_output, success, agent_id)
            .await?;

        Ok(())
    }

    pub async fn suggest_improvements(&self, text: &str) -> Result<Vec<String>> {
        let processed = self.process_text(text).await?;
        let mut suggestions = Vec::new();

        // Check against learned failure patterns
        let patterns = self.learned_patterns.read().await;
        for pattern in patterns.values() {
            if matches!(pattern.pattern_type, PatternType::FailureIndicator) {
                if self.text_matches_pattern(&processed, pattern) {
                    suggestions.push(format!(
                        "Consider avoiding patterns similar to '{}' which have a {}% failure rate",
                        pattern.keywords.join(" "),
                        (1.0 - pattern.success_rate) * 100.0
                    ));
                }
            }
        }

        // Suggest successful patterns
        for pattern in patterns.values() {
            if matches!(pattern.pattern_type, PatternType::SuccessIndicator)
                && pattern.success_rate > 0.8
            {
                suggestions.push(format!(
                    "Consider incorporating elements from successful pattern: '{}'",
                    pattern.keywords.join(" ")
                ));
            }
        }

        Ok(suggestions)
    }

    pub async fn find_similar_experiences(&self, text: &str) -> Result<Vec<NLPInsight>> {
        let processed = self.process_text(text).await?;
        let insights = self.insights.read().await;

        let mut similar_insights = Vec::new();
        for insight in insights.iter() {
            let similarity = self.calculate_semantic_similarity(
                &processed.semantic_vector,
                &self.text_to_semantic_vector(&insight.context).await,
            );

            if similarity > 0.7 {
                similar_insights.push(insight.clone());
            }
        }

        // Sort by confidence
        similar_insights.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(similar_insights)
    }

    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|word| {
                word.chars()
                    .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                    .collect::<String>()
            })
            .filter(|word: &String| !word.is_empty())
            .collect()
    }

    async fn generate_semantic_vector(&self, tokens: &[String]) -> SemanticVector {
        // CPU-optimized bag-of-words approach with learned weights
        let vocabulary = self.vocabulary.read().await;
        let mut dimensions = vec![0.0; 100]; // Fixed dimension size

        // Vectorized approach for better cache utilization
        for (i, token) in tokens.iter().enumerate() {
            let weight = vocabulary.get(token).copied().unwrap_or(0.1);
            let index = (token.len() + i) % dimensions.len();
            dimensions[index] += weight;
        }

        // Use vectorized norm calculation
        let dimensions_f32: Vec<f32> = dimensions.iter().map(|&x| x as f32).collect();
        let magnitude = VectorizedOps::vector_norm(&dimensions_f32) as f64;

        SemanticVector {
            dimensions,
            magnitude,
        }
    }

    pub fn analyze_sentiment(&self, tokens: &[String]) -> f64 {
        // Simple sentiment analysis
        let positive_words = [
            "good",
            "great",
            "excellent",
            "success",
            "complete",
            "done",
            "perfect",
        ];
        let negative_words = [
            "bad",
            "fail",
            "error",
            "wrong",
            "problem",
            "issue",
            "difficult",
        ];

        let positive_count = tokens
            .iter()
            .filter(|token| positive_words.contains(&token.as_str()))
            .count() as f64;

        let negative_count = tokens
            .iter()
            .filter(|token| negative_words.contains(&token.as_str()))
            .count() as f64;

        if positive_count + negative_count == 0.0 {
            0.0 // Neutral
        } else {
            (positive_count - negative_count) / (positive_count + negative_count)
        }
    }

    pub async fn extract_keywords(&self, text: &str, limit: usize) -> Vec<String> {
        let tokens: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
        let vocabulary = self.vocabulary.read().await;
        let mut keyword_scores: Vec<(String, f64)> = tokens
            .iter()
            .map(|token| {
                let score = vocabulary.get(token).copied().unwrap_or(0.1);
                (token.clone(), score)
            })
            .collect();

        keyword_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        keyword_scores
            .into_iter()
            .take(limit)
            .map(|(word, _)| word)
            .collect()
    }

    async fn identify_patterns(&self, tokens: &[String]) -> Vec<Uuid> {
        let patterns = self.learned_patterns.read().await;
        let mut matching_patterns = Vec::new();

        for pattern in patterns.values() {
            let matches = pattern
                .keywords
                .iter()
                .filter(|keyword| tokens.contains(keyword))
                .count();

            if matches as f64 / pattern.keywords.len() as f64 > 0.5 {
                matching_patterns.push(pattern.pattern_id);
            }
        }

        matching_patterns
    }

    async fn update_vocabulary(&self, tokens: &[String], success: bool) {
        let mut vocabulary = self.vocabulary.write().await;
        let adjustment = if success { 0.1 } else { -0.05 };

        for token in tokens {
            let current_score = vocabulary.get(token).copied().unwrap_or(0.5);
            let new_score = (current_score + adjustment).clamp(0.0, 1.0);
            vocabulary.insert(token.clone(), new_score);
        }
    }

    async fn learn_success_pattern(
        &self,
        input: &ProcessedText,
        _output: &ProcessedText,
        _agent_id: Uuid,
    ) -> Result<()> {
        let pattern = LanguagePattern {
            pattern_id: Uuid::new_v4(),
            pattern_type: PatternType::SuccessIndicator,
            keywords: input.keywords.clone(),
            semantic_vector: input.semantic_vector.clone(),
            success_rate: 1.0,
            usage_count: 1,
        };

        let mut patterns = self.learned_patterns.write().await;
        let key = input.keywords.join("_");
        patterns.insert(key, pattern);

        Ok(())
    }

    async fn learn_failure_pattern(
        &self,
        input: &ProcessedText,
        _output: &ProcessedText,
        _agent_id: Uuid,
    ) -> Result<()> {
        let pattern = LanguagePattern {
            pattern_id: Uuid::new_v4(),
            pattern_type: PatternType::FailureIndicator,
            keywords: input.keywords.clone(),
            semantic_vector: input.semantic_vector.clone(),
            success_rate: 0.0,
            usage_count: 1,
        };

        let mut patterns = self.learned_patterns.write().await;
        let key = input.keywords.join("_");
        patterns.insert(key, pattern);

        Ok(())
    }

    async fn generate_insights(
        &self,
        input: &ProcessedText,
        output: &ProcessedText,
        success: bool,
        agent_id: Uuid,
    ) -> Result<()> {
        let insight = NLPInsight {
            pattern: format!(
                "Input: {} -> Output: {}",
                input.keywords.join(" "),
                output.keywords.join(" ")
            ),
            confidence: if success { 0.8 } else { 0.6 },
            context: input.original_text.clone(),
            learned_from: vec![agent_id],
        };

        let mut insights = self.insights.write().await;
        insights.push(insight);

        // Keep only the most recent 1000 insights
        if insights.len() > 1000 {
            insights.remove(0);
        }

        Ok(())
    }

    fn text_matches_pattern(&self, processed: &ProcessedText, pattern: &LanguagePattern) -> bool {
        let matches = pattern
            .keywords
            .iter()
            .filter(|keyword| processed.keywords.contains(keyword))
            .count();

        matches as f64 / pattern.keywords.len() as f64 > 0.3
    }

    fn calculate_semantic_similarity(&self, vec1: &SemanticVector, vec2: &SemanticVector) -> f64 {
        if vec1.magnitude == 0.0 || vec2.magnitude == 0.0 {
            return 0.0;
        }

        // Use CPU-optimized vectorized operations for better performance
        let vec1_f32: Vec<f32> = vec1.dimensions.iter().map(|&x| x as f32).collect();
        let vec2_f32: Vec<f32> = vec2.dimensions.iter().map(|&x| x as f32).collect();

        let similarity = VectorizedOps::cosine_similarity(&vec1_f32, &vec2_f32);
        similarity as f64
    }

    async fn text_to_semantic_vector(&self, text: &str) -> SemanticVector {
        let tokens = self.tokenize(text);
        self.generate_semantic_vector(&tokens).await
    }
}

#[derive(Debug, Clone)]
pub struct ProcessedText {
    pub original_text: String,
    pub tokens: Vec<String>,
    pub semantic_vector: SemanticVector,
    pub sentiment: f64,
    pub keywords: Vec<String>,
    pub identified_patterns: Vec<Uuid>,
}
