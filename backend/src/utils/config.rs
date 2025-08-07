use serde::{Deserialize, Serialize};
use std::env;

/// Configuration for the multiagent hive system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveConfig {
    pub server: ServerConfig,
    pub agents: AgentConfig,
    pub tasks: TaskConfig,
    pub resources: ResourceConfig,
    pub neural: NeuralConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub websocket_timeout_secs: u64,
    pub max_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub max_agents: usize,
    pub default_energy: f64,
    pub energy_decay_rate: f64,
    pub learning_rate: f64,
    pub max_memory_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    pub max_concurrent_tasks: usize,
    pub task_timeout_secs: u64,
    pub retry_attempts: u32,
    pub priority_levels: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub cpu_threshold: f64,
    pub memory_threshold: f64,
    pub auto_scaling_enabled: bool,
    pub monitoring_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralConfig {
    pub enable_advanced_neural: bool,
    pub batch_size: usize,
    pub learning_rate: f64,
    pub max_iterations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file_path: Option<String>,
    pub max_file_size_mb: u64,
}

impl Default for HiveConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3001,
                cors_origins: vec!["http://localhost:3000".to_string()],
                websocket_timeout_secs: 300,
                max_connections: 1000,
            },
            agents: AgentConfig {
                max_agents: 100,
                default_energy: 100.0,
                energy_decay_rate: 0.1,
                learning_rate: 0.01,
                max_memory_size: 1000,
            },
            tasks: TaskConfig {
                max_concurrent_tasks: 50,
                task_timeout_secs: 300,
                retry_attempts: 3,
                priority_levels: 4,
            },
            resources: ResourceConfig {
                cpu_threshold: 80.0,
                memory_threshold: 85.0,
                auto_scaling_enabled: true,
                monitoring_interval_secs: 30,
            },
            neural: NeuralConfig {
                enable_advanced_neural: false,
                batch_size: 32,
                learning_rate: 0.001,
                max_iterations: 1000,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                file_path: None,
                max_file_size_mb: 100,
            },
        }
    }
}

impl HiveConfig {
    /// Load configuration from environment variables and defaults
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // Server configuration
        if let Ok(host) = env::var("HIVE_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = env::var("HIVE_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                config.server.port = port_num;
            }
        }
        
        // Agent configuration
        if let Ok(max_agents) = env::var("HIVE_MAX_AGENTS") {
            if let Ok(max) = max_agents.parse::<usize>() {
                config.agents.max_agents = max;
            }
        }
        
        // Resource configuration
        if let Ok(auto_scaling) = env::var("HIVE_AUTO_SCALING") {
            config.resources.auto_scaling_enabled = auto_scaling.to_lowercase() == "true";
        }
        
        // Neural configuration
        if let Ok(advanced_neural) = env::var("HIVE_ADVANCED_NEURAL") {
            config.neural.enable_advanced_neural = advanced_neural.to_lowercase() == "true";
        }
        
        // Logging configuration
        if let Ok(log_level) = env::var("HIVE_LOG_LEVEL") {
            config.logging.level = log_level;
        }
        if let Ok(log_file) = env::var("HIVE_LOG_FILE") {
            config.logging.file_path = Some(log_file);
        }
        
        config
    }
    
    /// Validate configuration values
    pub fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }
        
        if self.agents.max_agents == 0 {
            return Err("Maximum agents must be greater than 0".to_string());
        }
        
        if self.agents.default_energy <= 0.0 {
            return Err("Default energy must be positive".to_string());
        }
        
        if self.resources.cpu_threshold < 0.0 || self.resources.cpu_threshold > 100.0 {
            return Err("CPU threshold must be between 0 and 100".to_string());
        }
        
        if self.resources.memory_threshold < 0.0 || self.resources.memory_threshold > 100.0 {
            return Err("Memory threshold must be between 0 and 100".to_string());
        }
        
        Ok(())
    }
}