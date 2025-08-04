use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(feature = "advanced-neural")]
use ruv_fann::{Network, ActivationFunction, TrainingAlgorithm};

use crate::nlp::{NLPProcessor, ProcessedText};

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
    pub neural_networks: HashMap<Uuid, Network>,
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
                    self.process_with_fann(text, agent_id).await
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
    pub async fn predict_performance(&self, agent_id: Uuid, task_description: &str) -> Result<f64> {
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
    fn create_fann_network(&self, config: &FANNConfig) -> Result<Network> {
        let mut network = Network::new(&config.layers)?;
        
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
    async fn process_with_fann(&self, text: &str, agent_id: Uuid) -> Result<ProcessedText> {
        // First get basic processing
        let mut processed = self.nlp_processor.process_text(text).await?;
        
        // Enhance with FANN network if available
        if let Some(network) = self.neural_networks.get(&agent_id) {
            // Convert semantic vector to network input
            let input: Vec<f32> = processed.semantic_vector.dimensions
                .iter()
                .take(network.get_num_input())
                .map(|&x| x as f32)
                .collect();
            
            if input.len() == network.get_num_input() {
                let output = network.run(&input)?;
                
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
        // For now, use basic processing with LSTM enhancements planned
        let processed = self.nlp_processor.process_text(text).await?;
        
        // TODO: Implement LSTM-based sequence processing
        // This would involve maintaining sequence history and using LSTM for prediction
        
        Ok(processed)
    }

    #[cfg(feature = "advanced-neural")]
    async fn train_fann_network(
        &mut self,
        agent_id: Uuid,
        input: &str,
        output: &str,
        success: bool,
    ) -> Result<()> {
        if let Some(network) = self.neural_networks.get_mut(&agent_id) {
            let processed_input = self.nlp_processor.process_text(input).await?;
            
            // Convert to training data
            let input_data: Vec<f32> = processed_input.semantic_vector.dimensions
                .iter()
                .take(network.get_num_input())
                .map(|&x| x as f32)
                .collect();
            
            let target_data = vec![if success { 1.0 } else { 0.0 }];
            
            if input_data.len() == network.get_num_input() && 
               target_data.len() == network.get_num_output() {
                // Train on single example
                network.train(&input_data, &target_data)?;
            }
        }
        
        Ok(())
    }

    #[cfg(feature = "advanced-neural")]
    async fn train_lstm_network(
        &mut self,
        agent_id: Uuid,
        input: &str,
        output: &str,
        success: bool,
    ) -> Result<()> {
        // TODO: Implement LSTM training
        // This would involve sequence-based training for time series prediction
        Ok(())
    }

    #[cfg(feature = "advanced-neural")]
    async fn predict_with_fann(&self, agent_id: Uuid, task_description: &str) -> Result<f64> {
        if let Some(network) = self.neural_networks.get(&agent_id) {
            let processed = self.nlp_processor.process_text(task_description).await?;
            
            let input: Vec<f32> = processed.semantic_vector.dimensions
                .iter()
                .take(network.get_num_input())
                .map(|&x| x as f32)
                .collect();
            
            if input.len() == network.get_num_input() {
                let output = network.run(&input)?;
                if !output.is_empty() {
                    return Ok(output[0] as f64);
                }
            }
        }
        
        Ok(0.5) // Default prediction
    }

    #[cfg(feature = "advanced-neural")]
    async fn predict_with_lstm(&self, agent_id: Uuid, task_description: &str) -> Result<f64> {
        // TODO: Implement LSTM prediction
        // This would use sequence history to predict future performance
        Ok(0.5) // Default prediction for now
    }
}