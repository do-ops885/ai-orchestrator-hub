use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};

/// Cache invalidation events for event-driven invalidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheInvalidationEvent {
    /// Invalidate by specific key
    KeyInvalidation { key: String },
    /// Invalidate by tag pattern
    TagInvalidation { tag: String },
    /// Invalidate by pattern matching
    PatternInvalidation { pattern: String },
    /// Invalidate all entries
    FullInvalidation,
    /// Invalidate entries older than specified duration
    TimeBasedInvalidation { max_age_seconds: u64 },
    /// Invalidate entries with low access frequency
    FrequencyBasedInvalidation { min_frequency: f64 },
}

/// Cache entry with tags and metadata for advanced invalidation
#[derive(Debug, Clone)]
pub struct TaggedCachedResponse {
    pub data: Value,
    pub timestamp: Instant,
    pub hits: u32,
    pub tags: HashSet<String>,
    pub access_frequency: f64,
    pub last_access: Instant,
    pub size_bytes: usize,
}

/// Serializable version of cache statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableCacheStats {
    pub total_entries: usize,
    pub total_hits: u64,
    pub total_misses: u64,
    pub hit_rate: f64,
    pub memory_usage_bytes: usize,
    pub average_entry_size: usize,
    pub entries_by_tag: HashMap<String, usize>,
    pub invalidation_events: u64,
    pub uptime_seconds: u64,
}

/// Cache statistics for monitoring and optimization
#[derive(Debug, Clone)]
pub struct EnhancedCacheStats {
    pub total_entries: usize,
    pub total_hits: u64,
    pub total_misses: u64,
    pub hit_rate: f64,
    pub memory_usage_bytes: usize,
    pub average_entry_size: usize,
    pub entries_by_tag: HashMap<String, usize>,
    pub invalidation_events: u64,
    pub last_cleanup: Option<Instant>,
    pub uptime_seconds: u64,
}

/// Legacy structure for backward compatibility
#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub data: Value,
    pub timestamp: Instant,
    pub hits: u32,
}

/// Enhanced MCP Cache with Event-Driven Invalidation and Tag-Based Management (Phase 2)
#[derive(Debug)]
pub struct MCPCache {
    /// Legacy cache for backward compatibility
    legacy_responses: Arc<RwLock<HashMap<String, CachedResponse>>>,
    /// Enhanced cache with tags and metadata
    responses: Arc<RwLock<HashMap<String, TaggedCachedResponse>>>,
    /// Tag to key mapping for efficient tag-based invalidation
    tag_index: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// Event broadcaster for cache invalidation events
    event_sender: broadcast::Sender<CacheInvalidationEvent>,
    /// Cache statistics
    stats: Arc<RwLock<EnhancedCacheStats>>,
    /// Default TTL for cache entries
    ttl: Duration,
    /// Maximum memory usage in bytes
    max_memory_bytes: usize,
    /// Cache start time for uptime tracking
    start_time: Instant,
}

impl MCPCache {
    /// Create a new enhanced MCP cache with event-driven invalidation
    #[must_use]
    pub fn new(ttl_seconds: u64) -> Self {
        let (event_sender, _) = broadcast::channel(100);

        Self {
            legacy_responses: Arc::new(RwLock::new(HashMap::new())),
            responses: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            stats: Arc::new(RwLock::new(EnhancedCacheStats {
                total_entries: 0,
                total_hits: 0,
                total_misses: 0,
                hit_rate: 0.0,
                memory_usage_bytes: 0,
                average_entry_size: 0,
                entries_by_tag: HashMap::new(),
                invalidation_events: 0,
                last_cleanup: None,
                uptime_seconds: 0,
            })),
            ttl: Duration::from_secs(ttl_seconds),
            max_memory_bytes: 100 * 1024 * 1024, // 100MB default
            start_time: Instant::now(),
        }
    }

    /// Create cache with custom memory limit
    #[must_use]
    pub fn with_memory_limit(ttl_seconds: u64, max_memory_bytes: usize) -> Self {
        let mut cache = Self::new(ttl_seconds);
        cache.max_memory_bytes = max_memory_bytes;
        cache
    }

    /// Get cached response or compute new one (legacy method for backward compatibility)
    pub async fn get_or_compute<F, Fut>(&self, key: &str, compute: F) -> Result<Value>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<Value>>,
    {
        self.get_or_compute_with_tags(key, compute, HashSet::new())
            .await
    }

    /// Get cached response or compute new one with tags
    pub async fn get_or_compute_with_tags<F, Fut>(
        &self,
        key: &str,
        compute: F,
        tags: HashSet<String>,
    ) -> Result<Value>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<Value>>,
    {
        // Check enhanced cache first
        {
            let mut cache = self.responses.write().await;
            if let Some(cached) = cache.get_mut(key) {
                if cached.timestamp.elapsed() < self.ttl {
                    cached.hits += 1;
                    cached.last_access = Instant::now();
                    cached.access_frequency = self.calculate_access_frequency(cached);

                    // Update stats
                    let mut stats = self.stats.write().await;
                    stats.total_hits += 1;

                    debug!("Cache hit for key: {} (hits: {})", key, cached.hits);
                    return Ok(cached.data.clone());
                }
                // Remove expired entry
                self.remove_entry_from_tag_index(key, &cached.tags).await;
                cache.remove(key);
                debug!("Cache entry expired for key: {}", key);
            }
        }

        // Check legacy cache for backward compatibility
        {
            let mut legacy_cache = self.legacy_responses.write().await;
            if let Some(cached) = legacy_cache.get_mut(key) {
                if cached.timestamp.elapsed() < self.ttl {
                    cached.hits += 1;

                    // Update stats
                    let mut stats = self.stats.write().await;
                    stats.total_hits += 1;

                    debug!("Legacy cache hit for key: {} (hits: {})", key, cached.hits);
                    return Ok(cached.data.clone());
                }
                // Remove expired entry
                legacy_cache.remove(key);
                debug!("Legacy cache entry expired for key: {}", key);
            }
        }

        // Compute new value
        debug!("Cache miss for key: {}, computing new value", key);
        let result = compute().await?;

        // Store in enhanced cache
        let size_bytes = self.calculate_size(&result);
        let cached = TaggedCachedResponse {
            data: result.clone(),
            timestamp: Instant::now(),
            hits: 1,
            tags: tags.clone(),
            access_frequency: 0.0,
            last_access: Instant::now(),
            size_bytes,
        };

        // Check memory limits before storing
        self.enforce_memory_limits().await;

        // Update tag index
        self.add_entry_to_tag_index(key, &tags).await;

        self.responses.write().await.insert(key.to_string(), cached);

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_misses += 1;
        stats.total_entries += 1;
        stats.memory_usage_bytes += size_bytes;

        // Update tag statistics
        for tag in &tags {
            *stats.entries_by_tag.entry(tag.clone()).or_insert(0) += 1;
        }

        debug!("Cached new value for key: {} with tags: {:?}", key, tags);
        Ok(result)
    }

    /// Invalidate cache entry (legacy method)
    pub async fn invalidate(&self, key: &str) {
        self.invalidate_key(key).await;
    }

    /// Invalidate cache entry with event broadcasting
    pub async fn invalidate_key(&self, key: &str) {
        // Remove from enhanced cache
        let removed_entry = {
            let mut cache = self.responses.write().await;
            cache.remove(key)
        };

        if let Some(entry) = removed_entry {
            // Update tag index
            self.remove_entry_from_tag_index(key, &entry.tags).await;

            // Update stats
            let mut stats = self.stats.write().await;
            stats.total_entries = stats.total_entries.saturating_sub(1);
            stats.memory_usage_bytes = stats.memory_usage_bytes.saturating_sub(entry.size_bytes);
            stats.invalidation_events += 1;

            // Update tag statistics
            for tag in &entry.tags {
                if let Some(count) = stats.entries_by_tag.get_mut(tag) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        stats.entries_by_tag.remove(tag);
                    }
                }
            }
        }

        // Also remove from legacy cache
        self.legacy_responses.write().await.remove(key);

        // Broadcast invalidation event
        let event = CacheInvalidationEvent::KeyInvalidation {
            key: key.to_string(),
        };
        let _ = self.event_sender.send(event);

        debug!("Invalidated cache entry for key: {}", key);
    }

    /// Invalidate entries by tag
    pub async fn invalidate_by_tag(&self, tag: &str) -> usize {
        let keys_to_invalidate = {
            let tag_index = self.tag_index.read().await;
            tag_index.get(tag).cloned().unwrap_or_default()
        };

        let mut invalidated_count = 0;
        for key in &keys_to_invalidate {
            self.invalidate_key(key).await;
            invalidated_count += 1;
        }

        // Broadcast invalidation event
        let event = CacheInvalidationEvent::TagInvalidation {
            tag: tag.to_string(),
        };
        let _ = self.event_sender.send(event);

        info!(
            "Invalidated {} entries with tag: {}",
            invalidated_count, tag
        );
        invalidated_count
    }

    /// Invalidate entries by pattern
    pub async fn invalidate_by_pattern(&self, pattern: &str) -> usize {
        let keys_to_invalidate = {
            let cache = self.responses.read().await;
            cache
                .keys()
                .filter(|key| key.contains(pattern))
                .cloned()
                .collect::<Vec<_>>()
        };

        let mut invalidated_count = 0;
        for key in &keys_to_invalidate {
            self.invalidate_key(key).await;
            invalidated_count += 1;
        }

        // Broadcast invalidation event
        let event = CacheInvalidationEvent::PatternInvalidation {
            pattern: pattern.to_string(),
        };
        let _ = self.event_sender.send(event);

        info!(
            "Invalidated {} entries matching pattern: {}",
            invalidated_count, pattern
        );
        invalidated_count
    }

    /// Invalidate all entries
    pub async fn invalidate_all(&self) -> usize {
        let keys_to_invalidate = {
            let cache = self.responses.read().await;
            cache.keys().cloned().collect::<Vec<_>>()
        };

        let invalidated_count = keys_to_invalidate.len();
        for key in &keys_to_invalidate {
            self.invalidate_key(key).await;
        }

        // Clear legacy cache
        self.legacy_responses.write().await.clear();

        // Clear tag index
        self.tag_index.write().await.clear();

        // Broadcast invalidation event
        let event = CacheInvalidationEvent::FullInvalidation;
        let _ = self.event_sender.send(event);

        info!("Invalidated all {} cache entries", invalidated_count);
        invalidated_count
    }

    /// Subscribe to cache invalidation events
    pub fn subscribe_events(&self) -> broadcast::Receiver<CacheInvalidationEvent> {
        self.event_sender.subscribe()
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        self.responses.write().await.clear();
        debug!("Cleared all cache entries");
    }

    /// Get enhanced cache statistics
    pub async fn enhanced_stats(&self) -> EnhancedCacheStats {
        let mut stats = self.stats.read().await.clone();

        // Update uptime
        stats.uptime_seconds = self.start_time.elapsed().as_secs();

        // Calculate hit rate
        let total_accesses = stats.total_hits + stats.total_misses;
        stats.hit_rate = if total_accesses > 0 {
            stats.total_hits as f64 / total_accesses as f64
        } else {
            0.0
        };

        // Calculate average entry size
        if stats.total_entries > 0 {
            stats.average_entry_size = stats.memory_usage_bytes / stats.total_entries;
        }

        stats
    }

    /// Get serializable cache statistics for external monitoring
    pub async fn serializable_stats(&self) -> SerializableCacheStats {
        let stats = self.enhanced_stats().await;
        SerializableCacheStats {
            total_entries: stats.total_entries,
            total_hits: stats.total_hits,
            total_misses: stats.total_misses,
            hit_rate: stats.hit_rate,
            memory_usage_bytes: stats.memory_usage_bytes,
            average_entry_size: stats.average_entry_size,
            entries_by_tag: stats.entries_by_tag,
            invalidation_events: stats.invalidation_events,
            uptime_seconds: stats.uptime_seconds,
        }
    }

    /// Get cache statistics (legacy method for backward compatibility)
    pub async fn stats(&self) -> CacheStats {
        let enhanced = self.enhanced_stats().await;
        let cache = self.responses.read().await;
        let expired_count = cache
            .values()
            .filter(|v| v.timestamp.elapsed() >= self.ttl)
            .count();

        CacheStats {
            total_entries: enhanced.total_entries,
            total_hits: enhanced.total_hits as u32,
            expired_count,
            ttl_seconds: self.ttl.as_secs(),
        }
    }

    /// Clean up expired entries with enhanced tracking
    pub async fn cleanup_expired(&self) {
        let mut cache = self.responses.write().await;
        let mut tag_index = self.tag_index.write().await;
        let mut stats = self.stats.write().await;

        let before_count = cache.len();
        let mut removed_memory = 0usize;

        cache.retain(|key, entry| {
            if entry.timestamp.elapsed() >= self.ttl {
                // Remove from tag index
                for tag in &entry.tags {
                    if let Some(keys) = tag_index.get_mut(tag) {
                        keys.remove(key);
                        if keys.is_empty() {
                            tag_index.remove(tag);
                        }
                    }
                }

                // Update stats
                removed_memory += entry.size_bytes;
                for tag in &entry.tags {
                    if let Some(count) = stats.entries_by_tag.get_mut(tag) {
                        *count = count.saturating_sub(1);
                        if *count == 0 {
                            stats.entries_by_tag.remove(tag);
                        }
                    }
                }

                false // Remove this entry
            } else {
                true // Keep this entry
            }
        });

        let after_count = cache.len();
        let removed = before_count - after_count;

        stats.total_entries = stats.total_entries.saturating_sub(removed);
        stats.memory_usage_bytes = stats.memory_usage_bytes.saturating_sub(removed_memory);
        stats.last_cleanup = Some(Instant::now());

        if removed > 0 {
            debug!(
                "Cleaned up {} expired cache entries, freed {} bytes",
                removed, removed_memory
            );
        }
    }

    /// Add entry to tag index
    async fn add_entry_to_tag_index(&self, key: &str, tags: &HashSet<String>) {
        let mut tag_index = self.tag_index.write().await;
        for tag in tags {
            tag_index
                .entry(tag.clone())
                .or_default()
                .insert(key.to_string());
        }
    }

    /// Remove entry from tag index
    async fn remove_entry_from_tag_index(&self, key: &str, tags: &HashSet<String>) {
        let mut tag_index = self.tag_index.write().await;
        for tag in tags {
            if let Some(keys) = tag_index.get_mut(tag) {
                keys.remove(key);
                if keys.is_empty() {
                    tag_index.remove(tag);
                }
            }
        }
    }

    /// Enforce memory limits by evicting least recently used entries
    async fn enforce_memory_limits(&self) {
        let current_memory = {
            let stats = self.stats.read().await;
            stats.memory_usage_bytes
        };

        if current_memory >= self.max_memory_bytes {
            warn!(
                "Cache memory limit exceeded ({} bytes), evicting entries",
                current_memory
            );

            let mut cache = self.responses.write().await;
            let mut tag_index = self.tag_index.write().await;
            let mut stats = self.stats.write().await;

            // Sort entries by last access time (oldest first)
            let mut entries_to_evict: Vec<_> = cache
                .iter()
                .map(|(key, entry)| (key.clone(), entry.last_access))
                .collect();

            entries_to_evict.sort_by_key(|(_, last_access)| *last_access);

            let mut evicted_memory = 0usize;
            let mut evicted_count = 0usize;

            for (key, _) in entries_to_evict {
                if let Some(entry) = cache.remove(&key) {
                    // Remove from tag index
                    for tag in &entry.tags {
                        if let Some(keys) = tag_index.get_mut(tag) {
                            keys.remove(&key);
                            if keys.is_empty() {
                                tag_index.remove(tag);
                            }
                        }
                    }

                    // Update stats
                    evicted_memory += entry.size_bytes;
                    evicted_count += 1;

                    for tag in &entry.tags {
                        if let Some(count) = stats.entries_by_tag.get_mut(tag) {
                            *count = count.saturating_sub(1);
                            if *count == 0 {
                                stats.entries_by_tag.remove(tag);
                            }
                        }
                    }

                    // Check if we've freed enough memory
                    if current_memory - evicted_memory <= self.max_memory_bytes * 9 / 10 {
                        break; // Keep 10% buffer
                    }
                }
            }

            stats.total_entries = stats.total_entries.saturating_sub(evicted_count);
            stats.memory_usage_bytes = stats.memory_usage_bytes.saturating_sub(evicted_memory);
            stats.invalidation_events += evicted_count as u64;

            info!(
                "Evicted {} entries to free {} bytes of memory",
                evicted_count, evicted_memory
            );
        }
    }

    /// Calculate approximate size of a value in bytes
    fn calculate_size(&self, value: &Value) -> usize {
        // Rough estimation based on JSON string length
        serde_json::to_string(value).map(|s| s.len()).unwrap_or(128)
    }

    /// Calculate access frequency for an entry
    fn calculate_access_frequency(&self, entry: &TaggedCachedResponse) -> f64 {
        let age_seconds = entry.timestamp.elapsed().as_secs_f64();
        if age_seconds > 0.0 {
            entry.hits as f64 / age_seconds
        } else {
            0.0
        }
    }

    /// Get entries by tag for monitoring
    pub async fn get_entries_by_tag(&self, tag: &str) -> Vec<String> {
        let tag_index = self.tag_index.read().await;
        tag_index
            .get(tag)
            .map(|keys| keys.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all tags currently in use
    pub async fn get_all_tags(&self) -> Vec<String> {
        let tag_index = self.tag_index.read().await;
        tag_index.keys().cloned().collect()
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_hits: u32,
    pub expired_count: usize,
    pub ttl_seconds: u64,
}

/// Generate cache key for MCP requests
#[must_use]
pub fn generate_cache_key(method: &str, params: &Option<Value>) -> String {
    match params {
        Some(p) => format!(
            "{}:{}",
            method,
            serde_json::to_string(p).unwrap_or_default()
        ),
        None => method.to_string(),
    }
}

/// MCP Cache Invalidation Manager for event-driven cache management
#[derive(Clone)]
pub struct MCPCacheInvalidationManager {
    cache: Arc<MCPCache>,
    invalidation_rules: Arc<RwLock<Vec<InvalidationRule>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidationRule {
    pub pattern: String,
    pub tags: Vec<String>,
    pub ttl_override: Option<Duration>,
    pub priority: u8,
}

impl MCPCacheInvalidationManager {
    /// Create a new cache invalidation manager
    pub fn new(cache: Arc<MCPCache>) -> Self {
        Self {
            cache,
            invalidation_rules: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add an invalidation rule
    pub async fn add_rule(&self, rule: InvalidationRule) {
        let mut rules = self.invalidation_rules.write().await;
        rules.push(rule);
        rules.sort_by(|a, b| b.priority.cmp(&a.priority)); // Higher priority first
    }

    /// Process invalidation events
    pub async fn process_events(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut receiver = self.cache.subscribe_events();
        while let Ok(event) = receiver.try_recv() {
            self.handle_event(event).await?;
        }
        Ok(())
    }

    /// Handle a single invalidation event
    async fn handle_event(
        &self,
        event: CacheInvalidationEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match event {
            CacheInvalidationEvent::KeyInvalidation { key } => {
                info!("Processing key invalidation event for: {}", key);
                // Apply rules that match this key
                self.apply_rules_for_key(&key).await;
            }
            CacheInvalidationEvent::TagInvalidation { tag } => {
                info!("Processing tag invalidation event for: {}", tag);
                // Additional logic for tag-based rules can be added here
            }
            CacheInvalidationEvent::PatternInvalidation { pattern } => {
                info!("Processing pattern invalidation event for: {}", pattern);
                // Additional logic for pattern-based rules can be added here
            }
            CacheInvalidationEvent::FullInvalidation => {
                info!("Processing full cache invalidation event");
                // Additional logic for full invalidation can be added here
            }
            _ => {
                debug!("Received unhandled cache invalidation event: {:?}", event);
            }
        }
        Ok(())
    }

    /// Apply invalidation rules for a specific key
    async fn apply_rules_for_key(&self, key: &str) {
        let rules = self.invalidation_rules.read().await;

        for rule in &*rules {
            if key.contains(&rule.pattern) {
                // Apply TTL override if specified
                if let Some(ttl_override) = rule.ttl_override {
                    // This would require modifying the cache entry's TTL
                    // For now, we'll just log it
                    debug!(
                        "TTL override rule applied for key {}: {:?}",
                        key, ttl_override
                    );
                }

                // Additional rule processing can be added here
            }
        }
    }

    /// Get current invalidation rules
    pub async fn get_rules(&self) -> Vec<InvalidationRule> {
        self.invalidation_rules.read().await.clone()
    }

    /// Start background event processing
    pub fn start_background_processing(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let manager = Arc::clone(&self);
            let mut interval = tokio::time::interval(Duration::from_millis(100));

            loop {
                interval.tick().await;
                if let Err(e) = manager.process_events().await {
                    warn!("Error processing cache invalidation events: {}", e);
                }
            }
        })
    }
}

/// Cache warming strategies for proactive cache population
pub struct MCPCacheWarmer {
    cache: Arc<MCPCache>,
    warming_strategies: Arc<RwLock<Vec<CacheWarmingStrategy>>>,
}

#[derive(Debug, Clone)]
pub enum CacheWarmingStrategy {
    /// Warm cache with frequently accessed keys
    FrequentAccess { keys: Vec<String>, priority: u8 },
    /// Warm cache with predicted access patterns
    Predictive {
        patterns: Vec<String>,
        confidence_threshold: f64,
    },
    /// Warm cache with static data
    Static {
        key_value_pairs: Vec<(String, Value, HashSet<String>)>,
    },
}

impl MCPCacheWarmer {
    pub fn new(cache: Arc<MCPCache>) -> Self {
        Self {
            cache,
            warming_strategies: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a warming strategy
    pub async fn add_strategy(&self, strategy: CacheWarmingStrategy) {
        let mut strategies = self.warming_strategies.write().await;
        strategies.push(strategy);
        strategies.sort_by(|a, b| {
            let priority_a = match a {
                CacheWarmingStrategy::FrequentAccess { priority, .. } => *priority,
                CacheWarmingStrategy::Predictive { .. } => 5,
                CacheWarmingStrategy::Static { .. } => 1,
            };
            let priority_b = match b {
                CacheWarmingStrategy::FrequentAccess { priority, .. } => *priority,
                CacheWarmingStrategy::Predictive { .. } => 5,
                CacheWarmingStrategy::Static { .. } => 1,
            };
            priority_b.cmp(&priority_a) // Higher priority first
        });
    }

    /// Execute cache warming
    pub async fn warm_cache(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let strategies = self.warming_strategies.read().await.clone();

        for strategy in strategies {
            match strategy {
                CacheWarmingStrategy::FrequentAccess { keys, .. } => {
                    for key in keys {
                        // Check if key exists, if not, it will be computed on demand
                        let _ = self
                            .cache
                            .get_or_compute(&key, || async {
                                // This would need to be implemented with actual computation logic
                                Ok(Value::Null)
                            })
                            .await;
                    }
                }
                CacheWarmingStrategy::Predictive { patterns, .. } => {
                    // Implement predictive warming based on patterns
                    for pattern in patterns {
                        debug!("Predictive warming for pattern: {}", pattern);
                        // This would analyze access patterns and warm accordingly
                    }
                }
                CacheWarmingStrategy::Static { key_value_pairs } => {
                    for (key, value, tags) in key_value_pairs {
                        let _ = self
                            .cache
                            .get_or_compute_with_tags(&key, || async { Ok(value.clone()) }, tags)
                            .await;
                    }
                }
            }
        }

        info!("Cache warming completed");
        Ok(())
    }

    /// Start periodic cache warming
    pub fn start_periodic_warming(
        self: Arc<Self>,
        interval: Duration,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                if let Err(e) = self.warm_cache().await {
                    warn!("Error during periodic cache warming: {}", e);
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashSet;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_cache_hit_miss() {
        let cache = MCPCache::new(5); // 5 second TTL
        let key = "test_key";

        // First call should be a miss
        let result1 = cache
            .get_or_compute(key, || async { Ok(json!({"value": "computed"})) })
            .await
            .expect("replaced unwrap");

        assert_eq!(result1, json!({"value": "computed"}));

        // Second call should be a hit
        let result2 = cache
            .get_or_compute(key, || async { Ok(json!({"value": "different"})) })
            .await
            .expect("replaced unwrap");

        assert_eq!(result2, json!({"value": "computed"})); // Should return cached value
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = MCPCache::new(1); // 1 second TTL
        let key = "test_key";

        // Store value
        let _result1 = cache
            .get_or_compute(key, || async { Ok(json!({"value": "first"})) })
            .await
            .expect("replaced unwrap");

        // Wait for expiration
        sleep(Duration::from_millis(1100)).await;

        // Should compute new value
        let result2 = cache
            .get_or_compute(key, || async { Ok(json!({"value": "second"})) })
            .await
            .expect("replaced unwrap");

        assert_eq!(result2, json!({"value": "second"}));
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = MCPCache::new(5);

        // Add some entries
        cache
            .get_or_compute("key1", || async { Ok(json!(1)) })
            .await
            .expect("replaced unwrap");
        cache
            .get_or_compute("key2", || async { Ok(json!(2)) })
            .await
            .expect("replaced unwrap");
        cache
            .get_or_compute("key1", || async { Ok(json!(999)) })
            .await
            .expect("replaced unwrap"); // Should hit cache

        let stats = cache.stats().await;
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.total_hits, 2); // One for initial key1, one for cache hit
    }

    #[tokio::test]
    async fn test_tag_based_caching() {
        let cache = MCPCache::new(5);
        let key = "test_key";
        let mut tags = HashSet::new();
        tags.insert("api".to_string());
        tags.insert("user_data".to_string());

        // Store with tags
        let result1 = cache
            .get_or_compute_with_tags(key, || async { Ok(json!({"value": "tagged"})) }, tags)
            .await
            .expect("replaced unwrap");

        assert_eq!(result1, json!({"value": "tagged"}));

        // Check tag index
        let api_entries = cache.get_entries_by_tag("api").await;
        assert_eq!(api_entries.len(), 1);
        assert_eq!(api_entries[0], key);

        let user_entries = cache.get_entries_by_tag("user_data").await;
        assert_eq!(user_entries.len(), 1);
        assert_eq!(user_entries[0], key);
    }

    #[tokio::test]
    async fn test_tag_based_invalidation() {
        let cache = MCPCache::new(5);

        // Store entries with different tags
        let mut api_tags = HashSet::new();
        api_tags.insert("api".to_string());

        let mut user_tags = HashSet::new();
        user_tags.insert("user".to_string());

        cache
            .get_or_compute_with_tags("api_key", || async { Ok(json!({"type": "api"})) }, api_tags)
            .await
            .expect("replaced unwrap");

        cache
            .get_or_compute_with_tags(
                "user_key",
                || async { Ok(json!({"type": "user"})) },
                user_tags,
            )
            .await
            .expect("replaced unwrap");

        // Verify both entries exist
        let stats = cache.enhanced_stats().await;
        assert_eq!(stats.total_entries, 2);

        // Invalidate by tag
        let invalidated = cache.invalidate_by_tag("api").await;
        assert_eq!(invalidated, 1);

        // Check that only API entry was removed
        let stats_after = cache.enhanced_stats().await;
        assert_eq!(stats_after.total_entries, 1);

        let api_entries = cache.get_entries_by_tag("api").await;
        assert_eq!(api_entries.len(), 0);

        let user_entries = cache.get_entries_by_tag("user").await;
        assert_eq!(user_entries.len(), 1);
    }

    #[tokio::test]
    async fn test_event_driven_invalidation() {
        let cache = MCPCache::new(5);
        let mut receiver = cache.subscribe_events();

        // Store an entry
        cache
            .get_or_compute("test_key", || async { Ok(json!({"value": "test"})) })
            .await
            .expect("replaced unwrap");

        // Invalidate it
        cache.invalidate_key("test_key").await;

        // Check that we received the invalidation event
        let event = receiver.try_recv().expect("replaced unwrap");
        match event {
            CacheInvalidationEvent::KeyInvalidation { key } => {
                assert_eq!(key, "test_key");
            }
            _ => panic!("Expected KeyInvalidation event"),
        }
    }

    #[tokio::test]
    async fn test_enhanced_stats() {
        let cache = MCPCache::new(5);

        // Add entries with tags
        let mut tags1 = HashSet::new();
        tags1.insert("api".to_string());

        let mut tags2 = HashSet::new();
        tags2.insert("user".to_string());

        cache
            .get_or_compute_with_tags(
                "key1",
                || async { Ok(json!({"data": "large_payload_to_test_size_calculation"})) },
                tags1,
            )
            .await
            .expect("replaced unwrap");

        cache
            .get_or_compute_with_tags(
                "key2",
                || async { Ok(json!({"data": "another_payload"})) },
                tags2,
            )
            .await
            .expect("replaced unwrap");

        // Access one entry multiple times
        cache
            .get_or_compute("key1", || async {
                Ok(json!({"data": "should_not_compute"}))
            })
            .await
            .expect("replaced unwrap");

        let stats = cache.enhanced_stats().await;
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.total_hits, 1);
        assert_eq!(stats.total_misses, 2);
        assert!(stats.hit_rate > 0.0);
        assert!(stats.memory_usage_bytes > 0);
        assert!(stats.average_entry_size > 0);
        assert_eq!(stats.entries_by_tag.len(), 2);
        assert_eq!(
            stats.entries_by_tag.get("api").expect("replaced unwrap"),
            &1
        );
        assert_eq!(
            stats.entries_by_tag.get("user").expect("replaced unwrap"),
            &1
        );
    }

    #[tokio::test]
    async fn test_generate_cache_key() {
        let key1 = generate_cache_key("test_method", &None);
        assert_eq!(key1, "test_method");

        let params = Some(json!({"param": "value"}));
        let key2 = generate_cache_key("test_method", &params);
        assert!(key2.starts_with("test_method:"));
        assert!(key2.contains("param"));
    }
}
