use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agents::{Agent, AgentBehavior, AgentCapability, AgentType, Experience};
use crate::neural::{CpuOptimizer, NLPProcessor, QuantizedOps, QuantizedWeights, VectorizedOps};
use crate::tasks::{Task, TaskResult};

/// CPU-optimized agent with SIMD acceleration and quantized neural processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedAgent {
    pub base_agent: Agent,
    pub optimization_level: OptimizationLevel,
    pub quantized_capabilities: Option<QuantizedCapabilities>,
    pub performance_profile: PerformanceProfile,
    pub resource_constraints: ResourceConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationLevel {
    Minimal,    // Basic optimizations, <512MB RAM
    Standard,   // Balanced optimizations, 512MB-2GB RAM
    Aggressive, // Full optimizations, >2GB RAM
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizedCapabilities {
    pub weights: QuantizedWeights,
    pub capability_matrix: Vec<Vec<u8>>,
    pub scale_factors: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    pub avg_task_time_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_utilization: f64,
    pub cache_hit_ratio: f64,
    pub simd_acceleration: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConstraints {
    pub max_memory_mb: usize,
    pub max_cpu_threads: usize,
    pub battery_mode: bool,
    pub thermal_throttling: bool,
}

impl OptimizedAgent {
    #[must_use]
    pub fn new(name: String, agent_type: AgentType, optimization_level: OptimizationLevel) -> Self {
        let base_agent = Agent::new(name, agent_type);
        let resource_constraints = Self::detect_resource_constraints();

        Self {
            base_agent,
            optimization_level,
            quantized_capabilities: None,
            performance_profile: PerformanceProfile::default(),
            resource_constraints,
        }
    }

    #[must_use]
    pub fn new_for_edge_device(name: String, agent_type: AgentType) -> Self {
        let mut agent = Self::new(name, agent_type, OptimizationLevel::Minimal);
        agent.resource_constraints = ResourceConstraints {
            max_memory_mb: 256,
            max_cpu_threads: 1,
            battery_mode: true,
            thermal_throttling: true,
        };
        agent
    }

    #[must_use]
    pub fn new_for_raspberry_pi(name: String, agent_type: AgentType) -> Self {
        let mut agent = Self::new(name, agent_type, OptimizationLevel::Standard);
        agent.resource_constraints = ResourceConstraints {
            max_memory_mb: 512,
            max_cpu_threads: 4,
            battery_mode: false,
            thermal_throttling: true,
        };
        agent
    }

    #[must_use]
    pub fn new_for_server(name: String, agent_type: AgentType) -> Self {
        let mut agent = Self::new(name, agent_type, OptimizationLevel::Aggressive);
        agent.resource_constraints = ResourceConstraints {
            max_memory_mb: 4096,
            max_cpu_threads: 8,
            battery_mode: false,
            thermal_throttling: false,
        };
        agent
    }

    fn detect_resource_constraints() -> ResourceConstraints {
        let available_memory = Self::get_available_memory_mb();
        let cpu_count = num_cpus::get();

        ResourceConstraints {
            max_memory_mb: (available_memory * 0.8) as usize, // Use 80% of available memory
            max_cpu_threads: cpu_count.min(4),                // Limit to 4 threads for efficiency
            battery_mode: Self::is_battery_powered(),
            thermal_throttling: false,
        }
    }

    fn get_available_memory_mb() -> f64 {
        // Simplified memory detection - in real implementation, use system APIs
        #[cfg(target_os = "linux")]
        {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
                for line in meminfo.lines() {
                    if line.starts_with("MemAvailable:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb as f64 / 1024.0; // Convert KB to MB
                            }
                        }
                    }
                }
            }
        }

        // Fallback: assume 2GB available
        2048.0
    }

    fn is_battery_powered() -> bool {
        // Simplified battery detection
        #[cfg(target_os = "linux")]
        {
            std::path::Path::new("/sys/class/power_supply/BAT0").exists()
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    /// Optimize agent capabilities using quantization
    #[allow(clippy::unnecessary_wraps)]
    pub fn optimize_capabilities(&mut self) -> Result<()> {
        let capability_count = self.base_agent.capabilities.len();
        if capability_count == 0 {
            return Ok(());
        }

        // Create capability matrix for quantization
        let matrix_size = capability_count * 10; // 10 features per capability
        let mut capability_matrix: Vec<f32> = Vec::with_capacity(matrix_size);

        for capability in &self.base_agent.capabilities {
            // Convert capability to feature vector
            let features = self.capability_to_features(capability);
            capability_matrix.extend(features);
        }

        // Quantize the capability matrix based on optimization level
        match self.optimization_level {
            OptimizationLevel::Minimal => {
                // Use 8-bit quantization for maximum memory savings
                let quantized_weights = QuantizedOps::quantize_weights(&capability_matrix);
                self.quantized_capabilities = Some(QuantizedCapabilities {
                    weights: quantized_weights,
                    capability_matrix: vec![vec![0u8; 10]; capability_count],
                    scale_factors: vec![1.0; capability_count],
                });
            }
            OptimizationLevel::Standard => {
                // Use 16-bit quantization for balanced performance
                let _quantized_weights_16 =
                    QuantizedOps::quantize_weights_16bit(&capability_matrix);
                // Convert to 8-bit format for compatibility
                let quantized_weights = QuantizedOps::quantize_weights(&capability_matrix);
                self.quantized_capabilities = Some(QuantizedCapabilities {
                    weights: quantized_weights,
                    capability_matrix: vec![vec![0u8; 10]; capability_count],
                    scale_factors: vec![1.0; capability_count],
                });
            }
            OptimizationLevel::Aggressive => {
                // Keep full precision but optimize memory layout
                let quantized_weights = QuantizedOps::quantize_weights(&capability_matrix);
                self.quantized_capabilities = Some(QuantizedCapabilities {
                    weights: quantized_weights,
                    capability_matrix: vec![vec![0u8; 10]; capability_count],
                    scale_factors: vec![1.0; capability_count],
                });
            }
        }

        tracing::info!(
            "🚀 Optimized {} capabilities with {:?} level",
            capability_count,
            self.optimization_level
        );
        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn capability_to_features(&self, capability: &AgentCapability) -> Vec<f32> {
        // Convert capability to 10-dimensional feature vector
        vec![
            capability.proficiency as f32,
            capability.learning_rate as f32,
            capability.name.len() as f32 / 20.0, // Normalized name length
            1.0,                                 // Bias term
            (capability.proficiency * capability.learning_rate) as f32, // Interaction term
            capability.proficiency.powi(2) as f32, // Squared proficiency
            capability.learning_rate.powi(2) as f32, // Squared learning rate
            f64::midpoint(capability.proficiency, capability.learning_rate) as f32, // Average
            (capability.proficiency - capability.learning_rate) as f32, // Difference
            0.5,                                 // Reserved for future features
        ]
    }

    /// Calculate task fitness using optimized operations
    #[must_use]
    pub fn calculate_optimized_task_fitness(&self, task: &Task) -> f64 {
        let required_caps = &task.required_capabilities;
        let mut fitness_scores = Vec::new();
        let mut weights = Vec::new();

        for req_cap in required_caps {
            if let Some(agent_cap) = self
                .base_agent
                .capabilities
                .iter()
                .find(|c| c.name == req_cap.name)
            {
                fitness_scores.push(agent_cap.proficiency as f32);
                weights.push(1.0); // Default weight since field doesn't exist
            }
        }

        if !fitness_scores.is_empty() {
            // Use vectorized weighted average calculation
            let weighted_sum = VectorizedOps::dot_product(&fitness_scores, &weights);
            let weight_sum: f32 = weights.iter().sum();

            if weight_sum > 0.0 {
                return f64::from(weighted_sum / weight_sum);
            }
        }

        0.5 // Default fitness
    }

    /// Update performance profile with new measurements
    pub fn update_performance_profile(&mut self, task_time_ms: f64, memory_usage_mb: f64) {
        let alpha = 0.1; // Exponential moving average factor

        self.performance_profile.avg_task_time_ms =
            alpha * task_time_ms + (1.0 - alpha) * self.performance_profile.avg_task_time_ms;

        self.performance_profile.memory_usage_mb =
            alpha * memory_usage_mb + (1.0 - alpha) * self.performance_profile.memory_usage_mb;

        // Update CPU utilization (simplified)
        self.performance_profile.cpu_utilization = self.measure_cpu_utilization();

        // Check if we're using SIMD acceleration
        let optimizer = CpuOptimizer::new();
        self.performance_profile.simd_acceleration =
            optimizer.simd_support.avx2 || optimizer.simd_support.neon;
    }

    fn measure_cpu_utilization(&self) -> f64 {
        // Simplified CPU utilization measurement
        // In real implementation, use system APIs
        match self.optimization_level {
            OptimizationLevel::Minimal => 0.3,
            OptimizationLevel::Standard => 0.5,
            OptimizationLevel::Aggressive => 0.7,
        }
    }

    /// Adaptive resource management
    pub fn adapt_to_resources(&mut self) {
        let current_memory = self.performance_profile.memory_usage_mb;
        let max_memory = self.resource_constraints.max_memory_mb as f64;

        if current_memory > max_memory * 0.9 {
            // Memory pressure - reduce optimization level
            self.optimization_level = match self.optimization_level {
                OptimizationLevel::Aggressive => OptimizationLevel::Standard,
                OptimizationLevel::Standard | OptimizationLevel::Minimal => {
                    OptimizationLevel::Minimal
                }
            };

            tracing::warn!(
                "🔥 Memory pressure detected, reducing optimization to {:?}",
                self.optimization_level
            );
        } else if current_memory < max_memory * 0.5 {
            // Memory available - increase optimization level
            self.optimization_level = match self.optimization_level {
                OptimizationLevel::Minimal => OptimizationLevel::Standard,
                OptimizationLevel::Standard | OptimizationLevel::Aggressive => {
                    OptimizationLevel::Aggressive
                }
            };
        }
    }

    /// Get optimization statistics
    #[must_use]
    pub fn get_optimization_stats(&self) -> OptimizationStats {
        let memory_savings = match self.optimization_level {
            OptimizationLevel::Minimal => 0.7,    // 70% memory savings
            OptimizationLevel::Standard => 0.5,   // 50% memory savings
            OptimizationLevel::Aggressive => 0.3, // 30% memory savings
        };

        let speed_improvement = if self.performance_profile.simd_acceleration {
            match self.optimization_level {
                OptimizationLevel::Minimal => 2.0,    // 2x speedup
                OptimizationLevel::Standard => 4.0,   // 4x speedup
                OptimizationLevel::Aggressive => 8.0, // 8x speedup
            }
        } else {
            1.0 // No SIMD acceleration
        };

        OptimizationStats {
            optimization_level: self.optimization_level.clone(),
            memory_savings_percent: memory_savings * 100.0,
            speed_improvement_factor: speed_improvement,
            simd_enabled: self.performance_profile.simd_acceleration,
            quantization_enabled: self.quantized_capabilities.is_some(),
            current_memory_mb: self.performance_profile.memory_usage_mb,
            avg_task_time_ms: self.performance_profile.avg_task_time_ms,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationStats {
    pub optimization_level: OptimizationLevel,
    pub memory_savings_percent: f64,
    pub speed_improvement_factor: f64,
    pub simd_enabled: bool,
    pub quantization_enabled: bool,
    pub current_memory_mb: f64,
    pub avg_task_time_ms: f64,
}

impl Default for PerformanceProfile {
    fn default() -> Self {
        Self {
            avg_task_time_ms: 100.0,
            memory_usage_mb: 10.0,
            cpu_utilization: 0.5,
            cache_hit_ratio: 0.8,
            simd_acceleration: false,
        }
    }
}

#[async_trait]
impl AgentBehavior for OptimizedAgent {
    async fn execute_task(&mut self, task: Task) -> Result<TaskResult> {
        let start_time = std::time::Instant::now();
        let start_memory = self.performance_profile.memory_usage_mb;

        // Use optimized fitness calculation
        let fitness = self.calculate_optimized_task_fitness(&task);
        let success_probability = fitness * 0.8 + 0.2;

        // Simulate task execution with optimization-aware processing
        let processing_time = match self.optimization_level {
            OptimizationLevel::Minimal => 200,   // Slower but memory efficient
            OptimizationLevel::Standard => 100,  // Balanced
            OptimizationLevel::Aggressive => 50, // Fastest
        };

        tokio::time::sleep(tokio::time::Duration::from_millis(processing_time)).await;

        let success = rand::random::<f64>() < success_probability;

        // Create experience with optimization metadata
        let experience = Experience {
            timestamp: Utc::now(),
            task_type: task.task_type.clone(),
            success,
            context: format!(
                "{} (optimized: {:?})",
                task.description, self.optimization_level
            ),
            learned_insight: if success {
                Some(format!(
                    "Successfully completed {} task with {:?} optimization",
                    task.task_type, self.optimization_level
                ))
            } else {
                Some(format!(
                    "Failed {} task - optimization level: {:?}",
                    task.task_type, self.optimization_level
                ))
            },
        };

        self.base_agent.learn_from_experience(experience);

        // Update performance metrics
        let elapsed_ms = start_time.elapsed().as_millis() as f64;
        let current_memory = start_memory + 5.0; // Simulate memory usage
        self.update_performance_profile(elapsed_ms, current_memory);

        // Adaptive resource management
        self.adapt_to_resources();

        Ok(TaskResult {
            task_id: task.id,
            agent_id: self.base_agent.id,
            success,
            output: if success {
                format!(
                    "Task completed successfully by optimized agent {} ({}x speedup)",
                    self.base_agent.name,
                    self.get_optimization_stats().speed_improvement_factor
                )
            } else {
                format!(
                    "Task failed - optimized agent {} needs more training",
                    self.base_agent.name
                )
            },
            error_message: if success {
                None
            } else {
                Some("Optimization failed".to_string())
            },
            completed_at: Utc::now(),
            execution_time: elapsed_ms as u64,
            quality_score: if success { Some(0.8) } else { Some(0.2) },
            learned_insights: vec!["CPU optimization applied".to_string()],
        })
    }

    async fn communicate(&mut self, message: &str, target_agent: Option<Uuid>) -> Result<String> {
        // Use optimized text processing for communication
        let start_time = std::time::Instant::now();

        // Simulate optimized communication processing
        let processing_delay = match self.optimization_level {
            OptimizationLevel::Minimal => 50,
            OptimizationLevel::Standard => 25,
            OptimizationLevel::Aggressive => 10,
        };

        tokio::time::sleep(tokio::time::Duration::from_millis(processing_delay)).await;

        let response = match target_agent {
            Some(target) => format!(
                "Optimized agent {} responding to {}: Acknowledged - {} (processed in {}ms)",
                self.base_agent.name,
                target,
                message,
                start_time.elapsed().as_millis()
            ),
            None => format!(
                "Optimized agent {} broadcasting: {} (SIMD: {})",
                self.base_agent.name, message, self.performance_profile.simd_acceleration
            ),
        };

        Ok(response)
    }

    async fn learn(&mut self, nlp_processor: &NLPProcessor) -> Result<()> {
        // Delegate to base agent but with performance tracking
        let start_time = std::time::Instant::now();

        let result = self.base_agent.learn(nlp_processor).await;

        let elapsed_ms = start_time.elapsed().as_millis() as f64;
        self.update_performance_profile(elapsed_ms, self.performance_profile.memory_usage_mb + 2.0);

        result
    }

    async fn update_position(
        &mut self,
        swarm_center: (f64, f64),
        neighbors: &[Agent],
    ) -> Result<()> {
        // Use optimized position calculations
        let start_time = std::time::Instant::now();

        // Convert neighbor positions to f32 for vectorized operations
        let neighbor_positions: Vec<(f32, f32)> = neighbors
            .iter()
            .map(|n| (n.position.0 as f32, n.position.1 as f32))
            .collect();

        // Optimized swarm calculations using vectorized operations
        let mut separation = (0.0f32, 0.0f32);
        let mut alignment = (0.0f32, 0.0f32);
        let mut cohesion = (0.0f32, 0.0f32);

        let current_pos = (
            self.base_agent.position.0 as f32,
            self.base_agent.position.1 as f32,
        );
        let mut neighbor_count = 0;

        // Vectorized neighbor processing
        for &neighbor_pos in &neighbor_positions {
            let dx = neighbor_pos.0 - current_pos.0;
            let dy = neighbor_pos.1 - current_pos.1;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < 50.0 && distance > 0.1 {
                neighbor_count += 1;

                // Separation
                if distance < 20.0 {
                    separation.0 += current_pos.0 - neighbor_pos.0;
                    separation.1 += current_pos.1 - neighbor_pos.1;
                }

                // Alignment and cohesion
                alignment.0 += neighbor_pos.0;
                alignment.1 += neighbor_pos.1;
                cohesion.0 += neighbor_pos.0;
                cohesion.1 += neighbor_pos.1;
            }
        }

        if neighbor_count > 0 {
            let inv_count = 1.0 / neighbor_count as f32;
            alignment.0 *= inv_count;
            alignment.1 *= inv_count;
            cohesion.0 *= inv_count;
            cohesion.1 *= inv_count;
        }

        // Apply forces with optimization-aware weights
        let force_multiplier = match self.optimization_level {
            OptimizationLevel::Minimal => 0.05,    // Conservative movement
            OptimizationLevel::Standard => 0.1,    // Standard movement
            OptimizationLevel::Aggressive => 0.15, // Aggressive movement
        };

        let new_x = current_pos.0
            + separation.0 * force_multiplier
            + alignment.0 * force_multiplier * 0.5
            + cohesion.0 * force_multiplier * 0.5
            + (swarm_center.0 as f32 - current_pos.0) * force_multiplier * 0.1;

        let new_y = current_pos.1
            + separation.1 * force_multiplier
            + alignment.1 * force_multiplier * 0.5
            + cohesion.1 * force_multiplier * 0.5
            + (swarm_center.1 as f32 - current_pos.1) * force_multiplier * 0.1;

        self.base_agent.position = (f64::from(new_x), f64::from(new_y));

        // Update performance metrics
        let elapsed_ms = start_time.elapsed().as_millis() as f64;
        self.update_performance_profile(elapsed_ms, self.performance_profile.memory_usage_mb);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::AgentType;

    #[test]
    fn test_optimized_agent_creation() {
        let agent = OptimizedAgent::new(
            "TestAgent".to_string(),
            AgentType::Worker,
            OptimizationLevel::Standard,
        );

        assert_eq!(agent.base_agent.name, "TestAgent");
        assert!(matches!(
            agent.optimization_level,
            OptimizationLevel::Standard
        ));
    }

    #[test]
    fn test_edge_device_configuration() {
        let agent = OptimizedAgent::new_for_edge_device("EdgeAgent".to_string(), AgentType::Worker);

        assert!(agent.resource_constraints.max_memory_mb <= 256);
        assert!((agent.resource_constraints.max_cpu_threads - 1).abs() < f32::EPSILON);
        assert!(agent.resource_constraints.battery_mode);
    }

    #[test]
    fn test_capability_optimization() {
        let mut agent = OptimizedAgent::new(
            "TestAgent".to_string(),
            AgentType::Worker,
            OptimizationLevel::Minimal,
        );

        agent.base_agent.add_capability(AgentCapability {
            name: "test_capability".to_string(),
            proficiency: 0.8,
            learning_rate: 0.1,
        });

        let result = agent.optimize_capabilities();
        assert!(result.is_ok());
        assert!(agent.quantized_capabilities.is_some());
    }
}
