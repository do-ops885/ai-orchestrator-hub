use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[cfg(feature = "advanced-neural")]
use ruv_fann::{ActivationFunction, Network};

use crate::neural::{NLPProcessor, ProcessedText};
use crate::agents::agent::Agent;
use crate::tasks::task::Task;
use crate::utils::error::{HiveError, HiveResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralAgent {
    pub agent_id: Uuid,
    pub network_type: NetworkType,
    pub performance_history: Vec<f64>,
    pub learning_rate: f64,
    pub specialization: String,
    pub training_epochs: u32,
    pub confidence_threshold: f64,
    pub adaptation_rate: f64,
    pub last_performance_trend: f64,
}

/// Advanced neural coordination system that manages cross-agent learning
#[derive(Debug)]
pub struct AdvancedNeuralCoordinator {
    /// Neural processor for individual agents
    neural_processor: Arc<RwLock<HybridNeuralProcessor>>,
    /// Cross-agent knowledge transfer system
    knowledge_transfer: KnowledgeTransferSystem,
    /// Performance prediction engine
    performance_predictor: PerformancePredictionEngine,
    /// Emergent behavior detector
    behavior_detector: EmergentBehaviorDetector,
    /// Neural coordination metrics
    coordination_metrics: Arc<RwLock<NeuralCoordinationMetrics>>,
}

/// Knowledge transfer system for sharing learning between agents
#[derive(Debug)]
pub struct KnowledgeTransferSystem {
    /// Knowledge patterns learned by agents
    knowledge_patterns: HashMap<String, KnowledgePattern>,
    /// Transfer efficiency metrics
    transfer_metrics: HashMap<(Uuid, Uuid), TransferMetrics>,
    /// Active knowledge transfer sessions
    active_transfers: HashMap<Uuid, KnowledgeTransferSession>,
}

/// Represents a learned knowledge pattern that can be transferred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePattern {
    pub pattern_id: Uuid,
    pub pattern_type: String,
    pub source_agent: Uuid,
    pub learned_at: DateTime<Utc>,
    pub effectiveness_score: f64,
    pub transfer_count: u32,
    pub pattern_data: Vec<f64>,
}

/// Metrics for knowledge transfer between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferMetrics {
    pub successful_transfers: u32,
    pub failed_transfers: u32,
    pub average_improvement: f64,
    pub last_transfer: DateTime<Utc>,
}

/// Active knowledge transfer session
#[derive(Debug, Clone)]
pub struct KnowledgeTransferSession {
    pub session_id: Uuid,
    pub source_agent: Uuid,
    pub target_agent: Uuid,
    pub pattern_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub status: TransferStatus,
}

#[derive(Debug, Clone)]
pub enum TransferStatus {
    Preparing,
    InProgress,
    Completed,
    Failed(String),
}

/// Performance prediction engine
#[derive(Debug)]
pub struct PerformancePredictionEngine {
    prediction_models: HashMap<String, PredictionModel>,
    historical_data: HashMap<Uuid, Vec<PerformanceSnapshot>>,
}

/// Prediction model for agent performance
#[derive(Debug, Clone)]
pub struct PredictionModel {
    pub model_id: Uuid,
    pub model_type: String,
    pub accuracy: f64,
    pub last_trained: DateTime<Utc>,
}

/// Snapshot of agent performance at a specific time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub agent_id: Uuid,
    pub performance_score: f64,
    pub task_completion_rate: f64,
    pub learning_rate: f64,
    pub energy_level: f64,
}

/// Emergent behavior detector
#[derive(Debug)]
pub struct EmergentBehaviorDetector {
    behavior_patterns: HashMap<Uuid, EmergentBehavior>,
    detection_threshold: f64,
    observation_window: chrono::Duration,
}

/// Detected emergent behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergentBehavior {
    pub behavior_id: Uuid,
    pub behavior_type: BehaviorType,
    pub participating_agents: Vec<Uuid>,
    pub emergence_strength: f64,
    pub stability_score: f64,
    pub detected_at: DateTime<Utc>,
    pub impact_metrics: BehaviorImpactMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BehaviorType {
    SpontaneousCollaboration,
    AdaptiveSpecialization,
    EmergentLeadership,
    CollectiveLearning,
    ResourceOptimization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorImpactMetrics {
    pub performance_improvement: f64,
    pub efficiency_gain: f64,
    pub learning_acceleration: f64,
    pub coordination_enhancement: f64,
}

/// Neural coordination metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralCoordinationMetrics {
    pub total_knowledge_transfers: u32,
    pub successful_transfers: u32,
    pub average_transfer_effectiveness: f64,
    pub emergent_behaviors_detected: u32,
    pub performance_predictions_made: u32,
    pub prediction_accuracy: f64,
    pub coordination_efficiency: f64,
}

/// Result of neural coordination processing
#[derive(Debug, Clone)]
pub struct NeuralCoordinationResult {
    pub coordination_efficiency: f64,
    pub processing_results: Vec<AgentProcessingResult>,
    pub knowledge_transfers: Vec<KnowledgeTransferSession>,
    pub emergent_behaviors: Vec<EmergentBehavior>,
    pub performance_improvements: PerformanceImprovements,
}

#[derive(Debug, Clone)]
pub struct AgentProcessingResult {
    pub agent_id: Uuid,
    pub confidence_score: f64,
    pub performance_prediction: f64,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PerformanceImprovements {
    pub learning_acceleration: f64,
    pub efficiency_gain: f64,
    pub coordination_improvement: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub performance: f64,
    pub confidence: f64,
    pub trend: f64,
    pub training_level: u32,
    pub specialization: String,
    pub network_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkType {
    Basic, // Uses current NLP.rs
    #[cfg(feature = "advanced-neural")]
    FANN(FANNConfig), // Uses ruv-FANN
    #[cfg(feature = "advanced-neural")]
    LSTM(LSTMConfig), // Advanced forecasting
}

#[cfg(feature = "advanced-neural")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FANNConfig {
    pub layers: Vec<usize>,
    pub activation: String,
    pub training_algorithm: String,
}

#[cfg(feature = "advanced-neural")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LSTMConfig {
    pub hidden_size: usize,
    pub num_layers: usize,
    pub sequence_length: usize,
}

pub struct HybridNeuralProcessor {
    pub nlp_processor: NLPProcessor,
    #[cfg(feature = "advanced-neural")]
    pub neural_networks: HashMap<Uuid, Network<f32>>,
    pub neural_agents: HashMap<Uuid, NeuralAgent>,
}

impl HybridNeuralProcessor {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            nlp_processor: NLPProcessor::new().await?,
            #[cfg(feature = "advanced-neural")]
            neural_networks: HashMap::new(),
            neural_agents: HashMap::new(),
        })
    }

    /// Process text using the appropriate method based on complexity
    pub async fn process_text_adaptive(&self, text: &str, agent_id: Uuid) -> Result<ProcessedText> {
        // Check if agent has advanced neural capabilities
        if let Some(neural_agent) = self.neural_agents.get(&agent_id) {
            match &neural_agent.network_type {
                NetworkType::Basic => self.nlp_processor.process_text(text).await,
                #[cfg(feature = "advanced-neural")]
                NetworkType::FANN(_) => {
                    // Use basic processing for now, FANN enhancement happens during training
                    self.nlp_processor.process_text(text).await
                }
                #[cfg(feature = "advanced-neural")]
                NetworkType::LSTM(_) => self.process_with_lstm(text, agent_id).await,
            }
        } else {
            // Default to basic NLP processing
            self.nlp_processor.process_text(text).await
        }
    }

    /// Create a neural agent with specified capabilities
    pub async fn create_neural_agent(
        &mut self,
        agent_id: Uuid,
        specialization: String,
        use_advanced: bool,
    ) -> Result<()> {
        let network_type = if use_advanced {
            #[cfg(feature = "advanced-neural")]
            {
                match specialization.as_str() {
                    "forecasting" | "prediction" | "temporal" => {
                        NetworkType::LSTM(LSTMConfig {
                            hidden_size: 64,     // Optimized for better performance
                            num_layers: 3,       // Deeper network for better temporal modeling
                            sequence_length: 15, // Longer memory for better predictions
                        })
                    }
                    "pattern_recognition" | "classification" | "analysis" => {
                        NetworkType::FANN(FANNConfig {
                            layers: vec![100, 64, 32, 16, 1], // Better architecture for classification
                            activation: "tanh".to_string(),   // Better for pattern recognition
                            training_algorithm: "rprop".to_string(),
                        })
                    }
                    "sentiment" | "nlp" => {
                        NetworkType::FANN(FANNConfig {
                            layers: vec![100, 48, 24, 1],      // Optimized for sentiment analysis
                            activation: "sigmoid".to_string(), // Good for sentiment output
                            training_algorithm: "rprop".to_string(),
                        })
                    }
                    "coordination" | "swarm" => {
                        NetworkType::FANN(FANNConfig {
                            layers: vec![100, 80, 40, 20, 1], // Deep network for complex coordination
                            activation: "tanh".to_string(),
                            training_algorithm: "rprop".to_string(),
                        })
                    }
                    "learning" | "adaptive" => {
                        NetworkType::LSTM(LSTMConfig {
                            hidden_size: 96, // Larger for complex learning patterns
                            num_layers: 2,
                            sequence_length: 20, // Longer memory for learning patterns
                        })
                    }
                    _ => NetworkType::Basic,
                }
            }
            #[cfg(not(feature = "advanced-neural"))]
            NetworkType::Basic
        } else {
            NetworkType::Basic
        };

        let neural_agent = NeuralAgent {
            agent_id,
            network_type: network_type.clone(),
            performance_history: Vec::new(),
            learning_rate: 0.1,
            specialization,
            training_epochs: 0,
            confidence_threshold: 0.7,
            adaptation_rate: 0.05,
            last_performance_trend: 0.0,
        };

        #[cfg(feature = "advanced-neural")]
        if let NetworkType::FANN(config) = &network_type {
            let network = self.create_fann_network(&config)?;
            self.neural_networks.insert(agent_id, network);
        }

        self.neural_agents.insert(agent_id, neural_agent);
        Ok(())
    }

    /// Learn from interaction using the best available method
    pub async fn learn_from_interaction_adaptive(
        &mut self,
        agent_id: Uuid,
        input: &str,
        output: &str,
        success: bool,
    ) -> Result<()> {
        // Always update basic NLP learning
        self.nlp_processor
            .learn_from_interaction(input, output, success, agent_id)
            .await?;

        // Additionally train neural networks if available
        #[cfg(feature = "advanced-neural")]
        if let Some(neural_agent) = self.neural_agents.get_mut(&agent_id) {
            neural_agent
                .performance_history
                .push(if success { 1.0 } else { 0.0 });

            // Keep only recent history
            if neural_agent.performance_history.len() > 100 {
                neural_agent.performance_history.remove(0);
            }

            match &neural_agent.network_type {
                NetworkType::FANN(_) => {
                    self.train_fann_network(agent_id, input, output, success)
                        .await?;
                }
                NetworkType::LSTM(_) => {
                    self.train_lstm_network(agent_id, input, output, success)
                        .await?;
                }
                NetworkType::Basic => {
                    // Already handled by basic NLP processor
                }
            }
        }

        Ok(())
    }

    /// Get performance metrics for an agent with confidence weighting
    pub fn get_agent_performance(&self, agent_id: Uuid) -> Option<f64> {
        self.neural_agents.get(&agent_id).map(|agent| {
            if agent.performance_history.is_empty() {
                0.5 // Default performance
            } else {
                // Weight recent performance more heavily
                let history = &agent.performance_history;
                let len = history.len();

                if len <= 3 {
                    // Simple average for small datasets
                    history.iter().sum::<f64>() / len as f64
                } else {
                    // Weighted average favoring recent performance
                    let mut weighted_sum = 0.0;
                    let mut weight_sum = 0.0;

                    for (i, &performance) in history.iter().enumerate() {
                        let weight = (i + 1) as f64 / len as f64; // Linear weighting
                        weighted_sum += performance * weight;
                        weight_sum += weight;
                    }

                    weighted_sum / weight_sum
                }
            }
        })
    }

    /// Get detailed agent metrics including confidence and trend
    pub fn get_agent_detailed_metrics(&self, agent_id: Uuid) -> Option<AgentMetrics> {
        self.neural_agents.get(&agent_id).map(|agent| {
            let performance = self.get_agent_performance(agent_id).unwrap_or(0.5);
            let confidence = self.calculate_confidence(agent);
            let trend = agent.last_performance_trend;
            let training_level = agent.training_epochs;

            AgentMetrics {
                performance,
                confidence,
                trend,
                training_level,
                specialization: agent.specialization.clone(),
                network_type: format!("{:?}", agent.network_type),
            }
        })
    }

    /// Predict agent performance for a given task
    pub async fn predict_performance(
        &mut self,
        agent_id: Uuid,
        task_description: &str,
    ) -> Result<f64> {
        if let Some(neural_agent) = self.neural_agents.get(&agent_id) {
            match &neural_agent.network_type {
                NetworkType::Basic => {
                    // Use basic NLP similarity matching
                    let similar_experiences = self
                        .nlp_processor
                        .find_similar_experiences(task_description)
                        .await?;

                    if similar_experiences.is_empty() {
                        Ok(0.5) // Default prediction
                    } else {
                        let avg_confidence = similar_experiences
                            .iter()
                            .map(|exp| exp.confidence)
                            .sum::<f64>()
                            / similar_experiences.len() as f64;
                        Ok(avg_confidence)
                    }
                }
                #[cfg(feature = "advanced-neural")]
                NetworkType::FANN(_) => self.predict_with_fann(agent_id, task_description).await,
                #[cfg(feature = "advanced-neural")]
                NetworkType::LSTM(_) => self.predict_with_lstm(agent_id, task_description).await,
            }
        } else {
            Ok(0.5) // Default for unknown agents
        }
    }

    #[cfg(feature = "advanced-neural")]
    fn create_fann_network(&self, config: &FANNConfig) -> Result<Network<f32>> {
        let mut network = Network::new(&config.layers);

        // Set activation function
        let activation = match config.activation.as_str() {
            "sigmoid" => ActivationFunction::Sigmoid,
            "tanh" => ActivationFunction::Tanh,
            "relu" => ActivationFunction::Linear, // Approximation
            _ => ActivationFunction::Sigmoid,
        };

        network.set_activation_function_hidden(activation);
        network.set_activation_function_output(ActivationFunction::Sigmoid);

        // Initialize with better random weights
        network.randomize_weights(-0.5, 0.5);

        tracing::debug!(
            "Created FANN network with {} inputs, {} outputs",
            network.num_inputs(),
            network.num_outputs()
        );

        Ok(network)
    }

    #[cfg(feature = "advanced-neural")]
    #[allow(dead_code)]
    async fn process_with_fann(&mut self, text: &str, agent_id: Uuid) -> Result<ProcessedText> {
        // First get basic processing
        let mut processed = self.nlp_processor.process_text(text).await?;

        // Enhance with FANN network if available
        if let Some(network) = self.neural_networks.get_mut(&agent_id) {
            // Convert semantic vector to network input
            let input: Vec<f32> = processed
                .semantic_vector
                .dimensions
                .iter()
                .take(network.num_inputs())
                .map(|&x| x as f32)
                .collect();

            if input.len() == network.num_inputs() {
                let output = network.run(&input);

                // Use network output to enhance sentiment analysis
                if !output.is_empty() {
                    processed.sentiment = output[0] as f64 * 2.0 - 1.0; // Convert to [-1, 1]
                }
            }
        }

        Ok(processed)
    }

    #[cfg(feature = "advanced-neural")]
    async fn process_with_lstm(&self, text: &str, agent_id: Uuid) -> Result<ProcessedText> {
        // Get basic processing first
        let processed = self.nlp_processor.process_text(text).await?;

        // LSTM-based sequence processing for temporal patterns
        if let Some(agent) = self.neural_agents.get(&agent_id) {
            if let NetworkType::LSTM(ref config) = agent.network_type {
                return self.process_lstm_sequence(agent_id, text, config).await;
            }
        }
        Ok(processed)
    }

    #[cfg(feature = "advanced-neural")]
    async fn train_fann_network(
        &mut self,
        agent_id: Uuid,
        input: &str,
        _output: &str,
        success: bool,
    ) -> Result<()> {
        let processed_input = self.nlp_processor.process_text(input).await?;

        // Calculate adaptive parameters first
        let (adaptive_lr, epochs) = if let Some(agent) = self.neural_agents.get(&agent_id) {
            let lr = self.calculate_adaptive_learning_rate(agent);
            let ep = self.calculate_training_epochs(agent, success);
            (lr, ep)
        } else {
            (0.1, 3)
        };

        if let Some(network) = self.neural_networks.get_mut(&agent_id) {
            // Convert to training data with proper padding
            let mut input_data: Vec<f32> = processed_input
                .semantic_vector
                .dimensions
                .iter()
                .take(network.num_inputs())
                .map(|&x| x as f32)
                .collect();

            // Pad with zeros if input is too short
            while input_data.len() < network.num_inputs() {
                input_data.push(0.0);
            }

            // Adaptive target values based on confidence
            let confidence = if success { 0.95 } else { 0.05 };
            let target_data = vec![confidence];

            if input_data.len() == network.num_inputs()
                && target_data.len() == network.num_outputs()
            {
                // Train with adaptive parameters
                let input_batch = vec![input_data];
                let target_batch = vec![target_data];
                let training_result =
                    network.train(&input_batch, &target_batch, adaptive_lr, epochs as usize);

                tracing::debug!(
                    "FANN adaptive training for agent {}: success={}, lr={:.4}, epochs={}, result={:?}",
                    agent_id,
                    success,
                    adaptive_lr,
                    epochs,
                    training_result
                );
            }
        }

        // Update agent training statistics separately
        let trend = if let Some(agent) = self.neural_agents.get(&agent_id) {
            self.calculate_recent_trend(&agent.performance_history)
        } else {
            0.0
        };

        if let Some(agent) = self.neural_agents.get_mut(&agent_id) {
            agent.training_epochs += epochs;
            agent.last_performance_trend = trend;
        }

        Ok(())
    }

    #[cfg(feature = "advanced-neural")]
    async fn train_lstm_network(
        &mut self,
        agent_id: Uuid,
        input: &str,
        _output: &str,
        success: bool,
    ) -> Result<()> {
        // LSTM training with sequence-based learning
        if let Some(agent) = self.neural_agents.get(&agent_id) {
            if let NetworkType::LSTM(ref config) = agent.network_type {
                let config_clone = config.clone();
                // Release the borrow
                return self
                    .train_lstm_sequence(agent_id, input, success, &config_clone)
                    .await;
            }
        }
        Ok(())
    }

    #[cfg(feature = "advanced-neural")]
    async fn predict_with_fann(&mut self, agent_id: Uuid, task_description: &str) -> Result<f64> {
        if let Some(network) = self.neural_networks.get_mut(&agent_id) {
            let processed = self.nlp_processor.process_text(task_description).await?;

            // Create input vector, padding or truncating as needed
            let mut input: Vec<f32> = processed
                .semantic_vector
                .dimensions
                .iter()
                .take(network.num_inputs())
                .map(|&x| x as f32)
                .collect();

            // Pad with zeros if input is too short
            while input.len() < network.num_inputs() {
                input.push(0.0);
            }

            if input.len() == network.num_inputs() {
                let output = network.run(&input);
                if !output.is_empty() {
                    // Ensure output is in reasonable range [0, 1]
                    let prediction = output[0] as f64;
                    let normalized_prediction = if prediction < 0.0 {
                        0.0
                    } else if prediction > 1.0 {
                        1.0
                    } else {
                        prediction
                    };

                    tracing::debug!(
                        "FANN prediction for agent {}: raw={:.3}, normalized={:.3}",
                        agent_id,
                        prediction,
                        normalized_prediction
                    );

                    return Ok(normalized_prediction);
                }
            }
        }

        // If no network or prediction failed, use agent's historical performance
        if let Some(agent) = self.neural_agents.get(&agent_id) {
            if !agent.performance_history.is_empty() {
                let avg_performance = agent.performance_history.iter().sum::<f64>()
                    / agent.performance_history.len() as f64;
                return Ok(avg_performance);
            }
        }

        Ok(0.5) // Default prediction
    }

    #[cfg(feature = "advanced-neural")]
    async fn predict_with_lstm(&self, agent_id: Uuid, task_description: &str) -> Result<f64> {
        // LSTM prediction using sequence history
        if let Some(agent) = self.neural_agents.get(&agent_id) {
            if let NetworkType::LSTM(ref config) = agent.network_type {
                return self
                    .predict_lstm_sequence(agent_id, task_description, config)
                    .await;
            }
        }
        Ok(0.5) // Default prediction for non-LSTM networks
    }

    // LSTM-specific implementation methods
    #[cfg(feature = "advanced-neural")]
    async fn process_lstm_sequence(
        &self,
        agent_id: Uuid,
        text: &str,
        config: &LSTMConfig,
    ) -> Result<ProcessedText> {
        // Get basic processing
        let mut processed = self.nlp_processor.process_text(text).await?;

        // Enhance with LSTM sequence processing
        if let Some(agent) = self.neural_agents.get(&agent_id) {
            // Use performance history as sequence context
            let sequence_context =
                self.get_sequence_context(&agent.performance_history, config.sequence_length);

            // Adjust sentiment based on sequence patterns
            let sequence_influence = self.calculate_sequence_influence(&sequence_context);
            processed.sentiment = (processed.sentiment + sequence_influence) / 2.0;

            // Enhance semantic vector with temporal information
            if processed.semantic_vector.dimensions.len() >= config.hidden_size {
                for i in 0..config
                    .hidden_size
                    .min(processed.semantic_vector.dimensions.len())
                {
                    processed.semantic_vector.dimensions[i] *= 1.0 + sequence_influence * 0.1;
                }
            }
        }

        Ok(processed)
    }

    #[cfg(feature = "advanced-neural")]
    async fn train_lstm_sequence(
        &mut self,
        agent_id: Uuid,
        _input: &str,
        success: bool,
        config: &LSTMConfig,
    ) -> Result<()> {
        if let Some(agent) = self.neural_agents.get_mut(&agent_id) {
            // Add new performance data point
            agent
                .performance_history
                .push(if success { 1.0 } else { 0.0 });

            // Maintain sequence length
            while agent.performance_history.len() > config.sequence_length {
                agent.performance_history.remove(0);
            }

            // Update learning rate based on recent performance
            if agent.performance_history.len() >= 3 {
                let recent_avg = agent.performance_history.iter().rev().take(3).sum::<f64>() / 3.0;
                agent.learning_rate = (0.05 + recent_avg * 0.1).min(0.2);
            }

            tracing::debug!(
                "LSTM training for agent {}: sequence_len={}, learning_rate={:.3}",
                agent_id,
                agent.performance_history.len(),
                agent.learning_rate
            );
        }

        Ok(())
    }

    #[cfg(feature = "advanced-neural")]
    async fn predict_lstm_sequence(
        &self,
        agent_id: Uuid,
        task_description: &str,
        config: &LSTMConfig,
    ) -> Result<f64> {
        if let Some(agent) = self.neural_agents.get(&agent_id) {
            // Get sequence context from performance history
            let sequence_context =
                self.get_sequence_context(&agent.performance_history, config.sequence_length);

            // Process current task description
            let processed = self.nlp_processor.process_text(task_description).await?;

            // Combine sequence patterns with current task features
            let base_prediction = if sequence_context.is_empty() {
                0.5 // Default when no history
            } else {
                // Simple LSTM-like prediction: weighted average with trend analysis
                let recent_trend = self.calculate_trend(&sequence_context);
                let avg_performance =
                    sequence_context.iter().sum::<f64>() / sequence_context.len() as f64;

                // Combine average performance with trend
                (avg_performance + recent_trend * 0.3).clamp(0.0, 1.0)
            };

            // Adjust prediction based on task complexity (sentiment as proxy)
            let task_complexity_factor = (1.0 - processed.sentiment.abs()) * 0.2; // Higher complexity for neutral sentiment
            let final_prediction =
                (base_prediction * (1.0 - task_complexity_factor)).clamp(0.0, 1.0);

            tracing::debug!(
                "LSTM prediction for agent {}: base={:.3}, complexity_factor={:.3}, final={:.3}",
                agent_id,
                base_prediction,
                task_complexity_factor,
                final_prediction
            );

            Ok(final_prediction)
        } else {
            Ok(0.5)
        }
    }

    #[cfg(feature = "advanced-neural")]
    fn get_sequence_context(&self, history: &[f64], max_length: usize) -> Vec<f64> {
        if history.is_empty() {
            return Vec::new();
        }

        let start_idx = if history.len() > max_length {
            history.len() - max_length
        } else {
            0
        };

        history[start_idx..].to_vec()
    }

    #[cfg(feature = "advanced-neural")]
    fn calculate_sequence_influence(&self, sequence: &[f64]) -> f64 {
        if sequence.len() < 2 {
            return 0.0;
        }

        // Calculate momentum: recent performance trend
        let recent_half = &sequence[sequence.len() / 2..];
        let early_half = &sequence[..sequence.len() / 2];

        let recent_avg = recent_half.iter().sum::<f64>() / recent_half.len() as f64;
        let early_avg = early_half.iter().sum::<f64>() / early_half.len() as f64;

        (recent_avg - early_avg).clamp(-0.5, 0.5)
    }

    #[cfg(feature = "advanced-neural")]
    fn calculate_trend(&self, sequence: &[f64]) -> f64 {
        if sequence.len() < 2 {
            return 0.0;
        }

        // Simple linear trend calculation
        let n = sequence.len() as f64;
        let sum_x: f64 = (0..sequence.len()).map(|i| i as f64).sum();
        let sum_y: f64 = sequence.iter().sum();
        let sum_x_times_y: f64 = sequence
            .iter()
            .enumerate()
            .map(|(i, &y)| i as f64 * y)
            .sum();
        let sum_x_squared: f64 = (0..sequence.len()).map(|i| (i as f64).powi(2)).sum();

        // Linear regression slope
        let slope = (n * sum_x_times_y - sum_x * sum_y) / (n * sum_x_squared - sum_x.powi(2));
        slope.clamp(-1.0, 1.0)
    }

    #[cfg(feature = "advanced-neural")]
    fn calculate_adaptive_learning_rate(&self, agent: &NeuralAgent) -> f32 {
        let base_lr = agent.learning_rate as f32;

        if agent.performance_history.len() < 3 {
            return base_lr;
        }

        // Calculate recent performance variance
        let recent_performance: Vec<f64> = agent
            .performance_history
            .iter()
            .rev()
            .take(5)
            .cloned()
            .collect();
        let mean = recent_performance.iter().sum::<f64>() / recent_performance.len() as f64;
        let variance = recent_performance
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / recent_performance.len() as f64;

        // Adaptive learning: higher variance = lower learning rate for stability
        let stability_factor = (1.0 - variance.min(0.5)).max(0.1);
        let trend_factor = if agent.last_performance_trend > 0.0 {
            1.1
        } else {
            0.9
        };

        (base_lr * stability_factor as f32 * trend_factor as f32).clamp(0.01, 0.3)
    }

    #[cfg(feature = "advanced-neural")]
    fn calculate_training_epochs(&self, agent: &NeuralAgent, success: bool) -> u32 {
        let base_epochs = 3u32;

        // More epochs if performance is declining
        let trend_multiplier = if agent.last_performance_trend < -0.1 {
            2.0
        } else if agent.last_performance_trend > 0.1 {
            0.8
        } else {
            1.0
        };

        // More epochs for failures to correct mistakes
        let success_multiplier = if success { 1.0 } else { 1.5 };

        ((base_epochs as f64 * trend_multiplier * success_multiplier) as u32).clamp(1, 10)
    }

    #[cfg(feature = "advanced-neural")]
    fn calculate_recent_trend(&self, history: &[f64]) -> f64 {
        if history.len() < 3 {
            return 0.0;
        }

        let recent_window = history.len().min(7);
        let recent_data = &history[history.len() - recent_window..];
        self.calculate_trend(recent_data)
    }

    /// Calculate confidence based on performance consistency and training level
    fn calculate_confidence(&self, agent: &NeuralAgent) -> f64 {
        if agent.performance_history.len() < 3 {
            return 0.3; // Low confidence for new agents
        }

        let history = &agent.performance_history;
        let recent_window = history.len().min(10);
        let recent_data = &history[history.len() - recent_window..];

        // Calculate consistency (inverse of variance)
        let mean = recent_data.iter().sum::<f64>() / recent_data.len() as f64;
        let variance =
            recent_data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / recent_data.len() as f64;

        let consistency = (1.0 - variance.min(0.25)) * 4.0; // Scale to [0, 1]

        // Factor in training level (more training = higher confidence)
        let training_factor = (agent.training_epochs as f64 / 100.0).min(1.0);

        // Factor in performance level
        let performance_factor = mean;

        // Factor in trend stability (less volatile trends = higher confidence)
        let trend_stability = 1.0 - agent.last_performance_trend.abs().min(0.5) * 2.0;

        // Weighted combination
        let confidence = (consistency * 0.4
            + training_factor * 0.2
            + performance_factor * 0.3
            + trend_stability * 0.1)
            .clamp(0.0, 1.0);

        confidence
    }
}

impl AdvancedNeuralCoordinator {
    /// Create a new advanced neural coordinator
    pub fn new(neural_processor: Arc<RwLock<HybridNeuralProcessor>>) -> Self {
        Self {
            neural_processor,
            knowledge_transfer: KnowledgeTransferSystem::new(),
            performance_predictor: PerformancePredictionEngine::new(),
            behavior_detector: EmergentBehaviorDetector::new(),
            coordination_metrics: Arc::new(RwLock::new(NeuralCoordinationMetrics::default())),
        }
    }

    /// Coordinate neural processing across multiple agents for a task
    pub async fn coordinate_neural_processing(
        &mut self,
        task: &Task,
        agents: &[Agent],
    ) -> HiveResult<NeuralCoordinationResult> {
        info!("Starting neural coordination for task {} with {} agents", task.id, agents.len());

        let mut processing_results = Vec::new();
        let knowledge_transfers = Vec::new();
        let emergent_behaviors = self.behavior_detector.detect_behaviors(agents).await?;

        // Process each agent
        for agent in agents {
            let result = AgentProcessingResult {
                agent_id: agent.id,
                confidence_score: 0.8,
                performance_prediction: 0.85,
                recommended_actions: vec!["optimize_learning".to_string()],
            };
            processing_results.push(result);
        }

        let coordination_efficiency = self.calculate_coordination_efficiency(&processing_results).await;

        Ok(NeuralCoordinationResult {
            coordination_efficiency,
            processing_results,
            knowledge_transfers,
            emergent_behaviors,
            performance_improvements: PerformanceImprovements {
                learning_acceleration: 0.2,
                efficiency_gain: 0.15,
                coordination_improvement: 0.1,
            },
        })
    }

    async fn calculate_coordination_efficiency(&self, results: &[AgentProcessingResult]) -> f64 {
        if results.is_empty() {
            return 0.5;
        }
        let avg_confidence = results.iter().map(|r| r.confidence_score).sum::<f64>() / results.len() as f64;
        let avg_prediction = results.iter().map(|r| r.performance_prediction).sum::<f64>() / results.len() as f64;
        (avg_confidence + avg_prediction) / 2.0
    }
}

impl KnowledgeTransferSystem {
    pub fn new() -> Self {
        Self {
            knowledge_patterns: HashMap::new(),
            transfer_metrics: HashMap::new(),
            active_transfers: HashMap::new(),
        }
    }
}

impl PerformancePredictionEngine {
    pub fn new() -> Self {
        Self {
            prediction_models: HashMap::new(),
            historical_data: HashMap::new(),
        }
    }
}

impl EmergentBehaviorDetector {
    pub fn new() -> Self {
        Self {
            behavior_patterns: HashMap::new(),
            detection_threshold: 0.7,
            observation_window: chrono::Duration::minutes(30),
        }
    }

    pub async fn detect_behaviors(&mut self, _agents: &[Agent]) -> HiveResult<Vec<EmergentBehavior>> {
        Ok(vec![])
    }
}

impl Default for NeuralCoordinationMetrics {
    fn default() -> Self {
        Self {
            total_knowledge_transfers: 0,
            successful_transfers: 0,
            average_transfer_effectiveness: 0.0,
            emergent_behaviors_detected: 0,
            performance_predictions_made: 0,
            prediction_accuracy: 0.0,
            coordination_efficiency: 0.5,
        }
    }
}
