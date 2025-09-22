use super::mcp::MCPToolHandler;
use super::mcp_cache::{generate_cache_key, MCPCache};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tracing::debug;

/// Cached wrapper for MCP tool handlers (Phase 1.2)
pub struct CachedMCPToolHandler<T: MCPToolHandler> {
    inner: T,
    cache: Arc<MCPCache>,
}

impl<T: MCPToolHandler> CachedMCPToolHandler<T> {
    pub fn new(inner: T, cache: Arc<MCPCache>) -> Self {
        Self { inner, cache }
    }
}

#[async_trait]
impl<T: MCPToolHandler> MCPToolHandler for CachedMCPToolHandler<T> {
    async fn execute(&self, params: &Value) -> Result<Value> {
        // Generate cache key
        let cache_key = format!(
            "tool:{}:{}",
            self.inner
                .get_description()
                .replace(' ', "_")
                .to_lowercase(),
            generate_cache_key("execute", &Some(params.clone()))
        );

        // Use cache for read-only operations (avoid caching state-changing operations)
        let is_cacheable = self.is_cacheable_operation(params);

        if !is_cacheable {
            debug!("Skipping cache for non-cacheable operation");
            return self.inner.execute(params).await;
        }

        // Try to get from cache or compute
        self.cache
            .get_or_compute(&cache_key, || {
                let inner = &self.inner;
                async move { inner.execute(params).await }
            })
            .await
    }

    fn get_schema(&self) -> Value {
        self.inner.get_schema()
    }

    fn get_description(&self) -> String {
        format!("{} (cached)", self.inner.get_description())
    }
}

impl<T: MCPToolHandler> CachedMCPToolHandler<T> {
    /// Determine if an operation should be cached based on its nature
    fn is_cacheable_operation(&self, _params: &Value) -> bool {
        let description = self.inner.get_description().to_lowercase();

        // Read-only operations that are safe to cache
        let cacheable_patterns = [
            "get_", "list_", "analyze_", "status", "info", "details", "echo",
        ];

        // State-changing operations that should not be cached
        let non_cacheable_patterns = [
            "create_",
            "assign_",
            "batch_create",
            "coordinate_",
            "delete_",
            "update_",
        ];

        // Check for non-cacheable operations first
        for pattern in &non_cacheable_patterns {
            if description.contains(pattern) {
                return false;
            }
        }

        // Check for cacheable operations
        for pattern in &cacheable_patterns {
            if description.contains(pattern) {
                return true;
            }
        }

        // For specialized workflows and performance analytics, check if it's read-only
        if description.contains("analytics") || description.contains("performance") {
            // These are typically read operations
            return true;
        }

        // Default to not caching unless we're sure it's safe
        false
    }
}

/// Tool registry with caching support (Phase 2.1)
pub struct CachedMCPToolRegistry {
    cache: Arc<MCPCache>,
}

impl CachedMCPToolRegistry {
    #[must_use]
    pub fn new(cache_ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(MCPCache::new(cache_ttl_seconds)),
        }
    }

    pub fn wrap_tool<T: MCPToolHandler + 'static>(&self, tool: T) -> Box<dyn MCPToolHandler> {
        Box::new(CachedMCPToolHandler::new(tool, Arc::clone(&self.cache)))
    }

    pub async fn get_cache_stats(&self) -> super::mcp_cache::CacheStats {
        self.cache.stats().await
    }

    pub async fn clear_cache(&self) {
        self.cache.clear().await;
    }

    pub async fn cleanup_expired(&self) {
        self.cache.cleanup_expired().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Mock tool for testing
    struct MockTool {
        call_count: std::sync::Arc<std::sync::atomic::AtomicU32>,
    }

    impl MockTool {
        fn new() -> Self {
            Self {
                call_count: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
            }
        }

        fn get_call_count(&self) -> u32 {
            self.call_count.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl MCPToolHandler for MockTool {
        async fn execute(&self, params: &Value) -> Result<Value> {
            self.call_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(json!({"result": "mock", "params": params}))
        }

        fn get_schema(&self) -> Value {
            json!({"type": "object"})
        }

        fn get_description(&self) -> String {
            "get_mock_data".to_string() // Cacheable operation
        }
    }

    #[tokio::test]
    async fn test_cached_tool_handler() {
        let mock_tool = MockTool::new();
        let cache = Arc::new(MCPCache::new(5));
        let cached_tool = CachedMCPToolHandler::new(mock_tool, cache);

        let params = json!({"test": "value"});

        // First call should execute the tool
        let result1 = cached_tool.execute(&params).await.expect("replaced unwrap");
        assert_eq!(cached_tool.inner.get_call_count(), 1);

        // Second call should use cache
        let result2 = cached_tool.execute(&params).await.expect("replaced unwrap");
        assert_eq!(cached_tool.inner.get_call_count(), 1); // Should not increase

        assert_eq!(result1, result2);
    }

    #[tokio::test]
    async fn test_non_cacheable_operation() {
        struct CreateTool;

        #[async_trait]
        impl MCPToolHandler for CreateTool {
            async fn execute(&self, _params: &Value) -> Result<Value> {
                Ok(json!({"created": true}))
            }

            fn get_schema(&self) -> Value {
                json!({})
            }
            fn get_description(&self) -> String {
                "create_agent".to_string()
            }
        }

        let create_tool = CreateTool;
        let cache = Arc::new(MCPCache::new(5));
        let cached_tool = CachedMCPToolHandler::new(create_tool, cache.clone());

        let params = json!({"type": "worker"});

        // Should not use cache for create operations
        assert!(!cached_tool.is_cacheable_operation(&params));

        // Verify cache is empty after execution
        cached_tool.execute(&params).await.expect("replaced unwrap");
        let stats = cache.stats().await;
        assert_eq!(stats.total_entries, 0);
    }
}
