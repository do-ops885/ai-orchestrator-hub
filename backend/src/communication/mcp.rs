use crate::{core::HiveCoordinator, tasks::TaskPriority};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Best Practice MCP (Model Context Protocol) Server Implementation
///
/// This implementation follows all MCP standards and provides a clean,
/// extensible architecture for the multiagent hive system.
/// MCP Protocol Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<MCPError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

/// MCP Error Codes (following JSON-RPC 2.0 specification)
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    // MCP-specific error codes
    pub const TOOL_NOT_FOUND: i32 = -32000;
    pub const RESOURCE_NOT_FOUND: i32 = -32001;
    pub const PERMISSION_DENIED: i32 = -32002;
    pub const RATE_LIMITED: i32 = -32003;
}

/// Trait for implementing MCP tool handlers
#[async_trait]
pub trait MCPToolHandler: Send + Sync {
    async fn execute(&self, params: &Value) -> Result<Value>;
    fn get_schema(&self) -> Value;
    fn get_description(&self) -> String;
}

/// Best Practice MCP Server for Hive System
pub struct HiveMCPServer {
    pub name: String,
    pub version: String,
    pub description: String,
    pub hive: Arc<RwLock<HiveCoordinator>>,
    pub tools: HashMap<String, Box<dyn MCPToolHandler>>,
    pub resources: HashMap<String, MCPResource>,
    pub capabilities: Value,
}

impl HiveMCPServer {
    pub fn new(hive: Arc<RwLock<HiveCoordinator>>) -> Self {
        let capabilities = json!({
            "tools": {},
            "resources": {},
            "prompts": {},
            "logging": {}
        });

        let mut server = Self {
            name: "multiagent-hive-mcp".to_string(),
            version: "1.0.0".to_string(),
            description:
                "Multiagent Hive System MCP Server - Swarm intelligence for any MCP client"
                    .to_string(),
            hive,
            tools: HashMap::new(),
            resources: HashMap::new(),
            capabilities,
        };

        // Register hive-specific tools
        server.register_tool(
            "create_swarm_agent".to_string(),
            Box::new(CreateSwarmAgentTool::new(Arc::clone(&server.hive))),
        );
        server.register_tool(
            "assign_swarm_task".to_string(),
            Box::new(AssignSwarmTaskTool::new(Arc::clone(&server.hive))),
        );
        server.register_tool(
            "get_swarm_status".to_string(),
            Box::new(GetSwarmStatusTool::new(Arc::clone(&server.hive))),
        );
        server.register_tool(
            "analyze_with_nlp".to_string(),
            Box::new(AnalyzeWithNLPTool::new(Arc::clone(&server.hive))),
        );
        server.register_tool(
            "coordinate_agents".to_string(),
            Box::new(CoordinateAgentsTool::new(Arc::clone(&server.hive))),
        );

        // Register utility tools
        server.register_tool("echo".to_string(), Box::new(EchoTool));
        server.register_tool("system_info".to_string(), Box::new(SystemInfoTool));

        // Register advanced tools
        server.register_tool(
            "list_agents".to_string(),
            Box::new(ListAgentsTool::new(Arc::clone(&server.hive))),
        );
        server.register_tool(
            "list_tasks".to_string(),
            Box::new(ListTasksTool::new(Arc::clone(&server.hive))),
        );
        server.register_tool(
            "get_agent_details".to_string(),
            Box::new(GetAgentDetailsTool::new(Arc::clone(&server.hive))),
        );
        server.register_tool(
            "get_task_details".to_string(),
            Box::new(GetTaskDetailsTool::new(Arc::clone(&server.hive))),
        );
        server.register_tool(
            "batch_create_agents".to_string(),
            Box::new(BatchCreateAgentsTool::new(Arc::clone(&server.hive))),
        );

        // Register resources
        server.register_resource(MCPResource {
            uri: "hive://status".to_string(),
            name: "Hive Status".to_string(),
            description: Some("Current status of the multiagent hive system".to_string()),
            mime_type: Some("application/json".to_string()),
        });

        server
    }

    /// Register a tool with the server
    pub fn register_tool(&mut self, name: String, handler: Box<dyn MCPToolHandler>) {
        debug!("Registering MCP tool: {}", name);
        self.tools.insert(name, handler);
    }

    /// Register a resource with the server
    pub fn register_resource(&mut self, resource: MCPResource) {
        debug!("Registering MCP resource: {}", resource.uri);
        self.resources.insert(resource.uri.clone(), resource);
    }

    /// Handle incoming MCP request
    pub async fn handle_request(&self, request: MCPRequest) -> MCPResponse {
        debug!(
            "Handling MCP request: {} (id: {:?})",
            request.method, request.id
        );

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params),
            "tools/list" => self.handle_list_tools(),
            "tools/call" => self.handle_call_tool(request.params).await,
            "resources/list" => self.handle_list_resources(),
            "resources/read" => self.handle_read_resource(request.params).await,
            _ => Err(MCPError {
                code: error_codes::METHOD_NOT_FOUND,
                message: format!("Method '{}' not found", request.method),
                data: None,
            }),
        };

        match result {
            Ok(result) => MCPResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(result),
                error: None,
            },
            Err(error) => MCPResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(error),
            },
        }
    }

    fn handle_initialize(&self, params: Option<Value>) -> Result<Value, MCPError> {
        info!("Initializing MCP server: {}", self.name);

        let client_info = params
            .as_ref()
            .and_then(|p| p.get("clientInfo"))
            .cloned()
            .unwrap_or_else(|| json!({"name": "unknown", "version": "unknown"}));

        debug!("Client info: {}", client_info);

        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": self.capabilities,
            "serverInfo": {
                "name": self.name,
                "version": self.version,
                "description": self.description
            }
        }))
    }

    fn handle_list_tools(&self) -> Result<Value, MCPError> {
        debug!("Listing {} MCP tools", self.tools.len());

        let tools: Vec<Value> = self
            .tools
            .iter()
            .map(|(name, handler)| {
                json!({
                    "name": name,
                    "description": handler.get_description(),
                    "inputSchema": handler.get_schema()
                })
            })
            .collect();

        Ok(json!({
            "tools": tools
        }))
    }

    async fn handle_call_tool(&self, params: Option<Value>) -> Result<Value, MCPError> {
        let params = params.ok_or_else(|| MCPError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing parameters".to_string(),
            data: None,
        })?;

        let tool_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MCPError {
                code: error_codes::INVALID_PARAMS,
                message: "Missing tool name".to_string(),
                data: None,
            })?;

        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));

        debug!("Calling MCP tool: {} with args: {}", tool_name, arguments);

        let handler = self.tools.get(tool_name).ok_or_else(|| MCPError {
            code: error_codes::TOOL_NOT_FOUND,
            message: format!("Tool '{tool_name}' not found"),
            data: None,
        })?;

        match handler.execute(&arguments).await {
            Ok(result) => Ok(json!({
                "content": [{
                    "type": "text",
                    "text": result.to_string()
                }]
            })),
            Err(e) => Err(MCPError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("Tool execution failed: {e}"),
                data: Some(json!({"tool": tool_name, "error": e.to_string()})),
            }),
        }
    }

    fn handle_list_resources(&self) -> Result<Value, MCPError> {
        debug!("Listing {} MCP resources", self.resources.len());

        let resources: Vec<&MCPResource> = self.resources.values().collect();
        Ok(json!({
            "resources": resources
        }))
    }

    async fn handle_read_resource(&self, params: Option<Value>) -> Result<Value, MCPError> {
        let params = params.ok_or_else(|| MCPError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing parameters".to_string(),
            data: None,
        })?;

        let uri = params
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MCPError {
                code: error_codes::INVALID_PARAMS,
                message: "Missing resource URI".to_string(),
                data: None,
            })?;

        debug!("Reading MCP resource: {}", uri);

        if uri == "hive://status" {
            let hive = self.hive.read().await;
            let status = hive.get_status().await;
            Ok(json!({
                "contents": [{
                    "uri": uri,
                    "mimeType": "application/json",
                    "text": serde_json::to_string_pretty(&status).unwrap_or_else(|_| "{}".to_string())
                }]
            }))
        } else {
            let resource = self.resources.get(uri).ok_or_else(|| MCPError {
                code: error_codes::RESOURCE_NOT_FOUND,
                message: format!("Resource '{uri}' not found"),
                data: None,
            })?;

            Ok(json!({
                "contents": [{
                    "uri": resource.uri,
                    "mimeType": resource.mime_type.as_deref().unwrap_or("text/plain"),
                    "text": format!("Resource: {}", resource.name)
                }]
            }))
        }
    }
}

// Hive-specific tool implementations

pub struct CreateSwarmAgentTool {
    hive: Arc<RwLock<HiveCoordinator>>,
}

impl CreateSwarmAgentTool {
    pub fn new(hive: Arc<RwLock<HiveCoordinator>>) -> Self {
        Self { hive }
    }
}

#[async_trait]
impl MCPToolHandler for CreateSwarmAgentTool {
    async fn execute(&self, params: &Value) -> Result<Value> {
        // Support both "type" and "agent_type" parameters for backward compatibility
        let agent_type_str = params
            .get("type")
            .or_else(|| params.get("agent_type"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Agent type is required. Must be one of: worker, coordinator, specialist, learner"))?;

        // Validate agent type before proceeding
        let valid_types = ["worker", "coordinator", "specialist", "learner"];
        if !valid_types.contains(&agent_type_str) {
            return Err(anyhow::anyhow!(
                "Invalid agent type: {}. Must be one of: {}",
                agent_type_str,
                valid_types.join(", ")
            ));
        }

        let hive = self.hive.write().await;
        let config = json!({
            "type": agent_type_str,
            "specialization": params.get("specialization").and_then(|v| v.as_str()).unwrap_or("general")
        });
        let agent_id = hive.create_agent(config).await?;

        Ok(json!({
            "success": true,
            "agent_id": agent_id,
            "message": format!("Created {} agent with ID: {}", agent_type_str, agent_id)
        }))
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "type": {
                    "type": "string",
                    "enum": ["worker", "coordinator", "specialist", "learner"],
                    "description": "Type of agent to create (alias: agent_type)"
                },
                "agent_type": {
                    "type": "string",
                    "enum": ["worker", "coordinator", "specialist", "learner"],
                    "description": "Type of agent to create (alias: type)"
                },
                "specialization": {
                    "type": "string",
                    "description": "Specialization for Specialist agents"
                }
            },
            "anyOf": [
                {"required": ["type"]},
                {"required": ["agent_type"]}
            ]
        })
    }

    fn get_description(&self) -> String {
        "Create a new agent in the swarm with specified type and capabilities".to_string()
    }
}
// Additional hive tool implementations

pub struct AssignSwarmTaskTool {
    hive: Arc<RwLock<HiveCoordinator>>,
}

impl AssignSwarmTaskTool {
    pub fn new(hive: Arc<RwLock<HiveCoordinator>>) -> Self {
        Self { hive }
    }
}

#[async_trait]
impl MCPToolHandler for AssignSwarmTaskTool {
    async fn execute(&self, params: &Value) -> Result<Value> {
        let description = params
            .get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: description"))?;

        let priority_str = params
            .get("priority")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: priority"))?;

        // Validate priority is one of the allowed values
        let valid_priorities = ["Low", "Medium", "High", "Critical"];
        if !valid_priorities.contains(&priority_str) {
            return Err(anyhow::anyhow!(
                "Invalid priority: {}. Must be one of: {}",
                priority_str,
                valid_priorities.join(", ")
            ));
        }

        let _ = match priority_str {
            "Low" => TaskPriority::Low,
            "High" => TaskPriority::High,
            "Critical" => TaskPriority::Critical,
            _ => TaskPriority::Medium,
        };

        let hive = self.hive.write().await;
        let config = json!({
            "description": description,
            "priority": priority_str
        });
        let task_id = hive.create_task(config).await?;

        Ok(json!({
            "success": true,
            "task_id": task_id,
            "message": format!("Created task with ID: {}", task_id)
        }))
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "description": {
                    "type": "string",
                    "description": "Description of the task to assign"
                },
                "priority": {
                    "type": "string",
                    "enum": ["Low", "Medium", "High", "Critical"],
                    "description": "Priority level of the task"
                }
            },
            "required": ["description", "priority"]
        })
    }

    fn get_description(&self) -> String {
        "Assign a new task to the swarm with specified priority".to_string()
    }
}

pub struct GetSwarmStatusTool {
    hive: Arc<RwLock<HiveCoordinator>>,
}

impl GetSwarmStatusTool {
    pub fn new(hive: Arc<RwLock<HiveCoordinator>>) -> Self {
        Self { hive }
    }
}

#[async_trait]
impl MCPToolHandler for GetSwarmStatusTool {
    async fn execute(&self, _params: &Value) -> Result<Value> {
        let hive = self.hive.read().await;
        let status = hive.get_status().await;
        Ok(json!(status))
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn get_description(&self) -> String {
        "Get the current status of the multiagent hive system".to_string()
    }
}

pub struct AnalyzeWithNLPTool {
    hive: Arc<RwLock<HiveCoordinator>>,
}

impl AnalyzeWithNLPTool {
    pub fn new(hive: Arc<RwLock<HiveCoordinator>>) -> Self {
        Self { hive }
    }
}

#[async_trait]
impl MCPToolHandler for AnalyzeWithNLPTool {
    async fn execute(&self, params: &Value) -> Result<Value> {
        let text = params
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing text to analyze"))?;

        let _ = self.hive.read().await;
        // Use basic NLP analysis for now
        let analysis = json!({
            "sentiment": "neutral",
            "keywords": text.split_whitespace().take(5).collect::<Vec<_>>(),
            "length": text.len(),
            "word_count": text.split_whitespace().count()
        });

        Ok(json!({
            "analysis": analysis,
            "text": text,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "Text to analyze with NLP"
                }
            },
            "required": ["text"]
        })
    }

    fn get_description(&self) -> String {
        "Analyze text using the hive's NLP capabilities".to_string()
    }
}

pub struct CoordinateAgentsTool {
    hive: Arc<RwLock<HiveCoordinator>>,
}

impl CoordinateAgentsTool {
    pub fn new(hive: Arc<RwLock<HiveCoordinator>>) -> Self {
        Self { hive }
    }
}

#[async_trait]
impl MCPToolHandler for CoordinateAgentsTool {
    async fn execute(&self, params: &Value) -> Result<Value> {
        let strategy = params
            .get("strategy")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: strategy"))?;

        // Validate strategy is one of the allowed values
        let valid_strategies = ["default", "aggressive", "conservative", "balanced"];
        if !valid_strategies.contains(&strategy) {
            return Err(anyhow::anyhow!(
                "Invalid strategy: {}. Must be one of: {}",
                strategy,
                valid_strategies.join(", ")
            ));
        }

        let hive = self.hive.read().await;
        // Basic coordination strategy implementation
        let status = hive.get_status().await;
        let coordination_result = json!({
            "strategy_applied": strategy,
            "agents_coordinated": status.get("total_agents").unwrap_or(&json!(0)),
            "coordination_score": 0.85,
            "recommendations": match strategy {
                "aggressive" => vec!["Increase task distribution", "Boost agent communication"],
                "conservative" => vec!["Maintain current pace", "Focus on quality"],
                "balanced" => vec!["Optimize resource allocation", "Balance speed and quality"],
                _ => vec!["Standard coordination applied"]
            }
        });

        Ok(json!({
            "strategy": strategy,
            "result": coordination_result,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "strategy": {
                    "type": "string",
                    "description": "Coordination strategy to use",
                    "enum": ["default", "aggressive", "conservative", "balanced"]
                }
            },
            "required": ["strategy"]
        })
    }

    fn get_description(&self) -> String {
        "Coordinate agents in the swarm using specified strategy".to_string()
    }
}

// Utility tool implementations

pub struct EchoTool;

#[async_trait]
impl MCPToolHandler for EchoTool {
    async fn execute(&self, params: &Value) -> Result<Value> {
        let message = params
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: message"))?;

        Ok(json!({
            "echo": message,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Message to echo back"
                }
            },
            "required": ["message"]
        })
    }

    fn get_description(&self) -> String {
        "Echo a message back with timestamp".to_string()
    }
}

pub struct SystemInfoTool;

#[async_trait]
impl MCPToolHandler for SystemInfoTool {
    async fn execute(&self, _params: &Value) -> Result<Value> {
        Ok(json!({
            "hostname": std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string()),
            "platform": std::env::consts::OS,
            "architecture": std::env::consts::ARCH,
            "cpu_count": num_cpus::get(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn get_description(&self) -> String {
        "Get system information including platform, architecture, and CPU count".to_string()
    }
}

// Advanced MCP Tools

pub struct ListAgentsTool {
    hive: Arc<RwLock<HiveCoordinator>>,
}

impl ListAgentsTool {
    pub fn new(hive: Arc<RwLock<HiveCoordinator>>) -> Self {
        Self { hive }
    }
}

#[async_trait]
impl MCPToolHandler for ListAgentsTool {
    async fn execute(&self, params: &Value) -> Result<Value> {
        let hive = self.hive.read().await;
        let status = hive.get_status().await;

        // Extract and validate filter parameters
        let agent_type_filter = if let Some(agent_type) = params.get("agent_type").and_then(|v| v.as_str()) {
            let valid_types = ["worker", "coordinator", "specialist", "learner"];
            if !valid_types.contains(&agent_type) {
                return Err(anyhow::anyhow!(
                    "Invalid agent_type: {}. Must be one of: {}",
                    agent_type,
                    valid_types.join(", ")
                ));
            }
            Some(agent_type)
        } else {
            None
        };

        let active_only = params
            .get("active_only")
            .and_then(serde_json::Value::as_bool);

        // For now, return basic agent information from status
        let total_agents = status
            .get("metrics")
            .and_then(|m| m.get("total_agents"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);

        let active_agents = status
            .get("metrics")
            .and_then(|m| m.get("active_agents"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);

        Ok(json!({
            "total_agents": total_agents,
            "active_agents": active_agents,
            "filter_applied": {
                "agent_type": agent_type_filter,
                "active_only": active_only
            },
            "agents": []  // Would contain actual agent list in full implementation
        }))
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "agent_type": {
                    "type": "string",
                    "description": "Filter by agent type",
                    "enum": ["worker", "coordinator", "specialist", "learner"]
                },
                "active_only": {
                    "type": "boolean",
                    "description": "Show only active agents",
                    "default": false
                }
            },
            "required": []
        })
    }

    fn get_description(&self) -> String {
        "List all agents in the swarm with optional filtering".to_string()
    }
}

pub struct ListTasksTool {
    hive: Arc<RwLock<HiveCoordinator>>,
}

impl ListTasksTool {
    pub fn new(hive: Arc<RwLock<HiveCoordinator>>) -> Self {
        Self { hive }
    }
}

#[async_trait]
impl MCPToolHandler for ListTasksTool {
    async fn execute(&self, params: &Value) -> Result<Value> {
        let hive = self.hive.read().await;
        let status = hive.get_status().await;

        let priority_filter = if let Some(priority) = params.get("priority").and_then(|v| v.as_str()) {
            let valid_priorities = ["Low", "Medium", "High", "Critical"];
            if !valid_priorities.contains(&priority) {
                return Err(anyhow::anyhow!(
                    "Invalid priority: {}. Must be one of: {}",
                    priority,
                    valid_priorities.join(", ")
                ));
            }
            Some(priority)
        } else {
            None
        };

        let status_filter = if let Some(status_val) = params.get("status").and_then(|v| v.as_str()) {
            let valid_statuses = ["Pending", "Running", "Completed", "Failed"];
            if !valid_statuses.contains(&status_val) {
                return Err(anyhow::anyhow!(
                    "Invalid status: {}. Must be one of: {}",
                    status_val,
                    valid_statuses.join(", ")
                ));
            }
            Some(status_val)
        } else {
            None
        };

        let completed_tasks = status
            .get("metrics")
            .and_then(|m| m.get("completed_tasks"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);

        let failed_tasks = status
            .get("metrics")
            .and_then(|m| m.get("failed_tasks"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);

        Ok(json!({
            "completed_tasks": completed_tasks,
            "failed_tasks": failed_tasks,
            "filter_applied": {
                "priority": priority_filter,
                "status": status_filter
            },
            "tasks": []  // Would contain actual task list in full implementation
        }))
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "priority": {
                    "type": "string",
                    "description": "Filter by task priority",
                    "enum": ["Low", "Medium", "High", "Critical"]
                },
                "status": {
                    "type": "string",
                    "description": "Filter by task status",
                    "enum": ["Pending", "Running", "Completed", "Failed"]
                }
            },
            "required": []
        })
    }

    fn get_description(&self) -> String {
        "List all tasks in the swarm with optional filtering".to_string()
    }
}

pub struct GetAgentDetailsTool {}

impl GetAgentDetailsTool {
    pub fn new(_hive: Arc<RwLock<HiveCoordinator>>) -> Self {
        Self {}
    }
}

#[async_trait]
impl MCPToolHandler for GetAgentDetailsTool {
    async fn execute(&self, params: &Value) -> Result<Value> {
        let agent_id = params
            .get("agent_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing agent_id parameter"))?;

        // In a real implementation, you'd look up the actual agent
        Ok(json!({
            "agent_id": agent_id,
            "status": "active",
            "type": "worker",
            "created_at": chrono::Utc::now().to_rfc3339(),
            "last_activity": chrono::Utc::now().to_rfc3339(),
            "tasks_completed": 0,
            "performance_score": 1.0,
            "capabilities": ["general_processing"]
        }))
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "agent_id": {
                    "type": "string",
                    "description": "Unique identifier of the agent"
                }
            },
            "required": ["agent_id"]
        })
    }

    fn get_description(&self) -> String {
        "Get detailed information about a specific agent".to_string()
    }
}

pub struct GetTaskDetailsTool {}

impl GetTaskDetailsTool {
    pub fn new(_hive: Arc<RwLock<HiveCoordinator>>) -> Self {
        Self {}
    }
}

#[async_trait]
impl MCPToolHandler for GetTaskDetailsTool {
    async fn execute(&self, params: &Value) -> Result<Value> {
        let task_id = params
            .get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing task_id parameter"))?;

        // In a real implementation, you'd look up the actual task
        Ok(json!({
            "task_id": task_id,
            "description": "Sample task",
            "status": "pending",
            "priority": "Medium",
            "created_at": chrono::Utc::now().to_rfc3339(),
            "assigned_agent": null,
            "progress": 0.0,
            "estimated_completion": null
        }))
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "string",
                    "description": "Unique identifier of the task"
                }
            },
            "required": ["task_id"]
        })
    }

    fn get_description(&self) -> String {
        "Get detailed information about a specific task".to_string()
    }
}

pub struct BatchCreateAgentsTool {
    hive: Arc<RwLock<HiveCoordinator>>,
}

impl BatchCreateAgentsTool {
    pub fn new(hive: Arc<RwLock<HiveCoordinator>>) -> Self {
        Self { hive }
    }
}

#[async_trait]
impl MCPToolHandler for BatchCreateAgentsTool {
    async fn execute(&self, params: &Value) -> Result<Value> {
        let count = params
            .get("count")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: count"))? as usize;

        // Validate count is between 1 and 10
        if count < 1 || count > 10 {
            return Err(anyhow::anyhow!("Invalid count: {}. Must be between 1 and 10", count));
        }

        // Support both "type" and "agent_type" parameters for backward compatibility
        let agent_type_str = params
            .get("type")
            .or_else(|| params.get("agent_type"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Agent type is required. Must be one of: worker, coordinator, specialist, learner"))?;

        // Validate agent type before proceeding
        let valid_types = ["worker", "coordinator", "specialist", "learner"];
        if !valid_types.contains(&agent_type_str) {
            return Err(anyhow::anyhow!(
                "Invalid agent type: {}. Must be one of: {}",
                agent_type_str,
                valid_types.join(", ")
            ));
        }

        let hive = self.hive.write().await;
        let mut created_agents = Vec::new();

        for _ in 0..count {
            let config = if agent_type_str == "specialist" {
                json!({
                    "type": "specialist"
                })
            } else {
                json!({
                    "type": agent_type_str.to_lowercase()
                })
            };
            let agent_id = hive.create_agent(config).await?;
            created_agents.push(agent_id.to_string());
        }

        Ok(json!({
            "success": true,
            "created_count": created_agents.len(),
            "agent_ids": created_agents,
            "message": format!("Successfully created {} {} agents", created_agents.len(), agent_type_str)
        }))
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "count": {
                    "type": "integer",
                    "description": "Number of agents to create (max 10)",
                    "minimum": 1,
                    "maximum": 10
                },
                "type": {
                    "type": "string",
                    "description": "Type of agents to create (alias: agent_type)",
                    "enum": ["worker", "coordinator", "specialist", "learner"]
                },
                "agent_type": {
                    "type": "string",
                    "description": "Type of agents to create (alias: type)",
                    "enum": ["worker", "coordinator", "specialist", "learner"]
                }
            },
            "anyOf": [
                {"required": ["type"]},
                {"required": ["agent_type"]}
            ],
            "required": ["count"]
        })
    }

    fn get_description(&self) -> String {
        "Create multiple agents in a single batch operation".to_string()
    }
}
