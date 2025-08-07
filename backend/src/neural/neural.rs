use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(feature = "advanced-neural")]
use ruv_fann::{Network, ActivationFunction};

use crate::neural::{NLPProcessor, ProcessedText};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralAgent {
    pub agent_id: Uuid,
    pub network_type: NetworkType,
    pub performance_history: Vec<f64>,
    pub learning_rate: f64,
    pub specialization: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkType {
    Basic,           // Uses current NLP.rs
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
                NetworkType::Basic => {
                    self.nlp_processor.process_text(text).await
                }
                #[cfg(feature = "advanced-neural")]
                NetworkType::FANN(_) => {
                    // Use basic processing for now, FANN enhancement happens during training
                    self.nlp_processor.process_text(text).await
                }
                #[cfg(feature = "advanced-neural")]
                NetworkType::LSTM(_) => {
                    self.process_with_lstm(text, agent_id).await
                }
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
                    "forecasting" | "prediction" => {
                        NetworkType::LSTM(LSTMConfig {
                            hidden_size: 128,
                            num_layers: 2,
                            sequence_length: 10,
                        })
                    }
                    "pattern_recognition" | "classification" => {
                        NetworkType::FANN(FANNConfig {
                            layers: vec![100, 50, 25, 10],
                            activation: "sigmoid".to_string(),
                            training_algorithm: "rprop".to_string(),
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
            neural_agent.performance_history.push(if success { 1.0 } else { 0.0 });
            
            // Keep only recent history
            if neural_agent.performance_history.len() > 100 {
                neural_agent.performance_history.remove(0);
            }

            match &neural_agent.network_type {
                NetworkType::FANN(_) => {
                    self.train_fann_network(agent_id, input, output, success).await?;
                }
                NetworkType::LSTM(_) => {
                    self.train_lstm_network(agent_id, input, output, success).await?;
                }
                NetworkType::Basic => {
                    // Already handled by basic NLP processor
                }
            }
        }

        Ok(())
    }

    /// Get performance metrics for an agent
    pub fn get_agent_performance(&self, agent_id: Uuid) -> Option<f64> {
        self.neural_agents.get(&agent_id).map(|agent| {
            if agent.performance_history.is_empty() {
                0.5 // Default performance
            } else {
                agent.performance_history.iter().sum::<f64>() / agent.performance_history.len() as f64
            }
        })
    }

    /// Predict agent performance for a given task
    pub async fn predict_performance(&mut self, agent_id: Uuid, task_description: &str) -> Result<f64> {
        if let Some(neural_agent) = self.neural_agents.get(&agent_id) {
            match &neural_agent.network_type {
                NetworkType::Basic => {
                    // Use basic NLP similarity matching
                    let similar_experiences = self.nlp_processor
                        .find_similar_experiences(task_description)
                        .await?;
                    
                    if similar_experiences.is_empty() {
                        Ok(0.5) // Default prediction
                    } else {
                        let avg_confidence = similar_experiences.iter()
                            .map(|exp| exp.confidence)
                            .sum::<f64>() / similar_experiences.len() as f64;
                        Ok(avg_confidence)
                    }
                }
                #[cfg(feature = "advanced-neural")]
                NetworkType::FANN(_) => {
                    self.predict_with_fann(agent_id, task_description).await
                }
                #[cfg(feature = "advanced-neural")]
                NetworkType::LSTM(_) => {
                    self.predict_with_lstm(agent_id, task_description).await
                }
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
        network.set_activation_function_output(activation);
        
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
            let input: Vec<f32> = processed.semantic_vector.dimensions
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
        if let Some(network) = self.neural_networks.get_mut(&agent_id) {
            let processed_input = self.nlp_processor.process_text(input).await?;
            
            // Convert to training data
            let input_data: Vec<f32> = processed_input.semantic_vector.dimensions
                .iter()
                .take(network.num_inputs())
                .map(|&x| x as f32)
                .collect();
            
            let target_data = vec![if success { 1.0 } else { 0.0 }];
            
            if input_data.len() == network.num_inputs() && 
               target_data.len() == network.num_outputs() {
                // Train on single example - FANN expects batch format
                let input_batch = vec![input_data];
                let target_batch = vec![target_data];
                let _ = network.train(&input_batch, &target_batch, 0.1, 1);
            }
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
                return self.train_lstm_sequence(agent_id, input, success, &config_clone).await;
            }
        }
        Ok(())
    }

    #[cfg(feature = "advanced-neural")]
    async fn predict_with_fann(&mut self, agent_id: Uuid, task_description: &str) -> Result<f64> {
        if let Some(network) = self.neural_networks.get_mut(&agent_id) {
            let processed = self.nlp_processor.process_text(task_description).await?;
            
            let input: Vec<f32> = processed.semantic_vector.dimensions
                .iter()
                .take(network.num_inputs())
                .map(|&x| x as f32)
                .collect();
            
            if input.len() == network.num_inputs() {
                let output = network.run(&input);
                if !output.is_empty() {
                    return Ok(output[0] as f64);
                }
            }
        }
        
        Ok(0.5) // Default prediction
    }

    #[cfg(feature = "advanced-neural")]
    async fn predict_with_lstm(&self, agent_id: Uuid, task_description: &str) -> Result<f64> {
        // LSTM prediction using sequence history
        if let Some(agent) = self.neural_agents.get(&agent_id) {
            if let NetworkType::LSTM(ref config) = agent.network_type {
                return self.predict_lstm_sequence(agent_id, task_description, config).await;
            }
        }
        Ok(0.5) // Default prediction for non-LSTM networks
    }

    // LSTM-specific implementation methods
    #[cfg(feature = "advanced-neural")]
    async fn process_lstm_sequence(&self, agent_id: Uuid, text: &str, config: &LSTMConfig) -> Result<ProcessedText> {
        // Get basic processing
        let mut processed = self.nlp_processor.process_text(text).await?;
        
        // Enhance with LSTM sequence processing
        if let Some(agent) = self.neural_agents.get(&agent_id) {
            // Use performance history as sequence context
            let sequence_context = self.get_sequence_context(&agent.performance_history, config.sequence_length);
            
            // Adjust sentiment based on sequence patterns
            let sequence_influence = self.calculate_sequence_influence(&sequence_context);
            processed.sentiment = (processed.sentiment + sequence_influence) / 2.0;
            
            // Enhance semantic vector with temporal information
            if processed.semantic_vector.dimensions.len() >= config.hidden_size {
                for i in 0..config.hidden_size.min(processed.semantic_vector.dimensions.len()) {
                    processed.semantic_vector.dimensions[i] *= 1.0 + sequence_influence * 0.1;
                }
            }
        }
        
        Ok(processed)
    }

    #[cfg(feature = "advanced-neural")]
    async fn train_lstm_sequence(&mut self, agent_id: Uuid, _input: &str, success: bool, config: &LSTMConfig) -> Result<()> {
        if let Some(agent) = self.neural_agents.get_mut(&agent_id) {
            // Add new performance data point
            agent.performance_history.push(if success { 1.0 } else { 0.0 });
            
            // Maintain sequence length
            while agent.performance_history.len() > config.sequence_length {
                agent.performance_history.remove(0);
            }
            
            // Update learning rate based on recent performance
            if agent.performance_history.len() >= 3 {
                let recent_avg = agent.performance_history.iter().rev().take(3).sum::<f64>() / 3.0;
                agent.learning_rate = (0.05 + recent_avg * 0.1).min(0.2);
            }
            
            tracing::debug!("LSTM training for agent {}: sequence_len={}, learning_rate={:.3}", 
                          agent_id, agent.performance_history.len(), agent.learning_rate);
        }
        
        Ok(())
    }

    #[cfg(feature = "advanced-neural")]
    async fn predict_lstm_sequence(&self, agent_id: Uuid, task_description: &str, config: &LSTMConfig) -> Result<f64> {
        if let Some(agent) = self.neural_agents.get(&agent_id) {
            // Get sequence context from performance history
            let sequence_context = self.get_sequence_context(&agent.performance_history, config.sequence_length);
            
            // Process current task description
            let processed = self.nlp_processor.process_text(task_description).await?;
            
            // Combine sequence patterns with current task features
            let base_prediction = if sequence_context.is_empty() {
                0.5 // Default when no history
            } else {
                // Simple LSTM-like prediction: weighted average with trend analysis
                let recent_trend = self.calculate_trend(&sequence_context);
                let avg_performance = sequence_context.iter().sum::<f64>() / sequence_context.len() as f64;
                
                // Combine average performance with trend
                (avg_performance + recent_trend * 0.3).clamp(0.0, 1.0)
            };
            
            // Adjust prediction based on task complexity (sentiment as proxy)
            let task_complexity_factor = (1.0 - processed.sentiment.abs()) * 0.2; // Higher complexity for neutral sentiment
            let final_prediction = (base_prediction * (1.0 - task_complexity_factor)).clamp(0.0, 1.0);
            
            tracing::debug!("LSTM prediction for agent {}: base={:.3}, complexity_factor={:.3}, final={:.3}", 
                          agent_id, base_prediction, task_complexity_factor, final_prediction);
            
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
        let recent_half = &sequence[sequence.len()/2..];
        let early_half = &sequence[..sequence.len()/2];
        
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
        let sum_xy: f64 = sequence.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let sum_x2: f64 = (0..sequence.len()).map(|i| (i as f64).powi(2)).sum();
        
        // Linear regression slope
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        slope.clamp(-1.0, 1.0)
    }
}