//! System initialization and component setup
//!
//! This module handles the initialization of all system components,
//! configuration loading, and dependency injection setup.

use anyhow::Context;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn, Level};

use crate::infrastructure::performance_optimizer::PerformanceConfig;
use crate::infrastructure::persistence::PersistenceConfig;
use crate::infrastructure::{IntelligentAlertConfig, StorageBackend};
use crate::neural::AdaptiveLearningConfig;
use crate::utils::config::HiveConfig;
use crate::utils::security::SecurityConfig;
use crate::utils::structured_logging::{SecurityEventDetails, SecurityEventType, StructuredLogger};
use crate::{
    AdaptiveLearningSystem, AgentRecoveryManager, AppState, CircuitBreaker, HiveCoordinator,
    IntelligentAlertingSystem, MetricsCollector, PerformanceOptimizer, PersistenceManager,
    RateLimiter, SecurityAuditor, SwarmIntelligenceEngine,
};

use chrono::Utc;
use std::time::Duration;

/// Initialize the entire system with all components
pub async fn initialize_system() -> anyhow::Result<AppState> {
    // Load and validate configuration with enhanced error handling
    let config = Arc::new(match HiveConfig::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("❌ Configuration error: {}", e);
            return Err(e.into());
        }
    });

    info!("✅ Configuration loaded and validated successfully");
    debug!(
        "Server will start on {}:{}",
        config.server.host, config.server.port
    );

    // Initialize structured logging based on configuration
    let log_level = match config.logging.level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    info!("🚀 Starting Multiagent Hive System v2.0 - Enhanced Edition");
    info!("📊 Configuration loaded: CPU-native, GPU-optional");
    info!("🔧 Initializing enhanced infrastructure components...");

    // Initialize all components
    let metrics = initialize_metrics(&config).await?;
    let intelligent_alerting = initialize_alerting(&metrics).await?;
    let circuit_breaker = initialize_circuit_breaker();
    let recovery_manager = initialize_recovery_manager();
    let swarm_intelligence = initialize_swarm_intelligence().await?;
    let persistence_manager = initialize_persistence(&config).await?;
    let adaptive_learning = initialize_adaptive_learning().await?;
    let rate_limiter = initialize_rate_limiter();
    let performance_optimizer = initialize_performance_optimizer().await?;
    let security_auditor = initialize_security_auditor();

    // Initialize the hive coordinator with enhanced capabilities
    let hive = Arc::new(RwLock::new(
        HiveCoordinator::new()
            .await
            .context("hive coordinator initialization")?,
    ));
    info!("✅ Hive coordinator initialized with enhanced error handling");

    // Log security event for system startup
    StructuredLogger::log_security_event(
        &SecurityEventType::AuthenticationSuccess,
        &SecurityEventDetails {
            client_id: "system".to_string(),
            endpoint: "startup".to_string(),
            user_agent: None,
            ip_address: None,
            timestamp: Utc::now(),
            additional_info: {
                let mut info = std::collections::HashMap::new();
                info.insert("event".to_string(), "system_startup".to_string());
                info
            },
        },
    );

    let app_state = AppState {
        hive,
        config,
        metrics: metrics.clone(),
        advanced_metrics: metrics,
        intelligent_alerting,
        circuit_breaker,
        recovery_manager,
        swarm_intelligence,
        persistence_manager,
        adaptive_learning,
        rate_limiter,
        performance_optimizer,
        security_auditor,
    };

    info!("🎯 All enhanced components initialized successfully");

    Ok(app_state)
}

/// Initialize metrics collection system
async fn initialize_metrics(config: &HiveConfig) -> anyhow::Result<Arc<MetricsCollector>> {
    // Initialize enhanced metrics collector with custom thresholds
    let metric_thresholds = crate::infrastructure::metrics::MetricThresholds {
        cpu_warning: config.performance.cpu_warning_threshold.unwrap_or(70.0),
        cpu_critical: config.performance.cpu_critical_threshold.unwrap_or(90.0),
        memory_warning: config.performance.memory_warning_threshold.unwrap_or(80.0),
        memory_critical: config.performance.memory_critical_threshold.unwrap_or(95.0),
        task_failure_rate_warning: 10.0,
        task_failure_rate_critical: 25.0,
        agent_failure_rate_warning: 5.0,
        agent_failure_rate_critical: 15.0,
        response_time_warning: 1000.0,
        response_time_critical: 5000.0,
    };
    let metrics = Arc::new(MetricsCollector::with_thresholds(1000, metric_thresholds));
    info!("✅ Enhanced metrics collector initialized with custom thresholds");

    Ok(metrics)
}

/// Initialize intelligent alerting system
async fn initialize_alerting(
    _metrics: &Arc<MetricsCollector>,
) -> anyhow::Result<Arc<IntelligentAlertingSystem>> {
    // Initialize advanced metrics collector with predictive analytics
    let advanced_metrics = Arc::new(MetricsCollector::new(2000));
    info!("🔮 Advanced metrics collector initialized with predictive analytics");

    // Initialize intelligent alerting system
    let alert_config = IntelligentAlertConfig::default();
    let intelligent_alerting = Arc::new(IntelligentAlertingSystem::new(
        advanced_metrics.clone(),
        alert_config,
    ));

    // Initialize default alert rules and notification channels
    intelligent_alerting.initialize_default_rules().await;

    // Add console notification channel
    let console_channel = crate::infrastructure::intelligent_alerting::NotificationChannel {
        id: uuid::Uuid::new_v4(),
        name: "Console".to_string(),
        channel_type: crate::infrastructure::intelligent_alerting::ChannelType::Console,
        config: crate::infrastructure::intelligent_alerting::ChannelConfig {
            endpoint: None,
            headers: std::collections::HashMap::new(),
            template: None,
            rate_limit_per_hour: None,
        },
        enabled: true,
        severity_filter: vec![], // Accept all severity levels
    };
    intelligent_alerting
        .add_notification_channel(console_channel)
        .await;
    info!("🚨 Intelligent alerting system initialized with default rules");

    Ok(intelligent_alerting)
}

/// Initialize circuit breaker for resilience
fn initialize_circuit_breaker() -> Arc<CircuitBreaker> {
    let circuit_breaker = Arc::new(CircuitBreaker::new(
        5,                       // failure threshold
        Duration::from_secs(30), // recovery timeout
    ));
    info!("✅ Circuit breaker initialized (threshold: 5, timeout: 30s)");
    circuit_breaker
}

/// Initialize agent recovery manager
fn initialize_recovery_manager() -> Arc<AgentRecoveryManager> {
    let recovery_manager = Arc::new(AgentRecoveryManager::new());
    info!("✅ Agent recovery manager initialized");
    recovery_manager
}

/// Initialize swarm intelligence engine
async fn initialize_swarm_intelligence() -> anyhow::Result<Arc<RwLock<SwarmIntelligenceEngine>>> {
    let swarm_intelligence = Arc::new(RwLock::new(SwarmIntelligenceEngine::new()));
    info!("✅ Swarm intelligence engine initialized");
    Ok(swarm_intelligence)
}

/// Initialize persistence system
async fn initialize_persistence(_config: &HiveConfig) -> anyhow::Result<Arc<PersistenceManager>> {
    // Load encryption key from secure sources
    let encryption_key = PersistenceManager::load_encryption_key();
    let encryption_enabled = encryption_key.is_some();

    if encryption_enabled {
        info!("🔐 Encryption enabled with secure key management");
    } else {
        warn!("🔓 Encryption disabled - consider enabling for production use");
    }

    // Initialize persistence system
    let persistence_config = PersistenceConfig {
        storage_backend: StorageBackend::SQLite {
            database_path: std::path::PathBuf::from("./data/hive_persistence.db"),
        },
        checkpoint_interval_minutes: 5, // Checkpoint every 5 minutes
        max_snapshots: 20,
        compression_enabled: true,
        encryption_enabled,
        backup_enabled: true,
        storage_path: std::path::PathBuf::from("./data"),
        encryption_key,
        compression_level: 6,
        backup_retention_days: 7,
        backup_location: Some(std::path::PathBuf::from("./data/backups")),
        incremental_backup: true,
    };

    // Create data directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all("./data") {
        warn!("Failed to create data directory: {}", e);
    }
    if let Err(e) = std::fs::create_dir_all("./data/backups") {
        warn!("Failed to create backup directory: {}", e);
    }

    let persistence_manager = Arc::new(match PersistenceManager::new(persistence_config).await {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to initialize persistence manager: {}", e);
            return Err(e.into());
        }
    });
    info!("✅ Persistence manager initialized with SQLite backend");

    Ok(persistence_manager)
}

/// Initialize adaptive learning system
async fn initialize_adaptive_learning() -> anyhow::Result<Arc<RwLock<AdaptiveLearningSystem>>> {
    // Initialize adaptive learning system
    let adaptive_learning_config = AdaptiveLearningConfig {
        learning_rate: 0.01,
        momentum: 0.9,
        decay_factor: 0.95,
        min_confidence_threshold: 0.7,
        pattern_retention_days: 30,
        max_patterns: 10000,
    };
    let adaptive_learning = match AdaptiveLearningSystem::new(adaptive_learning_config).await {
        Ok(system) => Arc::new(RwLock::new(system)),
        Err(e) => {
            error!("Failed to initialize adaptive learning system: {}", e);
            return Err(e);
        }
    };
    info!("✅ Adaptive learning system initialized");

    Ok(adaptive_learning)
}

/// Initialize rate limiter for API protection
fn initialize_rate_limiter() -> Arc<RateLimiter> {
    let rate_limiter = Arc::new(RateLimiter::new(
        1000, // requests per minute
        Duration::from_secs(60),
    ));
    info!("🛡️ Rate limiter initialized for API protection");
    rate_limiter
}

/// Initialize performance optimizer
async fn initialize_performance_optimizer() -> anyhow::Result<Arc<PerformanceOptimizer>> {
    // Initialize performance optimizer
    let performance_config = PerformanceConfig::default();
    let performance_optimizer = Arc::new(PerformanceOptimizer::new(performance_config));
    performance_optimizer.start_optimization().await;
    info!(
        "⚡ Performance optimizer initialized with connection pooling, caching, and CPU optimization"
    );
    Ok(performance_optimizer)
}

/// Initialize security auditor
fn initialize_security_auditor() -> Arc<SecurityAuditor> {
    let security_config = SecurityConfig::default();
    let security_auditor = Arc::new(SecurityAuditor::new(security_config.audit_logging_enabled));
    info!("🔒 Security auditor initialized with audit logging");
    security_auditor
}
