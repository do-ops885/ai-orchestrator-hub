use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use std::fs;
use crate::utils::error::{HiveError, HiveResult};

/// Configuration for the multiagent hive system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveConfig {
    pub server: ServerConfig,
    pub agents: AgentConfig,
    pub tasks: TaskConfig,
    pub resources: ResourceConfig,
    pub neural: NeuralConfig,
    pub logging: LoggingConfig,
    pub performance: PerformanceConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub cpu_warning_threshold: Option<f64>,
    pub cpu_critical_threshold: Option<f64>,
    pub memory_warning_threshold: Option<f64>,
    pub memory_critical_threshold: Option<f64>,
    pub metrics_collection_interval_ms: u64,
    pub alert_check_interval_ms: u64,
    pub circuit_breaker_failure_threshold: u64,
    pub circuit_breaker_recovery_timeout_ms: u64,
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
            performance: PerformanceConfig {
                cpu_warning_threshold: Some(70.0),
                cpu_critical_threshold: Some(90.0),
                memory_warning_threshold: Some(80.0),
                memory_critical_threshold: Some(95.0),
                metrics_collection_interval_ms: 5000,
                alert_check_interval_ms: 30000,
                circuit_breaker_failure_threshold: 5,
                circuit_breaker_recovery_timeout_ms: 30000,
            },
        }
    }
}

impl HiveConfig {
    /// Load configuration from environment variables and defaults
    pub fn from_env() -> HiveResult<Self> {
        let mut config = Self::default();
        config.load_from_env()?;
        config.validate()?;
        Ok(config)
    }
    
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> HiveResult<Self> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| HiveError::ConfigurationError { 
                reason: format!("Failed to read config file: {}", e) 
            })?;
            
        let config: Self = toml::from_str(&content)
            .map_err(|e| HiveError::ConfigurationError { 
                reason: format!("Failed to parse config file: {}", e) 
            })?;
            
        config.validate()?;
        Ok(config)
    }
    
    /// Load configuration with precedence: file -> env -> defaults
    pub fn load() -> HiveResult<Self> {
        // Start with defaults
        let mut config = Self::default();
        
        // Try to load from config file if specified
        if let Ok(config_path) = env::var("HIVE_CONFIG_FILE") {
            if Path::new(&config_path).exists() {
                config = Self::from_file(config_path)?;
            }
        }
        
        // Override with environment variables
        config.load_from_env()?;
        
        // Validate final configuration
        config.validate()?;
        
        Ok(config)
    }
    
    /// Load configuration from environment variables (internal helper)
    fn load_from_env(&mut self) -> HiveResult<()> {
        // Server configuration
        if let Ok(host) = env::var("HIVE_HOST") {
            self.server.host = host;
        }
        if let Ok(port) = env::var("HIVE_PORT") {
            self.server.port = port.parse()
                .map_err(|_| HiveError::ConfigurationError { 
                    reason: format!("Invalid port number: {}", port) 
                })?;
        }
        
        // Agent configuration
        if let Ok(max_agents) = env::var("HIVE_MAX_AGENTS") {
            self.agents.max_agents = max_agents.parse()
                .map_err(|_| HiveError::ConfigurationError { 
                    reason: format!("Invalid max_agents value: {}", max_agents) 
                })?;
        }
        
        // Resource configuration
        if let Ok(auto_scaling) = env::var("HIVE_AUTO_SCALING") {
            self.resources.auto_scaling_enabled = auto_scaling.to_lowercase() == "true";
        }
        
        // Neural configuration
        if let Ok(neural_enabled) = env::var("HIVE_NEURAL_ENABLED") {
            self.neural.enabled = neural_enabled.to_lowercase() == "true";
        }
        
        Ok(())
    }
    
    /// Validate configuration values
    pub fn validate(&self) -> HiveResult<()> {
        // Validate server configuration
        if self.server.port == 0 {
            return Err(HiveError::ConfigurationError { 
                reason: "Server port cannot be 0".to_string() 
            });
        }
        
        if self.server.host.is_empty() {
            return Err(HiveError::ConfigurationError { 
                reason: "Server host cannot be empty".to_string() 
            });
        }
        
        // Validate agent configuration
        if self.agents.max_agents == 0 {
            return Err(HiveError::ConfigurationError { 
                reason: "max_agents must be greater than 0".to_string() 
            });
        }
        
        if self.agents.max_agents > 10000 {
            return Err(HiveError::ConfigurationError { 
                reason: "max_agents cannot exceed 10000".to_string() 
            });
        }
        
        // Validate task configuration
        if self.tasks.max_pending_tasks == 0 {
            return Err(HiveError::ConfigurationError { 
                reason: "max_pending_tasks must be greater than 0".to_string() 
            });
        }
        
        // Validate neural configuration
        if self.neural.enabled && self.neural.learning_rate <= 0.0 {
            return Err(HiveError::ConfigurationError { 
                reason: "Neural learning rate must be positive".to_string() 
            });
        }
        
        Ok(())
    }
    
    /// Save configuration to a TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> HiveResult<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| HiveError::ConfigurationError { 
                reason: format!("Failed to serialize config: {}", e) 
            })?;
            
        fs::write(path.as_ref(), content)
            .map_err(|e| HiveError::ConfigurationError { 
                reason: format!("Failed to write config file: {}", e) 
            })?;
            
        Ok(())
    }
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
        
        // Performance configuration
        if let Ok(cpu_warning) = env::var("HIVE_CPU_WARNING_THRESHOLD") {
            if let Ok(threshold) = cpu_warning.parse::<f64>() {
                config.performance.cpu_warning_threshold = Some(threshold);
            }
        }
        if let Ok(cpu_critical) = env::var("HIVE_CPU_CRITICAL_THRESHOLD") {
            if let Ok(threshold) = cpu_critical.parse::<f64>() {
                config.performance.cpu_critical_threshold = Some(threshold);
            }
        }
        if let Ok(memory_warning) = env::var("HIVE_MEMORY_WARNING_THRESHOLD") {
            if let Ok(threshold) = memory_warning.parse::<f64>() {
                config.performance.memory_warning_threshold = Some(threshold);
            }
        }
        if let Ok(memory_critical) = env::var("HIVE_MEMORY_CRITICAL_THRESHOLD") {
            if let Ok(threshold) = memory_critical.parse::<f64>() {
                config.performance.memory_critical_threshold = Some(threshold);
            }
        }
        if let Ok(interval) = env::var("HIVE_METRICS_INTERVAL") {
            if let Ok(ms) = interval.parse::<u64>() {
                config.performance.metrics_collection_interval_ms = ms;
            }
        }
        if let Ok(threshold) = env::var("HIVE_CIRCUIT_BREAKER_THRESHOLD") {
            if let Ok(t) = threshold.parse::<u64>() {
                config.performance.circuit_breaker_failure_threshold = t;
            }
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