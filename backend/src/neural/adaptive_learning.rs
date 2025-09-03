use crate::agents::agent::{Agent, AgentMemory};
use crate::neural::NLPProcessor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningPattern {
    pub pattern_id: String,
    pub input_features: Vec<f64>,
    pub expected_output: Vec<f64>,
    pub confidence: f64,
    pub frequency: u32,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveLearningConfig {
    pub learning_rate: f64,
    pub momentum: f64,
    pub decay_factor: f64,
    pub min_confidence_threshold: f64,
    pub pattern_retention_days: u32,
    pub max_patterns: usize,
}

impl Default for AdaptiveLearningConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.01,
            momentum: 0.9,
            decay_factor: 0.95,
            min_confidence_threshold: 0.7,
            pattern_retention_days: 30,
            max_patterns: 10000,
        }
    }
}

pub struct AdaptiveLearningSystem {
    config: AdaptiveLearningConfig,
    patterns: HashMap<String, LearningPattern>,
    neural_processor: NLPProcessor,
    learning_history: Vec<LearningEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub pattern_id: String,
    pub outcome: f64,
    pub confidence_before: f64,
    pub confidence_after: f64,
}

impl AdaptiveLearningSystem {
    pub async fn new(config: AdaptiveLearningConfig) -> anyhow::Result<Self> {
        Ok(Self {
            config,
            patterns: HashMap::new(),
            neural_processor: NLPProcessor::new().await?,
            learning_history: Vec::new(),
        })
    }

    pub async fn learn_from_interaction(
        &mut self,
        agent: &Agent,
        context: &str,
        outcome: f64,
    ) -> anyhow::Result<()> {
        let features = self.extract_features(agent, context).await?;
        let pattern_id = self.generate_pattern_id(&features);

        debug!(
            "Learning from interaction for agent {} with outcome {}",
            agent.id, outcome
        );

        let confidence_before = self
            .patterns
            .get(&pattern_id)
            .map(|p| p.confidence)
            .unwrap_or(0.5);

        let pattern = self
            .patterns
            .entry(pattern_id.clone())
            .or_insert_with(|| LearningPattern {
                pattern_id: pattern_id.clone(),
                input_features: features.clone(),
                expected_output: vec![outcome],
                confidence: 0.5,
                frequency: 0,
                last_seen: chrono::Utc::now(),
            });

        // Update pattern with new data
        pattern.frequency += 1;
        pattern.last_seen = chrono::Utc::now();

        // Adaptive confidence calculation using exponential moving average
        let success_rate = outcome.clamp(0.0, 1.0);
        let frequency = pattern.frequency; // Store frequency before borrowing self

        // Calculate learning factor separately to avoid borrowing conflicts
        let learning_factor = {
            let base_rate = self.config.learning_rate;
            base_rate / (1.0 + (frequency as f64 * 0.1))
        };

        pattern.confidence =
            (pattern.confidence * (1.0 - learning_factor)) + (success_rate * learning_factor);

        // Update expected output with weighted average
        if !pattern.expected_output.is_empty() {
            pattern.expected_output[0] = (pattern.expected_output[0] * 0.8) + (outcome * 0.2);
        } else {
            pattern.expected_output = vec![outcome];
        }

        // Record learning event
        self.learning_history.push(LearningEvent {
            timestamp: chrono::Utc::now(),
            pattern_id: pattern_id.clone(),
            outcome,
            confidence_before,
            confidence_after: pattern.confidence,
        });

        // Update neural network if confidence is high enough
        if pattern.confidence >= self.config.min_confidence_threshold {
            // For now, we'll skip neural network training as NLPProcessor doesn't have this method
            // In a full implementation, you would add training capabilities to NLPProcessor
            debug!("High-confidence pattern recorded: {}", pattern_id);
        }

        // Cleanup if we have too many patterns
        if self.patterns.len() > self.config.max_patterns {
            self.cleanup_low_confidence_patterns();
        }

        Ok(())
    }

    pub async fn predict_outcome(&self, agent: &Agent, context: &str) -> anyhow::Result<f64> {
        let features = self.extract_features(agent, context).await?;
        let pattern_id = self.generate_pattern_id(&features);

        // Check for exact pattern match first
        if let Some(pattern) = self.patterns.get(&pattern_id) {
            if pattern.confidence >= self.config.min_confidence_threshold {
                debug!(
                    "Using cached pattern {} with confidence {}",
                    pattern_id, pattern.confidence
                );
                return Ok(pattern.expected_output[0]);
            }
        }

        // Use simple heuristic for prediction since NLPProcessor doesn't have predict method
        // In a full implementation, you would add prediction capabilities to NLPProcessor
        let prediction = features.iter().sum::<f64>() / features.len() as f64;
        debug!("Heuristic prediction: {}", prediction);
        Ok(prediction.clamp(0.0, 1.0))
    }

    pub async fn get_learning_insights(&self, agent_id: uuid::Uuid) -> LearningInsights {
        let agent_patterns: Vec<&LearningPattern> = self
            .patterns
            .values()
            .filter(|p| p.pattern_id.contains(&agent_id.to_string()[..8]))
            .collect();

        let total_patterns = agent_patterns.len();
        let high_confidence_patterns = agent_patterns
            .iter()
            .filter(|p| p.confidence >= self.config.min_confidence_threshold)
            .count();

        let average_confidence = if !agent_patterns.is_empty() {
            agent_patterns.iter().map(|p| p.confidence).sum::<f64>() / agent_patterns.len() as f64
        } else {
            0.0
        };

        let recent_learning_events = self
            .learning_history
            .iter()
            .filter(|e| e.timestamp > chrono::Utc::now() - chrono::Duration::hours(24))
            .count();

        LearningInsights {
            total_patterns,
            high_confidence_patterns,
            average_confidence,
            recent_learning_events,
            learning_velocity: self.calculate_learning_velocity(),
        }
    }

    async fn extract_features(&self, agent: &Agent, context: &str) -> anyhow::Result<Vec<f64>> {
        let mut features = Vec::new();

        // Agent features
        features.push(agent.energy);
        features.push(agent.capabilities.len() as f64);
        features.push(agent.memory.experiences.len() as f64);
        features.push(agent.memory.social_connections.len() as f64);

        // Agent type encoding
        features.push(match agent.agent_type {
            crate::agents::agent::AgentType::Worker => 0.0,
            crate::agents::agent::AgentType::Coordinator => 1.0,
            crate::agents::agent::AgentType::Specialist(_) => 2.0,
            crate::agents::agent::AgentType::Learner => 3.0,
        });

        // Position features
        features.push(agent.position.0);
        features.push(agent.position.1);

        // Capability features (average proficiency and learning rate)
        if !agent.capabilities.is_empty() {
            let avg_proficiency = agent
                .capabilities
                .iter()
                .map(|c| c.proficiency)
                .sum::<f64>()
                / agent.capabilities.len() as f64;
            let avg_learning_rate = agent
                .capabilities
                .iter()
                .map(|c| c.learning_rate)
                .sum::<f64>()
                / agent.capabilities.len() as f64;

            features.push(avg_proficiency);
            features.push(avg_learning_rate);
        } else {
            features.push(0.0);
            features.push(0.0);
        }

        // Context features (basic NLP processing)
        let processed_text = self.neural_processor.process_text(context).await?;

        // Extract numerical features from processed text
        features.push(processed_text.tokens.len() as f64);
        features.push(processed_text.sentiment);
        features.push(processed_text.keywords.len() as f64);

        // Add semantic vector (always available in SemanticVector)
        let vector = &processed_text.semantic_vector.dimensions;
        let vector_features: Vec<f64> = vector.iter().take(5).copied().collect();
        features.extend(vector_features.iter());

        // Pad with zeros if vector is smaller than 5 dimensions
        for _ in vector_features.len()..5 {
            features.push(0.0);
        }

        Ok(features)
    }

    fn generate_pattern_id(&self, features: &[f64]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for &feature in features {
            // Quantize features to reduce noise in pattern matching
            let quantized = (feature * 100.0).round() as i64;
            quantized.hash(&mut hasher);
        }
        format!("pattern_{:x}", hasher.finish())
    }

    #[allow(dead_code)]
    fn calculate_learning_factor(&self, frequency: u32) -> f64 {
        // Higher frequency patterns should have lower learning rates (more stable)
        let base_rate = self.config.learning_rate;
        base_rate / (1.0 + (frequency as f64 * 0.1))
    }

    fn calculate_learning_velocity(&self) -> f64 {
        if self.learning_history.len() < 2 {
            return 0.0;
        }

        let recent_events = self
            .learning_history
            .iter()
            .filter(|e| e.timestamp > chrono::Utc::now() - chrono::Duration::hours(1))
            .count();

        recent_events as f64 / 60.0 // Events per minute
    }

    pub fn cleanup_old_patterns(&mut self) {
        let cutoff =
            chrono::Utc::now() - chrono::Duration::days(self.config.pattern_retention_days as i64);
        let initial_count = self.patterns.len();

        self.patterns
            .retain(|_, pattern| pattern.last_seen > cutoff);

        let removed_count = initial_count - self.patterns.len();
        if removed_count > 0 {
            info!("Cleaned up {} old patterns", removed_count);
        }
    }

    fn cleanup_low_confidence_patterns(&mut self) {
        let initial_count = self.patterns.len();

        // Remove patterns with very low confidence and low frequency
        self.patterns
            .retain(|_, pattern| pattern.confidence >= 0.3 || pattern.frequency >= 5);

        let removed_count = initial_count - self.patterns.len();
        if removed_count > 0 {
            warn!(
                "Cleaned up {} low-confidence patterns to manage memory",
                removed_count
            );
        }
    }

    pub fn get_pattern_statistics(&self) -> PatternStatistics {
        let total_patterns = self.patterns.len();
        let high_confidence = self
            .patterns
            .values()
            .filter(|p| p.confidence >= self.config.min_confidence_threshold)
            .count();

        let average_confidence = if total_patterns > 0 {
            self.patterns.values().map(|p| p.confidence).sum::<f64>() / total_patterns as f64
        } else {
            0.0
        };

        let total_frequency: u32 = self.patterns.values().map(|p| p.frequency).sum();

        PatternStatistics {
            total_patterns,
            high_confidence_patterns: high_confidence,
            average_confidence,
            total_interactions: total_frequency,
            recent_learning_events: self.learning_history.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningInsights {
    pub total_patterns: usize,
    pub high_confidence_patterns: usize,
    pub average_confidence: f64,
    pub recent_learning_events: usize,
    pub learning_velocity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStatistics {
    pub total_patterns: usize,
    pub high_confidence_patterns: usize,
    pub average_confidence: f64,
    pub total_interactions: u32,
    pub recent_learning_events: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::agent::{AgentCapability, AgentState, AgentType};
    use uuid::Uuid;

    fn create_test_agent() -> Agent {
        Agent {
            id: Uuid::new_v4(),
            name: "test_agent".to_string(),
            agent_type: AgentType::Worker,
            state: AgentState::Idle,
            capabilities: vec![AgentCapability {
                name: "test_capability".to_string(),
                proficiency: 0.8,
                learning_rate: 0.1,
            }],
            position: (1.0, 2.0),
            energy: 0.7,
            memory: AgentMemory::new(),
            created_at: chrono::Utc::now(),
            last_active: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_learning_from_interaction() {
        let mut learning_system = AdaptiveLearningSystem::new(AdaptiveLearningConfig::default())
            .await
            .unwrap();
        let agent = create_test_agent();

        let result = learning_system
            .learn_from_interaction(&agent, "test context", 0.8)
            .await;
        assert!(result.is_ok());
        assert!(!learning_system.patterns.is_empty());
    }

    #[tokio::test]
    async fn test_pattern_confidence_update() {
        let mut learning_system = AdaptiveLearningSystem::new(AdaptiveLearningConfig::default())
            .await
            .unwrap();
        let agent = create_test_agent();

        // Learn from multiple interactions
        for i in 0..5 {
            let outcome = if i < 4 { 0.9 } else { 0.1 }; // Mostly successful
            learning_system
                .learn_from_interaction(&agent, "consistent context", outcome)
                .await
                .unwrap();
        }

        let prediction = learning_system
            .predict_outcome(&agent, "consistent context")
            .await
            .unwrap();
        assert!(prediction > 0.5); // Should predict success
    }

    #[tokio::test]
    async fn test_pattern_cleanup() {
        let config = AdaptiveLearningConfig {
            pattern_retention_days: 0, // Immediate cleanup
            ..Default::default()
        };
        let mut learning_system = AdaptiveLearningSystem::new(config).await.unwrap();
        let agent = create_test_agent();

        learning_system
            .learn_from_interaction(&agent, "test", 0.5)
            .await
            .unwrap();
        assert!((learning_system.patterns.len() - 1).abs() < f32::EPSILON);

        learning_system.cleanup_old_patterns();
        assert!((learning_system.patterns.len() - 0).abs() < f32::EPSILON);
    }
}
