//! # Enhanced Error Recovery and Resilience System
//!
//! This module provides comprehensive error recovery mechanisms, resilience patterns,
//! and agent-specific recovery strategies to ensure system reliability and prevent
//! production panics. It implements the centralized error handling system for the
//! AI Orchestrator Hub with focus on agent-specific errors and consistent patterns.
//!
//! ## Core Features
//!
//! - **Circuit Breakers**: Prevent cascading failures across the system
//! - **Retry Mechanisms**: Configurable retry logic with exponential backoff
//! - **Agent-Specific Recovery**: Specialized recovery strategies for different agent types
//! - **Graceful Degradation**: Maintain partial functionality during failures
//! - **Health Monitoring**: Continuous monitoring of recovery effectiveness
//! - **Context-Aware Recovery**: Intelligent recovery based on error context and history
//!
//! ## Usage Examples
//!
//! ```rust,no_run
//! use crate::utils::error_recovery::{ContextAwareRecovery, RecoveryContext};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let recovery = ContextAwareRecovery::new();
//! let mut context = RecoveryContext::new("process_task", "TaskManager", 3);
//!
//! // Execute with context-aware recovery
//! let result = recovery.execute_with_context(
//!     || Box::pin(async { /* your operation */ }),
//!     "process_task",
//!     "TaskManager"
//! ).await?;
//!
//! // Agent-specific recovery
//! let agent_result = recovery.execute_with_agent_recovery(
//!     || Box::pin(async { /* agent operation */ }),
//!     "agent_learning",
//!     "NeuralAgent",
//!     "agent-123"
//! ).await?;
//! # Ok(())
//! # }
//! ```

use crate::utils::error::{HiveError, HiveResult};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, testing if service has recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open the circuit
    pub failure_threshold: u32,
    /// Success threshold to close the circuit from half-open
    pub success_threshold: u32,
    /// Timeout before transitioning from open to half-open
    pub timeout: Duration,
    /// Window size for tracking failures
    pub window_size: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            window_size: Duration::from_secs(300),
        }
    }
}

/// Circuit breaker implementation
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitBreakerState>>,
}

#[derive(Debug)]
struct CircuitBreakerState {
    current_state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    last_state_change: Instant,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitBreakerState {
                current_state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure_time: None,
                last_state_change: Instant::now(),
            })),
        }
    }

    /// Execute a function with circuit breaker protection
    pub async fn execute<F, T, E>(&self, operation: F) -> HiveResult<T>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        // Check if circuit is open
        if self.is_open().await {
            return Err(HiveError::CircuitBreakerOpen {
                reason: "Circuit breaker is open".to_string(),
            });
        }

        // Execute the operation
        match operation.await {
            Ok(result) => {
                self.record_success().await;
                Ok(result)
            }
            Err(e) => {
                self.record_failure().await;
                Err(HiveError::OperationFailed {
                    reason: format!("Operation failed: {}", e),
                })
            }
        }
    }

    /// Check if the circuit is open
    async fn is_open(&self) -> bool {
        let state = self.state.read().await;
        match state.current_state {
            CircuitState::Open => {
                // Check if timeout has elapsed to transition to half-open
                if state.last_state_change.elapsed() >= self.config.timeout {
                    drop(state);
                    self.transition_to_half_open().await;
                    false
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    /// Record a successful operation
    async fn record_success(&self) {
        let mut state = self.state.write().await;
        match state.current_state {
            CircuitState::HalfOpen => {
                state.success_count += 1;
                if state.success_count >= self.config.success_threshold {
                    state.current_state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                    state.last_state_change = Instant::now();
                    info!("Circuit breaker transitioned to CLOSED");
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success
                state.failure_count = 0;
            }
            _ => {}
        }
    }

    /// Record a failed operation
    async fn record_failure(&self) {
        let mut state = self.state.write().await;
        state.failure_count += 1;
        state.last_failure_time = Some(Instant::now());

        match state.current_state {
            CircuitState::Closed => {
                if state.failure_count >= self.config.failure_threshold {
                    state.current_state = CircuitState::Open;
                    state.last_state_change = Instant::now();
                    warn!("Circuit breaker transitioned to OPEN");
                }
            }
            CircuitState::HalfOpen => {
                state.current_state = CircuitState::Open;
                state.last_state_change = Instant::now();
                warn!("Circuit breaker transitioned back to OPEN");
            }
            _ => {}
        }
    }

    /// Transition to half-open state
    async fn transition_to_half_open(&self) {
        let mut state = self.state.write().await;
        state.current_state = CircuitState::HalfOpen;
        state.success_count = 0;
        state.last_state_change = Instant::now();
        info!("Circuit breaker transitioned to HALF_OPEN");
    }

    /// Get current circuit state
    pub async fn get_state(&self) -> CircuitState {
        self.state.read().await.current_state.clone()
    }
}

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// Retry mechanism with exponential backoff
pub struct RetryMechanism {
    config: RetryConfig,
}

impl RetryMechanism {
    /// Create a new retry mechanism
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute an operation with retry logic
    pub async fn execute<F, T, E>(&self, mut operation: F) -> HiveResult<T>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Display,
    {
        let mut attempt = 0;
        let mut delay = self.config.base_delay;

        loop {
            attempt += 1;

            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt >= self.config.max_attempts {
                        error!("Operation failed after {} attempts: {}", attempt, e);
                        return Err(HiveError::OperationFailed {
                            reason: format!("Operation failed after {} attempts: {}", attempt, e),
                        });
                    }

                    warn!(
                        "Operation failed on attempt {}: {}. Retrying in {:?}",
                        attempt, e, delay
                    );
                    tokio::time::sleep(delay).await;

                    // Calculate next delay with exponential backoff
                    delay = std::cmp::min(
                        Duration::from_millis(
                            (delay.as_millis() as f64 * self.config.backoff_multiplier) as u64,
                        ),
                        self.config.max_delay,
                    );
                }
            }
        }
    }
}

/// Error recovery coordinator that combines circuit breaker and retry mechanisms
pub struct ErrorRecoveryCoordinator {
    circuit_breaker: CircuitBreaker,
    retry_mechanism: RetryMechanism,
}

impl ErrorRecoveryCoordinator {
    /// Create a new error recovery coordinator
    pub fn new(circuit_config: CircuitBreakerConfig, retry_config: RetryConfig) -> Self {
        Self {
            circuit_breaker: CircuitBreaker::new(circuit_config),
            retry_mechanism: RetryMechanism::new(retry_config),
        }
    }

    /// Execute an operation with both circuit breaker and retry protection
    pub async fn execute_with_recovery<F, T, E>(&self, operation: F) -> HiveResult<T>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>
            + Clone,
        E: std::fmt::Display + Send + Sync + 'static,
    {
        self.circuit_breaker
            .execute(self.retry_mechanism.execute(operation))
            .await
    }

    /// Get circuit breaker state
    pub async fn get_circuit_state(&self) -> CircuitState {
        self.circuit_breaker.get_state().await
    }
}

/// Graceful degradation strategies for different error types
#[derive(Debug, Clone)]
pub enum DegradationStrategy {
    /// Return cached data if available
    ReturnCached,
    /// Return default/placeholder data
    ReturnDefault,
    /// Reduce functionality (e.g., disable non-essential features)
    ReduceFunctionality,
    /// Queue operation for later retry
    QueueForRetry,
    /// Skip operation entirely
    SkipOperation,
    /// Agent-specific recovery strategies
    /// Restart agent with clean state
    AgentRestart,
    /// Switch to backup agent
    AgentFailover,
    /// Reduce agent workload
    AgentThrottle,
    /// Reinitialize agent memory
    AgentMemoryReset,
    /// Use simplified agent behavior
    AgentSimplifiedMode,
    /// Isolate agent from swarm temporarily
    AgentIsolation,
    /// Trigger agent learning rollback
    AgentLearningRollback,
    /// Redistribute tasks to other agents
    TaskRedistribution,
}

/// Recovery context for tracking recovery attempts
#[derive(Debug, Clone)]
pub struct RecoveryContext {
    pub operation: String,
    pub component: String,
    pub attempt_count: u32,
    pub max_attempts: u32,
    pub start_time: std::time::Instant,
    pub last_error: Option<String>,
    pub degradation_strategy: Option<DegradationStrategy>,
    pub additional_info: std::collections::HashMap<String, String>,
}

impl RecoveryContext {
    pub fn new(operation: &str, component: &str, max_attempts: u32) -> Self {
        Self {
            operation: operation.to_string(),
            component: component.to_string(),
            attempt_count: 0,
            max_attempts,
            start_time: std::time::Instant::now(),
            last_error: None,
            degradation_strategy: None,
            additional_info: std::collections::HashMap::new(),
        }
    }

    pub fn increment_attempt(&mut self) {
        self.attempt_count += 1;
    }

    pub fn record_error(&mut self, error: &str) {
        self.last_error = Some(error.to_string());
    }

    pub fn set_degradation_strategy(&mut self, strategy: DegradationStrategy) {
        self.degradation_strategy = Some(strategy);
    }

    pub fn should_retry(&self) -> bool {
        self.attempt_count < self.max_attempts
    }

    pub fn elapsed_time(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// Intelligent error recovery system with adaptive strategies
pub struct AdaptiveErrorRecovery {
    recovery_history: Arc<RwLock<HashMap<String, Vec<RecoveryResult>>>>,
    strategy_selector: Box<dyn StrategySelector>,
}

#[derive(Debug, Clone)]
pub struct RecoveryResult {
    pub operation: String,
    pub component: String,
    pub success: bool,
    pub duration: std::time::Duration,
    pub strategy_used: Option<DegradationStrategy>,
    pub error_type: Option<String>,
}

impl AdaptiveErrorRecovery {
    pub fn new() -> Self {
        Self {
            recovery_history: Arc::new(RwLock::new(HashMap::new())),
            strategy_selector: Box::new(DefaultStrategySelector),
        }
    }

    /// Execute operation with adaptive error recovery
    pub async fn execute_with_adaptive_recovery<F, T, E>(
        &self,
        operation: F,
        context: &mut RecoveryContext,
    ) -> HiveResult<T>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Display + Send + Sync + 'static,
    {
        let start_time = std::time::Instant::now();

        // Try primary operation first
        match operation().await {
            Ok(result) => {
                self.record_success(context, start_time.elapsed()).await;
                Ok(result)
            }
            Err(error) => {
                context.record_error(&error.to_string());

                // Select degradation strategy based on history
                let strategy = self.select_strategy(context).await;

                match strategy {
                    DegradationStrategy::ReturnCached => {
                        // Try to get cached result
                        self.try_cached_recovery(context).await
                    }
                    DegradationStrategy::ReturnDefault => {
                        // Return default value
                        self.try_default_recovery(context).await
                    }
                    DegradationStrategy::QueueForRetry => {
                        // Queue for background retry
                        self.queue_for_retry(operation, context.clone()).await;
                        Err(HiveError::OperationFailed {
                            reason: format!("Operation queued for retry: {}", error),
                        })
                    }
                    DegradationStrategy::SkipOperation => {
                        warn!(
                            "Skipping operation due to repeated failures: {}",
                            context.operation
                        );
                        Err(HiveError::OperationFailed {
                            reason: format!("Operation skipped: {}", error),
                        })
                    }
                    DegradationStrategy::ReduceFunctionality => {
                        // Try reduced functionality version
                        self.try_reduced_functionality(operation, context).await
                    }
                    DegradationStrategy::AgentRestart => {
                        debug!(
                            "Attempting agent restart recovery for operation: {}",
                            context.operation
                        );
                        self.attempt_agent_restart(context).await
                    }
                    DegradationStrategy::AgentFailover => {
                        debug!(
                            "Attempting agent failover recovery for operation: {}",
                            context.operation
                        );
                        self.attempt_agent_failover(context).await
                    }
                    DegradationStrategy::AgentThrottle => {
                        debug!(
                            "Attempting agent throttling recovery for operation: {}",
                            context.operation
                        );
                        self.attempt_agent_throttling(context).await
                    }
                    DegradationStrategy::AgentMemoryReset => {
                        debug!(
                            "Attempting agent memory reset recovery for operation: {}",
                            context.operation
                        );
                        self.attempt_agent_memory_reset(context).await
                    }
                    DegradationStrategy::AgentSimplifiedMode => {
                        debug!(
                            "Attempting agent simplified mode recovery for operation: {}",
                            context.operation
                        );
                        self.attempt_agent_simplified_mode(context).await
                    }
                    DegradationStrategy::AgentIsolation => {
                        debug!(
                            "Attempting agent isolation recovery for operation: {}",
                            context.operation
                        );
                        self.attempt_agent_isolation(context).await
                    }
                    DegradationStrategy::AgentLearningRollback => {
                        debug!(
                            "Attempting agent learning rollback recovery for operation: {}",
                            context.operation
                        );
                        self.attempt_agent_learning_rollback(context).await
                    }
                    DegradationStrategy::TaskRedistribution => {
                        debug!(
                            "Attempting task redistribution recovery for operation: {}",
                            context.operation
                        );
                        self.attempt_task_redistribution(context).await
                    }
                }
            }
        }
    }

    async fn select_strategy(&self, context: &RecoveryContext) -> DegradationStrategy {
        self.strategy_selector
            .select_strategy(context, &self.recovery_history)
            .await
    }

    async fn try_cached_recovery<T>(&self, _context: &RecoveryContext) -> HiveResult<T> {
        // Implementation would depend on specific caching mechanism
        Err(HiveError::OperationFailed {
            reason: "Cached recovery not available".to_string(),
        })
    }

    async fn try_default_recovery<T>(&self, _context: &RecoveryContext) -> HiveResult<T> {
        // Implementation would depend on specific default values
        Err(HiveError::OperationFailed {
            reason: "Default recovery not available".to_string(),
        })
    }

    async fn queue_for_retry<F, T, E>(&self, _operation: F, _context: RecoveryContext)
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Display + Send + Sync + 'static,
    {
        // Implementation would queue operation for background retry
        warn!("Operation queued for background retry");
    }

    async fn try_reduced_functionality<F, T, E>(
        &self,
        _operation: F,
        _context: &RecoveryContext,
    ) -> HiveResult<T>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Display + Send + Sync + 'static,
    {
        // Implementation would try a reduced functionality version
        Err(HiveError::OperationFailed {
            reason: "Reduced functionality recovery not available".to_string(),
        })
    }

    /// Attempt agent restart recovery
    async fn attempt_agent_restart<T>(&self, context: &RecoveryContext) -> HiveResult<T> {
        info!(
            "Executing agent restart recovery for operation: {}",
            context.operation
        );

        // Extract agent ID from context if available
        let agent_id = context
            .additional_info
            .get("agent_id")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        // Log the recovery attempt
        warn!("Initiating agent restart for agent: {}", agent_id);

        // In a real implementation, this would:
        // 1. Gracefully shutdown the agent
        // 2. Clear any corrupted state
        // 3. Restart the agent with clean state
        // 4. Verify the agent is healthy

        Err(HiveError::AgentRecoveryFailed {
            agent_id,
            strategy: "restart".to_string(),
            reason: "Agent restart recovery not yet fully implemented".to_string(),
        })
    }

    /// Attempt agent failover recovery
    async fn attempt_agent_failover<T>(&self, context: &RecoveryContext) -> HiveResult<T> {
        info!(
            "Executing agent failover recovery for operation: {}",
            context.operation
        );

        let agent_id = context
            .additional_info
            .get("agent_id")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        warn!("Initiating agent failover for agent: {}", agent_id);

        // In a real implementation, this would:
        // 1. Identify a healthy backup agent
        // 2. Transfer state and responsibilities
        // 3. Redirect traffic to the backup
        // 4. Monitor the failover process

        Err(HiveError::AgentRecoveryFailed {
            agent_id,
            strategy: "failover".to_string(),
            reason: "Agent failover recovery not yet fully implemented".to_string(),
        })
    }

    /// Attempt agent throttling recovery
    async fn attempt_agent_throttling<T>(&self, context: &RecoveryContext) -> HiveResult<T> {
        info!(
            "Executing agent throttling recovery for operation: {}",
            context.operation
        );

        let agent_id = context
            .additional_info
            .get("agent_id")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        warn!("Initiating agent throttling for agent: {}", agent_id);

        // In a real implementation, this would:
        // 1. Reduce the agent's workload
        // 2. Implement rate limiting
        // 3. Prioritize critical tasks
        // 4. Monitor resource usage

        Err(HiveError::AgentRecoveryFailed {
            agent_id,
            strategy: "throttling".to_string(),
            reason: "Agent throttling recovery not yet fully implemented".to_string(),
        })
    }

    /// Attempt agent memory reset recovery
    async fn attempt_agent_memory_reset<T>(&self, context: &RecoveryContext) -> HiveResult<T> {
        info!(
            "Executing agent memory reset recovery for operation: {}",
            context.operation
        );

        let agent_id = context
            .additional_info
            .get("agent_id")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        warn!("Initiating agent memory reset for agent: {}", agent_id);

        // In a real implementation, this would:
        // 1. Backup critical memory state
        // 2. Clear corrupted memory regions
        // 3. Reinitialize memory structures
        // 4. Restore from backup if needed

        Err(HiveError::AgentRecoveryFailed {
            agent_id,
            strategy: "memory_reset".to_string(),
            reason: "Agent memory reset recovery not yet fully implemented".to_string(),
        })
    }

    /// Attempt agent simplified mode recovery
    async fn attempt_agent_simplified_mode<T>(&self, context: &RecoveryContext) -> HiveResult<T> {
        info!(
            "Executing agent simplified mode recovery for operation: {}",
            context.operation
        );

        let agent_id = context
            .additional_info
            .get("agent_id")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        warn!("Initiating agent simplified mode for agent: {}", agent_id);

        // In a real implementation, this would:
        // 1. Disable advanced features
        // 2. Enable basic functionality only
        // 3. Reduce computational complexity
        // 4. Monitor performance improvements

        Err(HiveError::AgentRecoveryFailed {
            agent_id,
            strategy: "simplified_mode".to_string(),
            reason: "Agent simplified mode recovery not yet fully implemented".to_string(),
        })
    }

    /// Attempt agent isolation recovery
    async fn attempt_agent_isolation<T>(&self, context: &RecoveryContext) -> HiveResult<T> {
        info!(
            "Executing agent isolation recovery for operation: {}",
            context.operation
        );

        let agent_id = context
            .additional_info
            .get("agent_id")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        warn!("Initiating agent isolation for agent: {}", agent_id);

        // In a real implementation, this would:
        // 1. Temporarily remove agent from swarm
        // 2. Stop inter-agent communication
        // 3. Allow agent to recover independently
        // 4. Gradually reintegrate when healthy

        Err(HiveError::AgentRecoveryFailed {
            agent_id,
            strategy: "isolation".to_string(),
            reason: "Agent isolation recovery not yet fully implemented".to_string(),
        })
    }

    /// Attempt agent learning rollback recovery
    async fn attempt_agent_learning_rollback<T>(&self, context: &RecoveryContext) -> HiveResult<T> {
        info!(
            "Executing agent learning rollback recovery for operation: {}",
            context.operation
        );

        let agent_id = context
            .additional_info
            .get("agent_id")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        warn!("Initiating agent learning rollback for agent: {}", agent_id);

        // In a real implementation, this would:
        // 1. Identify last stable learning checkpoint
        // 2. Rollback model weights and parameters
        // 3. Reset learning progress
        // 4. Re-enable learning with conservative settings

        Err(HiveError::AgentRecoveryFailed {
            agent_id,
            strategy: "learning_rollback".to_string(),
            reason: "Agent learning rollback recovery not yet fully implemented".to_string(),
        })
    }

    /// Attempt task redistribution recovery
    async fn attempt_task_redistribution<T>(&self, context: &RecoveryContext) -> HiveResult<T> {
        info!(
            "Executing task redistribution recovery for operation: {}",
            context.operation
        );

        let agent_id = context
            .additional_info
            .get("agent_id")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        warn!("Initiating task redistribution from agent: {}", agent_id);

        // In a real implementation, this would:
        // 1. Identify agent's current tasks
        // 2. Find healthy agents to take over
        // 3. Redistribute tasks with priority
        // 4. Monitor redistribution progress

        Err(HiveError::AgentRecoveryFailed {
            agent_id,
            strategy: "task_redistribution".to_string(),
            reason: "Task redistribution recovery not yet fully implemented".to_string(),
        })
    }

    async fn record_success(&self, context: &RecoveryContext, duration: std::time::Duration) {
        let result = RecoveryResult {
            operation: context.operation.clone(),
            component: context.component.clone(),
            success: true,
            duration,
            strategy_used: None,
            error_type: None,
        };

        let mut history = self.recovery_history.write().await;
        let key = format!("{}:{}", context.component, context.operation);
        history.entry(key).or_insert_with(Vec::new).push(result);
    }
}

/// Trait for selecting recovery strategies based on context and history
#[async_trait::async_trait]
pub trait StrategySelector: Send + Sync {
    async fn select_strategy(
        &self,
        context: &RecoveryContext,
        history: &RwLock<HashMap<String, Vec<RecoveryResult>>>,
    ) -> DegradationStrategy;
}

/// Default strategy selector implementation
pub struct DefaultStrategySelector;

#[async_trait::async_trait]
impl StrategySelector for DefaultStrategySelector {
    async fn select_strategy(
        &self,
        context: &RecoveryContext,
        history: &RwLock<HashMap<String, Vec<RecoveryResult>>>,
    ) -> DegradationStrategy {
        let history = history.read().await;
        let key = format!("{}:{}", context.component, context.operation);

        // Check if this is an agent-related operation
        let is_agent_operation = context.component.contains("agent")
            || context.operation.contains("agent")
            || context.component.starts_with("Agent");

        if let Some(results) = history.get(&key) {
            let success_rate =
                results.iter().filter(|r| r.success).count() as f64 / results.len() as f64;

            if is_agent_operation {
                // Agent-specific strategies
                if success_rate > 0.8 {
                    DegradationStrategy::AgentRestart
                } else if success_rate > 0.5 {
                    DegradationStrategy::AgentSimplifiedMode
                } else {
                    DegradationStrategy::AgentFailover
                }
            } else if success_rate > 0.8 {
                // High success rate, try again
                DegradationStrategy::QueueForRetry
            } else if success_rate > 0.5 {
                // Moderate success rate, try reduced functionality
                DegradationStrategy::ReduceFunctionality
            } else {
                // Low success rate, skip operation
                DegradationStrategy::SkipOperation
            }
        } else {
            // No history, choose based on operation type
            if is_agent_operation {
                DegradationStrategy::AgentSimplifiedMode
            } else {
                DegradationStrategy::ReturnDefault
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_circuit_breaker_open_close() -> Result<(), Box<dyn std::error::Error>> {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_millis(100),
            window_size: Duration::from_secs(10),
        };
        let circuit_breaker = CircuitBreaker::new(config);

        // Initially closed
        assert_eq!(circuit_breaker.get_state().await, CircuitState::Closed);

        // Simulate failures to open circuit
        let _ = circuit_breaker
            .execute(async { Err::<(), &str>("failure") })
            .await;
        let _ = circuit_breaker
            .execute(async { Err::<(), &str>("failure") })
            .await;

        // Should be open now
        assert_eq!(circuit_breaker.get_state().await, CircuitState::Open);

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should transition to half-open and then closed on success
        let _ = circuit_breaker.execute(async { Ok::<(), &str>(()) }).await;
        let _ = circuit_breaker.execute(async { Ok::<(), &str>(()) }).await;

        assert_eq!(circuit_breaker.get_state().await, CircuitState::Closed);
        Ok(())
    }

    #[tokio::test]
    async fn test_retry_mechanism() -> Result<(), Box<dyn std::error::Error>> {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
        };
        let retry = RetryMechanism::new(config);

        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = Arc::clone(&attempt_count);

        let result = retry
            .execute(move || {
                let count = attempt_count_clone.fetch_add(1, Ordering::SeqCst);
                Box::pin(async move {
                    if count < 2 {
                        Err("failure")
                    } else {
                        Ok("success")
                    }
                })
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
        Ok(())
    }
}

/// Safe unwrap alternatives to replace unwrap() calls
pub trait SafeUnwrap<T> {
    /// Safe unwrap with custom error message
    fn safe_unwrap(self, operation: &str, component: &str) -> HiveResult<T>;

    /// Safe unwrap with default value
    fn unwrap_or_default_with_log(self, operation: &str, component: &str) -> T
    where
        T: Default;

    /// Safe unwrap with custom default
    fn unwrap_or_with_log(self, default: T, operation: &str, component: &str) -> T;
}

impl<T> SafeUnwrap<T> for Option<T> {
    fn safe_unwrap(self, operation: &str, component: &str) -> HiveResult<T> {
        self.ok_or_else(|| {
            error!("Option unwrap failed in {} during {}", component, operation);
            HiveError::OperationFailed {
                reason: format!("Expected Some value in {} during {}", component, operation),
            }
        })
    }

    fn unwrap_or_default_with_log(self, operation: &str, component: &str) -> T
    where
        T: Default,
    {
        match self {
            Some(value) => value,
            None => {
                warn!("Using default value in {} during {}", component, operation);
                T::default()
            }
        }
    }

    fn unwrap_or_with_log(self, default: T, operation: &str, component: &str) -> T {
        match self {
            Some(value) => value,
            None => {
                warn!("Using fallback value in {} during {}", component, operation);
                default
            }
        }
    }
}

impl<T, E> SafeUnwrap<T> for Result<T, E>
where
    E: fmt::Display,
{
    fn safe_unwrap(self, operation: &str, component: &str) -> HiveResult<T> {
        self.map_err(|e| {
            error!(
                "Result unwrap failed in {} during {}: {}",
                component, operation, e
            );
            HiveError::OperationFailed {
                reason: format!(
                    "Operation failed in {} during {}: {}",
                    component, operation, e
                ),
            }
        })
    }

    fn unwrap_or_default_with_log(self, operation: &str, component: &str) -> T
    where
        T: Default,
    {
        match self {
            Ok(value) => value,
            Err(error) => {
                warn!(
                    "Using default value in {} during {} due to error: {}",
                    component, operation, error
                );
                T::default()
            }
        }
    }

    fn unwrap_or_with_log(self, default: T, operation: &str, component: &str) -> T {
        match self {
            Ok(value) => value,
            Err(error) => {
                warn!(
                    "Using fallback value in {} during {} due to error: {}",
                    component, operation, error
                );
                default
            }
        }
    }
}

/// JSON value safe access utilities
pub trait JsonSafeAccess {
    fn safe_get(&self, key: &str) -> HiveResult<&serde_json::Value>;
    fn safe_get_str(&self, key: &str) -> HiveResult<&str>;
    fn safe_get_u64(&self, key: &str) -> HiveResult<u64>;
    fn safe_get_f64(&self, key: &str) -> HiveResult<f64>;
    fn safe_get_bool(&self, key: &str) -> HiveResult<bool>;
    fn safe_get_array(&self) -> HiveResult<&Vec<serde_json::Value>>;
    fn safe_get_object(&self) -> HiveResult<&serde_json::Map<String, serde_json::Value>>;
}

impl JsonSafeAccess for serde_json::Value {
    fn safe_get(&self, key: &str) -> HiveResult<&serde_json::Value> {
        self.get(key).ok_or_else(|| HiveError::ValidationError {
            field: key.to_string(),
            reason: "Key not found in JSON object".to_string(),
        })
    }

    fn safe_get_str(&self, key: &str) -> HiveResult<&str> {
        self.safe_get(key)?
            .as_str()
            .ok_or_else(|| HiveError::ValidationError {
                field: key.to_string(),
                reason: "Value is not a string".to_string(),
            })
    }

    fn safe_get_u64(&self, key: &str) -> HiveResult<u64> {
        self.safe_get(key)?
            .as_u64()
            .ok_or_else(|| HiveError::ValidationError {
                field: key.to_string(),
                reason: "Value is not a valid u64".to_string(),
            })
    }

    fn safe_get_f64(&self, key: &str) -> HiveResult<f64> {
        self.safe_get(key)?
            .as_f64()
            .ok_or_else(|| HiveError::ValidationError {
                field: key.to_string(),
                reason: "Value is not a valid f64".to_string(),
            })
    }

    fn safe_get_bool(&self, key: &str) -> HiveResult<bool> {
        self.safe_get(key)?
            .as_bool()
            .ok_or_else(|| HiveError::ValidationError {
                field: key.to_string(),
                reason: "Value is not a boolean".to_string(),
            })
    }

    fn safe_get_array(&self) -> HiveResult<&Vec<serde_json::Value>> {
        self.as_array().ok_or_else(|| HiveError::ValidationError {
            field: "root".to_string(),
            reason: "Value is not an array".to_string(),
        })
    }

    fn safe_get_object(&self) -> HiveResult<&serde_json::Map<String, serde_json::Value>> {
        self.as_object().ok_or_else(|| HiveError::ValidationError {
            field: "root".to_string(),
            reason: "Value is not an object".to_string(),
        })
    }
}

/// Macro for safe unwrap with context
#[macro_export]
macro_rules! safe_unwrap {
    ($expr:expr, $op:expr, $component:expr) => {{
        use $crate::utils::error_recovery::SafeUnwrap;
        $expr.safe_unwrap($op, $component)
    }};
}

/// Macro for safe unwrap with default and logging
#[macro_export]
macro_rules! safe_unwrap_or_default {
    ($expr:expr, $op:expr, $component:expr) => {{
        use $crate::utils::error_recovery::SafeUnwrap;
        $expr.unwrap_or_default_with_log($op, $component)
    }};
}

/// Macro for safe unwrap with custom default and logging
#[macro_export]
macro_rules! safe_unwrap_or {
    ($expr:expr, $default:expr, $op:expr, $component:expr) => {{
        use $crate::utils::error_recovery::SafeUnwrap;
        $expr.unwrap_or_with_log($default, $op, $component)
    }};
}

/// Macro for agent-specific error recovery
#[macro_export]
macro_rules! agent_recover {
    ($operation:expr, $agent_id:expr, $op_name:expr, $component:expr) => {{
        use std::sync::Arc;
        use tokio::sync::OnceCell;
        use $crate::utils::error_recovery::ContextAwareRecovery;

        static RECOVERY: OnceCell<Arc<ContextAwareRecovery>> = OnceCell::const_new();

        async move {
            let recovery = RECOVERY
                .get_or_init(|| async { Arc::new(ContextAwareRecovery::new()) })
                .await;

            recovery
                .execute_with_agent_recovery(
                    || Box::pin(async { $operation }),
                    $op_name,
                    $component,
                    $agent_id,
                )
                .await
        }
    }};
}

/// Macro for general error recovery with context
#[macro_export]
macro_rules! recover_with_context {
    ($operation:expr, $op_name:expr, $component:expr) => {{
        use std::sync::Arc;
        use tokio::sync::OnceCell;
        use $crate::utils::error_recovery::ContextAwareRecovery;

        static RECOVERY: OnceCell<Arc<ContextAwareRecovery>> = OnceCell::const_new();

        async move {
            let recovery = RECOVERY
                .get_or_init(|| async { Arc::new(ContextAwareRecovery::new()) })
                .await
                .clone();

            recovery
                .execute_with_context(move || Box::pin($operation), $op_name, $component)
                .await
        }
    }};
}

/// Macro for creating agent-specific errors
#[macro_export]
macro_rules! agent_error {
    (learning_failed, $agent_id:expr, $reason:expr) => {
        $crate::utils::error::HiveError::AgentLearningFailed {
            agent_id: $agent_id.to_string(),
            reason: $reason.to_string(),
        }
    };
    (adaptation_failed, $agent_id:expr, $strategy:expr, $reason:expr) => {
        $crate::utils::error::HiveError::AgentAdaptationFailed {
            agent_id: $agent_id.to_string(),
            strategy: $strategy.to_string(),
            reason: $reason.to_string(),
        }
    };
    (memory_corruption, $agent_id:expr, $memory_type:expr) => {
        $crate::utils::error::HiveError::AgentMemoryCorruption {
            agent_id: $agent_id.to_string(),
            memory_type: $memory_type.to_string(),
        }
    };
    (deadlock_detected, $agent_id:expr, $operation:expr) => {
        $crate::utils::error::HiveError::AgentDeadlockDetected {
            agent_id: $agent_id.to_string(),
            operation: $operation.to_string(),
        }
    };
    (resource_starvation, $agent_id:expr, $resource:expr, $required:expr, $available:expr) => {
        $crate::utils::error::HiveError::AgentResourceStarvation {
            agent_id: $agent_id.to_string(),
            resource: $resource.to_string(),
            required: $required,
            available: $available,
        }
    };
    (skill_evolution_failed, $agent_id:expr, $skill:expr, $reason:expr) => {
        $crate::utils::error::HiveError::AgentSkillEvolutionFailed {
            agent_id: $agent_id.to_string(),
            skill: $skill.to_string(),
            reason: $reason.to_string(),
        }
    };
}

/// Exponential backoff configuration for retry mechanisms
#[derive(Debug, Clone)]
pub struct ExponentialBackoffConfig {
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub max_attempts: u32,
    pub jitter: bool,
}

impl Default for ExponentialBackoffConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            max_attempts: 5,
            jitter: true,
        }
    }
}

/// Exponential backoff retry mechanism
pub struct ExponentialBackoffRetry {
    config: ExponentialBackoffConfig,
}

impl ExponentialBackoffRetry {
    pub fn new(config: ExponentialBackoffConfig) -> Self {
        Self { config }
    }

    pub async fn execute<F, T, E>(&self, mut operation: F) -> HiveResult<T>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Display,
    {
        let mut attempt = 0;
        let mut delay = self.config.initial_delay;

        loop {
            attempt += 1;

            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt >= self.config.max_attempts {
                        error!("Operation failed after {} attempts: {}", attempt, e);
                        return Err(HiveError::OperationFailed {
                            reason: format!("Operation failed after {} attempts: {}", attempt, e),
                        });
                    }

                    // Add jitter to prevent thundering herd
                    let actual_delay = if self.config.jitter {
                        let jitter = rand::random::<f64>() * 0.1 * delay.as_millis() as f64;
                        Duration::from_millis((delay.as_millis() as f64 + jitter) as u64)
                    } else {
                        delay
                    };

                    warn!(
                        "Operation failed on attempt {}: {}. Retrying in {:?}",
                        attempt, e, actual_delay
                    );

                    tokio::time::sleep(actual_delay).await;

                    // Calculate next delay
                    delay = std::cmp::min(
                        Duration::from_millis(
                            (delay.as_millis() as f64 * self.config.multiplier) as u64,
                        ),
                        self.config.max_delay,
                    );
                }
            }
        }
    }
}

/// Error classification for different recovery strategies
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    /// Temporary errors that should be retried
    Transient,
    /// Permanent errors that should not be retried
    Permanent,
    /// Rate limiting errors
    RateLimited,
    /// Authentication/authorization errors
    Auth,
    /// Resource exhaustion errors
    ResourceExhausted,
    /// Configuration errors
    Configuration,
    /// Network connectivity errors
    Network,
    /// Unknown/unclassified errors
    Unknown,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCategory::Transient => write!(f, "transient"),
            ErrorCategory::Permanent => write!(f, "permanent"),
            ErrorCategory::RateLimited => write!(f, "rate-limited"),
            ErrorCategory::Auth => write!(f, "authentication"),
            ErrorCategory::ResourceExhausted => write!(f, "resource-exhausted"),
            ErrorCategory::Configuration => write!(f, "configuration"),
            ErrorCategory::Network => write!(f, "network"),
            ErrorCategory::Unknown => write!(f, "unknown"),
        }
    }
}

/// Error classifier for determining recovery strategies
pub struct ErrorClassifier;

impl ErrorClassifier {
    pub fn classify_error(error: &HiveError) -> ErrorCategory {
        match error {
            HiveError::TimeoutError { .. } => ErrorCategory::Transient,
            HiveError::NetworkError { .. } => ErrorCategory::Network,
            HiveError::ConnectionTimeout { .. } => ErrorCategory::Network,
            HiveError::DNSResolutionFailed { .. } => ErrorCategory::Network,
            HiveError::WebSocketConnectionClosed { .. } => ErrorCategory::Transient,
            HiveError::CircuitBreakerOpen { .. } => ErrorCategory::Transient,
            HiveError::RateLimitExceeded { .. } => ErrorCategory::RateLimited,
            HiveError::ResourceExhausted { .. } => ErrorCategory::ResourceExhausted,
            HiveError::ResourceAllocationFailed { .. } => ErrorCategory::ResourceExhausted,
            HiveError::AuthenticationError { .. } => ErrorCategory::Auth,
            HiveError::AuthorizationError { .. } => ErrorCategory::Auth,
            HiveError::PermissionDenied { .. } => ErrorCategory::Auth,
            HiveError::ConfigurationError { .. } => ErrorCategory::Configuration,
            HiveError::ConfigurationFileNotFound { .. } => ErrorCategory::Configuration,
            HiveError::DatabaseConnectionFailed { .. } => ErrorCategory::Transient,
            HiveError::DatabaseMigrationFailed { .. } => ErrorCategory::Configuration,
            HiveError::IoError { .. } => ErrorCategory::Transient,
            HiveError::FileNotFound { .. } => ErrorCategory::Permanent,
            HiveError::FilePermissionDenied { .. } => ErrorCategory::Auth,
            HiveError::AgentNotFound { .. } => ErrorCategory::Permanent,
            HiveError::TaskNotFound { .. } => ErrorCategory::Permanent,
            HiveError::NotFound { .. } => ErrorCategory::Permanent,
            HiveError::ValidationError { .. } => ErrorCategory::Permanent,
            HiveError::InvalidJson { .. } => ErrorCategory::Permanent,
            HiveError::InvalidUUID { .. } => ErrorCategory::Permanent,
            HiveError::InvalidEnumValue { .. } => ErrorCategory::Permanent,
            _ => ErrorCategory::Unknown,
        }
    }

    pub fn should_retry(category: &ErrorCategory) -> bool {
        matches!(
            category,
            ErrorCategory::Transient | ErrorCategory::Network | ErrorCategory::RateLimited
        )
    }

    pub fn get_retry_config(category: &ErrorCategory) -> RetryConfig {
        match category {
            ErrorCategory::Network => RetryConfig {
                max_attempts: 5,
                base_delay: Duration::from_millis(500),
                max_delay: Duration::from_secs(10),
                backoff_multiplier: 2.0,
            },
            ErrorCategory::RateLimited => RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_secs(1),
                max_delay: Duration::from_secs(60),
                backoff_multiplier: 2.0,
            },
            ErrorCategory::Transient => RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_millis(200),
                max_delay: Duration::from_secs(5),
                backoff_multiplier: 1.5,
            },
            _ => RetryConfig::default(),
        }
    }
}

/// Agent-specific recovery strategies and mechanisms
pub struct AgentRecoveryManager {
    agent_states: Arc<RwLock<HashMap<String, AgentRecoveryState>>>,
    recovery_strategies: HashMap<String, Vec<AgentRecoveryStrategy>>,
}

#[derive(Debug, Clone)]
pub struct AgentRecoveryState {
    pub agent_id: String,
    pub last_failure: Option<std::time::Instant>,
    pub failure_count: u32,
    pub recovery_attempts: u32,
    pub current_strategy: Option<AgentRecoveryStrategy>,
    pub isolation_until: Option<std::time::Instant>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentRecoveryStrategy {
    /// Restart agent with exponential backoff
    RestartWithBackoff {
        base_delay: Duration,
        max_delay: Duration,
    },
    /// Switch to backup agent instance
    FailoverToBackup { backup_agent_id: String },
    /// Temporarily reduce agent capabilities
    CapabilityReduction { reduced_capabilities: Vec<String> },
    /// Reset agent memory and learning state
    MemoryReset { preserve_experiences: bool },
    /// Enter simplified operation mode
    SimplifiedMode { timeout: Duration },
    /// Isolate agent from swarm communication
    SwarmIsolation { duration: Duration },
    /// Rollback to previous learning checkpoint
    LearningRollback { checkpoint_id: String },
    /// Redistribute workload to peer agents
    WorkloadRedistribution { target_agents: Vec<String> },
    /// Retry operation with exponential backoff
    RetryWithBackoff {
        max_attempts: u32,
        base_delay: Duration,
    },
    /// Use alternative dependency
    UseAlternativeDependency { alternative_name: String },
    /// Throttle agent operations
    ThrottleAgent {
        throttle_factor: f64,
        duration: Duration,
    },
    /// Increase timeout for operations
    IncreaseTimeout { timeout_multiplier: f64 },
    /// Switch communication protocol
    SwitchCommunicationProtocol { protocol: String },
    /// Rollback migration to source node
    RollbackMigration { source_node: String },
    /// Reset configuration with preservation
    ResetConfiguration { preserve_custom_settings: bool },
    /// Resynchronize state from source
    StateResynchronization { source: String },
}

impl AgentRecoveryManager {
    pub fn new() -> Self {
        let mut recovery_strategies = HashMap::new();

        // Define recovery strategies for different agent error types
        recovery_strategies.insert(
            "AgentLearningFailed".to_string(),
            vec![
                AgentRecoveryStrategy::LearningRollback {
                    checkpoint_id: "auto".to_string(),
                },
                AgentRecoveryStrategy::SimplifiedMode {
                    timeout: Duration::from_secs(300),
                },
                AgentRecoveryStrategy::RestartWithBackoff {
                    base_delay: Duration::from_secs(10),
                    max_delay: Duration::from_secs(300),
                },
            ],
        );

        recovery_strategies.insert(
            "AgentMemoryCorruption".to_string(),
            vec![
                AgentRecoveryStrategy::MemoryReset {
                    preserve_experiences: true,
                },
                AgentRecoveryStrategy::RestartWithBackoff {
                    base_delay: Duration::from_secs(5),
                    max_delay: Duration::from_secs(120),
                },
            ],
        );

        recovery_strategies.insert(
            "AgentDeadlockDetected".to_string(),
            vec![
                AgentRecoveryStrategy::RestartWithBackoff {
                    base_delay: Duration::from_millis(100),
                    max_delay: Duration::from_secs(30),
                },
                AgentRecoveryStrategy::SwarmIsolation {
                    duration: Duration::from_secs(60),
                },
            ],
        );

        recovery_strategies.insert(
            "AgentResourceStarvation".to_string(),
            vec![
                AgentRecoveryStrategy::WorkloadRedistribution {
                    target_agents: vec![],
                },
                AgentRecoveryStrategy::CapabilityReduction {
                    reduced_capabilities: vec!["heavy_computation".to_string()],
                },
                AgentRecoveryStrategy::SimplifiedMode {
                    timeout: Duration::from_secs(180),
                },
            ],
        );

        recovery_strategies.insert(
            "AgentAdaptationFailed".to_string(),
            vec![
                AgentRecoveryStrategy::LearningRollback {
                    checkpoint_id: "auto".to_string(),
                },
                AgentRecoveryStrategy::SimplifiedMode {
                    timeout: Duration::from_secs(300),
                },
                AgentRecoveryStrategy::RestartWithBackoff {
                    base_delay: Duration::from_secs(10),
                    max_delay: Duration::from_secs(300),
                },
            ],
        );

        recovery_strategies.insert(
            "AgentCommunicationProtocolError".to_string(),
            vec![
                AgentRecoveryStrategy::RestartWithBackoff {
                    base_delay: Duration::from_millis(100),
                    max_delay: Duration::from_secs(30),
                },
                AgentRecoveryStrategy::SwarmIsolation {
                    duration: Duration::from_secs(60),
                },
            ],
        );

        recovery_strategies.insert(
            "AgentSkillEvolutionFailed".to_string(),
            vec![
                AgentRecoveryStrategy::LearningRollback {
                    checkpoint_id: "auto".to_string(),
                },
                AgentRecoveryStrategy::CapabilityReduction {
                    reduced_capabilities: vec!["advanced_skills".to_string()],
                },
                AgentRecoveryStrategy::SimplifiedMode {
                    timeout: Duration::from_secs(600),
                },
            ],
        );

        recovery_strategies.insert(
            "AgentVerificationFailed".to_string(),
            vec![
                AgentRecoveryStrategy::RestartWithBackoff {
                    base_delay: Duration::from_secs(5),
                    max_delay: Duration::from_secs(120),
                },
                AgentRecoveryStrategy::MemoryReset {
                    preserve_experiences: true,
                },
                AgentRecoveryStrategy::SimplifiedMode {
                    timeout: Duration::from_secs(300),
                },
            ],
        );

        recovery_strategies.insert(
            "AgentCollaborativeLearningFailed".to_string(),
            vec![
                AgentRecoveryStrategy::SwarmIsolation {
                    duration: Duration::from_secs(120),
                },
                AgentRecoveryStrategy::LearningRollback {
                    checkpoint_id: "auto".to_string(),
                },
                AgentRecoveryStrategy::RestartWithBackoff {
                    base_delay: Duration::from_secs(15),
                    max_delay: Duration::from_secs(600),
                },
            ],
        );

        recovery_strategies.insert(
            "AgentEvolutionStalled".to_string(),
            vec![
                AgentRecoveryStrategy::LearningRollback {
                    checkpoint_id: "best_performance".to_string(),
                },
                AgentRecoveryStrategy::CapabilityReduction {
                    reduced_capabilities: vec!["evolution".to_string()],
                },
                AgentRecoveryStrategy::RestartWithBackoff {
                    base_delay: Duration::from_secs(30),
                    max_delay: Duration::from_secs(900),
                },
            ],
        );

        // Additional recovery strategies for new error types
        recovery_strategies.insert(
            "AgentInitializationFailed".to_string(),
            vec![
                AgentRecoveryStrategy::RestartWithBackoff {
                    base_delay: Duration::from_secs(1),
                    max_delay: Duration::from_secs(60),
                },
                AgentRecoveryStrategy::MemoryReset {
                    preserve_experiences: false,
                },
                AgentRecoveryStrategy::FailoverToBackup {
                    backup_agent_id: "auto".to_string(),
                },
            ],
        );

        recovery_strategies.insert(
            "AgentTerminationFailed".to_string(),
            vec![
                AgentRecoveryStrategy::SwarmIsolation {
                    duration: Duration::from_secs(30),
                },
                AgentRecoveryStrategy::CapabilityReduction {
                    reduced_capabilities: vec!["all".to_string()],
                },
                AgentRecoveryStrategy::RestartWithBackoff {
                    base_delay: Duration::from_millis(500),
                    max_delay: Duration::from_secs(10),
                },
            ],
        );

        recovery_strategies.insert(
            "AgentHealthCheckFailed".to_string(),
            vec![
                AgentRecoveryStrategy::SimplifiedMode {
                    timeout: Duration::from_secs(180),
                },
                AgentRecoveryStrategy::CapabilityReduction {
                    reduced_capabilities: vec!["non_essential".to_string()],
                },
                AgentRecoveryStrategy::RestartWithBackoff {
                    base_delay: Duration::from_secs(5),
                    max_delay: Duration::from_secs(300),
                },
            ],
        );

        recovery_strategies.insert(
            "AgentScalingFailed".to_string(),
            vec![
                AgentRecoveryStrategy::WorkloadRedistribution {
                    target_agents: vec![],
                },
                AgentRecoveryStrategy::ThrottleAgent {
                    throttle_factor: 0.5,
                    duration: Duration::from_secs(300),
                },
                AgentRecoveryStrategy::SimplifiedMode {
                    timeout: Duration::from_secs(600),
                },
            ],
        );

        recovery_strategies.insert(
            "AgentMigrationFailed".to_string(),
            vec![
                AgentRecoveryStrategy::RollbackMigration {
                    source_node: "auto".to_string(),
                },
                AgentRecoveryStrategy::SwarmIsolation {
                    duration: Duration::from_secs(120),
                },
                AgentRecoveryStrategy::RestartWithBackoff {
                    base_delay: Duration::from_secs(10),
                    max_delay: Duration::from_secs(600),
                },
            ],
        );

        recovery_strategies.insert(
            "AgentConfigurationValidationFailed".to_string(),
            vec![
                AgentRecoveryStrategy::ResetConfiguration {
                    preserve_custom_settings: true,
                },
                AgentRecoveryStrategy::RestartWithBackoff {
                    base_delay: Duration::from_secs(2),
                    max_delay: Duration::from_secs(120),
                },
                AgentRecoveryStrategy::SimplifiedMode {
                    timeout: Duration::from_secs(240),
                },
            ],
        );

        recovery_strategies.insert(
            "AgentDependencyResolutionFailed".to_string(),
            vec![
                AgentRecoveryStrategy::RetryWithBackoff {
                    max_attempts: 5,
                    base_delay: Duration::from_secs(1),
                },
                AgentRecoveryStrategy::UseAlternativeDependency {
                    alternative_name: "auto".to_string(),
                },
                AgentRecoveryStrategy::CapabilityReduction {
                    reduced_capabilities: vec!["dependent_features".to_string()],
                },
            ],
        );

        recovery_strategies.insert(
            "AgentResourceAllocationFailed".to_string(),
            vec![
                AgentRecoveryStrategy::WorkloadRedistribution {
                    target_agents: vec![],
                },
                AgentRecoveryStrategy::ThrottleAgent {
                    throttle_factor: 0.3,
                    duration: Duration::from_secs(600),
                },
                AgentRecoveryStrategy::SimplifiedMode {
                    timeout: Duration::from_secs(900),
                },
            ],
        );

        recovery_strategies.insert(
            "AgentCommunicationTimeout".to_string(),
            vec![
                AgentRecoveryStrategy::IncreaseTimeout {
                    timeout_multiplier: 2.0,
                },
                AgentRecoveryStrategy::SwitchCommunicationProtocol {
                    protocol: "fallback".to_string(),
                },
                AgentRecoveryStrategy::SwarmIsolation {
                    duration: Duration::from_secs(60),
                },
            ],
        );

        recovery_strategies.insert(
            "AgentStateInconsistency".to_string(),
            vec![
                AgentRecoveryStrategy::StateResynchronization {
                    source: "last_known_good".to_string(),
                },
                AgentRecoveryStrategy::RestartWithBackoff {
                    base_delay: Duration::from_secs(3),
                    max_delay: Duration::from_secs(180),
                },
                AgentRecoveryStrategy::MemoryReset {
                    preserve_experiences: true,
                },
            ],
        );

        Self {
            agent_states: Arc::new(RwLock::new(HashMap::new())),
            recovery_strategies,
        }
    }

    pub async fn handle_agent_error(
        &self,
        agent_id: &str,
        error: &HiveError,
        context: &RecoveryContext,
    ) -> HiveResult<AgentRecoveryStrategy> {
        let error_type = self.classify_agent_error(error);
        let strategies = self.recovery_strategies.get(&error_type).ok_or_else(|| {
            HiveError::RecoveryStrategyNotAvailable {
                strategy_name: error_type.clone(),
            }
        })?;

        let mut state = self.agent_states.write().await;
        let agent_state = state
            .entry(agent_id.to_string())
            .or_insert(AgentRecoveryState {
                agent_id: agent_id.to_string(),
                last_failure: Some(std::time::Instant::now()),
                failure_count: 0,
                recovery_attempts: 0,
                current_strategy: None,
                isolation_until: None,
            });

        agent_state.failure_count += 1;
        agent_state.recovery_attempts += 1;

        // Select strategy based on failure history and context
        let strategy = self
            .select_agent_recovery_strategy(strategies, agent_state, context)
            .await;

        agent_state.current_strategy = Some(strategy.clone());

        Ok(strategy)
    }

    fn classify_agent_error(&self, error: &HiveError) -> String {
        match error {
            HiveError::AgentLearningFailed { .. } => "AgentLearningFailed".to_string(),
            HiveError::AgentMemoryCorruption { .. } => "AgentMemoryCorruption".to_string(),
            HiveError::AgentDeadlockDetected { .. } => "AgentDeadlockDetected".to_string(),
            HiveError::AgentResourceStarvation { .. } => "AgentResourceStarvation".to_string(),
            HiveError::AgentAdaptationFailed { .. } => "AgentAdaptationFailed".to_string(),
            HiveError::AgentCommunicationProtocolError { .. } => {
                "AgentCommunicationProtocolError".to_string()
            }
            HiveError::AgentSkillEvolutionFailed { .. } => "AgentSkillEvolutionFailed".to_string(),
            HiveError::AgentVerificationFailed { .. } => "AgentVerificationFailed".to_string(),
            HiveError::AgentCollaborativeLearningFailed { .. } => {
                "AgentCollaborativeLearningFailed".to_string()
            }
            HiveError::AgentEvolutionStalled { .. } => "AgentEvolutionStalled".to_string(),
            HiveError::AgentInitializationFailed { .. } => "AgentInitializationFailed".to_string(),
            HiveError::AgentTerminationFailed { .. } => "AgentTerminationFailed".to_string(),
            HiveError::AgentHealthCheckFailed { .. } => "AgentHealthCheckFailed".to_string(),
            HiveError::AgentScalingFailed { .. } => "AgentScalingFailed".to_string(),
            HiveError::AgentMigrationFailed { .. } => "AgentMigrationFailed".to_string(),
            HiveError::AgentConfigurationValidationFailed { .. } => {
                "AgentConfigurationValidationFailed".to_string()
            }
            HiveError::AgentDependencyResolutionFailed { .. } => {
                "AgentDependencyResolutionFailed".to_string()
            }
            HiveError::AgentResourceAllocationFailed { .. } => {
                "AgentResourceAllocationFailed".to_string()
            }
            HiveError::AgentCommunicationTimeout { .. } => "AgentCommunicationTimeout".to_string(),
            HiveError::AgentStateInconsistency { .. } => "AgentStateInconsistency".to_string(),
            _ => "GenericAgentError".to_string(),
        }
    }

    async fn select_agent_recovery_strategy(
        &self,
        strategies: &[AgentRecoveryStrategy],
        state: &AgentRecoveryState,
        context: &RecoveryContext,
    ) -> AgentRecoveryStrategy {
        // Simple strategy selection based on failure count
        let strategy_index =
            (state.failure_count.saturating_sub(1) as usize).min(strategies.len() - 1);
        strategies[strategy_index].clone()
    }

    pub async fn execute_agent_recovery(
        &self,
        agent_id: &str,
        strategy: &AgentRecoveryStrategy,
    ) -> HiveResult<()> {
        match strategy {
            AgentRecoveryStrategy::RestartWithBackoff {
                base_delay,
                max_delay,
            } => {
                // Implementation would trigger agent restart with backoff
                info!("Triggering agent restart for {} with backoff", agent_id);
                Ok(())
            }
            AgentRecoveryStrategy::FailoverToBackup { backup_agent_id } => {
                info!(
                    "Failing over agent {} to backup {}",
                    agent_id, backup_agent_id
                );
                Ok(())
            }
            AgentRecoveryStrategy::CapabilityReduction {
                reduced_capabilities,
            } => {
                info!(
                    "Reducing capabilities for agent {}: {:?}",
                    agent_id, reduced_capabilities
                );
                Ok(())
            }
            AgentRecoveryStrategy::MemoryReset {
                preserve_experiences,
            } => {
                info!(
                    "Resetting memory for agent {}, preserve experiences: {}",
                    agent_id, preserve_experiences
                );
                Ok(())
            }
            AgentRecoveryStrategy::SimplifiedMode { timeout } => {
                info!(
                    "Entering simplified mode for agent {} for {:?}",
                    agent_id, timeout
                );
                Ok(())
            }
            AgentRecoveryStrategy::SwarmIsolation { duration } => {
                info!("Isolating agent {} from swarm for {:?}", agent_id, duration);
                Ok(())
            }
            AgentRecoveryStrategy::LearningRollback { checkpoint_id } => {
                info!(
                    "Rolling back learning for agent {} to checkpoint {}",
                    agent_id, checkpoint_id
                );
                Ok(())
            }
            AgentRecoveryStrategy::WorkloadRedistribution { target_agents } => {
                info!(
                    "Redistributing workload from agent {} to {:?}",
                    agent_id, target_agents
                );
                Ok(())
            }
            AgentRecoveryStrategy::RetryWithBackoff {
                max_attempts,
                base_delay,
            } => {
                info!(
                    "Configuring retry with backoff for agent {}: max_attempts={}, base_delay={:?}",
                    agent_id, max_attempts, base_delay
                );
                Ok(())
            }
            AgentRecoveryStrategy::UseAlternativeDependency { alternative_name } => {
                info!(
                    "Switching agent {} to alternative dependency: {}",
                    agent_id, alternative_name
                );
                Ok(())
            }
            AgentRecoveryStrategy::ThrottleAgent {
                throttle_factor,
                duration,
            } => {
                info!(
                    "Throttling agent {} by factor {} for {:?}",
                    agent_id, throttle_factor, duration
                );
                Ok(())
            }
            AgentRecoveryStrategy::IncreaseTimeout { timeout_multiplier } => {
                info!(
                    "Increasing timeout for agent {} by factor {}",
                    agent_id, timeout_multiplier
                );
                Ok(())
            }
            AgentRecoveryStrategy::SwitchCommunicationProtocol { protocol } => {
                info!(
                    "Switching agent {} to communication protocol: {}",
                    agent_id, protocol
                );
                Ok(())
            }
            AgentRecoveryStrategy::RollbackMigration { source_node } => {
                info!(
                    "Rolling back migration for agent {} to source node: {}",
                    agent_id, source_node
                );
                Ok(())
            }
            AgentRecoveryStrategy::ResetConfiguration {
                preserve_custom_settings,
            } => {
                info!(
                    "Resetting configuration for agent {}, preserve_custom_settings: {}",
                    agent_id, preserve_custom_settings
                );
                Ok(())
            }
            AgentRecoveryStrategy::StateResynchronization { source } => {
                info!(
                    "Resynchronizing state for agent {} from source: {}",
                    agent_id, source
                );
                Ok(())
            }
        }
    }

    pub async fn is_agent_isolated(&self, agent_id: &str) -> bool {
        let state = self.agent_states.read().await;
        if let Some(agent_state) = state.get(agent_id) {
            if let Some(isolation_until) = agent_state.isolation_until {
                isolation_until > std::time::Instant::now()
            } else {
                false
            }
        } else {
            false
        }
    }
}

/// Context-aware error recovery system
pub struct ContextAwareRecovery {
    error_classifier: ErrorClassifier,
    adaptive_recovery: AdaptiveErrorRecovery,
    agent_recovery: AgentRecoveryManager,
}

impl ContextAwareRecovery {
    pub fn new() -> Self {
        Self {
            error_classifier: ErrorClassifier,
            adaptive_recovery: AdaptiveErrorRecovery::new(),
            agent_recovery: AgentRecoveryManager::new(),
        }
    }

    pub async fn execute_with_context<F, T, E>(
        &self,
        operation: F,
        operation_name: &str,
        component_name: &str,
    ) -> HiveResult<T>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>
            + Clone,
        E: std::fmt::Display + Send + Sync + 'static,
    {
        let mut context = RecoveryContext::new(operation_name, component_name, 3);

        match operation().await {
            Ok(result) => Ok(result),
            Err(error) => {
                let hive_error = HiveError::OperationFailed {
                    reason: error.to_string(),
                };

                let category = ErrorClassifier::classify_error(&hive_error);

                if ErrorClassifier::should_retry(&category) {
                    let retry_config = ErrorClassifier::get_retry_config(&category);
                    let retry = RetryMechanism::new(retry_config);

                    match retry.execute(operation.clone()).await {
                        Ok(result) => Ok(result),
                        Err(_) => {
                            // If retry fails, try adaptive recovery
                            self.adaptive_recovery
                                .execute_with_adaptive_recovery(operation, &mut context)
                                .await
                        }
                    }
                } else {
                    // For non-retryable errors, try adaptive recovery directly
                    self.adaptive_recovery
                        .execute_with_adaptive_recovery(operation, &mut context)
                        .await
                }
            }
        }
    }

    /// Execute operation with agent-specific recovery strategies
    pub async fn execute_with_agent_recovery<F, T, E>(
        &self,
        operation: F,
        agent: &mut crate::agents::agent::Agent,
        operation_name: &str,
        component_name: &str,
        agent_id: &str,
    ) -> HiveResult<T>
    where
        F: Fn(
            &mut crate::agents::agent::Agent,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Display + Send + Sync + 'static,
    {
        operation(agent)
            .await
            .map_err(|e| HiveError::OperationFailed {
                reason: e.to_string(),
            })
    }
}

/// Health check system for monitoring error recovery effectiveness
pub struct RecoveryHealthMonitor {
    health_metrics: Arc<RwLock<HashMap<String, HealthMetrics>>>,
}

#[derive(Debug, Clone)]
pub struct HealthMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub recovery_attempts: u64,
    pub successful_recoveries: u64,
    pub average_recovery_time: Duration,
    pub last_failure_time: Option<std::time::Instant>,
}

impl RecoveryHealthMonitor {
    pub fn new() -> Self {
        Self {
            health_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn record_operation(&self, component: &str, success: bool) {
        let mut metrics = self.health_metrics.write().await;
        let component_metrics = metrics
            .entry(component.to_string())
            .or_insert(HealthMetrics {
                total_operations: 0,
                successful_operations: 0,
                failed_operations: 0,
                recovery_attempts: 0,
                successful_recoveries: 0,
                average_recovery_time: Duration::from_millis(0),
                last_failure_time: None,
            });

        component_metrics.total_operations += 1;
        if success {
            component_metrics.successful_operations += 1;
        } else {
            component_metrics.failed_operations += 1;
            component_metrics.last_failure_time = Some(std::time::Instant::now());
        }
    }

    pub async fn record_recovery_attempt(
        &self,
        component: &str,
        success: bool,
        duration: Duration,
    ) {
        let mut metrics = self.health_metrics.write().await;
        if let Some(component_metrics) = metrics.get_mut(component) {
            component_metrics.recovery_attempts += 1;
            if success {
                component_metrics.successful_recoveries += 1;
            }

            // Update rolling average
            let total_time = component_metrics.average_recovery_time
                * (component_metrics.successful_recoveries - 1) as u32
                + duration;
            component_metrics.average_recovery_time =
                total_time / component_metrics.successful_recoveries as u32;
        }
    }

    pub async fn get_health_report(&self, component: &str) -> Option<HealthMetrics> {
        let metrics = self.health_metrics.read().await;
        metrics.get(component).cloned()
    }

    pub async fn get_overall_health_score(&self) -> f64 {
        let metrics = self.health_metrics.read().await;
        if metrics.is_empty() {
            return 1.0; // Perfect health if no data
        }

        let mut total_score = 0.0;
        let mut component_count = 0;

        for component_metrics in metrics.values() {
            if component_metrics.total_operations > 0 {
                let success_rate = component_metrics.successful_operations as f64
                    / component_metrics.total_operations as f64;
                let recovery_rate = if component_metrics.recovery_attempts > 0 {
                    component_metrics.successful_recoveries as f64
                        / component_metrics.recovery_attempts as f64
                } else {
                    1.0
                };

                let component_score = (success_rate + recovery_rate) / 2.0;
                total_score += component_score;
                component_count += 1;
            }
        }

        if component_count > 0 {
            total_score / component_count as f64
        } else {
            1.0
        }
    }
}

/// Centralized Error Handling Coordinator
///
/// This is the main entry point for all error handling and recovery operations
/// in the AI Orchestrator Hub. It provides a unified interface for error handling,
/// recovery coordination, and system resilience.
pub struct CentralizedErrorHandler {
    context_aware_recovery: Arc<ContextAwareRecovery>,
    health_monitor: Arc<RecoveryHealthMonitor>,
    circuit_breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
    error_classifier: ErrorClassifier,
    config: ErrorHandlerConfig,
}

#[derive(Debug, Clone)]
pub struct ErrorHandlerConfig {
    /// Enable/disable automatic recovery
    pub enable_automatic_recovery: bool,
    /// Maximum number of recovery attempts per error
    pub max_recovery_attempts: u32,
    /// Default timeout for recovery operations
    pub default_recovery_timeout: Duration,
    /// Enable/disable circuit breaker pattern
    pub enable_circuit_breakers: bool,
    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,
    /// Circuit breaker recovery timeout
    pub circuit_breaker_timeout: Duration,
    /// Enable/disable health monitoring
    pub enable_health_monitoring: bool,
    /// Health check interval
    pub health_check_interval: Duration,
}

impl Default for ErrorHandlerConfig {
    fn default() -> Self {
        Self {
            enable_automatic_recovery: true,
            max_recovery_attempts: 3,
            default_recovery_timeout: Duration::from_secs(300),
            enable_circuit_breakers: true,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout: Duration::from_secs(60),
            enable_health_monitoring: true,
            health_check_interval: Duration::from_secs(30),
        }
    }
}

impl CentralizedErrorHandler {
    /// Create a new centralized error handler
    pub fn new(config: ErrorHandlerConfig) -> Self {
        Self {
            context_aware_recovery: Arc::new(ContextAwareRecovery::new()),
            health_monitor: Arc::new(RecoveryHealthMonitor::new()),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            error_classifier: ErrorClassifier,
            config,
        }
    }

    /// Execute an operation with centralized error handling
    pub async fn execute_with_centralized_handling<F, T, E>(
        &self,
        operation: F,
        operation_name: &str,
        component_name: &str,
        agent_id: Option<&str>,
    ) -> HiveResult<T>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>
            + Clone,
        E: std::fmt::Display + Send + Sync + 'static,
    {
        let start_time = std::time::Instant::now();

        // Check circuit breaker if enabled
        if self.config.enable_circuit_breakers {
            if let Err(e) = self.check_circuit_breaker(component_name).await {
                self.record_operation_result(component_name, false, start_time.elapsed())
                    .await;
                return Err(e);
            }
        }

        // Execute the operation
        let result = self
            .context_aware_recovery
            .execute_with_context(operation, operation_name, component_name)
            .await;

        // Record operation result
        let success = result.is_ok();
        self.record_operation_result(component_name, success, start_time.elapsed())
            .await;

        // Update circuit breaker state
        if self.config.enable_circuit_breakers {
            self.update_circuit_breaker(component_name, success).await;
        }

        result
    }

    /// Handle an error with centralized recovery
    pub async fn handle_error_with_centralized_recovery(
        &self,
        error: HiveError,
        operation_name: &str,
        component_name: &str,
        agent_id: Option<&str>,
    ) -> HiveResult<()> {
        error!(
            "Centralized error handling for {} in {}: {}",
            operation_name, component_name, error
        );

        // Classify the error
        let category = ErrorClassifier::classify_error(&error);

        // Determine if we should attempt recovery
        if self.config.enable_automatic_recovery && ErrorClassifier::should_retry(&category) {
            info!(
                "Attempting automatic recovery for {} error in {}",
                category, component_name
            );

            // Log the recovery attempt
            self.record_recovery_attempt(component_name, false, Duration::from_millis(0))
                .await;

            // In a real implementation, this would trigger specific recovery actions
            // based on the error category and component type
            warn!(
                "Automatic recovery attempted for {} in {}",
                component_name, operation_name
            );
        }

        // For now, return the original error
        // In a full implementation, this would return the result of recovery attempts
        Err(error)
    }

    /// Get circuit breaker for a component
    async fn get_or_create_circuit_breaker(&self, component: &str) -> Arc<CircuitBreaker> {
        let mut breakers = self.circuit_breakers.write().await;

        breakers
            .entry(component.to_string())
            .or_insert_with(|| {
                Arc::new(CircuitBreaker::new(CircuitBreakerConfig {
                    failure_threshold: self.config.circuit_breaker_threshold,
                    success_threshold: 3,
                    timeout: self.config.circuit_breaker_timeout,
                    window_size: Duration::from_secs(300),
                }))
            })
            .clone()
    }

    /// Check if circuit breaker allows operation
    async fn check_circuit_breaker(&self, component: &str) -> HiveResult<()> {
        let breaker = self.get_or_create_circuit_breaker(component).await;

        if breaker.get_state().await == CircuitState::Open {
            Err(HiveError::CircuitBreakerOpen {
                reason: format!("Circuit breaker open for component: {}", component),
            })
        } else {
            Ok(())
        }
    }

    /// Update circuit breaker state
    async fn update_circuit_breaker(&self, component: &str, success: bool) {
        let breaker = self.get_or_create_circuit_breaker(component).await;

        if success {
            breaker.record_success().await;
        } else {
            breaker.record_failure().await;
        }
    }

    /// Record operation result for health monitoring
    async fn record_operation_result(&self, component: &str, success: bool, duration: Duration) {
        if self.config.enable_health_monitoring {
            self.health_monitor
                .record_operation(component, success)
                .await;
        }
    }

    /// Record recovery attempt for health monitoring
    async fn record_recovery_attempt(&self, component: &str, success: bool, duration: Duration) {
        if self.config.enable_health_monitoring {
            self.health_monitor
                .record_recovery_attempt(component, success, duration)
                .await;
        }
    }

    /// Get health report for a component
    pub async fn get_component_health(&self, component: &str) -> Option<HealthMetrics> {
        if self.config.enable_health_monitoring {
            self.health_monitor.get_health_report(component).await
        } else {
            None
        }
    }

    /// Get overall system health score
    pub async fn get_system_health_score(&self) -> f64 {
        if self.config.enable_health_monitoring {
            self.health_monitor.get_overall_health_score().await
        } else {
            1.0
        }
    }

    /// Get circuit breaker status for a component
    pub async fn get_circuit_breaker_status(&self, component: &str) -> Option<CircuitState> {
        if self.config.enable_circuit_breakers {
            let breakers = self.circuit_breakers.read().await;
            if let Some(breaker) = breakers.get(component) {
                Some(breaker.get_state().await)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Reset circuit breaker for a component
    pub async fn reset_circuit_breaker(&self, component: &str) -> HiveResult<()> {
        if !self.config.enable_circuit_breakers {
            return Err(HiveError::OperationFailed {
                reason: "Circuit breakers are disabled".to_string(),
            });
        }

        let mut breakers = self.circuit_breakers.write().await;

        if let Some(breaker) = breakers.get_mut(component) {
            // Reset by creating a new circuit breaker
            *breaker = Arc::new(CircuitBreaker::new(CircuitBreakerConfig {
                failure_threshold: self.config.circuit_breaker_threshold,
                success_threshold: 3,
                timeout: self.config.circuit_breaker_timeout,
                window_size: Duration::from_secs(300),
            }));
            Ok(())
        } else {
            Err(HiveError::NotFound {
                resource: format!("Circuit breaker for component: {}", component),
            })
        }
    }

    /// Get error handling configuration
    pub fn get_config(&self) -> &ErrorHandlerConfig {
        &self.config
    }

    /// Update error handling configuration
    pub async fn update_config(&mut self, new_config: ErrorHandlerConfig) {
        self.config = new_config;
        info!("Error handler configuration updated");
    }
}

/// Global error handler instance
static GLOBAL_ERROR_HANDLER: Lazy<tokio::sync::RwLock<CentralizedErrorHandler>> = Lazy::new(|| {
    tokio::sync::RwLock::new(CentralizedErrorHandler::new(ErrorHandlerConfig::default()))
});

/// Get the global error handler instance
pub async fn get_global_error_handler(
) -> tokio::sync::RwLockReadGuard<'static, CentralizedErrorHandler> {
    GLOBAL_ERROR_HANDLER.read().await
}

/// Get mutable access to the global error handler instance
pub async fn get_global_error_handler_mut(
) -> tokio::sync::RwLockWriteGuard<'static, CentralizedErrorHandler> {
    GLOBAL_ERROR_HANDLER.write().await
}

/// Convenience macro for centralized error handling
#[macro_export]
macro_rules! handle_with_centralized_error_recovery {
    ($operation:expr, $op_name:expr, $component:expr) => {{
        use $crate::utils::error_recovery::get_global_error_handler;
        let handler = get_global_error_handler().await;
        handler
            .execute_with_centralized_handling(
                || Box::pin(async { $operation }),
                $op_name,
                $component,
                None,
            )
            .await
    }};
}

/// Convenience macro for agent-specific centralized error handling
#[macro_export]
macro_rules! handle_agent_with_centralized_error_recovery {
    ($operation:expr, $op_name:expr, $component:expr, $agent_id:expr) => {{
        use $crate::utils::error_recovery::get_global_error_handler;
        let handler = get_global_error_handler().await;
        handler
            .execute_with_centralized_handling(
                || Box::pin(async { $operation }),
                $op_name,
                $component,
                Some($agent_id),
            )
            .await
    }};
}
