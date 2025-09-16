use crate::utils::error::HiveResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    pub cpu_cores: usize,
    pub available_memory: u64,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub simd_capabilities: Vec<String>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceProfile {
    pub profile_name: String,
    pub max_agents: usize,
    pub neural_complexity: f64,
    pub batch_size: usize,
    pub update_frequency: u64, // milliseconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HardwareClass {
    EdgeDevice, // Raspberry Pi, IoT devices
    Desktop,    // Standard desktop/laptop
    Server,     // High-performance server
    Cloud,      // Cloud instance
}

#[derive(Debug)]
pub struct ResourceManager {
    pub system_resources: Arc<RwLock<SystemResources>>,
    pub current_profile: Arc<RwLock<ResourceProfile>>,
    pub hardware_class: HardwareClass,
    pub auto_optimization: bool,
}

impl ResourceManager {
    pub async fn new() -> HiveResult<Self> {
        let system_resources = Self::detect_system_resources().await?;
        let hardware_class = Self::classify_hardware(&system_resources);
        let profile = Self::create_optimal_profile(&hardware_class, &system_resources);

        Ok(Self {
            system_resources: Arc::new(RwLock::new(system_resources)),
            current_profile: Arc::new(RwLock::new(profile)),
            hardware_class,
            auto_optimization: true,
        })
    }

    async fn detect_system_resources() -> HiveResult<SystemResources> {
        let cpu_cores = num_cpus::get();
        let available_memory = Self::get_available_memory();
        let simd_capabilities = vec!["SSE4.1".to_string(), "AVX2".to_string()]; // Simplified for Phase 2

        Ok(SystemResources {
            cpu_cores,
            available_memory,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            simd_capabilities,
            last_updated: Utc::now(),
        })
    }

    fn classify_hardware(resources: &SystemResources) -> HardwareClass {
        match (resources.cpu_cores, resources.available_memory) {
            (1..=2, 0..=2_000_000_000) => HardwareClass::EdgeDevice,
            (3..=8, 2_000_000_001..=16_000_000_000) => HardwareClass::Desktop,
            (9..=32, 16_000_000_001..=64_000_000_000) => HardwareClass::Server,
            _ => HardwareClass::Cloud,
        }
    }

    fn create_optimal_profile(
        hardware_class: &HardwareClass,
        _resources: &SystemResources,
    ) -> ResourceProfile {
        match hardware_class {
            HardwareClass::EdgeDevice => ResourceProfile {
                profile_name: "Edge Optimized".to_string(),
                max_agents: 5,
                neural_complexity: 0.3,
                batch_size: 1,
                update_frequency: 10000, // 10 seconds
            },
            HardwareClass::Desktop => ResourceProfile {
                profile_name: "Desktop Balanced".to_string(),
                max_agents: 20,
                neural_complexity: 0.7,
                batch_size: 4,
                update_frequency: 5000, // 5 seconds
            },
            HardwareClass::Server => ResourceProfile {
                profile_name: "Server Performance".to_string(),
                max_agents: 100,
                neural_complexity: 1.0,
                batch_size: 16,
                update_frequency: 1000, // 1 second
            },
            HardwareClass::Cloud => ResourceProfile {
                profile_name: "Cloud Scalable".to_string(),
                max_agents: 500,
                neural_complexity: 1.0,
                batch_size: 32,
                update_frequency: 500, // 0.5 seconds
            },
        }
    }

    pub async fn update_system_metrics(&self) -> HiveResult<()> {
        let mut resources = self.system_resources.write().await;

        // Update CPU and memory usage
        resources.cpu_usage = Self::get_cpu_usage();
        resources.memory_usage = Self::get_memory_usage();
        resources.last_updated = Utc::now();

        // Auto-optimize if enabled
        if self.auto_optimization {
            self.auto_optimize(&resources).await?;
        }

        Ok(())
    }

    async fn auto_optimize(&self, resources: &SystemResources) -> HiveResult<()> {
        let mut profile = self.current_profile.write().await;

        // Reduce load if system is under stress
        if resources.cpu_usage > 80.0 || resources.memory_usage > 85.0 {
            profile.max_agents = (profile.max_agents as f64 * 0.8) as usize;
            profile.update_frequency = (profile.update_frequency as f64 * 1.5) as u64;
            tracing::warn!(
                "System under stress, reducing load: max_agents={}, update_freq={}ms",
                profile.max_agents,
                profile.update_frequency
            );
        }
        // Increase load if system has capacity
        else if resources.cpu_usage < 50.0 && resources.memory_usage < 60.0 {
            let optimal_profile = Self::create_optimal_profile(&self.hardware_class, resources);
            if profile.max_agents < optimal_profile.max_agents {
                profile.max_agents =
                    std::cmp::min(profile.max_agents + 5, optimal_profile.max_agents);
                profile.update_frequency = std::cmp::max(
                    profile.update_frequency - 500,
                    optimal_profile.update_frequency,
                );
            }
        }

        Ok(())
    }

    fn get_available_memory() -> u64 {
        // Simplified memory detection - in production, use system APIs
        8_000_000_000 // 8GB default
    }

    fn get_cpu_usage() -> f64 {
        // Simplified CPU usage - in production, use system monitoring
        rand::random::<f64>() * 100.0
    }

    fn get_memory_usage() -> f64 {
        // Simplified memory usage - in production, use system monitoring
        rand::random::<f64>() * 100.0
    }

    pub async fn get_current_profile(&self) -> ResourceProfile {
        self.current_profile.read().await.clone()
    }

    pub async fn get_system_info(&self) -> (SystemResources, ResourceProfile, HardwareClass) {
        let resources = self.system_resources.read().await.clone();
        let profile = self.current_profile.read().await.clone();
        (resources, profile, self.hardware_class.clone())
    }

    pub async fn set_profile(&self, new_profile: ResourceProfile) -> HiveResult<()> {
        let mut profile = self.current_profile.write().await;
        *profile = new_profile;
        tracing::info!("Resource profile updated: {}", profile.profile_name);
        Ok(())
    }
}
