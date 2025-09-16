use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Custom error types for the multiagent hive system
///
/// This enum provides comprehensive error handling for all system components
/// with structured error information and proper error chaining.
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum HiveError {
    /// Agent-related errors
    #[error("Agent not found: {id}")]
    AgentNotFound { id: String },

    #[error("Agent creation failed: {reason}")]
    AgentCreationFailed { reason: String },

    #[error("Agent execution failed: {reason}")]
    AgentExecutionFailed { reason: String },

    #[error("Agent state transition failed: {from} -> {to}, reason: {reason}")]
    AgentStateTransitionFailed {
        from: String,
        to: String,
        reason: String,
    },

    #[error("Agent memory limit exceeded: {agent_id}, limit: {limit_mb}MB")]
    AgentMemoryLimitExceeded { agent_id: String, limit_mb: u64 },

    /// Task-related errors
    #[error("Task not found: {id}")]
    TaskNotFound { id: String },

    #[error("Task creation failed: {reason}")]
    TaskCreationFailed { reason: String },

    #[error("Task execution failed: {reason}")]
    TaskExecutionFailed { reason: String },

    #[error("Task queue full: {queue_name}, capacity: {capacity}")]
    TaskQueueFull { queue_name: String, capacity: usize },

    #[error("Task dependency failed: {task_id}, dependency: {dependency_id}")]
    TaskDependencyFailed {
        task_id: String,
        dependency_id: String,
    },

    /// Resource management errors
    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },

    #[error("Resource initialization failed: {reason}")]
    ResourceInitializationFailed { reason: String },

    #[error("Resource allocation failed: {resource_type}, requested: {requested}, available: {available}")]
    ResourceAllocationFailed {
        resource_type: String,
        requested: u64,
        available: u64,
    },

    #[error("Resource cleanup failed: {resource_id}, reason: {reason}")]
    ResourceCleanupFailed { resource_id: String, reason: String },

    /// Communication errors
    #[error("Communication error: {reason}")]
    Communication { reason: String },

    #[error("WebSocket error: {reason}")]
    WebSocketError { reason: String },

    #[error("WebSocket connection closed: {code}, reason: {reason}")]
    WebSocketConnectionClosed { code: u16, reason: String },

    #[error("Message parsing error: {reason}")]
    MessageParsingError { reason: String },

    #[error("Message serialization error: {message_type}, reason: {reason}")]
    MessageSerializationError {
        message_type: String,
        reason: String,
    },

    #[error("MCP protocol error: {code}, message: {message}")]
    MCPProtocolError { code: i32, message: String },

    #[error("MCP tool not found: {tool_name}")]
    MCPToolNotFound { tool_name: String },

    #[error("MCP resource not found: {resource_uri}")]
    MCPResourceNotFound { resource_uri: String },

    /// System errors
    #[error("System overloaded: {reason}")]
    SystemOverloaded { reason: String },

    #[error("Configuration error: {reason}")]
    ConfigurationError { reason: String },

    #[error("Configuration file not found: {path}")]
    ConfigurationFileNotFound { path: String },

    #[error("Database error: {reason}")]
    DatabaseError { reason: String },

    #[error("Database connection failed: {database_url}")]
    DatabaseConnectionFailed { database_url: String },

    #[error("Database migration failed: {version}, reason: {reason}")]
    DatabaseMigrationFailed { version: String, reason: String },

    /// Neural processing errors
    #[error("Neural processing error: {reason}")]
    NeuralProcessingError { reason: String },

    #[error("Neural network training failed: {network_id}, epoch: {epoch}, reason: {reason}")]
    NeuralNetworkTrainingFailed {
        network_id: String,
        epoch: u32,
        reason: String,
    },

    #[error("Neural data preprocessing failed: {data_type}, reason: {reason}")]
    NeuralDataPreprocessingFailed { data_type: String, reason: String },

    #[error("Neural model serialization failed: {model_id}, reason: {reason}")]
    NeuralModelSerializationFailed { model_id: String, reason: String },

    #[error("Processing error: {reason}")]
    ProcessingError { reason: String },

    #[error("NLP error: {reason}")]
    NLPError { reason: String },

    #[error("NLP model loading failed: {model_name}, reason: {reason}")]
    NLPModelLoadingFailed { model_name: String, reason: String },

    #[error("NLP text processing failed: {text_length} chars, reason: {reason}")]
    NLPTextProcessingFailed { text_length: usize, reason: String },

    /// Circuit breaker errors
    #[error("Circuit breaker open: {reason}")]
    CircuitBreakerOpen { reason: String },

    #[error("Circuit breaker configuration invalid: {field}, reason: {reason}")]
    CircuitBreakerConfigInvalid { field: String, reason: String },

    /// Cache and storage errors
    #[error("Cache error: {operation}, key: {key}, reason: {reason}")]
    CacheError {
        operation: String,
        key: String,
        reason: String,
    },

    #[error("Cache miss: {key}")]
    CacheMiss { key: String },

    #[error("Cache invalidation failed: {pattern}, reason: {reason}")]
    CacheInvalidationFailed { pattern: String, reason: String },

    #[error("Persistence error: {operation}, reason: {reason}")]
    PersistenceError { operation: String, reason: String },

    #[error("Snapshot creation failed: {snapshot_id}, reason: {reason}")]
    SnapshotCreationFailed { snapshot_id: String, reason: String },

    #[error("Snapshot restoration failed: {snapshot_id}, reason: {reason}")]
    SnapshotRestorationFailed { snapshot_id: String, reason: String },

    /// Metrics and monitoring errors
    #[error("Metrics collection failed: {metric_name}, reason: {reason}")]
    MetricsCollectionFailed { metric_name: String, reason: String },

    #[error("Metrics export failed: {format}, destination: {destination}, reason: {reason}")]
    MetricsExportFailed {
        format: String,
        destination: String,
        reason: String,
    },

    #[error("Alerting system error: {alert_type}, reason: {reason}")]
    AlertingSystemError { alert_type: String, reason: String },

    /// Security and authentication errors
    #[error("Authentication error: {reason}")]
    AuthenticationError { reason: String },

    #[error("Authorization error: {reason}")]
    AuthorizationError { reason: String },

    #[error("Security error: {reason}")]
    SecurityError { reason: String },

    #[error("Rate limit exceeded: {operation}, limit: {limit}, window: {window_secs}s")]
    RateLimitExceeded {
        operation: String,
        limit: u32,
        window_secs: u64,
    },

    #[error("Permission denied: {resource}, action: {action}")]
    PermissionDenied { resource: String, action: String },

    /// IO and file system errors
    #[error("IO error: {reason}")]
    IoError { reason: String },

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("File permission denied: {path}, operation: {operation}")]
    FilePermissionDenied { path: String, operation: String },

    #[error("Directory creation failed: {path}, reason: {reason}")]
    DirectoryCreationFailed { path: String, reason: String },

    /// Network and connectivity errors
    #[error("Network error: {operation}, reason: {reason}")]
    NetworkError { operation: String, reason: String },

    #[error("Connection timeout: {host}:{port}, timeout: {timeout_ms}ms")]
    ConnectionTimeout {
        host: String,
        port: u16,
        timeout_ms: u64,
    },

    #[error("DNS resolution failed: {hostname}")]
    DNSResolutionFailed { hostname: String },

    /// Validation errors
    #[error("Invalid input: {field} - {reason}")]
    ValidationError { field: String, reason: String },

    #[error("Invalid JSON: {json_path}, reason: {reason}")]
    InvalidJson { json_path: String, reason: String },

    #[error("Invalid UUID: {value}")]
    InvalidUUID { value: String },

    #[error("Invalid enum value: {field}, value: {value}, expected: {expected}")]
    InvalidEnumValue {
        field: String,
        value: String,
        expected: String,
    },

    /// Not found errors
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },

    #[error("Agent type not found: {agent_type}")]
    AgentTypeNotFound { agent_type: String },

    #[error("Task type not found: {task_type}")]
    TaskTypeNotFound { task_type: String },

    /// Timeout errors
    #[error("Timeout error: {reason}")]
    Timeout { reason: String },

    #[error("Operation timed out: {operation} after {duration_ms}ms")]
    TimeoutError { operation: String, duration_ms: u64 },

    #[error("Task execution timeout: {task_id}, timeout: {timeout_ms}ms")]
    TaskExecutionTimeout { task_id: String, timeout_ms: u64 },

    /// Generic operation errors
    #[error("Operation failed: {reason}")]
    OperationFailed { reason: String },

    #[error("Operation cancelled: {operation}")]
    OperationCancelled { operation: String },

    #[error("Operation not supported: {operation}")]
    OperationNotSupported { operation: String },

    /// Swarm intelligence errors
    #[error("Swarm coordination failed: {reason}")]
    SwarmCoordinationFailed { reason: String },

    #[error("Swarm communication failed: {agent_id}, reason: {reason}")]
    SwarmCommunicationFailed { agent_id: String, reason: String },

    #[error("Swarm optimization failed: {algorithm}, reason: {reason}")]
    SwarmOptimizationFailed { algorithm: String, reason: String },

    /// Agent-specific errors with recovery strategies
    #[error("Agent learning failed: {agent_id}, reason: {reason}")]
    AgentLearningFailed { agent_id: String, reason: String },

    #[error("Agent adaptation failed: {agent_id}, strategy: {strategy}, reason: {reason}")]
    AgentAdaptationFailed {
        agent_id: String,
        strategy: String,
        reason: String,
    },

    #[error(
        "Agent communication protocol error: {agent_id}, protocol: {protocol}, reason: {reason}"
    )]
    AgentCommunicationProtocolError {
        agent_id: String,
        protocol: String,
        reason: String,
    },

    #[error("Agent skill evolution failed: {agent_id}, skill: {skill}, reason: {reason}")]
    AgentSkillEvolutionFailed {
        agent_id: String,
        skill: String,
        reason: String,
    },

    #[error("Agent memory corruption: {agent_id}, memory_type: {memory_type}")]
    AgentMemoryCorruption {
        agent_id: String,
        memory_type: String,
    },

    #[error("Agent deadlock detected: {agent_id}, operation: {operation}")]
    AgentDeadlockDetected { agent_id: String, operation: String },

    #[error("Agent resource starvation: {agent_id}, resource: {resource}, required: {required}, available: {available}")]
    AgentResourceStarvation {
        agent_id: String,
        resource: String,
        required: u64,
        available: u64,
    },

    #[error("Agent verification failed: {agent_id}, verification_type: {verification_type}, reason: {reason}")]
    AgentVerificationFailed {
        agent_id: String,
        verification_type: String,
        reason: String,
    },

    #[error(
        "Agent collaborative learning failed: {agent_id}, peer_id: {peer_id}, reason: {reason}"
    )]
    AgentCollaborativeLearningFailed {
        agent_id: String,
        peer_id: String,
        reason: String,
    },

    #[error("Agent evolution stalled: {agent_id}, generations_without_improvement: {generations}")]
    AgentEvolutionStalled { agent_id: String, generations: u32 },

    #[error("Agent recovery failed: {agent_id}, recovery_strategy: {strategy}, reason: {reason}")]
    AgentRecoveryFailed {
        agent_id: String,
        strategy: String,
        reason: String,
    },

    /// Recovery mechanism errors
    #[error("Recovery strategy not available: {strategy_name}")]
    RecoveryStrategyNotAvailable { strategy_name: String },

    #[error("Recovery timeout exceeded: {operation}, timeout_ms: {timeout_ms}")]
    RecoveryTimeoutExceeded { operation: String, timeout_ms: u64 },

    #[error("Fallback mechanism failed: {primary_operation}, fallback: {fallback_operation}")]
    FallbackMechanismFailed {
        primary_operation: String,
        fallback_operation: String,
    },

    #[error("Circuit breaker recovery failed: {component}, reason: {reason}")]
    CircuitBreakerRecoveryFailed { component: String, reason: String },

    /// Adaptive learning errors
    #[error("Adaptive learning model corrupted: {model_id}")]
    AdaptiveLearningModelCorrupted { model_id: String },

    #[error("Adaptive learning convergence failed: {algorithm}, iterations: {iterations}")]
    AdaptiveLearningConvergenceFailed { algorithm: String, iterations: u32 },

    #[error(
        "Adaptive learning data insufficient: {required_samples}, available: {available_samples}"
    )]
    AdaptiveLearningDataInsufficient {
        required_samples: u32,
        available_samples: u32,
    },

    /// System resilience and recovery errors
    #[error("System resilience check failed: {component}, reason: {reason}")]
    SystemResilienceCheckFailed { component: String, reason: String },

    #[error("Recovery coordinator unavailable: {service}")]
    RecoveryCoordinatorUnavailable { service: String },

    #[error("Health check failed: {component}, status: {status}")]
    HealthCheckFailed { component: String, status: String },

    #[error("Graceful shutdown timeout: {component}, timeout_ms: {timeout_ms}")]
    GracefulShutdownTimeout { component: String, timeout_ms: u64 },

    #[error("Resource cleanup failed: {resource_type}, reason: {reason}")]
    ResourceTypeCleanupFailed {
        resource_type: String,
        reason: String,
    },

    #[error("State synchronization failed: {state_type}, reason: {reason}")]
    StateSynchronizationFailed { state_type: String, reason: String },

    #[error("Backup operation failed: {operation}, reason: {reason}")]
    BackupOperationFailed { operation: String, reason: String },

    #[error("Restore operation failed: {operation}, reason: {reason}")]
    RestoreOperationFailed { operation: String, reason: String },

    /// Agent lifecycle and orchestration errors
    #[error("Agent initialization failed: {agent_id}, reason: {reason}")]
    AgentInitializationFailed { agent_id: String, reason: String },

    #[error("Agent termination failed: {agent_id}, reason: {reason}")]
    AgentTerminationFailed { agent_id: String, reason: String },

    #[error("Agent health check failed: {agent_id}, health_score: {health_score}")]
    AgentHealthCheckFailed { agent_id: String, health_score: f64 },

    #[error("Agent scaling failed: {agent_type}, target_count: {target_count}, reason: {reason}")]
    AgentScalingFailed {
        agent_type: String,
        target_count: u32,
        reason: String,
    },

    #[error("Agent migration failed: {agent_id}, from_node: {from_node}, to_node: {to_node}")]
    AgentMigrationFailed {
        agent_id: String,
        from_node: String,
        to_node: String,
    },

    #[error("Agent configuration validation failed: {agent_id}, field: {field}, reason: {reason}")]
    AgentConfigurationValidationFailed {
        agent_id: String,
        field: String,
        reason: String,
    },

    #[error("Agent dependency resolution failed: {agent_id}, dependency: {dependency}")]
    AgentDependencyResolutionFailed {
        agent_id: String,
        dependency: String,
    },

    #[error("Agent resource allocation failed: {agent_id}, resource: {resource}, requested: {requested}")]
    AgentResourceAllocationFailed {
        agent_id: String,
        resource: String,
        requested: u64,
    },

    #[error("Agent communication timeout: {agent_id}, target_agent: {target_agent}, timeout_ms: {timeout_ms}")]
    AgentCommunicationTimeout {
        agent_id: String,
        target_agent: String,
        timeout_ms: u64,
    },

    #[error(
        "Agent state inconsistency: {agent_id}, expected_state: {expected}, actual_state: {actual}"
    )]
    AgentStateInconsistency {
        agent_id: String,
        expected: String,
        actual: String,
    },

    /// Task orchestration and workflow errors
    #[error("Task orchestration failed: {workflow_id}, reason: {reason}")]
    TaskOrchestrationFailed { workflow_id: String, reason: String },

    #[error("Task dependency cycle detected: {task_id}, cycle_path: {cycle_path}")]
    TaskDependencyCycleDetected { task_id: String, cycle_path: String },

    #[error("Task priority conflict: {task_id}, conflicting_task: {conflicting_task}")]
    TaskPriorityConflict {
        task_id: String,
        conflicting_task: String,
    },

    #[error("Task resource contention: {task_id}, resource: {resource}, competing_tasks: {competing_count}")]
    TaskResourceContention {
        task_id: String,
        resource: String,
        competing_count: u32,
    },

    #[error("Task deadline exceeded: {task_id}, deadline: {deadline}, actual_completion: {actual_completion}")]
    TaskDeadlineExceeded {
        task_id: String,
        deadline: String,
        actual_completion: String,
    },

    #[error("Task workflow validation failed: {workflow_id}, reason: {reason}")]
    TaskWorkflowValidationFailed { workflow_id: String, reason: String },

    #[error("Task batch processing failed: {batch_id}, processed: {processed}, failed: {failed}")]
    TaskBatchProcessingFailed {
        batch_id: String,
        processed: u32,
        failed: u32,
    },

    #[error("Task scheduling conflict: {task_id}, time_slot: {time_slot}")]
    TaskSchedulingConflict { task_id: String, time_slot: String },

    /// Swarm intelligence and coordination errors
    #[error(
        "Swarm formation failed: {formation_type}, agent_count: {agent_count}, reason: {reason}"
    )]
    SwarmFormationFailed {
        formation_type: String,
        agent_count: u32,
        reason: String,
    },

    #[error("Swarm consensus not reached: {proposal_id}, votes_for: {votes_for}, votes_against: {votes_against}")]
    SwarmConsensusNotReached {
        proposal_id: String,
        votes_for: u32,
        votes_against: u32,
    },

    #[error("Swarm leader election failed: {swarm_id}, reason: {reason}")]
    SwarmLeaderElectionFailed { swarm_id: String, reason: String },

    #[error("Swarm partition detected: {swarm_id}, partition_id: {partition_id}")]
    SwarmPartitionDetected {
        swarm_id: String,
        partition_id: String,
    },

    #[error("Swarm merge failed: {source_swarm}, target_swarm: {target_swarm}")]
    SwarmMergeFailed {
        source_swarm: String,
        target_swarm: String,
    },

    #[error("Swarm load balancing failed: {swarm_id}, reason: {reason}")]
    SwarmLoadBalancingFailed { swarm_id: String, reason: String },

    #[error("Swarm topology optimization failed: {swarm_id}, iterations: {iterations}")]
    SwarmTopologyOptimizationFailed { swarm_id: String, iterations: u32 },

    #[error("Swarm communication pattern invalid: {pattern_name}, reason: {reason}")]
    SwarmCommunicationPatternInvalid {
        pattern_name: String,
        reason: String,
    },

    /// External service integration errors
    #[error("External service unavailable: {service_name}, endpoint: {endpoint}")]
    ExternalServiceUnavailable {
        service_name: String,
        endpoint: String,
    },

    #[error("External service rate limited: {service_name}, retry_after: {retry_after_seconds}")]
    ExternalServiceRateLimited {
        service_name: String,
        retry_after_seconds: u64,
    },

    #[error("External service authentication failed: {service_name}")]
    ExternalServiceAuthenticationFailed { service_name: String },

    #[error("External service response invalid: {service_name}, status_code: {status_code}")]
    ExternalServiceResponseInvalid {
        service_name: String,
        status_code: u16,
    },

    #[error("External service timeout: {service_name}, timeout_ms: {timeout_ms}")]
    ExternalServiceTimeout {
        service_name: String,
        timeout_ms: u64,
    },

    #[error("External service configuration error: {service_name}, parameter: {parameter}")]
    ExternalServiceConfigurationError {
        service_name: String,
        parameter: String,
    },

    #[error("External service dependency missing: {service_name}")]
    ExternalServiceDependencyMissing { service_name: String },

    #[error("External service version incompatible: {service_name}, required: {required_version}, actual: {actual_version}")]
    ExternalServiceVersionIncompatible {
        service_name: String,
        required_version: String,
        actual_version: String,
    },

    /// Monitoring and observability errors
    #[error("Metrics collection disabled: {metrics_name}")]
    MetricsCollectionDisabled { metrics_name: String },

    #[error("Metrics export failed: {exporter_type}, destination: {destination}")]
    MetricsExporterFailed {
        exporter_type: String,
        destination: String,
    },

    #[error("Alert condition triggered: {alert_name}, severity: {severity}, value: {value}")]
    AlertConditionTriggered {
        alert_name: String,
        severity: String,
        value: f64,
    },

    #[error("Log aggregation failed: {aggregator_type}, reason: {reason}")]
    LogAggregationFailed {
        aggregator_type: String,
        reason: String,
    },

    #[error("Trace context missing: {operation}")]
    TraceContextMissing { operation: String },

    #[error("Distributed tracing failed: {operation}, reason: {reason}")]
    DistributedTracingFailed { operation: String, reason: String },

    #[error("Health check timeout: {component}, timeout_ms: {timeout_ms}")]
    HealthCheckTimeout { component: String, timeout_ms: u64 },

    #[error(
        "Performance threshold exceeded: {metric_name}, threshold: {threshold}, actual: {actual}"
    )]
    PerformanceThresholdExceeded {
        metric_name: String,
        threshold: f64,
        actual: f64,
    },

    #[error("Observability pipeline blocked: {pipeline_name}")]
    ObservabilityPipelineBlocked { pipeline_name: String },

    /// Configuration and deployment errors
    #[error("Configuration validation failed: {config_file}, error: {error_msg}")]
    ConfigurationValidationFailed {
        config_file: String,
        error_msg: String,
    },

    #[error("Feature flag not found: {flag_name}")]
    FeatureFlagNotFound { flag_name: String },

    #[error("Feature flag evaluation failed: {flag_name}, reason: {reason}")]
    FeatureFlagEvaluationFailed { flag_name: String, reason: String },

    #[error("Deployment rollback failed: {deployment_id}, reason: {reason}")]
    DeploymentRollbackFailed {
        deployment_id: String,
        reason: String,
    },

    #[error("Environment configuration missing: {envname}, required_key: {requiredkey}")]
    EnvironmentConfigurationMissing {
        envname: String,
        requiredkey: String,
    },

    #[error("Secret management failed: {secretname}, operation: {operation}")]
    SecretManagementFailed {
        secretname: String,
        operation: String,
    },

    #[error("Configuration drift detected: {configtype}, expected_version: {expectedversion}, actual_version: {actualversion}")]
    ConfigurationDriftDetected {
        configtype: String,
        expectedversion: String,
        actualversion: String,
    },

    #[error("Hot reload failed: {component}, reason: {reason}")]
    HotReloadFailed { component: String, reason: String },

    #[error(
        "Configuration override conflict: {overridekey}, source1: {source1}, source2: {source2}"
    )]
    ConfigurationOverrideConflict {
        overridekey: String,
        source1: String,
        source2: String,
    },
}

impl From<std::time::SystemTimeError> for HiveError {
    fn from(error: std::time::SystemTimeError) -> Self {
        HiveError::ValidationError {
            field: "timestamp".to_string(),
            reason: format!("System time conversion failed: {}", error),
        }
    }
}

// Note: anyhow already provides a blanket implementation for std::error::Error

/// Result type alias for the hive system
pub type HiveResult<T> = Result<T, HiveError>;

/// Error context for better debugging and observability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// The operation that was being performed when the error occurred
    pub operation: String,
    /// The component/module where the error originated
    pub component: String,
    /// When the error occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional contextual information
    pub additional_info: std::collections::HashMap<String, String>,
    /// Request ID for tracing (if applicable)
    pub request_id: Option<String>,
    /// User ID for user-specific errors (if applicable)
    pub user_id: Option<String>,
}

impl ErrorContext {
    /// Create a new error context
    #[must_use]
    pub fn new(operation: &str, component: &str) -> Self {
        Self {
            operation: operation.to_string(),
            component: component.to_string(),
            timestamp: chrono::Utc::now(),
            additional_info: std::collections::HashMap::new(),
            request_id: None,
            user_id: None,
        }
    }

    /// Add additional information to the error context
    #[must_use]
    pub fn with_info(mut self, key: &str, value: &str) -> Self {
        self.additional_info
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Add request ID for tracing
    #[must_use]
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Add user ID for user-specific errors
    #[must_use]
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
}

/// Helper macros for creating errors with context
#[macro_export]
macro_rules! hive_error {
    ($error_type:ident, $($field:ident: $value:expr),*) => {
        HiveError::$error_type {
            $($field: $value.to_string()),*
        }
    };
}

/// Helper function to convert anyhow errors to `HiveError`
#[must_use]
pub fn anyhow_to_hive_error(err: &anyhow::Error, operation: &str) -> HiveError {
    HiveError::OperationFailed {
        reason: format!("{operation} failed: {err}"),
    }
}

/// Helper trait for adding context to Results
pub trait ResultExt<T> {
    fn with_context(self, operation: &str, component: &str) -> Result<T, HiveError>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn with_context(self, operation: &str, component: &str) -> Result<T, HiveError> {
        self.map_err(|e| HiveError::OperationFailed {
            reason: format!("{operation} in {component}: {e}"),
        })
    }
}

/// Implement From<std::io::Error> for HiveError to support codec traits
impl From<std::io::Error> for HiveError {
    fn from(err: std::io::Error) -> Self {
        HiveError::IoError {
            reason: err.to_string(),
        }
    }
}

/// Implement From<anyhow::Error> for HiveError
impl From<anyhow::Error> for HiveError {
    fn from(err: anyhow::Error) -> Self {
        HiveError::OperationFailed {
            reason: err.to_string(),
        }
    }
}

/// Implement From<serde_json::Error> for HiveError
impl From<serde_json::Error> for HiveError {
    fn from(err: serde_json::Error) -> Self {
        HiveError::InvalidJson {
            json_path: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}
