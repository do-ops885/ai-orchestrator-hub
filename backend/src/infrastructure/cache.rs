use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// Add this import if SystemMetrics is defined in another file in the infrastructure module
use crate::infrastructure::metrics::SystemMetrics;

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
    eviction_strategy: EvictionStrategy,
    size_tracker: Arc<RwLock<SizeTracker>>,
}

/// Eviction strategy for cache entries
#[derive(Debug, Clone)]
pub enum EvictionStrategy {
    /// Least Recently Used (traditional LRU)
    LRU,
    /// Least Frequently Used (LFU)
    LFU,
    /// Size-aware LRU (considers entry size)
    SizeAwareLRU,
    /// Adaptive strategy (combines multiple factors)
    Adaptive {
        lru_weight: f64,
        lfu_weight: f64,
        size_weight: f64,
    },
    /// Time-aware LRU (considers time since last access)
    TimeAwareLRU { time_decay_factor: f64 },
}

/// Size tracker for memory usage monitoring
#[derive(Debug, Default)]
struct SizeTracker {
    total_entries: usize,
    total_size_bytes: usize,
    average_entry_size: usize,
    size_distribution: BTreeMap<usize, usize>, // size -> count
}

#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
    access_count: u64,
    last_accessed: Instant,
    size_bytes: usize,
    access_frequency: f64, // accesses per second
    creation_time: Instant,
    priority_score: f64, // For adaptive eviction
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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
    #[must_use]
    pub fn new(default_ttl: Duration, max_size: usize) -> Self {
        Self::with_strategy(
            default_ttl,
            max_size,
            EvictionStrategy::Adaptive {
                lru_weight: 0.4,
                lfu_weight: 0.4,
                size_weight: 0.2,
            },
        )
    }

    /// Create a new cache with custom eviction strategy
    #[must_use]
    pub fn with_strategy(
        default_ttl: Duration,
        max_size: usize,
        strategy: EvictionStrategy,
    ) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
            max_size,
            eviction_strategy: strategy,
            size_tracker: Arc::new(RwLock::new(SizeTracker::default())),
        }
    }

    /// Insert a value with default TTL
    pub async fn insert(&self, key: K, value: V) {
        self.insert_with_ttl(key, value, self.default_ttl).await;
    }

    /// Insert a value with custom TTL
    pub async fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) {
        let mut data = self.data.write().await;

        // Calculate entry size (rough estimate)
        let size_bytes = self.calculate_entry_size(&key, &value);

        // Check if we need to evict entries
        if data.len() >= self.max_size {
            self.evict_entries(&mut data, size_bytes).await;
        }

        let now = Instant::now();
        let entry = CacheEntry {
            value,
            expires_at: now + ttl,
            access_count: 0,
            last_accessed: now,
            size_bytes,
            access_frequency: 0.0,
            creation_time: now,
            priority_score: 0.0,
        };

        // Update size tracker
        self.update_size_tracker(&entry, true).await;

        data.insert(key, entry);
    }

    /// Get a value from the cache
    pub async fn get(&self, key: &K) -> Option<V> {
        let mut data = self.data.write().await;

        if let Some(entry) = data.get_mut(key) {
            // Check if entry has expired
            if Instant::now() > entry.expires_at {
                self.update_size_tracker(entry, false).await;
                data.remove(key);
                return None;
            }

            // Update access statistics
            let now = Instant::now();
            entry.access_count += 1;
            entry.last_accessed = now;

            // Update access frequency
            let time_since_creation = now.duration_since(entry.creation_time).as_secs_f64();
            if time_since_creation > 0.0 {
                entry.access_frequency = entry.access_count as f64 / time_since_creation;
            }

            // Update priority score for adaptive eviction
            entry.priority_score = self.calculate_priority_score(entry);

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
        let total_misses = total_accesses.saturating_sub(total_hits);

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

    /// Evict entries based on the configured strategy
    async fn evict_entries(&self, data: &mut HashMap<K, CacheEntry<V>>, new_entry_size: usize) {
        let mut entries_to_evict = Vec::new();
        let mut space_needed = new_entry_size;

        // Calculate how much space we need to free
        let current_size = {
            let tracker = self.size_tracker.read().await;
            tracker.total_size_bytes
        };

        // This is a simplified calculation - in practice you'd have a max_memory limit
        let max_memory = self.max_size * 1024; // Assume 1KB per entry on average
        if current_size + space_needed <= max_memory {
            return; // No eviction needed
        }

        space_needed = max_memory.saturating_sub(current_size) + space_needed;

        // Find entries to evict based on strategy
        while space_needed > 0 && !data.is_empty() {
            let eviction_candidate = self.find_eviction_candidate(data).await;

            if let Some((key, entry)) = eviction_candidate {
                entries_to_evict.push((key, entry.size_bytes));
                space_needed = space_needed.saturating_sub(entry.size_bytes);
            } else {
                break; // No more entries to evict
            }
        }

        // Evict the selected entries
        for (key, _size) in entries_to_evict {
            if let Some(entry) = data.remove(&key) {
                self.update_size_tracker(&entry, false).await;
            }
        }
    }

    /// Find the best candidate for eviction based on strategy
    async fn find_eviction_candidate(
        &self,
        data: &HashMap<K, CacheEntry<V>>,
    ) -> Option<(K, CacheEntry<V>)> {
        match &self.eviction_strategy {
            EvictionStrategy::LRU => data
                .iter()
                .min_by_key(|(_, entry)| entry.last_accessed)
                .map(|(k, v)| (k.clone(), v.clone())),
            EvictionStrategy::LFU => data
                .iter()
                .min_by_key(|(_, entry)| entry.access_count)
                .map(|(k, v)| (k.clone(), v.clone())),
            EvictionStrategy::SizeAwareLRU => {
                data.iter()
                    .min_by(|(_, a), (_, b)| {
                        // Combine recency and size (prefer evicting large, old entries)
                        let score_a = (a.last_accessed.elapsed().as_secs() as f64 * 0.7)
                            + (a.size_bytes as f64 * 0.3);
                        let score_b = (b.last_accessed.elapsed().as_secs() as f64 * 0.7)
                            + (b.size_bytes as f64 * 0.3);
                        score_a
                            .partial_cmp(&score_b)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(k, v)| (k.clone(), v.clone()))
            }
            EvictionStrategy::Adaptive {
                lru_weight,
                lfu_weight,
                size_weight,
            } => data
                .iter()
                .min_by(|(_, a), (_, b)| {
                    let score_a =
                        self.calculate_adaptive_score(a, *lru_weight, *lfu_weight, *size_weight);
                    let score_b =
                        self.calculate_adaptive_score(b, *lru_weight, *lfu_weight, *size_weight);
                    score_a
                        .partial_cmp(&score_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(k, v)| (k.clone(), v.clone())),
            EvictionStrategy::TimeAwareLRU { time_decay_factor } => {
                let now = Instant::now();
                data.iter()
                    .min_by_key(|(_, entry)| {
                        let time_since_access =
                            now.duration_since(entry.last_accessed).as_secs_f64();
                        let decayed_access = entry.access_count as f64
                            * (-time_decay_factor * time_since_access).exp();
                        (decayed_access * 1000.0) as u64
                    })
                    .map(|(k, v)| (k.clone(), v.clone()))
            }
        }
    }

    /// Calculate adaptive eviction score
    fn calculate_adaptive_score(
        &self,
        entry: &CacheEntry<V>,
        lru_weight: f64,
        lfu_weight: f64,
        size_weight: f64,
    ) -> f64 {
        let lru_score = entry.last_accessed.elapsed().as_secs_f64();
        let lfu_score = 1.0 / (entry.access_count as f64 + 1.0); // Lower access count = higher score (more likely to evict)
        let size_score = entry.size_bytes as f64;

        (lru_score * lru_weight) + (lfu_score * lfu_weight) + (size_score * size_weight)
    }

    /// Calculate priority score for adaptive eviction
    fn calculate_priority_score(&self, entry: &CacheEntry<V>) -> f64 {
        // Higher score = higher priority (less likely to evict)
        let recency_score = 1.0 / (entry.last_accessed.elapsed().as_secs_f64() + 1.0);
        let frequency_score = entry.access_frequency.min(10.0) / 10.0; // Normalize to 0-1
        let size_penalty = 1.0 / (entry.size_bytes as f64 + 1.0).log2(); // Penalize large entries

        (recency_score * 0.5) + (frequency_score * 0.3) + (size_penalty * 0.2)
    }

    /// Calculate approximate size of an entry
    fn calculate_entry_size(&self, key: &K, value: &V) -> usize {
        // Rough estimation - in practice you'd use more sophisticated sizing
        let key_size = std::mem::size_of_val(key);
        let value_size = std::mem::size_of_val(value);
        key_size + value_size + 128 // Overhead for HashMap entry
    }

    /// Update size tracker
    async fn update_size_tracker(&self, entry: &CacheEntry<V>, adding: bool) {
        let mut tracker = self.size_tracker.write().await;

        if adding {
            tracker.total_entries += 1;
            tracker.total_size_bytes += entry.size_bytes;
            *tracker
                .size_distribution
                .entry(entry.size_bytes)
                .or_insert(0) += 1;
        } else {
            tracker.total_entries = tracker.total_entries.saturating_sub(1);
            tracker.total_size_bytes = tracker.total_size_bytes.saturating_sub(entry.size_bytes);
            if let Some(count) = tracker.size_distribution.get_mut(&entry.size_bytes) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    tracker.size_distribution.remove(&entry.size_bytes);
                }
            }
        }

        // Update average entry size
        if tracker.total_entries > 0 {
            tracker.average_entry_size = tracker.total_size_bytes / tracker.total_entries;
        } else {
            tracker.average_entry_size = 0;
        }
    }

    /// Get cache size statistics
    pub async fn get_size_stats(&self) -> SizeStats {
        let tracker = self.size_tracker.read().await;
        let data = self.data.read().await;

        SizeStats {
            total_entries: data.len(),
            total_size_bytes: tracker.total_size_bytes,
            average_entry_size: tracker.average_entry_size,
            size_distribution: tracker.size_distribution.clone(),
            eviction_strategy: self.eviction_strategy.clone(),
        }
    }
}

/// Specialized cache for agent data
pub type AgentCache = Cache<uuid::Uuid, crate::agents::Agent>;

/// Specialized cache for task data
pub type TaskCache = Cache<uuid::Uuid, crate::tasks::Task>;

/// Specialized cache for hive status
pub type StatusCache = Cache<String, serde_json::Value>;

/// Cache size statistics
#[derive(Debug, Clone)]
pub struct SizeStats {
    pub total_entries: usize,
    pub total_size_bytes: usize,
    pub average_entry_size: usize,
    pub size_distribution: std::collections::BTreeMap<usize, usize>,
    pub eviction_strategy: EvictionStrategy,
}

/// High-performance cache for frequently accessed data
pub type PerformanceCache = Cache<String, Vec<u8>>;

/// Cache manager for the hive system
pub struct CacheManager {
    pub agents: AgentCache,
    pub tasks: TaskCache,
    pub status: StatusCache,
    pub metrics: Cache<String, SystemMetrics>,
    pub performance_cache: PerformanceCache,
}

impl CacheManager {
    /// Create a new cache manager with optimized settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            agents: Cache::new(Duration::from_secs(300), 1000), // 5 min TTL, 1000 agents max
            tasks: Cache::new(Duration::from_secs(600), 5000),  // 10 min TTL, 5000 tasks max
            status: Cache::new(Duration::from_secs(30), 100),   // 30 sec TTL, 100 status entries
            metrics: Cache::new(Duration::from_secs(60), 1000), // 1 min TTL, 1000 metrics
            performance_cache: Cache::new(Duration::from_secs(180), 10000), // 3 min TTL, 10k entries
        }
    }

    /// Start background cleanup task
    pub fn start_cleanup_task(self: Arc<Self>) {
        let cache_manager = Arc::clone(&self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;

                // Clean up expired entries in all caches
                cache_manager.agents.cleanup_expired().await;
                cache_manager.tasks.cleanup_expired().await;
                cache_manager.status.cleanup_expired().await;
                cache_manager.metrics.cleanup_expired().await;
                cache_manager.performance_cache.cleanup_expired().await;

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
        let performance_stats = self.performance_cache.stats().await;

        serde_json::json!({
            "agents": agent_stats,
            "tasks": task_stats,
            "status": status_stats,
            "metrics": metrics_stats,
            "performance": performance_stats,
            "total_memory_estimate":
                agent_stats.memory_usage_estimate +
                task_stats.memory_usage_estimate +
                status_stats.memory_usage_estimate +
                metrics_stats.memory_usage_estimate +
                performance_stats.memory_usage_estimate
        })
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}
