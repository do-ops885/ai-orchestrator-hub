use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::debug;

/// MCP-specific response caching implementation (Phase 1.3)
#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub data: Value,
    pub timestamp: Instant,
    pub hits: u32,
}

#[derive(Debug)]
pub struct MCPCache {
    responses: Arc<RwLock<HashMap<String, CachedResponse>>>,
    ttl: Duration,
}

impl MCPCache {
    #[must_use] 
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            responses: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Get cached response or compute new one
    pub async fn get_or_compute<F, Fut>(&self, key: &str, compute: F) -> Result<Value>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<Value>>,
    {
        // Check cache first
        {
            let mut cache = self.responses.write().await;
            if let Some(cached) = cache.get_mut(key) {
                if cached.timestamp.elapsed() < self.ttl {
                    cached.hits += 1;
                    debug!("Cache hit for key: {} (hits: {})", key, cached.hits);
                    return Ok(cached.data.clone());
                }
                // Remove expired entry
                cache.remove(key);
                debug!("Cache entry expired for key: {}", key);
            }
        }

        // Compute new value
        debug!("Cache miss for key: {}, computing new value", key);
        let result = compute().await?;
        
        // Store in cache
        let cached = CachedResponse {
            data: result.clone(),
            timestamp: Instant::now(),
            hits: 1,
        };
        
        self.responses.write().await.insert(key.to_string(), cached);
        debug!("Cached new value for key: {}", key);
        
        Ok(result)
    }

    /// Invalidate cache entry
    pub async fn invalidate(&self, key: &str) {
        self.responses.write().await.remove(key);
        debug!("Invalidated cache entry for key: {}", key);
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        self.responses.write().await.clear();
        debug!("Cleared all cache entries");
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let cache = self.responses.read().await;
        let total_entries = cache.len();
        let total_hits: u32 = cache.values().map(|v| v.hits).sum();
        let expired_count = cache
            .values()
            .filter(|v| v.timestamp.elapsed() >= self.ttl)
            .count();

        CacheStats {
            total_entries,
            total_hits,
            expired_count,
            ttl_seconds: self.ttl.as_secs(),
        }
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) {
        let mut cache = self.responses.write().await;
        let before_count = cache.len();
        cache.retain(|_, v| v.timestamp.elapsed() < self.ttl);
        let after_count = cache.len();
        let removed = before_count - after_count;
        
        if removed > 0 {
            debug!("Cleaned up {} expired cache entries", removed);
        }
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
        Some(p) => format!("{}:{}", method, serde_json::to_string(p).unwrap_or_default()),
        None => method.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_cache_hit_miss() {
        let cache = MCPCache::new(5); // 5 second TTL
        let key = "test_key";
        
        // First call should be a miss
        let result1 = cache.get_or_compute(key, || async {
            Ok(json!({"value": "computed"}))
        }).await.unwrap();
        
        assert_eq!(result1, json!({"value": "computed"}));
        
        // Second call should be a hit
        let result2 = cache.get_or_compute(key, || async {
            Ok(json!({"value": "different"}))
        }).await.unwrap();
        
        assert_eq!(result2, json!({"value": "computed"})); // Should return cached value
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = MCPCache::new(1); // 1 second TTL
        let key = "test_key";
        
        // Store value
        let _result1 = cache.get_or_compute(key, || async {
            Ok(json!({"value": "first"}))
        }).await.unwrap();
        
        // Wait for expiration
        sleep(Duration::from_millis(1100)).await;
        
        // Should compute new value
        let result2 = cache.get_or_compute(key, || async {
            Ok(json!({"value": "second"}))
        }).await.unwrap();
        
        assert_eq!(result2, json!({"value": "second"}));
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = MCPCache::new(5);
        
        // Add some entries
        cache.get_or_compute("key1", || async { Ok(json!(1)) }).await.unwrap();
        cache.get_or_compute("key2", || async { Ok(json!(2)) }).await.unwrap();
        cache.get_or_compute("key1", || async { Ok(json!(999)) }).await.unwrap(); // Should hit cache
        
        let stats = cache.stats().await;
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.total_hits, 2); // One for initial key1, one for cache hit
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