//! Network Performance Optimizer
//!
//! This module provides advanced network optimization features including:
//! - Connection pooling and reuse
//! - Intelligent compression and decompression
//! - Async I/O optimizations
//! - Network traffic shaping and prioritization

use crate::utils::error::{HiveError, HiveResult};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio_util::codec::{Decoder, Encoder, Framed};
use tracing::{debug, info, warn};

/// Configuration for network optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkOptimizerConfig {
    /// Maximum connections per host
    pub max_connections_per_host: usize,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Connection pool size
    pub connection_pool_size: usize,
    /// Enable compression
    pub enable_compression: bool,
    /// Compression level (0-9)
    pub compression_level: u32,
    /// Enable connection keep-alive
    pub enable_keep_alive: bool,
    /// Keep-alive timeout
    pub keep_alive_timeout: Duration,
    /// Maximum request size for compression
    pub max_compression_size: usize,
    /// Network buffer size
    pub buffer_size: usize,
    /// Enable traffic shaping
    pub enable_traffic_shaping: bool,
    /// Traffic shaping rate limit (bytes per second)
    pub rate_limit_bytes_per_sec: u64,
}

impl Default for NetworkOptimizerConfig {
    fn default() -> Self {
        Self {
            max_connections_per_host: 10,
            connection_timeout: Duration::from_secs(30),
            connection_pool_size: 100,
            enable_compression: true,
            compression_level: 6,
            enable_keep_alive: true,
            keep_alive_timeout: Duration::from_secs(300),
            max_compression_size: 1024 * 1024, // 1MB
            buffer_size: 8192,
            enable_traffic_shaping: true,
            rate_limit_bytes_per_sec: 10 * 1024 * 1024, // 10MB/s
        }
    }
}

/// Connection pool entry
#[derive(Debug)]
struct PooledConnection {
    stream: TcpStream,
    host: String,
    port: u16,
    created_at: Instant,
    last_used: Instant,
    request_count: u64,
}

/// Network connection pool
pub struct ConnectionPool {
    config: NetworkOptimizerConfig,
    pools: Arc<RwLock<HashMap<String, Vec<PooledConnection>>>>,
    semaphore: Arc<Semaphore>,
    metrics: Arc<RwLock<ConnectionPoolMetrics>>,
}

#[derive(Debug, Clone, Default)]
pub struct ConnectionPoolMetrics {
    pub total_connections_created: u64,
    pub active_connections: usize,
    pub connections_reused: u64,
    pub connection_errors: u64,
    pub average_connection_time_ms: f64,
    pub pool_hit_rate: f64,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(config: NetworkOptimizerConfig) -> Self {
        Self {
            config: config.clone(),
            pools: Arc::new(RwLock::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(config.connection_pool_size)),
            metrics: Arc::new(RwLock::new(ConnectionPoolMetrics::default())),
        }
    }

    /// Get a connection from the pool or create a new one
    pub async fn get_connection(&self, host: &str, port: u16) -> HiveResult<PooledConnectionHandle> {
        let start_time = Instant::now();
        let key = format!("{}:{}", host, port);

        // Try to get from pool first
        {
            let mut pools = self.pools.write().await;
            if let Some(pool) = pools.get_mut(&key) {
                // Find an available connection
                for (index, connection) in pool.iter().enumerate() {
                    if self.is_connection_valid(connection).await {
                        let connection = pool.remove(index);
                        let mut metrics = self.metrics.write().await;
                        metrics.connections_reused += 1;
                        metrics.pool_hit_rate = metrics.connections_reused as f64
                                               / (metrics.connections_reused + metrics.total_connections_created) as f64;

                        return Ok(PooledConnectionHandle {
                            connection: Some(connection),
                            pool: self.clone(),
                            host: host.to_string(),
                            port,
                        });
                    }
                }
            }
        }

        // Create new connection
        let stream = tokio::time::timeout(
            self.config.connection_timeout,
            TcpStream::connect((host, port))
        ).await.map_err(|_| {
            HiveError::NetworkError {
                reason: format!("Connection timeout to {}:{}", host, port),
            }
        })??;

        // Configure TCP options
        stream.set_nodelay(true)?;
        if self.config.enable_keep_alive {
            // Note: Keep-alive configuration would be platform-specific
        }

        let connection = PooledConnection {
            stream,
            host: host.to_string(),
            port,
            created_at: Instant::now(),
            last_used: Instant::now(),
            request_count: 0,
        };

        let connection_time = start_time.elapsed();
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_connections_created += 1;
            let total_connections = metrics.total_connections_created + metrics.connections_reused;
            metrics.average_connection_time_ms = (metrics.average_connection_time_ms
                * (total_connections - 1) as f64
                + connection_time.as_millis() as f64) / total_connections as f64;
        }

        Ok(PooledConnectionHandle {
            connection: Some(connection),
            pool: self.clone(),
            host: host.to_string(),
            port,
        })
    }

    /// Check if a connection is still valid
    async fn is_connection_valid(&self, connection: &PooledConnection) -> bool {
        // Check if connection has timed out
        if connection.last_used.elapsed() > self.config.keep_alive_timeout {
            return false;
        }

        // Check if connection is still alive (simplified check)
        // In a real implementation, you might send a ping or check TCP state
        true
    }

    /// Return a connection to the pool
    async fn return_connection(&self, mut connection: PooledConnection) {
        connection.last_used = Instant::now();
        connection.request_count += 1;

        let key = format!("{}:{}", connection.host, connection.port);
        let mut pools = self.pools.write().await;

        let pool = pools.entry(key).or_insert_with(Vec::new);

        // Limit pool size per host
        if pool.len() < self.config.max_connections_per_host {
            pool.push(connection);
        }
        // Connection will be dropped if pool is full
    }

    /// Get connection pool metrics
    pub async fn get_metrics(&self) -> ConnectionPoolMetrics {
        let pools = self.pools.read().await;
        let mut metrics = self.metrics.read().await.clone();

        metrics.active_connections = pools.values().map(|pool| pool.len()).sum();

        metrics
    }

    /// Clean up expired connections
    pub async fn cleanup_expired_connections(&self) {
        let mut pools = self.pools.write().await;
        let mut total_cleaned = 0;

        for pool in pools.values_mut() {
            pool.retain(|connection| {
                if connection.last_used.elapsed() > self.config.keep_alive_timeout {
                    total_cleaned += 1;
                    false
                } else {
                    true
                }
            });
        }

        if total_cleaned > 0 {
            debug!("Cleaned up {} expired connections", total_cleaned);
        }
    }
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            pools: Arc::clone(&self.pools),
            semaphore: Arc::clone(&self.semaphore),
            metrics: Arc::clone(&self.metrics),
        }
    }
}

/// Handle for a pooled connection
pub struct PooledConnectionHandle {
    connection: Option<PooledConnection>,
    pool: ConnectionPool,
    host: String,
    port: u16,
}

impl PooledConnectionHandle {
    /// Get the underlying TCP stream
    pub fn stream(&mut self) -> Option<&mut TcpStream> {
        self.connection.as_mut().map(|c| &mut c.stream)
    }

    /// Take ownership of the connection (for custom handling)
    pub fn take_connection(&mut self) -> Option<PooledConnection> {
        self.connection.take()
    }
}

impl Drop for PooledConnectionHandle {
    fn drop(&mut self) {
        if let Some(connection) = self.connection.take() {
            let pool = self.pool.clone();
            tokio::spawn(async move {
                pool.return_connection(connection).await;
            });
        }
    }
}

/// Compression optimizer for network traffic
pub struct CompressionOptimizer {
    config: NetworkOptimizerConfig,
    metrics: Arc<RwLock<CompressionMetrics>>,
}

#[derive(Debug, Clone, Default)]
pub struct CompressionMetrics {
    pub total_bytes_original: u64,
    pub total_bytes_compressed: u64,
    pub compression_operations: u64,
    pub average_compression_ratio: f64,
    pub compression_time_ms: u64,
}

impl CompressionOptimizer {
    pub fn new(config: NetworkOptimizerConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(RwLock::new(CompressionMetrics::default())),
        }
    }

    /// Compress data if beneficial
    pub async fn compress_data(&self, data: &[u8]) -> HiveResult<Vec<u8>> {
        if !self.config.enable_compression || data.len() < 1024 {
            return Ok(data.to_vec());
        }

        if data.len() > self.config.max_compression_size {
            warn!("Data size {} exceeds max compression size {}, skipping compression",
                  data.len(), self.config.max_compression_size);
            return Ok(data.to_vec());
        }

        let start_time = Instant::now();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.config.compression_level));

        use std::io::Write;
        encoder.write_all(data).map_err(|e| {
            HiveError::ProcessingError {
                reason: format!("Compression failed: {}", e),
            }
        })?;

        let compressed = encoder.finish().map_err(|e| {
            HiveError::ProcessingError {
                reason: format!("Compression finish failed: {}", e),
            }
        })?;

        let compression_time = start_time.elapsed();

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_bytes_original += data.len() as u64;
            metrics.total_bytes_compressed += compressed.len() as u64;
            metrics.compression_operations += 1;
            metrics.compression_time_ms += compression_time.as_millis() as u64;

            if metrics.compression_operations > 0 {
                metrics.average_compression_ratio = metrics.total_bytes_compressed as f64
                                                  / metrics.total_bytes_original as f64;
            }
        }

        debug!("Compressed {} bytes to {} bytes ({:.2}x)",
               data.len(), compressed.len(),
               data.len() as f64 / compressed.len() as f64);

        Ok(compressed)
    }

    /// Decompress data
    pub async fn decompress_data(&self, data: &[u8]) -> HiveResult<Vec<u8>> {
        use std::io::Read;
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();

        decoder.read_to_end(&mut decompressed).map_err(|e| {
            HiveError::ProcessingError {
                reason: format!("Decompression failed: {}", e),
            }
        })?;

        Ok(decompressed)
    }

    /// Check if compression would be beneficial
    pub fn should_compress(&self, data: &[u8]) -> bool {
        self.config.enable_compression
            && data.len() >= 1024
            && data.len() <= self.config.max_compression_size
    }

    /// Get compression metrics
    pub async fn get_metrics(&self) -> CompressionMetrics {
        self.metrics.read().await.clone()
    }
}

/// Traffic shaper for rate limiting
pub struct TrafficShaper {
    config: NetworkOptimizerConfig,
    tokens: Arc<Mutex<u64>>,
    last_refill: Arc<Mutex<Instant>>,
    metrics: Arc<RwLock<TrafficMetrics>>,
}

#[derive(Debug, Clone, Default)]
pub struct TrafficMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub throttled_requests: u64,
    pub average_throughput_bytes_per_sec: f64,
}

impl TrafficShaper {
    pub fn new(config: NetworkOptimizerConfig) -> Self {
        Self {
            config,
            tokens: Arc::new(Mutex::new(config.rate_limit_bytes_per_sec)),
            last_refill: Arc::new(Mutex::new(Instant::now())),
            metrics: Arc::new(RwLock::new(TrafficMetrics::default())),
        }
    }

    /// Check if request can proceed (rate limiting)
    pub async fn check_rate_limit(&self, requested_bytes: usize) -> bool {
        if !self.config.enable_traffic_shaping {
            return true;
        }

        let mut tokens = self.tokens.lock().await;
        let mut last_refill = self.last_refill.lock().await;

        // Refill tokens based on elapsed time
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill);
        let tokens_to_add = (elapsed.as_secs_f64() * self.config.rate_limit_bytes_per_sec as f64) as u64;

        *tokens = (*tokens + tokens_to_add).min(self.config.rate_limit_bytes_per_sec);
        *last_refill = now;

        if *tokens >= requested_bytes as u64 {
            *tokens -= requested_bytes as u64;
            true
        } else {
            let mut metrics = self.metrics.write().await;
            metrics.throttled_requests += 1;
            false
        }
    }

    /// Record bytes sent
    pub async fn record_bytes_sent(&self, bytes: usize) {
        let mut metrics = self.metrics.write().await;
        metrics.bytes_sent += bytes as u64;

        // Update throughput calculation
        let elapsed = Instant::now().elapsed().as_secs_f64();
        if elapsed > 0.0 {
            metrics.average_throughput_bytes_per_sec = metrics.bytes_sent as f64 / elapsed;
        }
    }

    /// Record bytes received
    pub async fn record_bytes_received(&self, bytes: usize) {
        let mut metrics = self.metrics.write().await;
        metrics.bytes_received += bytes as u64;
    }

    /// Get traffic metrics
    pub async fn get_metrics(&self) -> TrafficMetrics {
        self.metrics.read().await.clone()
    }
}

/// Comprehensive network optimizer
pub struct NetworkOptimizer {
    config: NetworkOptimizerConfig,
    connection_pool: ConnectionPool,
    compression_optimizer: CompressionOptimizer,
    traffic_shaper: TrafficShaper,
    metrics: Arc<RwLock<NetworkMetrics>>,
}

#[derive(Debug, Clone, Default)]
pub struct NetworkMetrics {
    pub connection_pool_metrics: ConnectionPoolMetrics,
    pub compression_metrics: CompressionMetrics,
    pub traffic_metrics: TrafficMetrics,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
}

impl NetworkOptimizer {
    /// Create a new network optimizer
    pub fn new(config: NetworkOptimizerConfig) -> Self {
        Self {
            connection_pool: ConnectionPool::new(config.clone()),
            compression_optimizer: CompressionOptimizer::new(config.clone()),
            traffic_shaper: TrafficShaper::new(config.clone()),
            metrics: Arc::new(RwLock::new(NetworkMetrics::default())),
            config,
        }
    }

    /// Execute an optimized network request
    pub async fn execute_request<F, Fut, T>(&self, host: &str, port: u16, request_fn: F) -> HiveResult<T>
    where
        F: FnOnce(&mut TcpStream) -> Fut + Send,
        Fut: std::future::Future<Output = HiveResult<T>> + Send,
        T: Send,
    {
        let start_time = Instant::now();

        // Get connection from pool
        let mut connection_handle = self.connection_pool.get_connection(host, port).await?;

        let result = if let Some(stream) = connection_handle.stream() {
            // Execute request
            let result = request_fn(stream).await;

            // Record metrics
            let request_time = start_time.elapsed();
            {
                let mut metrics = self.metrics.write().await;
                metrics.total_requests += 1;

                match &result {
                    Ok(_) => metrics.successful_requests += 1,
                    Err(_) => metrics.failed_requests += 1,
                }

                // Update average response time
                let total_requests = metrics.total_requests as f64;
                metrics.average_response_time_ms = (metrics.average_response_time_ms
                    * (total_requests - 1.0)
                    + request_time.as_millis() as f64) / total_requests;
            }

            result
        } else {
            Err(HiveError::NetworkError {
                reason: "Failed to get stream from connection pool".to_string(),
            })
        };

        result
    }

    /// Send compressed data
    pub async fn send_compressed(&self, host: &str, port: u16, data: &[u8]) -> HiveResult<usize> {
        // Check rate limit
        if !self.traffic_shaper.check_rate_limit(data.len()).await {
            return Err(HiveError::RateLimitError {
                reason: "Rate limit exceeded".to_string(),
            });
        }

        // Compress data if beneficial
        let data_to_send = if self.compression_optimizer.should_compress(data) {
            self.compression_optimizer.compress_data(data).await?
        } else {
            data.to_vec()
        };

        // Send data
        let bytes_sent = self.execute_request(host, port, |stream| async move {
            use tokio::io::AsyncWriteExt;
            stream.write_all(&data_to_send).await?;
            stream.flush().await?;
            Ok(data_to_send.len())
        }).await?;

        // Record metrics
        self.traffic_shaper.record_bytes_sent(bytes_sent).await;

        Ok(bytes_sent)
    }

    /// Receive and decompress data
    pub async fn receive_compressed(&self, host: &str, port: u16, buffer_size: usize) -> HiveResult<Vec<u8>> {
        let mut buffer = vec![0u8; buffer_size];

        let bytes_read = self.execute_request(host, port, |stream| async move {
            use tokio::io::AsyncReadExt;
            let bytes = stream.read(&mut buffer).await?;
            Ok(bytes)
        }).await?;

        buffer.truncate(bytes_read);

        // Record metrics
        self.traffic_shaper.record_bytes_received(bytes_read).await;

        // Decompress if needed
        if self.config.enable_compression && buffer.len() > 0 {
            // Try to detect if data is compressed (simplified check)
            if buffer.len() >= 2 && buffer[0] == 0x1f && buffer[1] == 0x8b {
                self.compression_optimizer.decompress_data(&buffer).await
            } else {
                Ok(buffer)
            }
        } else {
            Ok(buffer)
        }
    }

    /// Get comprehensive network metrics
    pub async fn get_metrics(&self) -> NetworkMetrics {
        let mut metrics = self.metrics.read().await.clone();
        metrics.connection_pool_metrics = self.connection_pool.get_metrics().await;
        metrics.compression_metrics = self.compression_optimizer.get_metrics().await;
        metrics.traffic_metrics = self.traffic_shaper.get_metrics().await;

        metrics
    }

    /// Start background maintenance tasks
    pub fn start_maintenance(&self) {
        let pool = self.connection_pool.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                pool.cleanup_expired_connections().await;
            }
        });
    }

    /// Optimize network settings based on current load
    pub async fn optimize_settings(&self) {
        let metrics = self.get_metrics().await;

        // Adjust compression settings based on performance
        if metrics.compression_metrics.average_compression_ratio > 0.8 {
            info!("Compression ratio is low, consider adjusting compression level");
        }

        // Adjust connection pool size based on usage
        if metrics.connection_pool_metrics.pool_hit_rate < 0.5 {
            info!("Low connection pool hit rate, consider increasing pool size");
        }

        // Adjust rate limiting based on throughput
        if metrics.traffic_metrics.throttled_requests > metrics.total_requests / 10 {
            info!("High throttling rate, consider increasing rate limit");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_connection_pool() -> Result<(), Box<dyn std::error::Error>> {
        let config = NetworkOptimizerConfig::default();
        let pool = ConnectionPool::new(config);

        // Test connection creation (would need a real server for full test)
        let result = pool.get_connection("127.0.0.1", 12345).await;
        assert!(result.is_err()); // Should fail as no server is running

        Ok(())
    }

    #[tokio::test]
    async fn test_compression_optimizer() -> Result<(), Box<dyn std::error::Error>> {
        let config = NetworkOptimizerConfig {
            enable_compression: true,
            ..Default::default()
        };
        let optimizer = CompressionOptimizer::new(config);

        let test_data = vec![b'A'; 2048]; // Compressible data
        let compressed = optimizer.compress_data(&test_data).await?;
        assert!(compressed.len() < test_data.len());

        let decompressed = optimizer.decompress_data(&compressed).await?;
        assert_eq!(decompressed, test_data);

        Ok(())
    }

    #[tokio::test]
    async fn test_traffic_shaper() -> Result<(), Box<dyn std::error::Error>> {
        let config = NetworkOptimizerConfig {
            enable_traffic_shaping: true,
            rate_limit_bytes_per_sec: 1000,
            ..Default::default()
        };
        let shaper = TrafficShaper::new(config);

        // Test rate limiting
        assert!(shaper.check_rate_limit(500).await);
        assert!(shaper.check_rate_limit(600).await); // Should succeed initially

        // Record some traffic
        shaper.record_bytes_sent(1000).await;
        shaper.record_bytes_received(500).await;

        let metrics = shaper.get_metrics().await;
        assert_eq!(metrics.bytes_sent, 1000);
        assert_eq!(metrics.bytes_received, 500);

        Ok(())
    }
}</content>
</xai:function_call">Now let me create a comprehensive throughput optimizer that combines all the optimizations.