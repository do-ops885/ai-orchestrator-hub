//! Memory Pool Implementation for High-Performance Object Reuse
//!
//! This module provides object pooling capabilities to reduce memory allocation overhead
//! and improve performance through object reuse patterns.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Configuration for memory pool behavior
#[derive(Debug, Clone)]
pub struct MemoryPoolConfig {
    /// Initial pool size
    pub initial_size: usize,
    /// Maximum pool size
    pub max_size: usize,
    /// Minimum pool size to maintain
    pub min_size: usize,
    /// Enable pool size auto-adjustment
    pub auto_adjust: bool,
    /// Pool cleanup interval in seconds
    pub cleanup_interval_secs: u64,
    /// Maximum object age before forced cleanup (seconds)
    pub max_object_age_secs: u64,
}

impl Default for MemoryPoolConfig {
    fn default() -> Self {
        Self {
            initial_size: 100,
            max_size: 1000,
            min_size: 10,
            auto_adjust: true,
            cleanup_interval_secs: 300, // 5 minutes
            max_object_age_secs: 3600,  // 1 hour
        }
    }
}

/// Statistics for memory pool usage and performance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryPoolStats {
    /// Total objects created
    pub objects_created: u64,
    /// Total objects reused from pool
    pub objects_reused: u64,
    /// Current pool size
    pub current_pool_size: usize,
    /// Peak pool size ever reached
    pub peak_pool_size: usize,
    /// Current objects in use (checked out)
    pub objects_in_use: usize,
    /// Pool hit ratio (reused / (created + reused))
    pub hit_ratio: f64,
    /// Average object lifetime in milliseconds
    pub avg_object_lifetime_ms: f64,
    /// Memory saved through reuse (estimated bytes)
    pub memory_saved_bytes: u64,
}

/// Pool efficiency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolEfficiency {
    pub hit_ratio_percent: f64,
    pub utilization_percent: f64,
    pub memory_efficiency_score: f64,
}

/// Simple object pool for basic types
pub struct SimpleObjectPool<T> {
    pool: Arc<Mutex<VecDeque<T>>>,
    max_size: usize,
    factory: Box<dyn Fn() -> T + Send + Sync>,
    stats: Arc<Mutex<MemoryPoolStats>>,
}

impl<T> SimpleObjectPool<T>
where
    T: Send + 'static,
{
    /// Create a new simple object pool
    pub fn new<F>(factory: F, max_size: usize) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::with_capacity(max_size / 2))),
            max_size,
            factory: Box::new(factory),
            stats: Arc::new(Mutex::new(MemoryPoolStats::default())),
        }
    }

    /// Get an object from the pool
    pub async fn get(&self) -> T {
        let mut pool = self.pool.lock().await;
        let mut stats = self.stats.lock().await;

        if let Some(object) = pool.pop_front() {
            stats.objects_reused += 1;
            stats.current_pool_size = pool.len();
            stats.objects_in_use += 1;
            
            // Update hit ratio
            let total = stats.objects_created + stats.objects_reused;
            stats.hit_ratio = stats.objects_reused as f64 / total as f64;
            
            object
        } else {
            stats.objects_created += 1;
            stats.objects_in_use += 1;
            
            // Update hit ratio
            let total = stats.objects_created + stats.objects_reused;
            stats.hit_ratio = stats.objects_reused as f64 / total as f64;
            
            (self.factory)()
        }
    }

    /// Return an object to the pool
    pub async fn put(&self, object: T) {
        let mut pool = self.pool.lock().await;
        let mut stats = self.stats.lock().await;
        
        stats.objects_in_use = stats.objects_in_use.saturating_sub(1);
        
        if pool.len() < self.max_size {
            pool.push_back(object);
            stats.current_pool_size = pool.len();
            stats.peak_pool_size = stats.peak_pool_size.max(stats.current_pool_size);
        }
        // Object is dropped if pool is full
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> MemoryPoolStats {
        self.stats.lock().await.clone()
    }
}

/// Specialized pools for common types used in swarm communication
pub struct SwarmMemoryPools {
    /// String pool for text messages
    pub string_pool: SimpleObjectPool<String>,
    /// Vec<u8> pool for binary data
    pub byte_vec_pool: SimpleObjectPool<Vec<u8>>,
    /// Small vec pool for recipient lists
    pub uuid_vec_pool: SimpleObjectPool<Vec<uuid::Uuid>>,
}

impl SwarmMemoryPools {
    /// Create new swarm memory pools
    #[must_use] 
    pub fn new() -> Self {
        Self {
            string_pool: SimpleObjectPool::new(
                || String::with_capacity(1024),
                500, // Max 500 strings
            ),
            byte_vec_pool: SimpleObjectPool::new(
                || Vec::with_capacity(4096),
                200, // Max 200 byte vectors
            ),
            uuid_vec_pool: SimpleObjectPool::new(
                || Vec::with_capacity(16),
                100, // Max 100 UUID vectors
            ),
        }
    }

    /// Get comprehensive statistics
    pub async fn get_comprehensive_stats(&self) -> SwarmPoolStats {
        SwarmPoolStats {
            string_pool_stats: self.string_pool.get_stats().await,
            byte_vec_pool_stats: self.byte_vec_pool.get_stats().await,
            uuid_vec_pool_stats: self.uuid_vec_pool.get_stats().await,
        }
    }
}

impl Default for SwarmMemoryPools {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for swarm memory pools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmPoolStats {
    pub string_pool_stats: MemoryPoolStats,
    pub byte_vec_pool_stats: MemoryPoolStats,
    pub uuid_vec_pool_stats: MemoryPoolStats,
}

impl SwarmPoolStats {
    /// Calculate overall memory efficiency
    #[must_use] 
    pub fn overall_efficiency(&self) -> f64 {
        let pools = [
            &self.string_pool_stats,
            &self.byte_vec_pool_stats,
            &self.uuid_vec_pool_stats,
        ];

        let avg_hit_ratio: f64 = pools.iter().map(|p| p.hit_ratio).sum::<f64>() / pools.len() as f64;
        avg_hit_ratio * 100.0
    }

    /// Calculate total memory saved
    #[must_use] 
    pub fn total_memory_saved_bytes(&self) -> u64 {
        self.string_pool_stats.memory_saved_bytes +
        self.byte_vec_pool_stats.memory_saved_bytes +
        self.uuid_vec_pool_stats.memory_saved_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_object_pool() {
        let pool = SimpleObjectPool::new(|| String::from("test"), 5);
        
        // Get object
        let obj1 = pool.get().await;
        assert_eq!(obj1, "test");
        
        // Return object
        pool.put(obj1).await;
        
        // Get object again - should be reused
        let obj2 = pool.get().await;
        
        let stats = pool.get_stats().await;
        assert_eq!(stats.objects_reused, 1);
        assert!(stats.hit_ratio > 0.0);
    }

    #[tokio::test]
    async fn test_swarm_memory_pools() {
        let pools = SwarmMemoryPools::new();
        
        // Test string pool
        let string1 = pools.string_pool.get().await;
        assert!(string1.capacity() >= 1024);
        
        // Test byte vec pool
        let vec1 = pools.byte_vec_pool.get().await;
        assert!(vec1.capacity() >= 4096);
        
        // Test UUID vec pool
        let uuid_vec1 = pools.uuid_vec_pool.get().await;
        assert!(uuid_vec1.capacity() >= 16);
        
        let stats = pools.get_comprehensive_stats().await;
        assert!(stats.overall_efficiency() >= 0.0);
    }
}