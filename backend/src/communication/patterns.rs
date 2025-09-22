//! # Standardized Communication Patterns
//!
//! This module defines standardized patterns for agent communication including:
//! - Message passing interfaces
//! - Error handling patterns
//! - Cancellation and timeout handling
//! - Performance optimizations
//! - Resource management

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// Message priority levels for communication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum MessagePriority {
    /// Low priority - can be delayed or dropped under load
    Low,
    /// Normal priority - standard processing
    Normal,
    /// High priority - expedited processing
    High,
    /// Critical priority - immediate processing required
    Critical,
}

/// Communication channel types
#[derive(Debug, Clone)]
pub enum CommunicationChannel {
    /// WebSocket-based communication
    WebSocket {
        sender: Arc<
            RwLock<
                futures_util::stream::SplitSink<
                    axum::extract::ws::WebSocket,
                    axum::extract::ws::Message,
                >,
            >,
        >,
        client_id: Uuid,
    },
    /// MCP-based communication
    MCP {
        sender: mpsc::UnboundedSender<serde_json::Value>,
        client_id: Uuid,
    },
    /// Internal agent-to-agent communication
    Internal {
        sender: mpsc::UnboundedSender<super::protocols::MessageEnvelope>,
        agent_id: Uuid,
    },
}

/// Standardized communication result type
pub type CommunicationResult<T> = Result<T, crate::utils::error::HiveError>;

/// Message delivery guarantees
#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum DeliveryGuarantee {
    /// At most once delivery
    AtMostOnce,
    /// At least once delivery
    AtLeastOnce,
    /// Exactly once delivery
    ExactlyOnce,
}

/// Communication configuration
#[derive(Debug, Clone)]
pub struct CommunicationConfig {
    pub default_timeout: std::time::Duration,
    pub max_retries: u32,
    pub retry_delay: std::time::Duration,
    pub max_concurrent_messages: usize,
    pub buffer_size: usize,
    pub enable_compression: bool,
    pub delivery_guarantee: DeliveryGuarantee,
}

impl Default for CommunicationConfig {
    fn default() -> Self {
        Self {
            default_timeout: std::time::Duration::from_secs(30),
            max_retries: 3,
            retry_delay: std::time::Duration::from_millis(100),
            max_concurrent_messages: 1000,
            buffer_size: 8192,
            enable_compression: false,
            delivery_guarantee: DeliveryGuarantee::AtLeastOnce,
        }
    }
}

/// Standardized async communication trait
#[async_trait]
pub trait AsyncCommunicator: Send + Sync {
    /// Send a message asynchronously with timeout and retry logic
    async fn send_message(
        &self,
        message: super::protocols::MessageEnvelope,
        config: &CommunicationConfig,
    ) -> CommunicationResult<()>;

    /// Receive a message asynchronously
    async fn receive_message(
        &self,
        timeout: std::time::Duration,
    ) -> CommunicationResult<super::protocols::MessageEnvelope>;

    /// Send a message and wait for response
    async fn request_response(
        &self,
        request: super::protocols::MessageEnvelope,
        timeout: std::time::Duration,
    ) -> CommunicationResult<super::protocols::MessageEnvelope>;

    /// Broadcast message to multiple recipients
    async fn broadcast(
        &self,
        message: super::protocols::MessageEnvelope,
        recipients: &[Uuid],
    ) -> CommunicationResult<()>;

    /// Get communication statistics
    async fn get_stats(&self) -> CommunicationResult<CommunicationStats>;
}

/// Communication statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub messages_failed: u64,
    pub average_response_time_ms: f64,
    pub active_connections: u64,
    pub total_bandwidth_bytes: u64,
    pub compression_ratio: f64,
    pub error_rate: f64,
}

impl Default for CommunicationStats {
    fn default() -> Self {
        Self {
            messages_sent: 0,
            messages_received: 0,
            messages_failed: 0,
            average_response_time_ms: 0.0,
            active_connections: 0,
            total_bandwidth_bytes: 0,
            compression_ratio: 1.0,
            error_rate: 0.0,
        }
    }
}

/// Standardized message handler trait
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle an incoming message
    async fn handle_message(
        &self,
        message: super::protocols::MessageEnvelope,
    ) -> CommunicationResult<Option<super::protocols::MessageEnvelope>>;

    /// Handle communication errors
    async fn handle_error(&self, error: &crate::utils::error::HiveError)
        -> CommunicationResult<()>;

    /// Check if handler can process the message type
    fn can_handle(&self, message_type: &super::protocols::MessageType) -> bool;
}

/// Connection pool for managing communication resources
pub struct ConnectionPool {
    connections: Arc<RwLock<std::collections::HashMap<String, CommunicationChannel>>>,
    config: CommunicationConfig,
    stats: Arc<RwLock<CommunicationStats>>,
}

impl ConnectionPool {
    /// Create a new connection pool
    #[must_use]
    pub fn new(config: CommunicationConfig) -> Self {
        Self {
            connections: Arc::new(RwLock::new(std::collections::HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(CommunicationStats::default())),
        }
    }

    /// Get or create a connection
    pub async fn get_connection(&self, key: &str) -> CommunicationResult<CommunicationChannel> {
        let connections = self.connections.read().await;
        if let Some(channel) = connections.get(key) {
            return Ok(channel.clone());
        }
        drop(connections);

        // Create new connection if it doesn't exist
        Err(crate::utils::error::HiveError::Communication {
            reason: format!("Connection not found for key: {key}"),
        })
    }

    /// Add a connection to the pool
    pub async fn add_connection(&self, key: String, channel: CommunicationChannel) {
        let mut connections = self.connections.write().await;
        connections.insert(key, channel);

        let mut stats = self.stats.write().await;
        stats.active_connections = connections.len() as u64;
    }

    /// Remove a connection from the pool
    pub async fn remove_connection(&self, key: &str) {
        let mut connections = self.connections.write().await;
        connections.remove(key);

        let mut stats = self.stats.write().await;
        stats.active_connections = connections.len() as u64;
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> CommunicationStats {
        self.stats.read().await.clone()
    }

    /// Cleanup idle connections
    pub async fn cleanup_idle(&self) {
        // Implementation would check connection last activity and remove idle ones
        // For now, this is a placeholder
    }
}

/// Circuit breaker for fault tolerance
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitBreakerState>>,
    config: CircuitBreakerConfig,
}

#[derive(Debug, Clone)]
struct CircuitBreakerState {
    failures: u32,
    last_failure_time: Option<std::time::Instant>,
    state: CircuitState,
}

#[derive(Debug, Clone)]
enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failing, requests rejected
    HalfOpen, // Testing if service recovered
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout: std::time::Duration,
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: std::time::Duration::from_secs(60),
            success_threshold: 3,
        }
    }
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    #[must_use]
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitBreakerState {
                failures: 0,
                last_failure_time: None,
                state: CircuitState::Closed,
            })),
            config,
        }
    }

    /// Execute an operation with circuit breaker protection
    pub async fn execute<F, Fut, T>(&self, operation: F) -> CommunicationResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = CommunicationResult<T>>,
    {
        let state = self.state.read().await.clone();

        match state.state {
            CircuitState::Open => {
                if let Some(last_failure) = state.last_failure_time {
                    if last_failure.elapsed() > self.config.recovery_timeout {
                        // Try to move to half-open
                        drop(state);
                        let mut state_mut = self.state.write().await;
                        state_mut.state = CircuitState::HalfOpen;
                        drop(state_mut);
                    } else {
                        return Err(crate::utils::error::HiveError::Communication {
                            reason: "Circuit breaker is open".to_string(),
                        });
                    }
                }
            }
            CircuitState::HalfOpen => {
                // Allow the request but be ready to close again on failure
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }

        match operation().await {
            Ok(result) => {
                let mut state = self.state.write().await;
                if let CircuitState::HalfOpen = state.state {
                    // Success in half-open state - close the circuit
                    state.state = CircuitState::Closed;
                    state.failures = 0;
                    state.last_failure_time = None;
                }
                Ok(result)
            }
            Err(e) => {
                let mut state = self.state.write().await;
                state.failures += 1;
                state.last_failure_time = Some(std::time::Instant::now());

                if state.failures >= self.config.failure_threshold {
                    state.state = CircuitState::Open;
                } else if matches!(state.state, CircuitState::HalfOpen) {
                    state.state = CircuitState::Open;
                }

                Err(e)
            }
        }
    }
}

/// Retry mechanism with exponential backoff
pub struct RetryMechanism {
    config: RetryConfig,
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: std::time::Duration,
    pub max_delay: std::time::Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: std::time::Duration::from_millis(100),
            max_delay: std::time::Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryMechanism {
    /// Create a new retry mechanism
    #[must_use]
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute an operation with retry logic
    pub async fn execute<F, Fut, T>(&self, mut operation: F) -> CommunicationResult<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = CommunicationResult<T>>,
    {
        let mut attempt = 0;
        let mut delay = self.config.initial_delay;

        loop {
            attempt += 1;

            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt >= self.config.max_attempts {
                        return Err(e);
                    }

                    tracing::warn!(
                        "Operation failed (attempt {}/{}): {}. Retrying in {:?}",
                        attempt,
                        self.config.max_attempts,
                        e,
                        delay
                    );

                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(
                        delay.mul_f64(self.config.backoff_multiplier),
                        self.config.max_delay,
                    );
                }
            }
        }
    }
}

/// Resource manager for communication resources
pub struct ResourceManager {
    semaphore: Arc<tokio::sync::Semaphore>,
    stats: Arc<RwLock<ResourceStats>>,
}

#[derive(Debug, Clone, Default)]
pub struct ResourceStats {
    pub active_connections: u64,
    pub total_connections_created: u64,
    pub peak_concurrent_connections: u64,
    pub average_connection_duration_ms: f64,
    pub memory_usage_bytes: u64,
}

impl ResourceManager {
    /// Create a new resource manager
    #[must_use]
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(tokio::sync::Semaphore::new(max_concurrent)),
            stats: Arc::new(RwLock::new(ResourceStats::default())),
        }
    }

    /// Acquire a resource permit
    pub async fn acquire(
        &self,
    ) -> Result<tokio::sync::SemaphorePermit<'_>, crate::utils::error::HiveError> {
        let permit = self.semaphore.acquire().await.map_err(|e| {
            crate::utils::error::HiveError::Communication {
                reason: format!("Failed to acquire resource permit: {e}"),
            }
        })?;

        let mut stats = self.stats.write().await;
        stats.active_connections += 1;
        stats.total_connections_created += 1;
        stats.peak_concurrent_connections = stats
            .peak_concurrent_connections
            .max(stats.active_connections);

        Ok(permit)
    }

    /// Get resource statistics
    pub async fn get_stats(&self) -> ResourceStats {
        self.stats.read().await.clone()
    }

    /// Update memory usage
    pub async fn update_memory_usage(&self, bytes: u64) {
        let mut stats = self.stats.write().await;
        stats.memory_usage_bytes = bytes;
    }
}
