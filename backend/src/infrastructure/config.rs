//! Phase 3: External Configuration Management
//!
//! Provides TOML/YAML configuration support with hot-reload capability
//! and environment variable overrides for operational flexibility.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{watch, RwLock};
use tracing::{debug, error, info, warn};

/// Main configuration structure for the MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfig {
    /// Server configuration
    pub server: ServerConfig,
    /// Metrics configuration
    pub metrics: MetricsConfig,
    /// Cache configuration
    pub cache: CacheConfig,
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Shutdown timeout in seconds
    pub shutdown_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable Prometheus metrics
    pub enabled: bool,
    /// Metrics endpoint path
    pub path: String,
    /// Metrics collection interval in seconds
    pub collection_interval_secs: u64,
    /// Tool execution histogram buckets
    pub tool_execution_buckets: Vec<f64>,
    /// Cache operation labels
    pub cache_labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Cache size in MB
    pub size_mb: usize,
    /// Default TTL in seconds
    pub default_ttl_secs: u64,
    /// Maximum TTL in seconds
    pub max_ttl_secs: u64,
    /// Cache warming enabled
    pub warming_enabled: bool,
    /// Cache invalidation enabled
    pub invalidation_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Health check enabled
    pub health_check_enabled: bool,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
    /// Dashboard enabled
    pub dashboard_enabled: bool,
    /// Dashboard port
    pub dashboard_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// CPU usage warning threshold (percentage)
    pub cpu_warning_percent: f64,
    /// CPU usage critical threshold (percentage)
    pub cpu_critical_percent: f64,
    /// Memory usage warning threshold (percentage)
    pub memory_warning_percent: f64,
    /// Memory usage critical threshold (percentage)
    pub memory_critical_percent: f64,
    /// Error rate warning threshold (errors per minute)
    pub error_rate_warning: f64,
    /// Error rate critical threshold (errors per minute)
    pub error_rate_critical: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Log format (json, text)
    pub format: String,
    /// Enable request tracing
    pub request_tracing: bool,
    /// Enable correlation IDs
    pub correlation_ids: bool,
    /// Log file path (optional)
    pub file_path: Option<String>,
    /// Maximum log file size in MB
    pub max_file_size_mb: usize,
    /// Maximum number of log files to keep
    pub max_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable authentication
    pub auth_enabled: bool,
    /// JWT secret (should be loaded from environment)
    pub jwt_secret: Option<String>,
    /// Token expiration in hours
    pub token_expiration_hours: u64,
    /// Rate limiting enabled
    pub rate_limiting_enabled: bool,
    /// Requests per minute limit
    pub requests_per_minute: u32,
    /// CORS allowed origins
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Task queue size
    pub task_queue_size: usize,
    /// CPU optimization enabled
    pub cpu_optimization_enabled: bool,
    /// Memory pool size in MB
    pub memory_pool_size_mb: usize,
    /// Connection pool size
    pub connection_pool_size: usize,
    /// Streaming enabled
    pub streaming_enabled: bool,
}

/// Configuration manager with hot-reload capability
#[derive(Clone)]
pub struct ConfigManager {
    /// Current configuration
    config: Arc<RwLock<MCPConfig>>,
    /// Configuration file path
    config_path: String,
    /// Configuration format
    format: ConfigFormat,
    /// Watch channel sender for configuration changes
    config_sender: Arc<RwLock<Option<watch::Sender<MCPConfig>>>>,
    /// Environment variable overrides
    env_overrides: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy)]
pub enum ConfigFormat {
    Toml,
    Yaml,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub async fn new(
        config_path: String,
        format: ConfigFormat,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let env_overrides = Self::load_env_overrides();

        let manager = Self {
            config: Arc::new(RwLock::new(MCPConfig::default())),
            config_path,
            format,
            config_sender: Arc::new(RwLock::new(None)),
            env_overrides,
        };

        // Load initial configuration
        manager.load_config().await?;

        Ok(manager)
    }

    /// Load configuration from file with environment overrides
    pub async fn load_config(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let config_content = tokio::fs::read_to_string(&self.config_path)
            .await
            .map_err(|e| format!("Failed to read config file {}: {}", self.config_path, e))?;

        let mut config: MCPConfig = match self.format {
            ConfigFormat::Toml => toml::from_str(&config_content)
                .map_err(|e| format!("Failed to parse TOML config: {}", e))?,
            ConfigFormat::Yaml => serde_yaml::from_str(&config_content)
                .map_err(|e| format!("Failed to parse YAML config: {}", e))?,
        };

        // Apply environment variable overrides
        self.apply_env_overrides(&mut config);

        // Update configuration
        *self.config.write().await = config.clone();

        // Notify watchers
        if let Some(ref tx) = *self.config_sender.read().await {
            let _ = tx.send(config);
        }

        info!("Configuration loaded from {}", self.config_path);
        Ok(())
    }

    /// Get current configuration
    pub async fn get_config(&self) -> MCPConfig {
        self.config.read().await.clone()
    }

    /// Watch for configuration changes
    pub async fn watch_config(
        &self,
    ) -> Result<watch::Receiver<MCPConfig>, Box<dyn std::error::Error + Send + Sync>> {
        let (_tx, rx) = watch::channel(self.get_config().await);
        *self.config_sender.write().await = Some(_tx);

        // Start file watcher
        let config_path = self.config_path.clone();
        let config_manager = self.clone();

        tokio::spawn(async move {
            use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

            let mut watcher = match RecommendedWatcher::new(
                move |res: Result<Event, notify::Error>| match res {
                    Ok(event) => {
                        if let EventKind::Modify(_) = event.kind {
                            info!("Configuration file changed, reloading...");
                            let config_manager = config_manager.clone();
                            tokio::spawn(async move {
                                if let Err(e) = config_manager.load_config().await {
                                    error!("Failed to reload configuration: {}", e);
                                }
                            });
                        }
                    }
                    Err(e) => error!("Watch error: {:?}", e),
                },
                Config::default(),
            ) {
                Ok(w) => w,
                Err(e) => {
                    error!("Failed to create file watcher: {}", e);
                    return;
                }
            };

            if let Err(e) = watcher.watch(Path::new(&config_path), RecursiveMode::NonRecursive) {
                error!("Failed to watch config file {}: {}", config_path, e);
            }

            // Keep the watcher alive
            tokio::signal::ctrl_c().await.ok();
        });

        Ok(rx)
    }

    /// Load environment variable overrides
    fn load_env_overrides() -> HashMap<String, String> {
        let mut overrides = HashMap::new();

        // Server overrides
        if let Ok(host) = std::env::var("MCP_SERVER_HOST") {
            overrides.insert("server.host".to_string(), host);
        }
        if let Ok(port) = std::env::var("MCP_SERVER_PORT") {
            overrides.insert("server.port".to_string(), port);
        }

        // Metrics overrides
        if let Ok(enabled) = std::env::var("MCP_METRICS_ENABLED") {
            overrides.insert("metrics.enabled".to_string(), enabled);
        }

        // Logging overrides
        if let Ok(level) = std::env::var("MCP_LOG_LEVEL") {
            overrides.insert("logging.level".to_string(), level);
        }

        // Security overrides
        if let Ok(jwt_secret) = std::env::var("MCP_JWT_SECRET") {
            overrides.insert("security.jwt_secret".to_string(), jwt_secret);
        }

        debug!("Loaded {} environment variable overrides", overrides.len());
        overrides
    }

    /// Apply environment variable overrides to configuration
    fn apply_env_overrides(&self, config: &mut MCPConfig) {
        for (key, value) in &self.env_overrides {
            match key.as_str() {
                "server.host" => config.server.host = value.clone(),
                "server.port" => {
                    if let Ok(port) = value.parse() {
                        config.server.port = port;
                    }
                }
                "metrics.enabled" => {
                    if let Ok(enabled) = value.parse() {
                        config.metrics.enabled = enabled;
                    }
                }
                "logging.level" => config.logging.level = value.clone(),
                "security.jwt_secret" => config.security.jwt_secret = Some(value.clone()),
                _ => warn!("Unknown environment override: {}", key),
            }
        }

        if !self.env_overrides.is_empty() {
            info!(
                "Applied {} environment variable overrides",
                self.env_overrides.len()
            );
        }
    }

    /// Validate configuration
    pub fn validate_config(&self, config: &MCPConfig) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Server validation
        if config.server.port == 0 {
            errors.push("Server port cannot be 0".to_string());
        }
        if config.server.max_connections == 0 {
            errors.push("Max connections cannot be 0".to_string());
        }

        // Metrics validation
        if config.metrics.collection_interval_secs == 0 {
            errors.push("Metrics collection interval cannot be 0".to_string());
        }

        // Cache validation
        if config.cache.size_mb == 0 {
            errors.push("Cache size cannot be 0".to_string());
        }

        // Performance validation
        if config.performance.max_concurrent_tasks == 0 {
            errors.push("Max concurrent tasks cannot be 0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for MCPConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            metrics: MetricsConfig::default(),
            cache: CacheConfig::default(),
            monitoring: MonitoringConfig::default(),
            logging: LoggingConfig::default(),
            security: SecurityConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3002,
            max_connections: 1000,
            request_timeout_secs: 30,
            shutdown_timeout_secs: 10,
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: "/metrics".to_string(),
            collection_interval_secs: 15,
            tool_execution_buckets: vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0],
            cache_labels: vec!["memory".to_string(), "disk".to_string()],
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            size_mb: 512,
            default_ttl_secs: 3600,
            max_ttl_secs: 86400,
            warming_enabled: true,
            invalidation_enabled: true,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            health_check_enabled: true,
            health_check_interval_secs: 30,
            alert_thresholds: AlertThresholds::default(),
            dashboard_enabled: true,
            dashboard_port: 8080,
        }
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_warning_percent: 70.0,
            cpu_critical_percent: 90.0,
            memory_warning_percent: 80.0,
            memory_critical_percent: 95.0,
            error_rate_warning: 5.0,
            error_rate_critical: 15.0,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            request_tracing: true,
            correlation_ids: true,
            file_path: None,
            max_file_size_mb: 100,
            max_files: 5,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            auth_enabled: false,
            jwt_secret: None,
            token_expiration_hours: 24,
            rate_limiting_enabled: true,
            requests_per_minute: 1000,
            cors_origins: vec!["*".to_string()],
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 100,
            task_queue_size: 1000,
            cpu_optimization_enabled: true,
            memory_pool_size_mb: 256,
            connection_pool_size: 50,
            streaming_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_config_loading() {
        let config_content = r#"
[server]
host = "127.0.0.1"
port = 3003

[metrics]
enabled = true

[logging]
level = "debug"
"#;

        let temp_file = NamedTempFile::new().expect("replaced unwrap");
        write!(temp_file.as_file(), "{}", config_content).expect("replaced unwrap");

        let config_manager = ConfigManager::new(
            temp_file.path().to_string_lossy().to_string(),
            ConfigFormat::Toml,
        )
        .await
        .expect("replaced unwrap");

        let config = config_manager.get_config().await;
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3003);
        assert_eq!(config.logging.level, "debug");
    }

    #[tokio::test]
    async fn test_env_overrides() {
        std::env::set_var("MCP_SERVER_PORT", "4000");
        std::env::set_var("MCP_LOG_LEVEL", "warn");

        let config_content = r#"
[server]
port = 3002

[logging]
level = "info"
"#;

        let temp_file = NamedTempFile::new().expect("replaced unwrap");
        write!(temp_file.as_file(), "{}", config_content).expect("replaced unwrap");

        let config_manager = ConfigManager::new(
            temp_file.path().to_string_lossy().to_string(),
            ConfigFormat::Toml,
        )
        .await
        .expect("replaced unwrap");

        let config = config_manager.get_config().await;
        assert_eq!(config.server.port, 4000);
        assert_eq!(config.logging.level, "warn");

        // Clean up
        std::env::remove_var("MCP_SERVER_PORT");
        std::env::remove_var("MCP_LOG_LEVEL");
    }

    #[tokio::test]
    async fn test_config_validation() {
        let config_manager = ConfigManager::new("nonexistent.toml".to_string(), ConfigFormat::Toml)
            .await
            .expect("replaced unwrap");

        let mut invalid_config = MCPConfig::default();
        invalid_config.server.port = 0;

        let validation_result = config_manager.validate_config(&invalid_config);
        assert!(validation_result.is_err());
        let errors = validation_result.unwrap_err();
        assert!(errors.contains(&"Server port cannot be 0".to_string()));
    }
}
