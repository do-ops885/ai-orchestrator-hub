use super::mcp_unified_error::{MCPErrorHandler, MCPUnifiedError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// MCP Plugin Architecture Foundation (Phase 4)
///
/// This module provides a comprehensive plugin system for extending MCP functionality
/// with security sandboxing, hot-swappable plugins, and capability-based access control.

/// Core trait defining the interface for MCP plugins
#[async_trait]
pub trait MCPPlugin: Send + Sync {
    /// Initialize the plugin with configuration
    async fn initialize(&mut self, config: &PluginConfig) -> Result<(), MCPUnifiedError>;

    /// Shutdown the plugin and cleanup resources
    async fn shutdown(&mut self) -> Result<(), MCPUnifiedError>;

    /// Execute a tool provided by this plugin
    async fn execute_tool(&self, tool_name: &str, params: &Value)
        -> Result<Value, MCPUnifiedError>;

    /// Get the plugin's capabilities
    fn get_capabilities(&self) -> PluginCapabilities;

    /// Get plugin metadata
    fn get_metadata(&self) -> PluginMetadata;

    /// Check if plugin supports a specific tool
    fn supports_tool(&self, tool_name: &str) -> bool;
}

/// Plugin capabilities defining what a plugin can do
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapabilities {
    /// Tools provided by this plugin
    pub tools: Vec<String>,
    /// Security requirements for execution
    pub security_requirements: Vec<SecurityRequirement>,
    /// Resource limits for the plugin
    pub resource_limits: ResourceLimits,
    /// Supported execution modes
    pub execution_modes: Vec<ExecutionMode>,
    /// Plugin type (native, external, etc.)
    pub plugin_type: PluginType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityRequirement {
    /// Requires specific permissions
    Permission(String),
    /// Requires sandboxed execution
    Sandboxed,
    /// Requires network access
    NetworkAccess,
    /// Requires file system access
    FileSystemAccess(String), // path pattern
    /// Requires specific environment variables
    EnvironmentVariable(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: f32,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
    /// Maximum concurrent executions
    pub max_concurrent_executions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Synchronous execution
    Sync,
    /// Asynchronous execution
    Async,
    /// Streaming execution
    Streaming,
    /// Batch execution
    Batch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginType {
    /// Native Rust plugin
    Native,
    /// External plugin (WASM, process, etc.)
    External,
    /// WebAssembly plugin
    Wasm,
    /// HTTP-based plugin
    Http,
}

/// Configuration for plugin initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Unique plugin identifier
    pub id: String,
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Configuration parameters
    pub parameters: HashMap<String, Value>,
    /// Security context
    pub security_context: SecurityContext,
    /// Resource allocation
    pub resource_allocation: ResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Allowed permissions
    pub allowed_permissions: Vec<String>,
    /// Trusted execution environment
    pub trusted_environment: bool,
    /// Sandbox level
    pub sandbox_level: SandboxLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SandboxLevel {
    /// No sandboxing
    None,
    /// Basic sandboxing
    Basic,
    /// Strict sandboxing
    Strict,
    /// Isolated execution
    Isolated,
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Author information
    pub author: Option<String>,
    /// Description
    pub description: String,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Repository URL
    pub repository: Option<String>,
    /// License
    pub license: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Core plugin manager for loading, managing, and executing plugins
pub struct PluginManager {
    /// Registered plugins
    plugins: Arc<RwLock<HashMap<String, Box<dyn MCPPlugin>>>>,
    /// Plugin registry for discovery
    registry: Arc<RwLock<PluginRegistry>>,
    /// Security manager
    security_manager: Arc<SecurityManager>,
    /// Plugin loader
    plugin_loader: Arc<PluginLoader>,
    /// Error handler
    error_handler: MCPErrorHandler,
    /// Active plugin executions
    active_executions: Arc<RwLock<HashMap<String, Vec<ExecutionContext>>>>,
}

#[derive(Debug, Clone)]
struct ExecutionContext {
    plugin_id: String,
    tool_name: String,
    start_time: std::time::Instant,
    execution_id: Uuid,
}

/// Plugin registry for discovery and metadata
#[derive(Debug, Clone)]
pub struct PluginRegistry {
    /// Available plugins
    available_plugins: HashMap<String, PluginMetadata>,
    /// Plugin categories
    categories: HashMap<String, Vec<String>>,
    /// Plugin versions
    versions: HashMap<String, Vec<String>>,
}

impl PluginRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            available_plugins: HashMap::new(),
            categories: HashMap::new(),
            versions: HashMap::new(),
        }
    }

    /// Register a plugin in the registry
    pub fn register_plugin(&mut self, metadata: PluginMetadata) {
        let plugin_id = metadata.name.clone();
        self.available_plugins
            .insert(plugin_id.clone(), metadata.clone());

        // Update categories
        for tag in &metadata.tags {
            self.categories
                .entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(plugin_id.clone());
        }

        // Update versions
        self.versions
            .entry(plugin_id)
            .or_insert_with(Vec::new)
            .push(metadata.version);
    }

    /// Get plugin metadata
    pub fn get_plugin_metadata(&self, plugin_id: &str) -> Option<&PluginMetadata> {
        self.available_plugins.get(plugin_id)
    }

    /// List plugins by category
    pub fn list_plugins_by_category(&self, category: &str) -> Vec<&PluginMetadata> {
        self.categories
            .get(category)
            .map(|plugin_ids| {
                plugin_ids
                    .iter()
                    .filter_map(|id| self.available_plugins.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// List all available plugins
    pub fn list_all_plugins(&self) -> Vec<&PluginMetadata> {
        self.available_plugins.values().collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Security manager for plugin execution validation
pub struct SecurityManager {
    /// Global security policies
    policies: Arc<RwLock<HashMap<String, SecurityPolicy>>>,
    /// Active security contexts
    active_contexts: Arc<RwLock<HashMap<String, SecurityContext>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Policy name
    pub name: String,
    /// Allowed permissions
    pub allowed_permissions: Vec<String>,
    /// Denied permissions
    pub denied_permissions: Vec<String>,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Sandbox level
    pub sandbox_level: SandboxLevel,
}

impl SecurityManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            active_contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Validate plugin execution against security policies
    pub async fn validate_execution(
        &self,
        plugin_id: &str,
        tool_name: &str,
        capabilities: &PluginCapabilities,
        security_context: &SecurityContext,
    ) -> Result<(), MCPUnifiedError> {
        // Check sandbox level compatibility
        match (&security_context.sandbox_level, &capabilities.plugin_type) {
            (SandboxLevel::None, _) => {
                // No restrictions for no sandbox
            }
            (SandboxLevel::Basic | SandboxLevel::Strict, PluginType::Native) => {
                // Native plugins can run in basic/strict sandbox
            }
            (SandboxLevel::Isolated, PluginType::Native | PluginType::Wasm) => {
                // Only native and WASM plugins in isolated mode
            }
            _ => {
                return Err(MCPUnifiedError::ResourceAccess {
                    resource: format!("plugin:{plugin_id}"),
                    reason: "Plugin type not compatible with sandbox level".to_string(),
                    required_permissions: vec!["execute".to_string()],
                    context_chain: vec!["security_validation".to_string()],
                });
            }
        }

        // Check resource limits
        if capabilities.resource_limits.max_memory_bytes
            > security_context.allowed_permissions.len() as u64 * 1024 * 1024
        {
            return Err(MCPUnifiedError::ResourceAccess {
                resource: format!("memory:{plugin_id}"),
                reason: "Plugin memory requirements exceed allowed limits".to_string(),
                required_permissions: vec!["memory_access".to_string()],
                context_chain: vec!["resource_validation".to_string()],
            });
        }

        // Check security requirements
        for requirement in &capabilities.security_requirements {
            match requirement {
                SecurityRequirement::Permission(perm) => {
                    if !security_context.allowed_permissions.contains(perm) {
                        return Err(MCPUnifiedError::ResourceAccess {
                            resource: format!("permission:{perm}"),
                            reason: format!("Missing required permission: {perm}"),
                            required_permissions: vec![perm.clone()],
                            context_chain: vec!["permission_check".to_string()],
                        });
                    }
                }
                SecurityRequirement::Sandboxed => {
                    if matches!(security_context.sandbox_level, SandboxLevel::None) {
                        return Err(MCPUnifiedError::ResourceAccess {
                            resource: format!("sandbox:{plugin_id}"),
                            reason: "Plugin requires sandboxed execution".to_string(),
                            required_permissions: vec!["sandboxed_execution".to_string()],
                            context_chain: vec!["sandbox_check".to_string()],
                        });
                    }
                }
                _ => {
                    // Other requirements can be checked here
                    debug!("Security requirement validated: {:?}", requirement);
                }
            }
        }

        info!(
            "Security validation passed for plugin {} tool {}",
            plugin_id, tool_name
        );
        Ok(())
    }

    /// Create execution context for monitoring
    pub async fn create_execution_context(
        &self,
        plugin_id: &str,
        tool_name: &str,
    ) -> Result<String, MCPUnifiedError> {
        let execution_id = Uuid::new_v4().to_string();
        let context = SecurityContext {
            allowed_permissions: vec!["execute".to_string()],
            trusted_environment: false,
            sandbox_level: SandboxLevel::Basic,
        };

        self.active_contexts
            .write()
            .await
            .insert(execution_id.clone(), context);
        debug!(
            "Created execution context {} for plugin {}",
            execution_id, plugin_id
        );

        Ok(execution_id)
    }

    /// Cleanup execution context
    pub async fn cleanup_execution_context(&self, execution_id: &str) {
        self.active_contexts.write().await.remove(execution_id);
        debug!("Cleaned up execution context {}", execution_id);
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin loader for dynamic loading from various sources
pub struct PluginLoader {
    /// Supported plugin formats
    supported_formats: Vec<String>,
    /// Plugin search paths
    search_paths: Vec<String>,
    /// Loaded plugin libraries (placeholder for future implementation)
    loaded_libraries: Arc<RwLock<HashMap<String, String>>>,
}

impl PluginLoader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            supported_formats: vec![
                "so".to_string(),    // Linux shared library
                "dylib".to_string(), // macOS dynamic library
                "dll".to_string(),   // Windows DLL
                "wasm".to_string(),  // WebAssembly
            ],
            search_paths: vec![
                "./plugins".to_string(),
                "/usr/local/lib/mcp/plugins".to_string(),
                "/opt/mcp/plugins".to_string(),
            ],
            loaded_libraries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load a plugin from a file path
    pub async fn load_plugin_from_path(
        &self,
        path: &str,
        _config: &PluginConfig,
    ) -> Result<Box<dyn MCPPlugin>, MCPUnifiedError> {
        // Validate file extension
        let extension = std::path::Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| {
                MCPUnifiedError::validation(
                    "plugin_path".to_string(),
                    "Invalid plugin file path".to_string(),
                    Some(json!(path)),
                    Some("Path must have a valid extension".to_string()),
                )
            })?;

        if !self.supported_formats.contains(&extension.to_string()) {
            return Err(MCPUnifiedError::validation(
                "plugin_format".to_string(),
                format!("Unsupported plugin format: {}", extension),
                Some(json!(extension)),
                Some(format!(
                    "Supported formats: {}",
                    self.supported_formats.join(", ")
                )),
            ));
        }

        // For now, return an error as actual plugin loading would require
        // more complex implementation with dynamic linking
        // This is a placeholder for the full implementation
        Err(MCPUnifiedError::validation(
            "plugin_loading".to_string(),
            "Plugin loading not yet implemented".to_string(),
            Some(json!(path)),
            Some("This is a Phase 4 feature placeholder".to_string()),
        ))
    }

    /// Load a plugin from registry
    pub async fn load_plugin_from_registry(
        &self,
        plugin_id: &str,
        _config: &PluginConfig,
    ) -> Result<Box<dyn MCPPlugin>, MCPUnifiedError> {
        // This would search the registry and load the plugin
        // For now, return an error as this requires the full registry implementation
        Err(MCPUnifiedError::validation(
            "plugin_registry".to_string(),
            format!("Plugin {} not found in registry", plugin_id),
            Some(json!(plugin_id)),
            Some("Plugin must be registered before loading".to_string()),
        ))
    }

    /// Unload a plugin
    pub async fn unload_plugin(&self, plugin_id: &str) -> Result<(), MCPUnifiedError> {
        let mut libraries = self.loaded_libraries.write().await;
        if libraries.remove(plugin_id).is_some() {
            info!("Unloaded plugin {}", plugin_id);
            Ok(())
        } else {
            Err(MCPUnifiedError::validation(
                "plugin_id".to_string(),
                format!("Plugin {} not loaded", plugin_id),
                Some(json!(plugin_id)),
                Some("Plugin must be loaded before unloading".to_string()),
            ))
        }
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            registry: Arc::new(RwLock::new(PluginRegistry::new())),
            security_manager: Arc::new(SecurityManager::new()),
            plugin_loader: Arc::new(PluginLoader::new()),
            error_handler: MCPErrorHandler::default(),
            active_executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a plugin with the manager
    pub async fn register_plugin(
        &self,
        mut plugin: Box<dyn MCPPlugin>,
        config: PluginConfig,
    ) -> Result<(), MCPUnifiedError> {
        let plugin_id = config.id.clone();
        let metadata = plugin.get_metadata();

        // Initialize the plugin
        plugin.initialize(&config).await?;

        // Register in registry
        self.registry.write().await.register_plugin(metadata);

        // Store the plugin
        self.plugins.write().await.insert(plugin_id.clone(), plugin);

        info!("Registered plugin {} version {}", plugin_id, config.version);
        Ok(())
    }

    /// Load and register a plugin from path
    pub async fn load_plugin_from_path(
        &self,
        path: &str,
        config: PluginConfig,
    ) -> Result<(), MCPUnifiedError> {
        let plugin = self
            .plugin_loader
            .load_plugin_from_path(path, &config)
            .await?;
        self.register_plugin(plugin, config).await
    }

    /// Unload a plugin
    pub async fn unload_plugin(&self, plugin_id: &str) -> Result<(), MCPUnifiedError> {
        // Check for active executions
        let active_execs = self.active_executions.read().await;
        if let Some(executions) = active_execs.get(plugin_id) {
            if !executions.is_empty() {
                return Err(MCPUnifiedError::validation(
                    "plugin_unload".to_string(),
                    format!("Cannot unload plugin {} with active executions", plugin_id),
                    Some(json!(executions.len())),
                    Some("Wait for all executions to complete".to_string()),
                ));
            }
        }

        // Shutdown the plugin
        if let Some(mut plugin) = self.plugins.write().await.remove(plugin_id) {
            plugin.shutdown().await?;
        }

        // Unload from loader
        self.plugin_loader.unload_plugin(plugin_id).await?;

        info!("Unloaded plugin {}", plugin_id);
        Ok(())
    }

    /// Execute a tool from a plugin
    pub async fn execute_tool(
        &self,
        plugin_id: &str,
        tool_name: &str,
        params: &Value,
        _request_id: Option<&str>,
        _client_id: Option<&str>,
    ) -> Result<Value, MCPUnifiedError> {
        let start_time = std::time::Instant::now();

        // Get the plugin
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(plugin_id).ok_or_else(|| {
            MCPUnifiedError::validation(
                "plugin_id".to_string(),
                format!("Plugin {} not found", plugin_id),
                Some(json!(plugin_id)),
                Some("Plugin must be registered and loaded".to_string()),
            )
        })?;

        // Check if plugin supports the tool
        if !plugin.supports_tool(tool_name) {
            return Err(MCPUnifiedError::validation(
                "tool_name".to_string(),
                format!("Tool {} not supported by plugin {}", tool_name, plugin_id),
                Some(json!(tool_name)),
                Some("Check available tools for this plugin".to_string()),
            ));
        }

        // Get capabilities for security validation
        let capabilities = plugin.get_capabilities();

        // Create security context (simplified for now)
        let security_context = SecurityContext {
            allowed_permissions: vec!["execute".to_string()],
            trusted_environment: false,
            sandbox_level: SandboxLevel::Basic,
        };

        // Validate security
        self.security_manager
            .validate_execution(plugin_id, tool_name, &capabilities, &security_context)
            .await?;

        // Create execution context
        let execution_id = self
            .security_manager
            .create_execution_context(plugin_id, tool_name)
            .await?;

        // Track active execution
        {
            let mut active_execs = self.active_executions.write().await;
            let exec_context = ExecutionContext {
                plugin_id: plugin_id.to_string(),
                tool_name: tool_name.to_string(),
                start_time,
                execution_id: Uuid::parse_str(&execution_id).unwrap_or(Uuid::new_v4()),
            };
            active_execs
                .entry(plugin_id.to_string())
                .or_insert_with(Vec::new)
                .push(exec_context);
        }

        // Execute the tool
        let result = plugin.execute_tool(tool_name, params).await;

        // Cleanup execution context
        {
            let mut active_execs = self.active_executions.write().await;
            if let Some(executions) = active_execs.get_mut(plugin_id) {
                executions.retain(|ctx| ctx.execution_id.to_string() != execution_id);
            }
        }

        self.security_manager
            .cleanup_execution_context(&execution_id)
            .await;

        let duration = start_time.elapsed();
        debug!(
            "Plugin {} tool {} executed in {}ms",
            plugin_id,
            tool_name,
            duration.as_millis()
        );

        result
    }

    /// Get plugin capabilities
    pub async fn get_plugin_capabilities(&self, plugin_id: &str) -> Option<PluginCapabilities> {
        let plugins = self.plugins.read().await;
        plugins.get(plugin_id).map(|p| p.get_capabilities())
    }

    /// List all registered plugins
    pub async fn list_plugins(&self) -> Vec<String> {
        let plugins = self.plugins.read().await;
        plugins.keys().cloned().collect()
    }

    /// Get plugin registry information
    pub async fn get_registry_info(&self) -> Value {
        let registry = self.registry.read().await;
        let plugins = registry.list_all_plugins();

        json!({
            "total_plugins": plugins.len(),
            "plugins": plugins.into_iter().map(|p| json!({
                "name": p.name,
                "version": p.version,
                "description": p.description,
                "tags": p.tags
            })).collect::<Vec<_>>(),
            "categories": registry.categories.keys().cloned().collect::<Vec<_>>()
        })
    }

    /// Get active executions
    pub async fn get_active_executions(&self) -> Value {
        let active_execs = self.active_executions.read().await;
        let mut executions = Vec::new();

        for (plugin_id, execs) in active_execs.iter() {
            for exec in execs {
                executions.push(json!({
                    "plugin_id": plugin_id,
                    "tool_name": exec.tool_name,
                    "execution_id": exec.execution_id,
                    "duration_ms": exec.start_time.elapsed().as_millis()
                }));
            }
        }

        json!({
            "active_executions": executions,
            "total_count": executions.len()
        })
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockPlugin {
        name: String,
        tools: Vec<String>,
    }

    #[async_trait]
    impl MCPPlugin for MockPlugin {
        async fn initialize(&mut self, _config: &PluginConfig) -> Result<(), MCPUnifiedError> {
            Ok(())
        }

        async fn shutdown(&mut self) -> Result<(), MCPUnifiedError> {
            Ok(())
        }

        async fn execute_tool(
            &self,
            tool_name: &str,
            _params: &Value,
        ) -> Result<Value, MCPUnifiedError> {
            if self.tools.contains(&tool_name.to_string()) {
                Ok(json!({"result": format!("executed {}", tool_name)}))
            } else {
                Err(MCPUnifiedError::validation(
                    "tool_name".to_string(),
                    format!("Tool {} not supported", tool_name),
                    Some(json!(tool_name)),
                    Some("Check supported tools".to_string()),
                ))
            }
        }

        fn get_capabilities(&self) -> PluginCapabilities {
            PluginCapabilities {
                tools: self.tools.clone(),
                security_requirements: vec![SecurityRequirement::Sandboxed],
                resource_limits: ResourceLimits {
                    max_memory_bytes: 1024 * 1024,
                    max_cpu_percent: 50.0,
                    max_execution_time_ms: 5000,
                    max_concurrent_executions: 5,
                },
                execution_modes: vec![ExecutionMode::Async],
                plugin_type: PluginType::Native,
            }
        }

        fn get_metadata(&self) -> PluginMetadata {
            PluginMetadata {
                name: self.name.clone(),
                version: "1.0.0".to_string(),
                author: Some("Test Author".to_string()),
                description: "Mock plugin for testing".to_string(),
                homepage: None,
                repository: None,
                license: Some("MIT".to_string()),
                tags: vec!["test".to_string(), "mock".to_string()],
            }
        }

        fn supports_tool(&self, tool_name: &str) -> bool {
            self.tools.contains(&tool_name.to_string())
        }
    }

    #[tokio::test]
    async fn test_plugin_registration() {
        let manager = PluginManager::new();
        let plugin = Box::new(MockPlugin {
            name: "test_plugin".to_string(),
            tools: vec!["test_tool".to_string()],
        });

        let config = PluginConfig {
            id: "test_plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            parameters: HashMap::new(),
            security_context: SecurityContext {
                allowed_permissions: vec!["execute".to_string()],
                trusted_environment: false,
                sandbox_level: SandboxLevel::Basic,
            },
            resource_allocation: ResourceLimits {
                max_memory_bytes: 1024 * 1024,
                max_cpu_percent: 50.0,
                max_execution_time_ms: 5000,
                max_concurrent_executions: 5,
            },
        };

        let result = manager.register_plugin(plugin, config).await;
        assert!(result.is_ok());

        let plugins = manager.list_plugins().await;
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0], "test_plugin");
    }

    #[tokio::test]
    async fn test_tool_execution() {
        let manager = PluginManager::new();
        let plugin = Box::new(MockPlugin {
            name: "test_plugin".to_string(),
            tools: vec!["test_tool".to_string()],
        });

        let config = PluginConfig {
            id: "test_plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            parameters: HashMap::new(),
            security_context: SecurityContext {
                allowed_permissions: vec!["execute".to_string()],
                trusted_environment: false,
                sandbox_level: SandboxLevel::Basic,
            },
            resource_allocation: ResourceLimits {
                max_memory_bytes: 1024 * 1024,
                max_cpu_percent: 50.0,
                max_execution_time_ms: 5000,
                max_concurrent_executions: 5,
            },
        };

        manager
            .register_plugin(plugin, config)
            .await
            .expect("plugin registration should succeed");

        let result = manager
            .execute_tool("test_plugin", "test_tool", &json!({}), None, None)
            .await;

        assert!(result.is_ok());
        let value = result.expect("result should be ok");
        assert_eq!(value["result"], "executed test_tool");
    }

    #[tokio::test]
    async fn test_plugin_capabilities() {
        let manager = PluginManager::new();
        let plugin = Box::new(MockPlugin {
            name: "test_plugin".to_string(),
            tools: vec!["test_tool".to_string()],
        });

        let config = PluginConfig {
            id: "test_plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            parameters: HashMap::new(),
            security_context: SecurityContext {
                allowed_permissions: vec!["execute".to_string()],
                trusted_environment: false,
                sandbox_level: SandboxLevel::Basic,
            },
            resource_allocation: ResourceLimits {
                max_memory_bytes: 1024 * 1024,
                max_cpu_percent: 50.0,
                max_execution_time_ms: 5000,
                max_concurrent_executions: 5,
            },
        };

        manager
            .register_plugin(plugin, config)
            .await
            .expect("plugin registration should succeed");

        let capabilities = manager.get_plugin_capabilities("test_plugin").await;
        assert!(capabilities.is_some());
        let caps = capabilities.expect("capabilities should be some");
        assert_eq!(caps.tools.len(), 1);
        assert_eq!(caps.tools[0], "test_tool");
    }

    #[tokio::test]
    async fn test_registry_info() {
        let manager = PluginManager::new();
        let plugin = Box::new(MockPlugin {
            name: "test_plugin".to_string(),
            tools: vec!["test_tool".to_string()],
        });

        let config = PluginConfig {
            id: "test_plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            parameters: HashMap::new(),
            security_context: SecurityContext {
                allowed_permissions: vec!["execute".to_string()],
                trusted_environment: false,
                sandbox_level: SandboxLevel::Basic,
            },
            resource_allocation: ResourceLimits {
                max_memory_bytes: 1024 * 1024,
                max_cpu_percent: 50.0,
                max_execution_time_ms: 5000,
                max_concurrent_executions: 5,
            },
        };

        manager
            .register_plugin(plugin, config)
            .await
            .expect("plugin registration should succeed");

        let registry_info = manager.get_registry_info().await;
        assert_eq!(registry_info["total_plugins"], 1);
        let plugins = registry_info["plugins"]
            .as_array()
            .expect("plugins should be array");
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0]["name"], "test_plugin");
    }
}
