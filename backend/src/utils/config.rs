use crate::utils::error::{HiveError, HiveResult};



use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

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
    pub monitoring: MonitoringConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct MonitoringConfig {
    pub monitoring_interval_secs: u64,
    pub metrics_retention_days: u64,
    pub alert_threshold: f64,
    pub metrics_endpoint: String,
    pub health_endpoint: String,
    pub enable_agent_discovery: bool,
    pub enable_health_monitoring: bool,
    pub enable_performance_monitoring: bool,
    pub enable_behavior_analysis: bool,
    pub enable_dashboards: bool,
    pub enable_alerting: bool,
    pub enable_diagnostics: bool,
    pub enable_reporting: bool,
    pub enable_automation: bool,
    pub enable_external_integration: bool,
    pub diagnostics: DiagnosticsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsConfig {
    pub component_health_scores: HashMap<String, f64>,
    pub component_issues: HashMap<String, Vec<String>>,
    pub component_recommendations: HashMap<String, Vec<String>>,
    pub network_components: Vec<String>,
    pub default_health_score: f64,
    pub performance_bottlenecks: Vec<String>,
    pub optimization_opportunities: Vec<String>,
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
            monitoring: MonitoringConfig {
                monitoring_interval_secs: 5,
                metrics_retention_days: 7,
                alert_threshold: 0.8,
                metrics_endpoint: "http://localhost:8000/metrics".to_string(),
                health_endpoint: "http://localhost:8000/health".to_string(),
                enable_agent_discovery: true,
                enable_health_monitoring: true,
                enable_performance_monitoring: true,
                enable_behavior_analysis: true,
                enable_dashboards: true,
                enable_alerting: true,
                enable_diagnostics: true,
                enable_reporting: true,
                enable_automation: false,
                enable_external_integration: false,
                diagnostics: DiagnosticsConfig {
                    component_health_scores: {
                        let mut scores = HashMap::new();
                        scores.insert("database".to_string(), 0.95);
                        scores.insert("cache".to_string(), 0.88);
                        scores.insert("network".to_string(), 0.92);
                        scores
                    },
                    component_issues: {
                        let mut issues = HashMap::new();
                        issues.insert(
                            "database".to_string(),
                            vec!["Slow query performance".to_string()],
                        );
                        issues.insert(
                            "cache".to_string(),
                            vec!["High cache miss rate".to_string()],
                        );
                        issues.insert(
                            "network".to_string(),
                            vec!["Intermittent connectivity".to_string()],
                        );
                        issues
                    },
                    component_recommendations: {
                        let mut recommendations = HashMap::new();
                        recommendations.insert(
                            "database".to_string(),
                            vec![
                                "Add database indexes".to_string(),
                                "Optimize query patterns".to_string(),
                            ],
                        );
                        recommendations.insert(
                            "cache".to_string(),
                            vec![
                                "Increase cache size".to_string(),
                                "Implement cache warming".to_string(),
                            ],
                        );
                        recommendations.insert(
                            "network".to_string(),
                            vec![
                                "Implement retry logic".to_string(),
                                "Monitor network latency".to_string(),
                            ],
                        );
                        recommendations
                    },
                    network_components: vec![
                        "internal_api".to_string(),
                        "external_services".to_string(),
                        "database".to_string(),
                    ],
                    default_health_score: 0.85,
                    performance_bottlenecks: vec![
                        "Database query optimization needed".to_string(),
                        "Memory usage spikes during peak hours".to_string(),
                        "Network latency affecting response times".to_string(),
                    ],
                    optimization_opportunities: vec![
                        "Implement database query caching".to_string(),
                        "Add memory pooling for frequent allocations".to_string(),
                        "Use CDN for static assets".to_string(),
                        "Implement horizontal scaling".to_string(),
                    ],
                },
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
        let content =
            fs::read_to_string(path.as_ref()).map_err(|e| HiveError::ConfigurationError {
                reason: format!("Failed to read config file: {e}"),
            })?;

        let config: Self = toml::from_str(&content).map_err(|e| HiveError::ConfigurationError {
            reason: format!("Failed to parse config file: {e}"),
        })?;

        config.validate()?;
        Ok(config)
    }

    /// Load configuration with precedence: file -> env -> defaults
    pub fn load() -> HiveResult<Self> {
        // Start with defaults
        let mut config = Self::default();

        // Determine environment (default, development, production)
        let environment = env::var("HIVE_ENV").unwrap_or_else(|_| "default".to_string());

        // Load default settings first
        let default_path = Path::new("settings/default.toml");
        if default_path.exists() {
            config = Self::from_file(default_path)?;
        }

        // Load environment-specific settings
        let env_path_str = format!("settings/{environment}.toml");
        let env_path = Path::new(&env_path_str);
        if env_path.exists() {
            let env_config = Self::from_file(env_path)?;
            // Merge environment config with default config
            config = Self::merge_configs(config, env_config);
        }

        // Try to load from custom config file if specified
        if let Ok(config_path) = env::var("HIVE_CONFIG_FILE") {
            if Path::new(&config_path).exists() {
                let custom_config = Self::from_file(config_path)?;
                config = Self::merge_configs(config, custom_config);
            }
        }

        // Override with environment variables
        config.load_from_env()?;

        // Validate final configuration
        config.validate()?;

        Ok(config)
    }

    /// Merge two configurations with precedence (second config overrides first)
    fn merge_configs(base: Self, override_config: Self) -> Self {
        Self {
            server: Self::merge_server_config(base.server, override_config.server),
            agents: Self::merge_agent_config(base.agents, override_config.agents),
            tasks: Self::merge_task_config(base.tasks, override_config.tasks),
            resources: Self::merge_resource_config(base.resources, override_config.resources),
            neural: Self::merge_neural_config(base.neural, override_config.neural),
            logging: Self::merge_logging_config(base.logging, override_config.logging),
            performance: Self::merge_performance_config(
                base.performance,
                override_config.performance,
            ),
            monitoring: Self::merge_monitoring_config(base.monitoring, override_config.monitoring),
        }
    }

    fn merge_server_config(base: ServerConfig, override_config: ServerConfig) -> ServerConfig {
        ServerConfig {
            host: if override_config.host == "0.0.0.0" {
                base.host
            } else {
                override_config.host
            },
            port: if override_config.port == 3001 {
                base.port
            } else {
                override_config.port
            },
            cors_origins: if override_config.cors_origins.is_empty() {
                base.cors_origins
            } else {
                override_config.cors_origins
            },
            websocket_timeout_secs: if override_config.websocket_timeout_secs == 300 {
                base.websocket_timeout_secs
            } else {
                override_config.websocket_timeout_secs
            },
            max_connections: if override_config.max_connections == 1000 {
                base.max_connections
            } else {
                override_config.max_connections
            },
        }
    }

    fn merge_agent_config(base: AgentConfig, override_config: AgentConfig) -> AgentConfig {
        AgentConfig {
            max_agents: if override_config.max_agents == 100 {
                base.max_agents
            } else {
                override_config.max_agents
            },
            default_energy: if (override_config.default_energy - 100.0).abs() > f64::EPSILON {
                override_config.default_energy
            } else {
                base.default_energy
            },
            energy_decay_rate: if (override_config.energy_decay_rate - 0.1).abs() > f64::EPSILON {
                override_config.energy_decay_rate
            } else {
                base.energy_decay_rate
            },
            learning_rate: if (override_config.learning_rate - 0.01).abs() > f64::EPSILON {
                override_config.learning_rate
            } else {
                base.learning_rate
            },
            max_memory_size: if override_config.max_memory_size == 1000 {
                base.max_memory_size
            } else {
                override_config.max_memory_size
            },
        }
    }

    fn merge_task_config(base: TaskConfig, override_config: TaskConfig) -> TaskConfig {
        TaskConfig {
            max_concurrent_tasks: if override_config.max_concurrent_tasks == 50 {
                base.max_concurrent_tasks
            } else {
                override_config.max_concurrent_tasks
            },
            task_timeout_secs: if override_config.task_timeout_secs == 300 {
                base.task_timeout_secs
            } else {
                override_config.task_timeout_secs
            },
            retry_attempts: if override_config.retry_attempts == 3 {
                base.retry_attempts
            } else {
                override_config.retry_attempts
            },
            priority_levels: if override_config.priority_levels == 4 {
                base.priority_levels
            } else {
                override_config.priority_levels
            },
        }
    }

    fn merge_resource_config(
        base: ResourceConfig,
        override_config: ResourceConfig,
    ) -> ResourceConfig {
        ResourceConfig {
            cpu_threshold: if (override_config.cpu_threshold - 80.0).abs() > f64::EPSILON {
                override_config.cpu_threshold
            } else {
                base.cpu_threshold
            },
            memory_threshold: if (override_config.memory_threshold - 85.0).abs() > f64::EPSILON {
                override_config.memory_threshold
            } else {
                base.memory_threshold
            },
            auto_scaling_enabled: override_config.auto_scaling_enabled, // Always use override value
            monitoring_interval_secs: if override_config.monitoring_interval_secs == 30 {
                base.monitoring_interval_secs
            } else {
                override_config.monitoring_interval_secs
            },
        }
    }

    fn merge_neural_config(base: NeuralConfig, override_config: NeuralConfig) -> NeuralConfig {
        NeuralConfig {
            enable_advanced_neural: override_config.enable_advanced_neural,
            batch_size: if override_config.batch_size == 32 {
                base.batch_size
            } else {
                override_config.batch_size
            },
            learning_rate: if (override_config.learning_rate - 0.001).abs() > f64::EPSILON {
                override_config.learning_rate
            } else {
                base.learning_rate
            },
            max_iterations: if override_config.max_iterations == 1000 {
                base.max_iterations
            } else {
                override_config.max_iterations
            },
        }
    }

    fn merge_logging_config(base: LoggingConfig, override_config: LoggingConfig) -> LoggingConfig {
        LoggingConfig {
            level: if override_config.level == "info" {
                base.level
            } else {
                override_config.level
            },
            format: if override_config.format == "json" {
                base.format
            } else {
                override_config.format
            },
            file_path: override_config.file_path.or(base.file_path),
            max_file_size_mb: if override_config.max_file_size_mb == 100 {
                base.max_file_size_mb
            } else {
                override_config.max_file_size_mb
            },
        }
    }

    fn merge_performance_config(
        base: PerformanceConfig,
        override_config: PerformanceConfig,
    ) -> PerformanceConfig {
        PerformanceConfig {
            cpu_warning_threshold: override_config
                .cpu_warning_threshold
                .or(base.cpu_warning_threshold),
            cpu_critical_threshold: override_config
                .cpu_critical_threshold
                .or(base.cpu_critical_threshold),
            memory_warning_threshold: override_config
                .memory_warning_threshold
                .or(base.memory_warning_threshold),
            memory_critical_threshold: override_config
                .memory_critical_threshold
                .or(base.memory_critical_threshold),
            metrics_collection_interval_ms: if override_config.metrics_collection_interval_ms
                == 5000
            {
                base.metrics_collection_interval_ms
            } else {
                override_config.metrics_collection_interval_ms
            },
            alert_check_interval_ms: if override_config.alert_check_interval_ms == 30000 {
                base.alert_check_interval_ms
            } else {
                override_config.alert_check_interval_ms
            },
            circuit_breaker_failure_threshold: if override_config.circuit_breaker_failure_threshold
                == 5
            {
                base.circuit_breaker_failure_threshold
            } else {
                override_config.circuit_breaker_failure_threshold
            },
            circuit_breaker_recovery_timeout_ms: if override_config
                .circuit_breaker_recovery_timeout_ms
                == 30000
            {
                base.circuit_breaker_recovery_timeout_ms
            } else {
                override_config.circuit_breaker_recovery_timeout_ms
            },
        }
    }

    fn merge_monitoring_config(
        base: MonitoringConfig,
        override_config: MonitoringConfig,
    ) -> MonitoringConfig {
        MonitoringConfig {
            monitoring_interval_secs: if override_config.monitoring_interval_secs == 5 {
                base.monitoring_interval_secs
            } else {
                override_config.monitoring_interval_secs
            },
            metrics_retention_days: if override_config.metrics_retention_days == 7 {
                base.metrics_retention_days
            } else {
                override_config.metrics_retention_days
            },
            alert_threshold: if (override_config.alert_threshold - 0.8).abs() > f64::EPSILON {
                override_config.alert_threshold
            } else {
                base.alert_threshold
            },
            metrics_endpoint: if override_config.metrics_endpoint == "http://localhost:8000/metrics"
            {
                base.metrics_endpoint
            } else {
                override_config.metrics_endpoint
            },
            health_endpoint: if override_config.health_endpoint == "http://localhost:8000/health" {
                base.health_endpoint
            } else {
                override_config.health_endpoint
            },
            enable_agent_discovery: override_config.enable_agent_discovery,
            enable_health_monitoring: override_config.enable_health_monitoring,
            enable_performance_monitoring: override_config.enable_performance_monitoring,
            enable_behavior_analysis: override_config.enable_behavior_analysis,
            enable_dashboards: override_config.enable_dashboards,
            enable_alerting: override_config.enable_alerting,
            enable_diagnostics: override_config.enable_diagnostics,
            enable_reporting: override_config.enable_reporting,
            enable_automation: override_config.enable_automation,
            enable_external_integration: override_config.enable_external_integration,
            diagnostics: Self::merge_diagnostics_config(
                base.diagnostics,
                override_config.diagnostics,
            ),
        }
    }

    fn merge_diagnostics_config(
        base: DiagnosticsConfig,
        override_config: DiagnosticsConfig,
    ) -> DiagnosticsConfig {
        let mut merged_health_scores = base.component_health_scores;
        for (key, value) in override_config.component_health_scores {
            merged_health_scores.insert(key, value);
        }

        let mut merged_issues = base.component_issues;
        for (key, value) in override_config.component_issues {
            merged_issues.insert(key, value);
        }

        let mut merged_recommendations = base.component_recommendations;
        for (key, value) in override_config.component_recommendations {
            merged_recommendations.insert(key, value);
        }

        DiagnosticsConfig {
            component_health_scores: merged_health_scores,
            component_issues: merged_issues,
            component_recommendations: merged_recommendations,
            network_components: if override_config.network_components.is_empty() {
                base.network_components
            } else {
                override_config.network_components
            },
            default_health_score: if (override_config.default_health_score - 0.85).abs()
                > f64::EPSILON
            {
                override_config.default_health_score
            } else {
                base.default_health_score
            },
            performance_bottlenecks: if override_config.performance_bottlenecks.is_empty() {
                base.performance_bottlenecks
            } else {
                override_config.performance_bottlenecks
            },
            optimization_opportunities: if override_config.optimization_opportunities.is_empty() {
                base.optimization_opportunities
            } else {
                override_config.optimization_opportunities
            },
        }
    }

    /// Load configuration from environment variables (internal helper)
    fn load_from_env(&mut self) -> HiveResult<()> {
        // Server configuration
        if let Ok(host) = env::var("HIVE_HOST") {
            self.server.host = host;
        }
        if let Ok(port) = env::var("HIVE_PORT") {
            self.server.port = port.parse().map_err(|_| HiveError::ConfigurationError {
                reason: format!("Invalid port number: {port}"),
            })?;
        }

        // Agent configuration
        if let Ok(max_agents) = env::var("HIVE_MAX_AGENTS") {
            self.agents.max_agents =
                max_agents
                    .parse()
                    .map_err(|_| HiveError::ConfigurationError {
                        reason: format!("Invalid max_agents value: {max_agents}"),
                    })?;
        }

        // Resource configuration
        if let Ok(auto_scaling) = env::var("HIVE_AUTO_SCALING") {
            self.resources.auto_scaling_enabled = auto_scaling.to_lowercase() == "true";
        }

        // Neural configuration
        if let Ok(neural_enabled) = env::var("HIVE_NEURAL_ENABLED") {
            self.neural.enable_advanced_neural = neural_enabled.to_lowercase() == "true";
        }

        // Monitoring configuration
        if let Ok(interval) = env::var("MONITORING_INTERVAL") {
            self.monitoring.monitoring_interval_secs = interval.parse().unwrap_or(5);
        }
        if let Ok(retention) = env::var("METRICS_RETENTION") {
            self.monitoring.metrics_retention_days = retention.parse().unwrap_or(7);
        }
        if let Ok(threshold) = env::var("ALERT_THRESHOLD") {
            self.monitoring.alert_threshold = threshold.parse().unwrap_or(0.8);
        }
        if let Ok(endpoint) = env::var("METRICS_ENDPOINT") {
            self.monitoring.metrics_endpoint = endpoint;
        }
        if let Ok(endpoint) = env::var("HEALTH_ENDPOINT") {
            self.monitoring.health_endpoint = endpoint;
        }

        Ok(())
    }

    /// Validate configuration values
    pub fn validate(&self) -> HiveResult<()> {
        // Validate server configuration
        if self.server.port == 0 {
            return Err(HiveError::ConfigurationError {
                reason: "Server port cannot be 0".to_string(),
            });
        }

        if self.server.host.is_empty() {
            return Err(HiveError::ConfigurationError {
                reason: "Server host cannot be empty".to_string(),
            });
        }

        // Validate agent configuration
        if self.agents.max_agents == 0 {
            return Err(HiveError::ConfigurationError {
                reason: "max_agents must be greater than 0".to_string(),
            });
        }

        if self.agents.max_agents > 10000 {
            return Err(HiveError::ConfigurationError {
                reason: "max_agents cannot exceed 10000".to_string(),
            });
        }

        // Validate task configuration
        if self.tasks.max_concurrent_tasks == 0 {
            return Err(HiveError::ConfigurationError {
                reason: "max_concurrent_tasks must be greater than 0".to_string(),
            });
        }

        // Validate neural configuration
        if self.neural.enable_advanced_neural && self.neural.learning_rate <= 0.0 {
            return Err(HiveError::ConfigurationError {
                reason: "Neural learning rate must be positive".to_string(),
            });
        }

        Ok(())
    }

    /// Save configuration to a TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> HiveResult<()> {
        let content = toml::to_string_pretty(self).map_err(|e| HiveError::ConfigurationError {
            reason: format!("Failed to serialize config: {e}"),
        })?;

        fs::write(path.as_ref(), content).map_err(|e| HiveError::ConfigurationError {
            reason: format!("Failed to write config file: {e}"),
        })?;

        Ok(())
    }
}
