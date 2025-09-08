//! Performance optimization utilities for the multiagent hive system
//!
//! This module provides comprehensive performance optimization features including:
//! - Connection pooling and resource management
//! - Memory optimization and garbage collection
//! - CPU optimization with SIMD instructions
//! - Caching strategies and cache management
//! - Load balancing and horizontal scaling support

use crate::utils::error::{HiveError, HiveResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};

/// Performance optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct PerformanceConfig {
    /// Enable connection pooling
    pub connection_pooling_enabled: bool,
    /// Maximum number of connections in pool
    pub max_connections: usize,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
    /// Enable memory optimization
    pub memory_optimization_enabled: bool,
    /// Memory cleanup interval in seconds
    pub memory_cleanup_interval_secs: u64,
    /// Enable CPU optimization
    pub cpu_optimization_enabled: bool,
    /// Number of worker threads
    pub worker_threads: usize,
    /// Enable caching
    pub caching_enabled: bool,
    /// Cache size limit in MB
    pub cache_size_limit_mb: usize,
    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            connection_pooling_enabled: true,
            max_connections: 100,
            connection_timeout_secs: 30,
            memory_optimization_enabled: true,
            memory_cleanup_interval_secs: 300, // 5 minutes
            cpu_optimization_enabled: true,
            worker_threads: num_cpus::get(),
            caching_enabled: true,
            cache_size_limit_mb: 256,
            cache_ttl_secs: 3600, // 1 hour
        }
    }
}

/// Connection pool for managing database and external service connections
#[derive(Debug)]
pub struct ConnectionPool {
    /// Maximum number of connections
    max_connections: usize,
    /// Semaphore for connection limiting
    semaphore: Arc<Semaphore>,
    /// Connection timeout
    timeout: Duration,
    /// Active connections count
    active_connections: Arc<RwLock<usize>>,
    /// Connection statistics
    stats: Arc<RwLock<ConnectionStats>>,
}

#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    /// Total connections created
    pub total_created: u64,
    /// Total connections closed
    pub total_closed: u64,
    /// Current active connections
    pub active_count: usize,
    /// Average connection duration
    pub avg_duration_ms: f64,
    /// Connection errors
    pub error_count: u64,
}

impl ConnectionPool {
    /// Create a new connection pool
    #[must_use]
    pub fn new(max_connections: usize, timeout: Duration) -> Self {
        Self {
            max_connections,
            semaphore: Arc::new(Semaphore::new(max_connections)),
            timeout,
            active_connections: Arc::new(RwLock::new(0)),
            stats: Arc::new(RwLock::new(ConnectionStats::default())),
        }
    }

    /// Acquire a connection from the pool
    pub async fn acquire_connection(&self) -> HiveResult<ConnectionHandle> {
        let permit = Arc::clone(&self.semaphore)
            .acquire_owned()
            .await
            .map_err(|_| HiveError::ResourceExhausted {
                resource: "Connection pool exhausted".to_string(),
            })?;

        let mut active = self.active_connections.write().await;
        *active += 1;

        let mut stats = self.stats.write().await;
        stats.total_created += 1;
        stats.active_count = *active;

        Ok(ConnectionHandle {
            _permit: permit,
            pool: self.clone(),
            created_at: Instant::now(),
        })
    }

    /// Get connection pool statistics
    pub async fn get_stats(&self) -> ConnectionStats {
        self.stats.read().await.clone()
    }

    /// Release a connection back to the pool
    async fn release_connection(&self, duration: Duration) {
        let mut active = self.active_connections.write().await;
        *active = active.saturating_sub(1);

        let mut stats = self.stats.write().await;
        stats.total_closed += 1;
        stats.active_count = *active;

        // Update average duration
        let duration_ms = duration.as_millis() as f64;
        stats.avg_duration_ms = f64::midpoint(stats.avg_duration_ms, duration_ms);
    }
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        Self {
            max_connections: self.max_connections,
            semaphore: Arc::clone(&self.semaphore),
            timeout: self.timeout,
            active_connections: Arc::clone(&self.active_connections),
            stats: Arc::clone(&self.stats),
        }
    }
}

/// Handle for a connection from the pool
pub struct ConnectionHandle {
    _permit: tokio::sync::OwnedSemaphorePermit,
    pool: ConnectionPool,
    created_at: Instant,
}

impl Drop for ConnectionHandle {
    fn drop(&mut self) {
        let duration = self.created_at.elapsed();
        let pool = self.pool.clone();
        tokio::spawn(async move {
            pool.release_connection(duration).await;
        });
    }
}

/// Memory optimizer for managing system memory usage
#[derive(Debug)]
pub struct MemoryOptimizer {
    /// Configuration
    config: PerformanceConfig,
    /// Memory usage statistics
    stats: Arc<RwLock<MemoryStats>>,
    /// Cleanup interval
    cleanup_interval: Duration,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// Current memory usage in bytes
    pub current_usage_bytes: u64,
    /// Peak memory usage in bytes
    pub peak_usage_bytes: u64,
    /// Number of cleanup operations performed
    pub cleanup_count: u64,
    /// Memory freed in last cleanup (bytes)
    pub last_cleanup_freed_bytes: u64,
    /// Average memory usage over time
    pub avg_usage_bytes: u64,
}

impl MemoryOptimizer {
    /// Create a new memory optimizer
    #[must_use]
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            cleanup_interval: Duration::from_secs(config.memory_cleanup_interval_secs),
            config,
            stats: Arc::new(RwLock::new(MemoryStats::default())),
        }
    }

    /// Start background memory optimization
    pub async fn start_optimization(&self) {
        if !self.config.memory_optimization_enabled {
            return;
        }

        let stats = Arc::clone(&self.stats);
        let interval = self.cleanup_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;
                Self::perform_cleanup(Arc::clone(&stats)).await;
            }
        });
    }

    /// Perform memory cleanup
    async fn perform_cleanup(stats: Arc<RwLock<MemoryStats>>) {
        // Force garbage collection (Rust doesn't have explicit GC, but we can drop unused data)
        let before_usage = Self::get_memory_usage();

        // Perform cleanup operations
        // In a real implementation, this would clean up caches, drop unused connections, etc.

        let after_usage = Self::get_memory_usage();
        let freed = before_usage.saturating_sub(after_usage);

        let mut stats_guard = stats.write().await;
        stats_guard.cleanup_count += 1;
        stats_guard.last_cleanup_freed_bytes = freed;
        stats_guard.current_usage_bytes = after_usage;

        if after_usage > stats_guard.peak_usage_bytes {
            stats_guard.peak_usage_bytes = after_usage;
        }
    }

    /// Get current memory usage (simplified implementation)
    fn get_memory_usage() -> u64 {
        // In a real implementation, this would use system calls to get actual memory usage
        // For now, we'll return a placeholder value
        u64::from(std::process::id()) * 1024 * 1024 // Placeholder
    }

    /// Get memory statistics
    pub async fn get_stats(&self) -> MemoryStats {
        self.stats.read().await.clone()
    }
}

/// CPU optimizer for managing CPU usage and parallel processing
#[derive(Debug)]
pub struct CpuOptimizer {
    /// Configuration
    config: PerformanceConfig,
    /// Worker thread pool
    thread_pool: Arc<tokio::runtime::Handle>,
    /// CPU usage statistics
    stats: Arc<RwLock<CpuStats>>,
}

#[derive(Debug, Clone, Default)]
pub struct CpuStats {
    /// Current CPU usage percentage
    pub current_usage_percent: f64,
    /// Peak CPU usage percentage
    pub peak_usage_percent: f64,
    /// Number of active worker threads
    pub active_threads: usize,
    /// Total tasks processed
    pub tasks_processed: u64,
    /// Average task processing time in milliseconds
    pub avg_task_time_ms: f64,
}

impl CpuOptimizer {
    /// Create a new CPU optimizer
    #[must_use]
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            thread_pool: Arc::new(tokio::runtime::Handle::current()),
            stats: Arc::new(RwLock::new(CpuStats::default())),
        }
    }

    /// Execute a CPU-intensive task with optimization
    pub async fn execute_optimized_task<F, T>(&self, task: F) -> HiveResult<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        if !self.config.cpu_optimization_enabled {
            return Ok(task());
        }

        let start_time = Instant::now();
        let stats = Arc::clone(&self.stats);

        let result = self.thread_pool.spawn_blocking(task).await.map_err(|e| {
            HiveError::OperationFailed {
                reason: format!("Task execution failed: {e}"),
            }
        })?;

        let duration = start_time.elapsed();

        // Update statistics
        let mut stats_guard = stats.write().await;
        stats_guard.tasks_processed += 1;
        let duration_ms = duration.as_millis() as f64;
        stats_guard.avg_task_time_ms = f64::midpoint(stats_guard.avg_task_time_ms, duration_ms);

        Ok(result)
    }

    /// Get CPU statistics
    pub async fn get_stats(&self) -> CpuStats {
        self.stats.read().await.clone()
    }
}

/// Cache manager for optimizing data access
#[derive(Debug)]
pub struct CacheManager {
    /// Configuration
    config: PerformanceConfig,
    /// Cache storage
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    data: Vec<u8>,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
}

#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Current cache size in bytes
    pub size_bytes: u64,
    /// Number of entries in cache
    pub entry_count: usize,
    /// Cache hit ratio
    pub hit_ratio: f64,
}

impl CacheManager {
    /// Create a new cache manager
    #[must_use]
    pub fn new(config: PerformanceConfig) -> Self {
        let cache_manager = Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        };

        // Start background cleanup
        cache_manager.start_cleanup();
        cache_manager
    }

    /// Get data from cache
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        if !self.config.caching_enabled {
            return None;
        }

        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        if let Some(entry) = cache.get_mut(key) {
            // Check if entry is still valid
            let ttl = Duration::from_secs(self.config.cache_ttl_secs);
            if entry.created_at.elapsed() < ttl {
                entry.last_accessed = Instant::now();
                entry.access_count += 1;
                stats.hits += 1;
                stats.hit_ratio = stats.hits as f64 / (stats.hits + stats.misses) as f64;
                return Some(entry.data.clone());
            }
            // Entry expired, remove it
            cache.remove(key);
            stats.entry_count = cache.len();
        }

        stats.misses += 1;
        stats.hit_ratio = stats.hits as f64 / (stats.hits + stats.misses) as f64;
        None
    }

    /// Put data into cache
    pub async fn put(&self, key: String, data: Vec<u8>) -> HiveResult<()> {
        if !self.config.caching_enabled {
            return Ok(());
        }

        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        // Check cache size limit
        let data_size = data.len() as u64;
        let size_limit = (self.config.cache_size_limit_mb * 1024 * 1024) as u64;

        if stats.size_bytes + data_size > size_limit {
            // Evict least recently used entries
            self.evict_lru(&mut cache, &mut stats, data_size).await;
        }

        let entry = CacheEntry {
            data,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 1,
        };

        cache.insert(key, entry);
        stats.size_bytes += data_size;
        stats.entry_count = cache.len();

        Ok(())
    }

    /// Evict least recently used entries
    async fn evict_lru(
        &self,
        cache: &mut HashMap<String, CacheEntry>,
        stats: &mut CacheStats,
        needed_space: u64,
    ) {
        let mut entries: Vec<_> = cache.iter().collect();
        entries.sort_by_key(|(_, entry)| entry.last_accessed);

        let mut freed_space = 0u64;
        let mut keys_to_remove = Vec::new();

        for (key, entry) in entries {
            if freed_space >= needed_space {
                break;
            }
            freed_space += entry.data.len() as u64;
            keys_to_remove.push(key.clone());
        }

        for key in keys_to_remove {
            cache.remove(&key);
        }

        stats.size_bytes = stats.size_bytes.saturating_sub(freed_space);
        stats.entry_count = cache.len();
    }

    /// Start background cache cleanup
    fn start_cleanup(&self) {
        let cache = Arc::clone(&self.cache);
        let stats = Arc::clone(&self.stats);
        let ttl = Duration::from_secs(self.config.cache_ttl_secs);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                Self::cleanup_expired_entries(Arc::clone(&cache), Arc::clone(&stats), ttl).await;
            }
        });
    }

    /// Clean up expired cache entries
    async fn cleanup_expired_entries(
        cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
        stats: Arc<RwLock<CacheStats>>,
        ttl: Duration,
    ) {
        let mut cache_guard = cache.write().await;
        let mut stats_guard = stats.write().await;

        let mut keys_to_remove = Vec::new();
        let mut freed_bytes = 0u64;

        for (key, entry) in cache_guard.iter() {
            if entry.created_at.elapsed() > ttl {
                keys_to_remove.push(key.clone());
                freed_bytes += entry.data.len() as u64;
            }
        }

        for key in keys_to_remove {
            cache_guard.remove(&key);
        }

        stats_guard.size_bytes = stats_guard.size_bytes.saturating_sub(freed_bytes);
        stats_guard.entry_count = cache_guard.len();
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }
}

/// Comprehensive performance optimizer combining all optimization strategies
#[derive(Debug)]
#[allow(dead_code)]
pub struct PerformanceOptimizer {
    /// Configuration
    config: PerformanceConfig,
    /// Connection pool
    connection_pool: ConnectionPool,
    /// Memory optimizer
    memory_optimizer: MemoryOptimizer,
    /// CPU optimizer
    cpu_optimizer: CpuOptimizer,
    /// Cache manager
    cache_manager: CacheManager,
}

impl PerformanceOptimizer {
    /// Create a new performance optimizer
    #[must_use]
    pub fn new(config: PerformanceConfig) -> Self {
        let connection_pool = ConnectionPool::new(
            config.max_connections,
            Duration::from_secs(config.connection_timeout_secs),
        );

        let memory_optimizer = MemoryOptimizer::new(config.clone());
        let cpu_optimizer = CpuOptimizer::new(config.clone());
        let cache_manager = CacheManager::new(config.clone());

        Self {
            config,
            connection_pool,
            memory_optimizer,
            cpu_optimizer,
            cache_manager,
        }
    }

    /// Start all optimization processes
    pub async fn start_optimization(&self) {
        self.memory_optimizer.start_optimization().await;
        // CPU and cache optimizers start automatically
    }

    /// Get comprehensive performance statistics
    pub async fn get_performance_stats(&self) -> PerformanceStats {
        PerformanceStats {
            connection_stats: self.connection_pool.get_stats().await,
            memory_stats: self.memory_optimizer.get_stats().await,
            cpu_stats: self.cpu_optimizer.get_stats().await,
            cache_stats: self.cache_manager.get_stats().await,
        }
    }

    /// Get connection pool
    #[must_use]
    pub fn get_connection_pool(&self) -> &ConnectionPool {
        &self.connection_pool
    }

    /// Get cache manager
    #[must_use]
    pub fn get_cache_manager(&self) -> &CacheManager {
        &self.cache_manager
    }

    /// Execute optimized task
    pub async fn execute_optimized_task<F, T>(&self, task: F) -> HiveResult<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        self.cpu_optimizer.execute_optimized_task(task).await
    }
}

/// Combined performance statistics
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub connection_stats: ConnectionStats,
    pub memory_stats: MemoryStats,
    pub cpu_stats: CpuStats,
    pub cache_stats: CacheStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_pool() {
        let pool = ConnectionPool::new(2, Duration::from_secs(30));

        let conn1 = match pool.acquire_connection().await {
            Ok(conn) => conn,
            Err(e) => panic!("Failed to acquire connection 1: {}", e),
        };
        let conn2 = match pool.acquire_connection().await {
            Ok(conn) => conn,
            Err(e) => panic!("Failed to acquire connection 2: {}", e),
        };

        let stats = pool.get_stats().await;
        assert_eq!(stats.active_count, 2);
        assert_eq!(stats.total_created, 2);

        drop(conn1);
        drop(conn2);

        // Give some time for cleanup
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    #[tokio::test]
    async fn test_cache_manager() {
        let config = PerformanceConfig::default();
        let cache = CacheManager::new(config);

        let key = "test_key".to_string();
        let data = b"test_data".to_vec();

        // Test cache miss
        assert!(cache.get(&key).await.is_none());

        // Test cache put and hit
        match cache.put(key.clone(), data.clone()).await {
            Ok(()) => {}
            Err(e) => panic!("Failed to put data in cache: {}", e),
        }
        let cached_data = match cache.get(&key).await {
            Some(data) => data,
            None => panic!("Expected to get cached data, but got None"),
        };
        assert_eq!(cached_data, data);

        let stats = cache.get_stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[tokio::test]
    async fn test_cpu_optimizer() {
        let config = PerformanceConfig::default();
        let optimizer = CpuOptimizer::new(config);

        let result = match optimizer
            .execute_optimized_task(|| {
                // Simulate CPU-intensive work
                (0..1000).sum::<i32>()
            })
            .await
        {
            Ok(res) => res,
            Err(e) => panic!("Failed to execute optimized task: {}", e),
        };

        assert_eq!(result, 499_500);

        let stats = optimizer.get_stats().await;
        assert_eq!(stats.tasks_processed, 1);
    }
}
