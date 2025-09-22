use super::mcp::MCPToolHandler;
use super::mcp_cache::MCPCache;
use super::mcp_cached_tools::CachedMCPToolHandler;
use super::mcp_unified_error::{MCPErrorHandler, MCPUnifiedError};
use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Centralized MCP Tool Registry (Phase 2.1)
///
/// This provides a unified registry for all MCP tools with categorization,
/// dependency management, and advanced features like tool composition.
#[derive(Clone)]
pub struct MCPToolRegistry {
    tools: Arc<RwLock<HashMap<String, Arc<dyn MCPToolHandler>>>>,
    categories: Arc<RwLock<HashMap<String, Vec<String>>>>,
    dependencies: Arc<RwLock<HashMap<String, Vec<String>>>>,
    tool_metadata: Arc<RwLock<HashMap<String, ToolMetadata>>>,
    cache: Arc<MCPCache>,
    error_handler: MCPErrorHandler,
    enabled_categories: Arc<RwLock<Vec<String>>>,
}

#[derive(Debug, Clone)]
pub struct ToolMetadata {
    pub name: String,
    pub category: String,
    pub version: String,
    pub author: Option<String>,
    pub description: String,
    pub tags: Vec<String>,
    pub dependencies: Vec<String>,
    pub deprecated: bool,
    pub experimental: bool,
    pub performance_tier: PerformanceTier,
    pub caching_strategy: CachingStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerformanceTier {
    Fast,   // < 100ms expected
    Medium, // 100ms - 1s expected
    Slow,   // 1s - 10s expected
    Heavy,  // > 10s expected
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CachingStrategy {
    Never,      // Never cache (state-changing operations)
    Short,      // Cache for 1 minute
    Medium,     // Cache for 5 minutes
    Long,       // Cache for 30 minutes
    Persistent, // Cache until explicitly invalidated
}

impl MCPToolRegistry {
    #[must_use]
    pub fn new(cache_ttl_seconds: u64) -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            categories: Arc::new(RwLock::new(HashMap::new())),
            dependencies: Arc::new(RwLock::new(HashMap::new())),
            tool_metadata: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(MCPCache::new(cache_ttl_seconds)),
            error_handler: MCPErrorHandler::default(),
            enabled_categories: Arc::new(RwLock::new(vec![
                "core".to_string(),
                "agent_management".to_string(),
                "task_management".to_string(),
                "analytics".to_string(),
                "utilities".to_string(),
            ])),
        }
    }

    /// Register a tool with comprehensive metadata
    pub async fn register_tool<T: MCPToolHandler + 'static>(
        &self,
        name: String,
        handler: T,
        metadata: ToolMetadata,
    ) -> Result<(), MCPUnifiedError> {
        // Validate tool name
        if name.is_empty() {
            return Err(MCPUnifiedError::validation(
                "tool_name".to_string(),
                "Tool name cannot be empty".to_string(),
                Some(json!(name)),
                Some("Non-empty string".to_string()),
            ));
        }

        // Check if tool already exists
        let tool_exists = self.tools.read().await.contains_key(&name);
        if tool_exists {
            warn!("Tool '{}' already exists, replacing", name);
        }

        // Validate category is enabled
        let enabled_categories = self.enabled_categories.read().await;
        if !enabled_categories.contains(&metadata.category) {
            return Err(MCPUnifiedError::validation(
                "category".to_string(),
                format!("Category '{}' is not enabled", metadata.category),
                Some(json!(metadata.category)),
                Some(format!(
                    "Enabled categories: {}",
                    enabled_categories.join(", ")
                )),
            ));
        }

        // Check dependencies exist
        for dep in &metadata.dependencies {
            if !self.tools.read().await.contains_key(dep) {
                return Err(MCPUnifiedError::validation(
                    "dependencies".to_string(),
                    format!("Dependency '{dep}' not found"),
                    Some(json!(dep)),
                    Some("All dependencies must be registered first".to_string()),
                ));
            }
        }

        // Temporarily add this tool to check for cycles
        // We'll validate the dependency graph after adding
        let mut temp_dependencies = self.dependencies.read().await.clone();
        temp_dependencies.insert(name.clone(), metadata.dependencies.clone());

        // Check for cycles by attempting topological sort on temp graph
        if let Err(_) = self
            .validate_dependencies_with_graph(&temp_dependencies)
            .await
        {
            return Err(MCPUnifiedError::validation(
                "dependencies".to_string(),
                format!("Adding tool '{}' would create a circular dependency", name),
                Some(json!({
                    "tool": name,
                    "dependencies": metadata.dependencies
                })),
                Some("Ensure the new tool does not create dependency cycles".to_string()),
            ));
        }

        // Wrap with caching based on strategy
        let cached_handler = if metadata.caching_strategy == CachingStrategy::Never {
            Box::new(handler) as Box<dyn MCPToolHandler>
        } else {
            let cache_ttl = match metadata.caching_strategy {
                CachingStrategy::Short => 60,
                CachingStrategy::Medium => 300,
                CachingStrategy::Long => 1800,
                CachingStrategy::Persistent => u64::MAX,
                CachingStrategy::Never => unreachable!(),
            };
            let cache = Arc::new(MCPCache::new(cache_ttl));
            Box::new(CachedMCPToolHandler::new(handler, cache)) as Box<dyn MCPToolHandler>
        };

        // Register the tool
        self.tools
            .write()
            .await
            .insert(name.clone(), Arc::from(cached_handler));

        // Update categories
        self.categories
            .write()
            .await
            .entry(metadata.category.clone())
            .or_insert_with(Vec::new)
            .push(name.clone());

        // Store dependencies
        self.dependencies
            .write()
            .await
            .insert(name.clone(), metadata.dependencies.clone());

        // Store metadata
        self.tool_metadata
            .write()
            .await
            .insert(name.clone(), metadata.clone());

        info!(
            "Registered tool '{}' in category '{}' with {} dependencies",
            name,
            metadata.category,
            metadata.dependencies.len()
        );

        Ok(())
    }

    /// Register a simple tool with default metadata
    pub async fn register_simple_tool<T: MCPToolHandler + 'static>(
        &self,
        name: String,
        handler: T,
        category: &str,
        description: String,
        caching_strategy: CachingStrategy,
    ) -> Result<(), MCPUnifiedError> {
        let metadata = ToolMetadata {
            name: name.clone(),
            category: category.to_string(),
            version: "1.0.0".to_string(),
            author: None,
            description,
            tags: vec![],
            dependencies: vec![],
            deprecated: false,
            experimental: false,
            performance_tier: PerformanceTier::Medium,
            caching_strategy,
        };

        self.register_tool(name, handler, metadata).await
    }

    /// Get a tool by name
    pub async fn get_tool(&self, name: &str) -> Option<Arc<dyn MCPToolHandler>> {
        self.tools.read().await.get(name).cloned()
    }

    /// Get all tools in a category
    pub async fn get_tools_by_category(&self, category: &str) -> Vec<String> {
        self.categories
            .read()
            .await
            .get(category)
            .cloned()
            .unwrap_or_default()
    }

    /// Get tools by performance tier
    pub async fn get_tools_by_performance_tier(&self, tier: PerformanceTier) -> Vec<String> {
        self.tool_metadata
            .read()
            .await
            .iter()
            .filter(|(_, metadata)| metadata.performance_tier == tier)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get tools with specific tags
    pub async fn get_tools_by_tags(&self, tags: &[String]) -> Vec<String> {
        self.tool_metadata
            .read()
            .await
            .iter()
            .filter(|(_, metadata)| tags.iter().any(|tag| metadata.tags.contains(tag)))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get tool metadata
    pub async fn get_tool_metadata(&self, name: &str) -> Option<ToolMetadata> {
        self.tool_metadata.read().await.get(name).cloned()
    }

    /// List all available tools with metadata
    pub async fn list_tools(&self) -> Value {
        let tools = self.tools.read().await;
        let metadata_map = self.tool_metadata.read().await;
        let tools: Vec<Value> = tools
            .iter()
            .map(|(name, handler)| {
                let metadata = metadata_map.get(name);
                json!({
                    "name": name,
                    "description": handler.get_description(),
                    "inputSchema": handler.get_schema(),
                    "category": metadata.as_ref().map_or(&"unknown".to_string(), |m| &m.category),
                    "version": metadata.as_ref().map_or(&"unknown".to_string(), |m| &m.version),
                    "performance_tier": format!("{:?}", metadata.as_ref().map_or(PerformanceTier::Medium, |m| m.performance_tier)),
                    "caching_strategy": format!("{:?}", metadata.as_ref().map_or(CachingStrategy::Medium, |m| m.caching_strategy)),
                    "deprecated": metadata.as_ref().is_some_and(|m| m.deprecated),
                    "experimental": metadata.as_ref().is_some_and(|m| m.experimental),
                    "dependencies": metadata.map_or(&vec![], |m| &m.dependencies),
                    "tags": metadata.map_or(&vec![], |m| &m.tags),
                })
            })
            .collect();

        json!({
            "tools": tools,
            "categories": self.get_category_summary().await,
            "total_count": tools.len(),
            "enabled_categories": self.enabled_categories.read().await.clone()
        })
    }

    /// Get category summary
    pub async fn get_category_summary(&self) -> Value {
        let categories = self.categories.read().await;
        let metadata_map = self.tool_metadata.read().await;
        let summary: HashMap<String, Value> = categories
            .iter()
            .map(|(category, tools)| {
                let tool_count = tools.len();
                let deprecated_count = tools
                    .iter()
                    .filter(|tool| {
                        metadata_map
                            .get(tool.as_str())
                            .is_some_and(|m| m.deprecated)
                    })
                    .count();
                let experimental_count = tools
                    .iter()
                    .filter(|tool| {
                        metadata_map
                            .get(tool.as_str())
                            .is_some_and(|m| m.experimental)
                    })
                    .count();

                (
                    category.clone(),
                    json!({
                        "total_tools": tool_count,
                        "deprecated_tools": deprecated_count,
                        "experimental_tools": experimental_count,
                        "tools": tools
                    }),
                )
            })
            .collect();

        json!(summary)
    }

    /// Validate tool dependencies (topological sort)
    pub async fn validate_dependencies(&self) -> Result<Vec<String>, MCPUnifiedError> {
        let dependencies = self.dependencies.read().await;
        let tools = self.tools.read().await;

        // Build adjacency list for topological sort
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        // Initialize in-degree for all tools
        for tool_name in tools.keys() {
            in_degree.insert(tool_name.clone(), 0);
            adj_list.insert(tool_name.clone(), Vec::new());
        }

        // Build the graph
        for (tool_name, deps) in dependencies.iter() {
            if let Some(neighbors) = adj_list.get_mut(tool_name) {
                for dep in deps {
                    neighbors.push(dep.clone());
                    *in_degree.entry(dep.clone()).or_insert(0) += 1;
                }
            }
        }

        // Perform topological sort using Kahn's algorithm
        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(name, _)| name.clone())
            .collect();

        let mut sorted_order = Vec::new();
        let mut visited_count = 0;

        while let Some(tool) = queue.pop() {
            sorted_order.push(tool.clone());
            visited_count += 1;

            if let Some(neighbors) = adj_list.get(&tool) {
                for neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(neighbor.clone());
                        }
                    }
                }
            }
        }

        // Check for cycles
        if visited_count != tools.len() {
            // Find tools involved in cycles
            let mut cycle_tools = Vec::new();
            for (tool, &deg) in &in_degree {
                if deg > 0 {
                    cycle_tools.push(tool.clone());
                }
            }

            return Err(MCPUnifiedError::validation(
                "tool_dependencies".to_string(),
                format!(
                    "Circular dependency detected involving tools: {}",
                    cycle_tools.join(", ")
                ),
                Some(serde_json::json!({
                    "cycle_participants": cycle_tools,
                    "total_tools": tools.len(),
                    "visited_tools": visited_count
                })),
                Some("Ensure no circular dependencies exist between tools".to_string()),
            ));
        }

        // Return the valid topological order
        Ok(sorted_order)
    }

    /// Validate dependencies with a custom graph (helper for registration)
    async fn validate_dependencies_with_graph(
        &self,
        custom_dependencies: &HashMap<String, Vec<String>>,
    ) -> Result<Vec<String>, MCPUnifiedError> {
        let tools = self.tools.read().await;

        // Build adjacency list for topological sort
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        // Initialize in-degree for all tools
        for tool_name in tools.keys() {
            in_degree.insert(tool_name.clone(), 0);
            adj_list.insert(tool_name.clone(), Vec::new());
        }

        // Build the graph using custom dependencies
        for (tool_name, deps) in custom_dependencies.iter() {
            if let Some(neighbors) = adj_list.get_mut(tool_name) {
                for dep in deps {
                    neighbors.push(dep.clone());
                    *in_degree.entry(dep.clone()).or_insert(0) += 1;
                }
            }
        }

        // Perform topological sort using Kahn's algorithm
        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(name, _)| name.clone())
            .collect();

        let mut sorted_order = Vec::new();
        let mut visited_count = 0;

        while let Some(tool) = queue.pop() {
            sorted_order.push(tool.clone());
            visited_count += 1;

            if let Some(neighbors) = adj_list.get(&tool) {
                for neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(neighbor.clone());
                        }
                    }
                }
            }
        }

        // Check for cycles
        if visited_count != tools.len() {
            return Err(MCPUnifiedError::validation(
                "tool_dependencies".to_string(),
                "Circular dependency detected".to_string(),
                None,
                Some("Dependency graph contains cycles".to_string()),
            ));
        }

        Ok(sorted_order)
    }

    /// Get dependency graph structure
    pub async fn get_dependency_graph(&self) -> Value {
        use serde_json::Map;

        let dependencies = self.dependencies.read().await;
        let tools = self.tools.read().await;
        let metadata = self.tool_metadata.read().await;

        let mut graph = Map::new();

        for (tool_name, deps) in dependencies.iter() {
            let mut node = Map::new();
            node.insert("name".to_string(), Value::String(tool_name.clone()));
            node.insert("dependencies".to_string(), serde_json::json!(deps));

            // Add metadata if available
            if let Some(meta) = metadata.get(tool_name) {
                node.insert("category".to_string(), Value::String(meta.category.clone()));
                node.insert("version".to_string(), Value::String(meta.version.clone()));
                node.insert(
                    "performance_tier".to_string(),
                    Value::String(format!("{:?}", meta.performance_tier)),
                );
            }

            // Check if tool exists
            node.insert(
                "exists".to_string(),
                Value::Bool(tools.contains_key(tool_name)),
            );

            graph.insert(tool_name.clone(), Value::Object(node));
        }

        serde_json::json!({
            "graph": graph,
            "total_tools": tools.len(),
            "total_dependencies": dependencies.len()
        })
    }

    /// Enable or disable tool categories
    pub fn set_enabled_categories(&mut self, categories: Vec<String>) {
        self.enabled_categories = Arc::new(RwLock::new(categories));
        info!("Updated enabled categories");
    }

    /// Mark a tool as deprecated
    pub async fn deprecate_tool(
        &mut self,
        name: &str,
        reason: Option<String>,
    ) -> Result<(), MCPUnifiedError> {
        let mut tool_metadata = self.tool_metadata.write().await;
        if let Some(metadata) = tool_metadata.get_mut(name) {
            metadata.deprecated = true;
            warn!(
                "Tool '{}' marked as deprecated: {}",
                name,
                reason.unwrap_or_else(|| "No reason provided".to_string())
            );
            Ok(())
        } else {
            Err(MCPUnifiedError::validation(
                "tool_name".to_string(),
                format!("Tool '{name}' not found"),
                Some(json!(name)),
                Some("Tool must be registered first".to_string()),
            ))
        }
    }

    /// Get registry statistics
    pub async fn get_statistics(&self) -> Value {
        let tools = self.tools.read().await;
        let categories = self.categories.read().await;
        let tool_metadata = self.tool_metadata.read().await;

        let total_tools = tools.len();
        let categories_count = categories.len();

        let deprecated_count = tool_metadata.values().filter(|m| m.deprecated).count();

        let experimental_count = tool_metadata.values().filter(|m| m.experimental).count();

        let performance_distribution: HashMap<String, usize> = [
            ("Fast".to_string(), 0),
            ("Medium".to_string(), 0),
            ("Slow".to_string(), 0),
            ("Heavy".to_string(), 0),
        ]
        .iter()
        .cloned()
        .collect();

        let mut perf_dist = performance_distribution;
        for metadata in tool_metadata.values() {
            let tier_name = format!("{:?}", metadata.performance_tier);
            *perf_dist.entry(tier_name).or_insert(0) += 1;
        }

        let enabled_categories = self.enabled_categories.read().await.clone();
        json!({
            "total_tools": total_tools,
            "categories_count": categories_count,
            "deprecated_tools": deprecated_count,
            "experimental_tools": experimental_count,
            "enabled_categories": enabled_categories,
            "performance_distribution": perf_dist,
            "cache_enabled": true
        })
    }

    /// Execute a tool with error handling and metrics
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        params: &Value,
        _request_id: Option<&str>,
        _client_id: Option<&str>,
    ) -> Result<Value, MCPUnifiedError> {
        let start_time = std::time::Instant::now();

        // Check if tool exists
        let tools = self.tools.read().await;
        let tool = tools.get(tool_name).ok_or_else(|| {
            MCPUnifiedError::validation(
                "tool_name".to_string(),
                format!("Tool '{tool_name}' not found"),
                Some(json!(tool_name)),
                Some(format!(
                    "Available tools: {}",
                    tools.keys().cloned().collect::<Vec<_>>().join(", ")
                )),
            )
        })?;

        // Check if tool is deprecated
        let tool_metadata = self.tool_metadata.read().await;
        if let Some(metadata) = tool_metadata.get(tool_name) {
            if metadata.deprecated {
                warn!("Using deprecated tool: {}", tool_name);
            }
        }

        // Execute tool with error handling
        let result = tool.execute(params).await.map_err(|e| {
            MCPUnifiedError::tool_execution(
                tool_name.to_string(),
                e.to_string(),
                Some(params.clone()),
                Some(start_time.elapsed().as_millis() as u64),
            )
        })?;

        let duration = start_time.elapsed();
        debug!(
            "Tool '{}' executed successfully in {}ms",
            tool_name,
            duration.as_millis()
        );

        Ok(result)
    }
}

impl Default for MCPToolRegistry {
    fn default() -> Self {
        Self::new(300) // 5 minute default cache TTL
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockTool {
        name: String,
    }

    #[async_trait]
    impl MCPToolHandler for MockTool {
        async fn execute(&self, _params: &Value) -> Result<Value> {
            Ok(json!({"tool": self.name}))
        }

        fn get_schema(&self) -> Value {
            json!({"type": "object"})
        }

        fn get_description(&self) -> String {
            format!("Mock tool: {}", self.name)
        }
    }

    #[tokio::test]
    async fn test_tool_registration() {
        let mut registry = MCPToolRegistry::new(300);

        let metadata = ToolMetadata {
            name: "test_tool".to_string(),
            category: "core".to_string(),
            version: "1.0.0".to_string(),
            author: Some("Test Author".to_string()),
            description: "A test tool".to_string(),
            tags: vec!["test".to_string()],
            dependencies: vec![],
            deprecated: false,
            experimental: false,
            performance_tier: PerformanceTier::Fast,
            caching_strategy: CachingStrategy::Medium,
        };

        let result = registry.register_tool(
            "test_tool".to_string(),
            MockTool {
                name: "test".to_string(),
            },
            metadata,
        );

        assert!(result.is_ok());
        assert!(registry.get_tool("test_tool").is_some());
    }

    #[tokio::test]
    async fn test_category_filtering() {
        let mut registry = MCPToolRegistry::new(300);

        let metadata1 = ToolMetadata {
            name: "tool1".to_string(),
            category: "core".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            description: "Tool 1".to_string(),
            tags: vec![],
            dependencies: vec![],
            deprecated: false,
            experimental: false,
            performance_tier: PerformanceTier::Fast,
            caching_strategy: CachingStrategy::Never,
        };

        let metadata2 = ToolMetadata {
            name: "tool2".to_string(),
            category: "analytics".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            description: "Tool 2".to_string(),
            tags: vec![],
            dependencies: vec![],
            deprecated: false,
            experimental: false,
            performance_tier: PerformanceTier::Medium,
            caching_strategy: CachingStrategy::Short,
        };

        registry
            .register_tool(
                "tool1".to_string(),
                MockTool {
                    name: "tool1".to_string(),
                },
                metadata1,
            )
            .expect("replaced unwrap");
        registry
            .register_tool(
                "tool2".to_string(),
                MockTool {
                    name: "tool2".to_string(),
                },
                metadata2,
            )
            .expect("replaced unwrap");

        let core_tools = registry.get_tools_by_category("core");
        assert_eq!(core_tools.len(), 1);
        assert_eq!(core_tools[0], "tool1");

        let analytics_tools = registry.get_tools_by_category("analytics");
        assert_eq!(analytics_tools.len(), 1);
        assert_eq!(analytics_tools[0], "tool2");
    }

    #[tokio::test]
    async fn test_dependency_validation() {
        let mut registry = MCPToolRegistry::new(300);

        // Register base tool
        let base_metadata = ToolMetadata {
            name: "base_tool".to_string(),
            category: "core".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            description: "Base tool".to_string(),
            tags: vec![],
            dependencies: vec![],
            deprecated: false,
            experimental: false,
            performance_tier: PerformanceTier::Fast,
            caching_strategy: CachingStrategy::Never,
        };

        registry
            .register_tool(
                "base_tool".to_string(),
                MockTool {
                    name: "base".to_string(),
                },
                base_metadata,
            )
            .expect("replaced unwrap");

        // Register dependent tool
        let dependent_metadata = ToolMetadata {
            name: "dependent_tool".to_string(),
            category: "core".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            description: "Dependent tool".to_string(),
            tags: vec![],
            dependencies: vec!["base_tool".to_string()],
            deprecated: false,
            experimental: false,
            performance_tier: PerformanceTier::Medium,
            caching_strategy: CachingStrategy::Short,
        };

        let result = registry.register_tool(
            "dependent_tool".to_string(),
            MockTool {
                name: "dependent".to_string(),
            },
            dependent_metadata,
        );
        assert!(result.is_ok());

        // Test dependency validation
        let validation_result = registry.validate_dependencies();
        assert!(validation_result.is_ok());
    }

    #[tokio::test]
    async fn test_tool_execution() {
        let mut registry = MCPToolRegistry::new(300);

        let metadata = ToolMetadata {
            name: "test_tool".to_string(),
            category: "core".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            description: "Test tool".to_string(),
            tags: vec![],
            dependencies: vec![],
            deprecated: false,
            experimental: false,
            performance_tier: PerformanceTier::Fast,
            caching_strategy: CachingStrategy::Never,
        };

        registry
            .register_tool(
                "test_tool".to_string(),
                MockTool {
                    name: "test".to_string(),
                },
                metadata,
            )
            .expect("replaced unwrap");

        let result = registry
            .execute_tool("test_tool", &json!({}), Some("req_123"), Some("client_456"))
            .await;

        assert!(result.is_ok());
        let value = result.expect("replaced unwrap");
        assert_eq!(value["tool"], "test");
    }

    #[tokio::test]
    async fn test_dependency_validation_no_cycles() {
        let registry = MCPToolRegistry::new(300);

        // Register tools with dependencies
        let base_metadata = ToolMetadata {
            name: "base_tool".to_string(),
            category: "core".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            description: "Base tool".to_string(),
            tags: vec![],
            dependencies: vec![],
            deprecated: false,
            experimental: false,
            performance_tier: PerformanceTier::Fast,
            caching_strategy: CachingStrategy::Never,
        };

        let dependent_metadata = ToolMetadata {
            name: "dependent_tool".to_string(),
            category: "core".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            description: "Dependent tool".to_string(),
            tags: vec![],
            dependencies: vec!["base_tool".to_string()],
            deprecated: false,
            experimental: false,
            performance_tier: PerformanceTier::Medium,
            caching_strategy: CachingStrategy::Short,
        };

        registry
            .register_tool(
                "base_tool".to_string(),
                MockTool {
                    name: "base".to_string(),
                },
                base_metadata,
            )
            .await
            .expect("replaced unwrap");
        registry
            .register_tool(
                "dependent_tool".to_string(),
                MockTool {
                    name: "dependent".to_string(),
                },
                dependent_metadata,
            )
            .await
            .expect("replaced unwrap");

        let result = registry.validate_dependencies().await;
        assert!(result.is_ok());
        let order = result.expect("replaced unwrap");
        assert_eq!(order.len(), 2);
        // base_tool should come before dependent_tool
        assert!(
            order
                .iter()
                .position(|x| x == "base_tool")
                .expect("replaced unwrap")
                < order
                    .iter()
                    .position(|x| x == "dependent_tool")
                    .expect("replaced unwrap")
        );
    }

    #[tokio::test]
    async fn test_dependency_validation_with_cycles() {
        let registry = MCPToolRegistry::new(300);

        // Register tools that would create a cycle
        let tool_a_metadata = ToolMetadata {
            name: "tool_a".to_string(),
            category: "core".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            description: "Tool A".to_string(),
            tags: vec![],
            dependencies: vec!["tool_b".to_string()], // Depends on B
            deprecated: false,
            experimental: false,
            performance_tier: PerformanceTier::Fast,
            caching_strategy: CachingStrategy::Never,
        };

        let tool_b_metadata = ToolMetadata {
            name: "tool_b".to_string(),
            category: "core".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            description: "Tool B".to_string(),
            tags: vec![],
            dependencies: vec!["tool_a".to_string()], // Depends on A - creates cycle
            deprecated: false,
            experimental: false,
            performance_tier: PerformanceTier::Medium,
            caching_strategy: CachingStrategy::Short,
        };

        registry
            .register_tool(
                "tool_a".to_string(),
                MockTool {
                    name: "a".to_string(),
                },
                tool_a_metadata,
            )
            .await
            .expect("replaced unwrap");
        let result = registry
            .register_tool(
                "tool_b".to_string(),
                MockTool {
                    name: "b".to_string(),
                },
                tool_b_metadata,
            )
            .await;
        assert!(result.is_err());

        if let Err(MCPUnifiedError::Validation { field, .. }) = result {
            assert_eq!(field, "dependencies");
        } else {
            panic!("Expected validation error");
        }
    }

    #[tokio::test]
    async fn test_get_dependency_graph() {
        let registry = MCPToolRegistry::new(300);

        let metadata = ToolMetadata {
            name: "test_tool".to_string(),
            category: "core".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            description: "Test tool".to_string(),
            tags: vec![],
            dependencies: vec![],
            deprecated: false,
            experimental: false,
            performance_tier: PerformanceTier::Fast,
            caching_strategy: CachingStrategy::Medium,
        };

        registry
            .register_tool(
                "test_tool".to_string(),
                MockTool {
                    name: "test".to_string(),
                },
                metadata,
            )
            .await
            .expect("replaced unwrap");

        let graph = registry.get_dependency_graph().await;
        assert!(graph.get("graph").is_some());
        assert!(graph.get("total_tools").is_some());
        assert!(graph.get("total_dependencies").is_some());

        let graph_obj = graph
            .get("graph")
            .expect("replaced unwrap")
            .as_object()
            .expect("replaced unwrap");
        assert!(graph_obj.contains_key("test_tool"));
    }
}
