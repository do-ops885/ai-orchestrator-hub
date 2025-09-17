use chrono::{DateTime, Utc};
use rusqlite::{Connection, OpenFlags};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::{timeout, Duration};

/// Configuration for database connection pool
#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    pub max_connections: usize,
    pub min_connections: usize,
    pub max_idle_time: Duration,
    pub connection_timeout: Duration,
    pub health_check_interval: Duration,
    pub database_path: String,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 2,
            max_idle_time: Duration::from_secs(300), // 5 minutes
            connection_timeout: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(60),
            database_path: "./data/hive.db".to_string(),
        }
    }
}

/// Pooled database connection with metadata
#[derive(Debug)]
pub struct PooledConnection {
    pub connection: Connection,
    pub created_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub in_use: bool,
}

impl PooledConnection {
    pub fn new(connection: Connection) -> Self {
        let now = Utc::now();
        Self {
            connection,
            created_at: now,
            last_used: now,
            in_use: false,
        }
    }

    pub fn mark_used(&mut self) {
        self.last_used = Utc::now();
        self.in_use = true;
    }

    pub fn mark_free(&mut self) {
        self.in_use = false;
    }

    pub fn is_expired(&self, max_idle_time: Duration) -> bool {
        let idle_time = Utc::now().signed_duration_since(self.last_used);
        idle_time > chrono::Duration::from_std(max_idle_time).unwrap_or(chrono::Duration::seconds(300))
    }
}

/// Database connection pool with health monitoring and automatic cleanup
pub struct ConnectionPool {
    config: ConnectionPoolConfig,
    pool: Arc<Mutex<VecDeque<PooledConnection>>>,
    semaphore: Arc<Semaphore>,
    stats: Arc<Mutex<PoolStats>>,
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub idle_connections: usize,
    pub connections_created: u64,
    pub connections_destroyed: u64,
    pub connection_requests: u64,
    pub connection_timeouts: u64,
    pub average_wait_time_ms: f64,
}

impl Default for PoolStats {
    fn default() -> Self {
        Self {
            total_connections: 0,
            active_connections: 0,
            idle_connections: 0,
            connections_created: 0,
            connections_destroyed: 0,
            connection_requests: 0,
            connection_timeouts: 0,
            average_wait_time_ms: 0.0,
        }
    }
}

impl ConnectionPool {
    /// Create a new connection pool
    pub async fn new(config: ConnectionPoolConfig) -> anyhow::Result<Self> {
        let pool = Arc::new(Mutex::new(VecDeque::new()));
        let semaphore = Arc::new(Semaphore::new(config.max_connections));
        let stats = Arc::new(Mutex::new(PoolStats::default()));

        let connection_pool = Self {
            config,
            pool,
            semaphore,
            stats,
        };

        // Initialize minimum connections
        connection_pool.initialize_pool().await?;

        // Start background cleanup task
        connection_pool.start_cleanup_task();

        Ok(connection_pool)
    }

    /// Initialize the pool with minimum connections
    async fn initialize_pool(&self) -> anyhow::Result<()> {
        for _ in 0..self.config.min_connections {
            let connection = self.create_connection().await?;
            let mut pool = self.pool.lock().await;
            pool.push_back(connection);

            let mut stats = self.stats.lock().await;
            stats.total_connections += 1;
            stats.idle_connections += 1;
            stats.connections_created += 1;
        }

        tracing::info!(
            "Initialized connection pool with {} connections",
            self.config.min_connections
        );

        Ok(())
    }

    /// Create a new database connection
    async fn create_connection(&self) -> anyhow::Result<PooledConnection> {
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_URI
            | OpenFlags::SQLITE_OPEN_NO_MUTEX; // We're handling our own synchronization

        let connection = Connection::open_with_flags(&self.config.database_path, flags)
            .map_err(|e| anyhow::anyhow!("Failed to create database connection: {}", e))?;

        // Configure connection for better performance
        connection.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = 1000;
            PRAGMA temp_store = memory;
            PRAGMA mmap_size = 268435456;
            "
        )?;

        Ok(PooledConnection::new(connection))
    }

    /// Get a connection from the pool
    pub async fn get_connection(&self) -> anyhow::Result<PooledConnectionHandle> {
        let start_time = std::time::Instant::now();

        // Acquire semaphore permit
        let permit = timeout(self.config.connection_timeout, self.semaphore.acquire()).await
            .map_err(|_| anyhow::anyhow!("Connection timeout"))?
            .map_err(|e| anyhow::anyhow!("Failed to acquire connection permit: {}", e))?;

        let mut stats = self.stats.lock().await;
        stats.connection_requests += 1;

        // Try to get an existing connection
        let mut pool = self.pool.lock().await;
        let connection = if let Some(mut conn) = pool.pop_front() {
            if conn.is_expired(self.config.max_idle_time) {
                // Connection is too old, create a new one
                drop(conn); // Explicitly drop the old connection
                stats.connections_destroyed += 1;
                match self.create_connection().await {
                    Ok(new_conn) => {
                        stats.connections_created += 1;
                        new_conn
                    }
                    Err(e) => {
                        // Failed to create new connection, try to use an existing one
                        if let Some(fallback_conn) = pool.pop_front() {
                            fallback_conn
                        } else {
                            return Err(anyhow::anyhow!("Failed to create connection and no fallback available: {}", e));
                        }
                    }
                }
            } else {
                conn
            }
        } else {
            // No available connections, create a new one
            drop(pool); // Release lock before creating connection
            let new_conn = self.create_connection().await?;
            stats.connections_created += 1;
            new_conn
        };

        stats.active_connections += 1;
        stats.idle_connections = stats.idle_connections.saturating_sub(1);

        let wait_time = start_time.elapsed().as_millis() as f64;
        stats.average_wait_time_ms = (stats.average_wait_time_ms + wait_time) / 2.0;

        Ok(PooledConnectionHandle {
            connection: Some(connection),
            pool: Arc::clone(&self.pool),
            semaphore: Arc::clone(&self.semaphore),
            stats: Arc::clone(&self.stats),
            _permit: permit,
        })
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> PoolStats {
        self.stats.lock().await.clone()
    }

    /// Start background cleanup task
    fn start_cleanup_task(&self) {
        let pool = Arc::clone(&self.pool);
        let stats = Arc::clone(&self.stats);
        let max_idle_time = self.config.max_idle_time;
        let max_connections = self.config.max_connections;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                let mut pool_guard = pool.lock().await;
                let mut stats_guard = stats.lock().await;

                // Remove expired connections
                let initial_len = pool_guard.len();
                pool_guard.retain(|conn| {
                    if conn.is_expired(max_idle_time) && !conn.in_use {
                        stats_guard.connections_destroyed += 1;
                        false
                    } else {
                        true
                    }
                });

                let removed = initial_len - pool_guard.len();
                if removed > 0 {
                    stats_guard.total_connections = stats_guard.total_connections.saturating_sub(removed);
                    stats_guard.idle_connections = stats_guard.idle_connections.saturating_sub(removed);
                }

                // Ensure we don't exceed max connections
                while pool_guard.len() > max_connections {
                    if let Some(conn) = pool_guard.pop_back() {
                        if !conn.in_use {
                            stats_guard.connections_destroyed += 1;
                            stats_guard.total_connections = stats_guard.total_connections.saturating_sub(1);
                            stats_guard.idle_connections = stats_guard.idle_connections.saturating_sub(1);
                        } else {
                            // Put it back if it's in use
                            pool_guard.push_back(conn);
                            break;
                        }
                    }
                }

                if removed > 0 {
                    tracing::debug!("Cleaned up {} expired connections", removed);
                }
            }
        });
    }

    /// Health check for the connection pool
    pub async fn health_check(&self) -> anyhow::Result<()> {
        // Try to get a connection to verify pool is working
        let handle = timeout(Duration::from_secs(5), self.get_connection()).await
            .map_err(|_| anyhow::anyhow!("Health check timeout"))?
            .map_err(|e| anyhow::anyhow!("Health check failed: {}", e))?;

        // Connection is automatically returned when handle is dropped
        drop(handle);

        Ok(())
    }
}

/// RAII handle for pooled connections
pub struct PooledConnectionHandle {
    connection: Option<PooledConnection>,
    pool: Arc<Mutex<VecDeque<PooledConnection>>>,
    semaphore: Arc<Semaphore>,
    stats: Arc<Mutex<PoolStats>>,
    _permit: tokio::sync::SemaphorePermit<'static>,
}

impl PooledConnectionHandle {
    /// Get mutable reference to the connection
    pub fn as_mut(&mut self) -> Result<&mut Connection, crate::utils::error::HiveError> {
        self.connection
            .as_mut()
            .map(|conn| &mut conn.connection)
            .ok_or_else(|| crate::utils::error::HiveError::OperationFailed {
                reason: "Connection handle is empty".to_string(),
            })
    }

    /// Mark the connection as used
    pub fn mark_used(&mut self) {
        if let Some(ref mut conn) = self.connection {
            conn.mark_used();
        }
    }
}

impl Drop for PooledConnectionHandle {
    fn drop(&mut self) {
        if let Some(mut conn) = self.connection.take() {
            conn.mark_free();

            // Return connection to pool
            let pool = Arc::clone(&self.pool);
            let stats = Arc::clone(&self.stats);

            tokio::spawn(async move {
                let mut pool_guard = pool.lock().await;
                pool_guard.push_back(conn);

                let mut stats_guard = stats.lock().await;
                stats_guard.active_connections = stats_guard.active_connections.saturating_sub(1);
                stats_guard.idle_connections += 1;
            });
        }
    }
}

/// Request caching system
pub struct RequestCache {
    cache: Arc<Mutex<HashMap<String, CachedResponse>>>,
    max_size: usize,
    ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub data: Vec<u8>,
    pub content_type: String,
    pub timestamp: DateTime<Utc>,
    pub ttl: Duration,
}

impl CachedResponse {
    pub fn is_expired(&self) -> bool {
        let elapsed = Utc::now().signed_duration_since(self.timestamp);
        elapsed > chrono::Duration::from_std(self.ttl).unwrap_or(chrono::Duration::seconds(300))
    }
}

impl RequestCache {
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            max_size,
            ttl,
        }
    }

    pub async fn get(&self, key: &str) -> Option<CachedResponse> {
        let mut cache = self.cache.lock().await;
        if let Some(response) = cache.get(key) {
            if !response.is_expired() {
                return Some(response.clone());
            } else {
                // Remove expired entry
                cache.remove(key);
            }
        }
        None
    }

    pub async fn set(&self, key: String, data: Vec<u8>, content_type: String) {
        let mut cache = self.cache.lock().await;

        // Remove expired entries
        cache.retain(|_, response| !response.is_expired());

        // Evict oldest entries if cache is full
        while cache.len() >= self.max_size {
            if let Some(oldest_key) = cache.keys().next().cloned() {
                cache.remove(&oldest_key);
            }
        }

        let response = CachedResponse {
            data,
            content_type,
            timestamp: Utc::now(),
            ttl: self.ttl,
        };

        cache.insert(key, response);
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }

    pub async fn stats(&self) -> (usize, usize) {
        let cache = self.cache.lock().await;
        let total_entries = cache.len();
        let expired_entries = cache.values().filter(|r| r.is_expired()).count();
        (total_entries, expired_entries)
    }
}

/// Optimized async task scheduler
pub struct AsyncTaskScheduler {
    semaphore: Arc<Semaphore>,
    stats: Arc<Mutex<SchedulerStats>>,
}

#[derive(Debug, Clone, Default)]
pub struct SchedulerStats {
    pub total_tasks: u64,
    pub active_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub average_execution_time_ms: f64,
}

impl AsyncTaskScheduler {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            stats: Arc::new(Mutex::new(SchedulerStats::default())),
        }
    }

    pub async fn schedule_task<F, Fut, T>(&self, task: F) -> anyhow::Result<T>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = anyhow::Result<T>> + Send + 'static,
        T: Send + 'static,
    {
        let permit = self.semaphore.acquire().await
            .map_err(|e| anyhow::anyhow!("Failed to acquire task permit: {}", e))?;

        let stats = Arc::clone(&self.stats);
        {
            let mut stats_guard = stats.lock().await;
            stats_guard.total_tasks += 1;
            stats_guard.active_tasks += 1;
        }

        let start_time = std::time::Instant::now();

        // Execute task
        let result = task().await;

        let execution_time = start_time.elapsed().as_millis() as f64;

        let mut stats_guard = stats.lock().await;
        stats_guard.active_tasks = stats_guard.active_tasks.saturating_sub(1);

        match &result {
            Ok(_) => {
                stats_guard.completed_tasks += 1;
            }
            Err(_) => {
                stats_guard.failed_tasks += 1;
            }
        }

        // Update average execution time
        stats_guard.average_execution_time_ms =
            (stats_guard.average_execution_time_ms + execution_time) / 2.0;

        drop(permit);

        result
    }

    pub async fn get_stats(&self) -> SchedulerStats {
        self.stats.lock().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_connection_pool_basic() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new().map_err(|e| format!("Failed to create temp dir: {}", e))?;
        let db_path = temp_dir.path().join("test.db").to_str()
            .ok_or("Failed to convert path to string")?
            .to_string();

        let config = ConnectionPoolConfig {
            max_connections: 5,
            min_connections: 2,
            database_path: db_path,
            ..Default::default()
        };

        let pool = ConnectionPool::new(config).await
            .map_err(|e| format!("Failed to create connection pool: {}", e))?;

        // Test getting a connection
        let handle = pool.get_connection().await
            .map_err(|e| format!("Failed to get connection: {}", e))?;
        assert!(handle.connection.is_some());

        // Connection should be returned to pool when handle is dropped
        drop(handle);

        // Check stats
        let stats = pool.get_stats().await;
        assert_eq!(stats.total_connections, 2); // Min connections
        assert_eq!(stats.active_connections, 0); // Connection returned
        assert_eq!(stats.idle_connections, 2);
        Ok(())
    }

    #[tokio::test]
    async fn test_request_cache() -> Result<(), Box<dyn std::error::Error>> {
        let cache = RequestCache::new(10, Duration::from_secs(60));

        let key = "test_key".to_string();
        let data = b"test data".to_vec();
        let content_type = "application/json".to_string();

        // Set and get
        cache.set(key.clone(), data.clone(), content_type.clone()).await;
        let cached = cache.get(&key).await
            .ok_or("Failed to get cached item")?;

        assert_eq!(cached.data, data);
        assert_eq!(cached.content_type, content_type);
        assert!(!cached.is_expired());
        Ok(())
    }

    #[tokio::test]
    async fn test_async_scheduler() -> Result<(), Box<dyn std::error::Error>> {
        let scheduler = AsyncTaskScheduler::new(2);

        let result = scheduler.schedule_task(|| async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(42)
        }).await.map_err(|e| format!("Failed to schedule task: {}", e))?;

        assert_eq!(result, 42);

        let stats = scheduler.get_stats().await;
        assert_eq!(stats.total_tasks, 1);
        assert_eq!(stats.completed_tasks, 1);
        assert_eq!(stats.active_tasks, 0);
        Ok(())
    }
}</content>
</xai:function_call">backend/src/infrastructure/connection_pool.rs
