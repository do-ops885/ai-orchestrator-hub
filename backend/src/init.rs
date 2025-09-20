//! System initialization and component setup
//!
//! This module handles the initialization of all system components,
//! configuration loading, and dependency injection setup.

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn, Level};

use crate::infrastructure::performance_optimizer::PerformanceConfig;
use crate::infrastructure::persistence::PersistenceConfig;
use crate::infrastructure::{IntelligentAlertConfig, StorageBackend};
use crate::neural::AdaptiveLearningConfig;
use crate::utils::config::HiveConfig;
use crate::utils::error::{HiveError, HiveResult};
use crate::utils::security::SecurityConfig;
use crate::utils::structured_logging::{SecurityEventDetails, SecurityEventType, StructuredLogger};
use crate::{
    core::hive::HiveCoordinator, AdaptiveLearningSystem, AgentRecoveryManager, AppState,
    CircuitBreaker, IntelligentAlertingSystem, MetricsCollector, PerformanceOptimizer,
    PersistenceManager, RateLimiter, SecurityAuditor, SwarmIntelligenceEngine,
};
use crate::utils::auth::AuthManager;

use chrono::Utc;

/// Initialize the entire system with all components
pub async fn initialize_system() -> HiveResult<AppState> {
    // Load and validate configuration with enhanced error handling
    let config = Arc::new(match HiveConfig::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("❌ Configuration error: {e}");
            return Err(HiveError::ConfigurationError {
                reason: format!("Failed to load configuration: {e}"),
            });
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
    let auth_manager = initialize_auth_manager().await?;
    let cache_manager = initialize_cache_manager().await?;

    // Initialize the hive coordinator with enhanced capabilities
    let hive = Arc::new(RwLock::new(HiveCoordinator::new().await.map_err(|e| {
        HiveError::ResourceInitializationFailed {
            reason: format!("Failed to initialize hive coordinator: {e}"),
        }
    })?));
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
        metrics: Arc::clone(&metrics),
        advanced_metrics: metrics,
        intelligent_alerting,
        circuit_breaker,
        recovery_manager,
        swarm_intelligence,
        persistence_manager,
        cache_manager,
        adaptive_learning,
        rate_limiter,
        performance_optimizer,
        security_auditor,
        auth_manager,
        update_tx: None,
    };

    info!("🎯 All enhanced components initialized successfully");

    Ok(app_state)
}

/// Initialize metrics collection system
async fn initialize_metrics(config: &HiveConfig) -> HiveResult<Arc<MetricsCollector>> {
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
) -> HiveResult<Arc<IntelligentAlertingSystem>> {
    // Initialize advanced metrics collector with predictive analytics
    let advanced_metrics = Arc::new(MetricsCollector::new(2000));
    info!("🔮 Advanced metrics collector initialized with predictive analytics");

    // Initialize intelligent alerting system
    let alert_config = IntelligentAlertConfig::default();
    let intelligent_alerting = Arc::new(IntelligentAlertingSystem::new(
        Arc::clone(&advanced_metrics),
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
        5,                             // failure threshold
        std::time::Duration::from_secs(30), // recovery timeout
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
async fn initialize_swarm_intelligence() -> HiveResult<Arc<RwLock<SwarmIntelligenceEngine>>> {
    let swarm_intelligence = Arc::new(RwLock::new(SwarmIntelligenceEngine::new()));
    info!("✅ Swarm intelligence engine initialized");
    Ok(swarm_intelligence)
}

/// Initialize persistence system
async fn initialize_persistence(_config: &HiveConfig) -> HiveResult<Arc<PersistenceManager>> {
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

    // Create data directory if it doesn't exist (async to avoid blocking)
    if let Err(e) = tokio::task::spawn_blocking(|| std::fs::create_dir_all("./data"))
        .await
        .unwrap_or_else(|_| {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "spawn_blocking failed",
            ))
        })
    {
        warn!("Failed to create data directory: {}", e);
    }
    if let Err(e) = tokio::task::spawn_blocking(|| std::fs::create_dir_all("./data/backups"))
        .await
        .unwrap_or_else(|_| {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "spawn_blocking failed",
            ))
        })
    {
        warn!("Failed to create backup directory: {}", e);
    }

    let persistence_manager = Arc::new(match PersistenceManager::new(persistence_config).await {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to initialize persistence manager: {}", e);
            return Err(HiveError::ResourceInitializationFailed {
                reason: format!("Failed to initialize persistence manager: {e}"),
            });
        }
    });
    info!("✅ Persistence manager initialized with SQLite backend");

    Ok(persistence_manager)
}

/// Initialize adaptive learning system
async fn initialize_adaptive_learning() -> HiveResult<Arc<RwLock<AdaptiveLearningSystem>>> {
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
        std::time::Duration::from_secs(60),
    ));
    info!("🛡️ Rate limiter initialized for API protection");
    rate_limiter
}

/// Initialize performance optimizer
async fn initialize_performance_optimizer() -> HiveResult<Arc<PerformanceOptimizer>> {
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

/// Initialize authentication manager
async fn initialize_auth_manager() -> HiveResult<Arc<AuthManager>> {
    let security_auditor = initialize_security_auditor();
    let auth_manager = Arc::new(AuthManager::new(
        "your-secret-key-here-change-in-production", // JWT secret - should be from config
        "hive-system".to_string(), // issuer
        "hive-api".to_string(),     // audience
        security_auditor,
    ));
    info!("🔐 Authentication manager initialized with JWT support");
    Ok(auth_manager)
}

/// Initialize intelligent cache manager
async fn initialize_cache_manager(
) -> HiveResult<Arc<crate::infrastructure::intelligent_cache::MultiTierCacheManager>> {
    let cache_manager =
        Arc::new(crate::infrastructure::intelligent_cache::MultiTierCacheManager::new());

    // Start cache optimization background tasks
    let (l1_handle, l2_handle) = cache_manager.start_optimization();

    // Don't block on these handles, just log that they've started
    tokio::spawn(async move {
        if let Err(e) = l1_handle.await {
            tracing::error!("L1 cache optimization task failed: {}", e);
        }
    });

    tokio::spawn(async move {
        if let Err(e) = l2_handle.await {
            tracing::error!("L2 cache optimization task failed: {}", e);
        }
    });

    info!("📋 Multi-tier intelligent cache manager initialized with optimization");
    Ok(cache_manager)
}
