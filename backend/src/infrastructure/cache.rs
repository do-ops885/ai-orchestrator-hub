use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// High-performance in-memory cache with TTL support
#[derive(Debug)]
pub struct Cache<K, V> 
where
    K: Clone + Eq + Hash + Send + Sync,
    V: Clone + Send + Sync,
{
    data: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    default_ttl: Duration,
    max_size: usize,
}

#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
    access_count: u64,
    last_accessed: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub total_hits: u64,
    pub total_misses: u64,
    pub evictions: u64,
    pub memory_usage_estimate: usize,
}

impl<K, V> Cache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync,
    V: Clone + Send + Sync,
{
    /// Create a new cache with default TTL and maximum size
    pub fn new(default_ttl: Duration, max_size: usize) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
            max_size,
        }
    }

    /// Insert a value with default TTL
    pub async fn insert(&self, key: K, value: V) {
        self.insert_with_ttl(key, value, self.default_ttl).await;
    }

    /// Insert a value with custom TTL
    pub async fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) {
        let mut data = self.data.write().await;
        
        // Check if we need to evict entries
        if data.len() >= self.max_size {
            self.evict_lru(&mut data).await;
        }

        let entry = CacheEntry {
            value,
            expires_at: Instant::now() + ttl,
            access_count: 0,
            last_accessed: Instant::now(),
        };

        data.insert(key, entry);
    }

    /// Get a value from the cache
    pub async fn get(&self, key: &K) -> Option<V> {
        let mut data = self.data.write().await;
        
        if let Some(entry) = data.get_mut(key) {
            // Check if entry has expired
            if Instant::now() > entry.expires_at {
                data.remove(key);
                return None;
            }

            // Update access statistics
            entry.access_count += 1;
            entry.last_accessed = Instant::now();
            
            Some(entry.value.clone())
        } else {
            None
        }
    }

    /// Remove a value from the cache
    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut data = self.data.write().await;
        data.remove(key).map(|entry| entry.value)
    }

    /// Clear all entries from the cache
    pub async fn clear(&self) {
        let mut data = self.data.write().await;
        data.clear();
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let data = self.data.read().await;
        
        let total_entries = data.len();
        let total_hits: u64 = data.values().map(|entry| entry.access_count).sum();
        let total_accesses = total_hits; // Simplified for demo
        let total_misses = if total_accesses > total_hits { total_accesses - total_hits } else { 0 };
        
        let hit_rate = if total_accesses > 0 {
            total_hits as f64 / total_accesses as f64
        } else {
            0.0
        };

        CacheStats {
            total_entries,
            hit_rate,
            miss_rate: 1.0 - hit_rate,
            total_hits,
            total_misses,
            evictions: 0, // Would track this in a real implementation
            memory_usage_estimate: total_entries * std::mem::size_of::<CacheEntry<V>>(),
        }
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) {
        let mut data = self.data.write().await;
        let now = Instant::now();
        
        data.retain(|_, entry| now <= entry.expires_at);
    }

    /// Evict least recently used entry
    async fn evict_lru(&self, data: &mut HashMap<K, CacheEntry<V>>) {
        if let Some((lru_key, _)) = data
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            data.remove(&lru_key);
        }
    }
}

/// Specialized cache for agent data
pub type AgentCache = Cache<uuid::Uuid, crate::agents::Agent>;

/// Specialized cache for task data  
pub type TaskCache = Cache<uuid::Uuid, crate::tasks::Task>;

/// Specialized cache for hive status
pub type StatusCache = Cache<String, serde_json::Value>;

/// Cache manager for the hive system
pub struct CacheManager {
    pub agents: AgentCache,
    pub tasks: TaskCache,
    pub status: StatusCache,
    pub metrics: Cache<String, crate::infrastructure::SystemMetrics>,
}

impl CacheManager {
    /// Create a new cache manager with optimized settings
    pub fn new() -> Self {
        Self {
            agents: Cache::new(Duration::from_secs(300), 1000), // 5 min TTL, 1000 agents max
            tasks: Cache::new(Duration::from_secs(600), 5000),  // 10 min TTL, 5000 tasks max
            status: Cache::new(Duration::from_secs(30), 100),   // 30 sec TTL, 100 status entries
            metrics: Cache::new(Duration::from_secs(60), 1000), // 1 min TTL, 1000 metrics
        }
    }

    /// Start background cleanup task
    pub fn start_cleanup_task(self: Arc<Self>) {
        let cache_manager = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                
                // Clean up expired entries in all caches
                cache_manager.agents.cleanup_expired().await;
                cache_manager.tasks.cleanup_expired().await;
                cache_manager.status.cleanup_expired().await;
                cache_manager.metrics.cleanup_expired().await;
                
                tracing::debug!("Cache cleanup completed");
            }
        });
    }

    /// Get comprehensive cache statistics
    pub async fn get_stats(&self) -> serde_json::Value {
        let agent_stats = self.agents.stats().await;
        let task_stats = self.tasks.stats().await;
        let status_stats = self.status.stats().await;
        let metrics_stats = self.metrics.stats().await;

        serde_json::json!({
            "agents": agent_stats,
            "tasks": task_stats,
            "status": status_stats,
            "metrics": metrics_stats,
            "total_memory_estimate": 
                agent_stats.memory_usage_estimate +
                task_stats.memory_usage_estimate +
                status_stats.memory_usage_estimate +
                metrics_stats.memory_usage_estimate
        })
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}