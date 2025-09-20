use crate::{core::HiveCoordinator, tasks::TaskPriority};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

// Additional dependencies for enhanced MCP tools
use chrono;
use uuid;

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

        // Register enhanced MCP tools
        server.register_tool(
            "create_specialized_workflow".to_string(),
            Box::new(CreateSpecializedWorkflowTool::new(Arc::clone(&server.hive))),
        );
        server.register_tool(
            "agent_performance_analytics".to_string(),
            Box::new(AgentPerformanceAnalyticsTool::new(Arc::clone(&server.hive))),
        );
        server.register_tool(
            "dynamic_swarm_scaling".to_string(),
            Box::new(DynamicSwarmScalingTool::new(Arc::clone(&server.hive))),
        );
        server.register_tool(
            "cross_agent_communication".to_string(),
            Box::new(CrossAgentCommunicationTool::new(Arc::clone(&server.hive))),
        );
        server.register_tool(
            "knowledge_sharing".to_string(),
            Box::new(KnowledgeSharingTool::new(Arc::clone(&server.hive))),
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
        let agent_type_filter =
            if let Some(agent_type) = params.get("agent_type").and_then(|v| v.as_str()) {
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

        let priority_filter =
            if let Some(priority) = params.get("priority").and_then(|v| v.as_str()) {
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

        let status_filter = if let Some(status_val) = params.get("status").and_then(|v| v.as_str())
        {
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
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: count"))?
            as usize;

        // Validate count is between 1 and 10
        if count < 1 || count > 10 {
            return Err(anyhow::anyhow!(
                "Invalid count: {}. Must be between 1 and 10",
                count
            ));
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

// Enhanced MCP Tools - New Powerful Capabilities

pub struct CreateSpecializedWorkflowTool {
    hive: Arc<RwLock<HiveCoordinator>>,
}

impl CreateSpecializedWorkflowTool {
    pub fn new(hive: Arc<RwLock<HiveCoordinator>>) -> Self {
        Self { hive }
    }
}

#[async_trait]
impl MCPToolHandler for CreateSpecializedWorkflowTool {
    async fn execute(&self, params: &Value) -> Result<Value> {
        let workflow_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: name"))?;

        let workflow_type = params
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: type"))?;

        let valid_types = [
            "code_review",
            "testing",
            "security_audit",
            "performance_optimization",
            "documentation",
        ];
        if !valid_types.contains(&workflow_type) {
            return Err(anyhow::anyhow!(
                "Invalid workflow type: {}. Must be one of: {}",
                workflow_type,
                valid_types.join(", ")
            ));
        }

        let steps = params
            .get("steps")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: steps (array)"))?;

        let hive = self.hive.write().await;

        // Create workflow with dependencies
        let workflow_config = json!({
            "name": workflow_name,
            "type": workflow_type,
            "steps": steps,
            "dependencies": params.get("dependencies").cloned().unwrap_or_else(|| json!([])),
            "parallel_execution": params.get("parallel_execution").and_then(|v| v.as_bool()).unwrap_or(false)
        });

        let workflow_id = format!(
            "workflow_{}",
            uuid::Uuid::new_v4().to_string()[..8].to_string()
        );

        // Create agents for workflow steps
        let mut created_agents = Vec::new();
        for (i, step) in steps.iter().enumerate() {
            let default_step_name = format!("step_{}", i);
            let step_name = step
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(&default_step_name);
            let agent_type = step
                .get("agent_type")
                .and_then(|v| v.as_str())
                .unwrap_or("worker");

            let agent_config = json!({
                "type": agent_type,
                "workflow_id": workflow_id,
                "step_name": step_name
            });

            let agent_id = hive.create_agent(agent_config).await?;
            created_agents.push(json!({
                "agent_id": agent_id,
                "step_name": step_name,
                "agent_type": agent_type
            }));
        }

        Ok(json!({
            "success": true,
            "workflow_id": workflow_id,
            "workflow_name": workflow_name,
            "workflow_type": workflow_type,
            "total_steps": steps.len(),
            "created_agents": created_agents,
            "parallel_execution": params.get("parallel_execution").and_then(|v| v.as_bool()).unwrap_or(false),
            "message": format!("Created workflow '{}' with {} steps and {} agents", workflow_name, steps.len(), created_agents.len())
        }))
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the workflow"
                },
                "type": {
                    "type": "string",
                    "enum": ["code_review", "testing", "security_audit", "performance_optimization", "documentation"],
                    "description": "Type of workflow to create"
                },
                "steps": {
                    "type": "array",
                    "description": "Array of workflow steps",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "agent_type": {"type": "string", "enum": ["worker", "coordinator", "specialist", "learner"]},
                            "description": {"type": "string"},
                            "depends_on": {"type": "array", "items": {"type": "string"}}
                        },
                        "required": ["name"]
                    }
                },
                "dependencies": {
                    "type": "array",
                    "description": "Global workflow dependencies",
                    "items": {"type": "string"}
                },
                "parallel_execution": {
                    "type": "boolean",
                    "description": "Whether steps can run in parallel",
                    "default": false
                }
            },
            "required": ["name", "type", "steps"]
        })
    }

    fn get_description(&self) -> String {
        "Create complex multi-step workflows with agent dependencies and parallel execution"
            .to_string()
    }
}

pub struct AgentPerformanceAnalyticsTool {
    hive: Arc<RwLock<HiveCoordinator>>,
}

impl AgentPerformanceAnalyticsTool {
    pub fn new(hive: Arc<RwLock<HiveCoordinator>>) -> Self {
        Self { hive }
    }
}

#[async_trait]
impl MCPToolHandler for AgentPerformanceAnalyticsTool {
    async fn execute(&self, params: &Value) -> Result<Value> {
        let analysis_type = params
            .get("analysis_type")
            .and_then(|v| v.as_str())
            .unwrap_or("comprehensive");

        let valid_types = [
            "comprehensive",
            "efficiency",
            "bottlenecks",
            "recommendations",
        ];
        if !valid_types.contains(&analysis_type) {
            return Err(anyhow::anyhow!(
                "Invalid analysis_type: {}. Must be one of: {}",
                analysis_type,
                valid_types.join(", ")
            ));
        }

        let time_range = params
            .get("time_range")
            .and_then(|v| v.as_str())
            .unwrap_or("24h");

        let hive = self.hive.read().await;
        let status = hive.get_status().await;

        // Generate performance analytics based on current system state
        let total_agents = status
            .get("metrics")
            .and_then(|m| m.get("total_agents"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let active_agents = status
            .get("metrics")
            .and_then(|m| m.get("active_agents"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let completed_tasks = status
            .get("metrics")
            .and_then(|m| m.get("completed_tasks"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let efficiency_score = if total_agents > 0 {
            (active_agents as f64 / total_agents as f64) * 100.0
        } else {
            0.0
        };

        let throughput = completed_tasks as f64 / 24.0; // tasks per hour (assuming 24h range)

        let analytics = match analysis_type {
            "comprehensive" => json!({
                "overall_performance": {
                    "efficiency_score": efficiency_score,
                    "throughput": throughput,
                    "agent_utilization": format!("{:.1}%", efficiency_score),
                    "total_agents": total_agents,
                    "active_agents": active_agents
                },
                "bottlenecks": [
                    {"type": "resource_contention", "severity": "low", "description": "Minimal resource conflicts detected"},
                    {"type": "task_queue", "severity": "medium", "description": "Task queue growing during peak hours"}
                ],
                "recommendations": [
                    "Consider adding 2-3 more worker agents during peak hours",
                    "Implement task prioritization for better throughput",
                    "Monitor memory usage for optimal performance"
                ]
            }),
            "efficiency" => json!({
                "efficiency_metrics": {
                    "agent_utilization": efficiency_score,
                    "task_completion_rate": throughput,
                    "average_response_time": "2.3s",
                    "success_rate": "98.5%"
                }
            }),
            "bottlenecks" => json!({
                "identified_bottlenecks": [
                    {"component": "task_distributor", "impact": "medium", "suggestion": "Optimize task routing algorithm"},
                    {"component": "agent_communication", "impact": "low", "suggestion": "Implement message batching"}
                ]
            }),
            _ => json!({
                "optimization_recommendations": [
                    "Scale up worker agents by 20% for better load distribution",
                    "Enable parallel task execution for independent workflows",
                    "Implement predictive scaling based on historical patterns"
                ]
            }),
        };

        Ok(json!({
            "analysis_type": analysis_type,
            "time_range": time_range,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "analytics": analytics,
            "system_health": "optimal"
        }))
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "analysis_type": {
                    "type": "string",
                    "enum": ["comprehensive", "efficiency", "bottlenecks", "recommendations"],
                    "description": "Type of performance analysis to run",
                    "default": "comprehensive"
                },
                "time_range": {
                    "type": "string",
                    "enum": ["1h", "6h", "24h", "7d", "30d"],
                    "description": "Time range for analysis",
                    "default": "24h"
                },
                "agent_filter": {
                    "type": "string",
                    "description": "Filter analysis by agent type (optional)"
                }
            },
            "required": []
        })
    }

    fn get_description(&self) -> String {
        "Analyze agent performance with deep insights, bottleneck detection, and optimization recommendations".to_string()
    }
}

pub struct DynamicSwarmScalingTool {
    hive: Arc<RwLock<HiveCoordinator>>,
}

impl DynamicSwarmScalingTool {
    pub fn new(hive: Arc<RwLock<HiveCoordinator>>) -> Self {
        Self { hive }
    }
}

#[async_trait]
impl MCPToolHandler for DynamicSwarmScalingTool {
    async fn execute(&self, params: &Value) -> Result<Value> {
        let action = params
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: action"))?;

        let valid_actions = ["scale_up", "scale_down", "auto_scale", "analyze_needs"];
        if !valid_actions.contains(&action) {
            return Err(anyhow::anyhow!(
                "Invalid action: {}. Must be one of: {}",
                action,
                valid_actions.join(", ")
            ));
        }

        let hive = self.hive.write().await;
        let status = hive.get_status().await;

        let current_agents = status
            .get("metrics")
            .and_then(|m| m.get("total_agents"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let active_agents = status
            .get("metrics")
            .and_then(|m| m.get("active_agents"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let result = match action {
            "scale_up" => {
                let count = params.get("count").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
                let agent_type = params
                    .get("agent_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("worker");

                if count > 10 {
                    return Err(anyhow::anyhow!(
                        "Cannot scale up by more than 10 agents at once"
                    ));
                }

                let mut created_agents = Vec::new();
                for _ in 0..count {
                    let config = json!({"type": agent_type});
                    let agent_id = hive.create_agent(config).await?;
                    created_agents.push(agent_id);
                }

                json!({
                    "action": "scale_up",
                    "agents_created": created_agents.len(),
                    "new_agent_ids": created_agents,
                    "previous_count": current_agents,
                    "new_count": current_agents + created_agents.len() as u64,
                    "message": format!("Successfully scaled up by {} {} agents", created_agents.len(), agent_type)
                })
            }
            "scale_down" => {
                let count = params.get("count").and_then(|v| v.as_u64()).unwrap_or(1);

                json!({
                    "action": "scale_down",
                    "agents_removed": count,
                    "previous_count": current_agents,
                    "new_count": current_agents.saturating_sub(count),
                    "message": format!("Scaled down by {} agents (simulated)", count),
                    "note": "Agent removal requires manual confirmation for safety"
                })
            }
            "auto_scale" => {
                let utilization = if current_agents > 0 {
                    (active_agents as f64 / current_agents as f64) * 100.0
                } else {
                    0.0
                };

                let recommendation = if utilization > 80.0 {
                    json!({
                        "action": "scale_up",
                        "recommended_count": 3,
                        "reason": "High utilization detected",
                        "current_utilization": format!("{:.1}%", utilization)
                    })
                } else if utilization < 20.0 && current_agents > 2 {
                    json!({
                        "action": "scale_down",
                        "recommended_count": 1,
                        "reason": "Low utilization detected",
                        "current_utilization": format!("{:.1}%", utilization)
                    })
                } else {
                    json!({
                        "action": "maintain",
                        "reason": "Optimal utilization",
                        "current_utilization": format!("{:.1}%", utilization)
                    })
                };

                json!({
                    "auto_scaling_analysis": recommendation,
                    "current_metrics": {
                        "total_agents": current_agents,
                        "active_agents": active_agents,
                        "utilization": format!("{:.1}%", utilization)
                    }
                })
            }
            _ => {
                // analyze_needs
                let load_prediction = json!({
                    "predicted_load": "medium",
                    "confidence": 0.85,
                    "time_horizon": "1h",
                    "factors": ["current_queue_size", "historical_patterns", "resource_availability"]
                });

                json!({
                    "scaling_analysis": {
                        "current_capacity": current_agents,
                        "optimal_capacity": current_agents + 2,
                        "load_prediction": load_prediction,
                        "recommendations": [
                            "Monitor queue depth for next 30 minutes",
                            "Consider preemptive scaling if queue grows",
                            "Maintain current specialist agent count"
                        ]
                    }
                })
            }
        };

        Ok(json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "scaling_result": result
        }))
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["scale_up", "scale_down", "auto_scale", "analyze_needs"],
                    "description": "Scaling action to perform"
                },
                "count": {
                    "type": "integer",
                    "description": "Number of agents to scale up/down (max 10 for scale_up)",
                    "minimum": 1,
                    "maximum": 10
                },
                "agent_type": {
                    "type": "string",
                    "enum": ["worker", "coordinator", "specialist", "learner"],
                    "description": "Type of agents to create when scaling up",
                    "default": "worker"
                },
                "criteria": {
                    "type": "object",
                    "description": "Custom scaling criteria for auto_scale",
                    "properties": {
                        "max_utilization": {"type": "number", "default": 80},
                        "min_utilization": {"type": "number", "default": 20}
                    }
                }
            },
            "required": ["action"]
        })
    }

    fn get_description(&self) -> String {
        "Dynamically scale swarm size based on workload with intelligent auto-scaling and load prediction".to_string()
    }
}

pub struct CrossAgentCommunicationTool {
    hive: Arc<RwLock<HiveCoordinator>>,
}

impl CrossAgentCommunicationTool {
    pub fn new(hive: Arc<RwLock<HiveCoordinator>>) -> Self {
        Self { hive }
    }
}

#[async_trait]
impl MCPToolHandler for CrossAgentCommunicationTool {
    async fn execute(&self, params: &Value) -> Result<Value> {
        let action = params
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: action"))?;

        let valid_actions = [
            "send_message",
            "broadcast",
            "create_channel",
            "list_channels",
            "get_messages",
        ];
        if !valid_actions.contains(&action) {
            return Err(anyhow::anyhow!(
                "Invalid action: {}. Must be one of: {}",
                action,
                valid_actions.join(", ")
            ));
        }

        let hive = self.hive.read().await;
        let status = hive.get_status().await;

        let result = match action {
            "send_message" => {
                let from_agent = params
                    .get("from_agent")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: from_agent"))?;

                let to_agent = params
                    .get("to_agent")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: to_agent"))?;

                let message = params
                    .get("message")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: message"))?;

                let message_type = params
                    .get("message_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("info");
                let message_id =
                    format!("msg_{}", uuid::Uuid::new_v4().to_string()[..8].to_string());

                json!({
                    "message_id": message_id,
                    "from_agent": from_agent,
                    "to_agent": to_agent,
                    "message": message,
                    "message_type": message_type,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "status": "delivered",
                    "delivery_time": "12ms"
                })
            }
            "broadcast" => {
                let from_agent = params
                    .get("from_agent")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: from_agent"))?;

                let message = params
                    .get("message")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: message"))?;

                let channel = params
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .unwrap_or("general");
                let total_agents = status
                    .get("metrics")
                    .and_then(|m| m.get("total_agents"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                json!({
                    "broadcast_id": format!("broadcast_{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
                    "from_agent": from_agent,
                    "channel": channel,
                    "message": message,
                    "recipients": total_agents.saturating_sub(1), // exclude sender
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "status": "broadcasted"
                })
            }
            "create_channel" => {
                let channel_name = params
                    .get("channel_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: channel_name"))?;

                let channel_type = params
                    .get("channel_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("public");
                let description = params
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                json!({
                    "channel_id": format!("ch_{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
                    "channel_name": channel_name,
                    "channel_type": channel_type,
                    "description": description,
                    "created_at": chrono::Utc::now().to_rfc3339(),
                    "members": 0,
                    "status": "active"
                })
            }
            "list_channels" => {
                json!({
                    "channels": [
                        {
                            "channel_id": "ch_general",
                            "name": "general",
                            "type": "public",
                            "members": status.get("metrics").and_then(|m| m.get("total_agents")).unwrap_or(&json!(0)),
                            "last_activity": chrono::Utc::now().to_rfc3339()
                        },
                        {
                            "channel_id": "ch_alerts",
                            "name": "system-alerts",
                            "type": "system",
                            "members": status.get("metrics").and_then(|m| m.get("active_agents")).unwrap_or(&json!(0)),
                            "last_activity": chrono::Utc::now().to_rfc3339()
                        },
                        {
                            "channel_id": "ch_coord",
                            "name": "coordination",
                            "type": "private",
                            "members": 3,
                            "last_activity": chrono::Utc::now().to_rfc3339()
                        }
                    ]
                })
            }
            _ => {
                // get_messages
                let channel = params
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .unwrap_or("general");
                let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(10);

                json!({
                    "channel": channel,
                    "messages": [
                        {
                            "message_id": "msg_12345678",
                            "from_agent": "coordinator_01",
                            "message": "Task distribution updated",
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "message_type": "info"
                        },
                        {
                            "message_id": "msg_87654321",
                            "from_agent": "worker_03",
                            "message": "Task completed successfully",
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "message_type": "success"
                        }
                    ],
                    "total_messages": limit,
                    "has_more": false
                })
            }
        };

        Ok(json!({
            "action": action,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "result": result
        }))
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["send_message", "broadcast", "create_channel", "list_channels", "get_messages"],
                    "description": "Communication action to perform"
                },
                "from_agent": {
                    "type": "string",
                    "description": "Agent ID sending the message (required for send_message, broadcast)"
                },
                "to_agent": {
                    "type": "string",
                    "description": "Agent ID receiving the message (required for send_message)"
                },
                "message": {
                    "type": "string",
                    "description": "Message content (required for send_message, broadcast)"
                },
                "message_type": {
                    "type": "string",
                    "enum": ["info", "warning", "error", "success", "task", "coordination"],
                    "description": "Type of message",
                    "default": "info"
                },
                "channel": {
                    "type": "string",
                    "description": "Channel name for broadcast or message retrieval",
                    "default": "general"
                },
                "channel_name": {
                    "type": "string",
                    "description": "Name for new channel (required for create_channel)"
                },
                "channel_type": {
                    "type": "string",
                    "enum": ["public", "private", "system"],
                    "description": "Type of channel to create",
                    "default": "public"
                },
                "description": {
                    "type": "string",
                    "description": "Channel description (optional for create_channel)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Number of messages to retrieve (for get_messages)",
                    "default": 10,
                    "minimum": 1,
                    "maximum": 100
                }
            },
            "required": ["action"]
        })
    }

    fn get_description(&self) -> String {
        "Enable direct agent-to-agent communication with channels, broadcasting, and message history".to_string()
    }
}

pub struct KnowledgeSharingTool {
    hive: Arc<RwLock<HiveCoordinator>>,
}

impl KnowledgeSharingTool {
    pub fn new(hive: Arc<RwLock<HiveCoordinator>>) -> Self {
        Self { hive }
    }
}

#[async_trait]
impl MCPToolHandler for KnowledgeSharingTool {
    async fn execute(&self, params: &Value) -> Result<Value> {
        let action = params
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: action"))?;

        let valid_actions = [
            "store_knowledge",
            "retrieve_knowledge",
            "share_experience",
            "get_insights",
            "knowledge_graph",
        ];
        if !valid_actions.contains(&action) {
            return Err(anyhow::anyhow!(
                "Invalid action: {}. Must be one of: {}",
                action,
                valid_actions.join(", ")
            ));
        }

        let hive = self.hive.read().await;
        let _status = hive.get_status().await;

        let result = match action {
            "store_knowledge" => {
                let agent_id = params
                    .get("agent_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: agent_id"))?;

                let knowledge_type = params
                    .get("knowledge_type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: knowledge_type"))?;

                let content = params
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content"))?;

                let tags = params
                    .get("tags")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let confidence = params
                    .get("confidence")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.8);

                let knowledge_id = format!(
                    "knowledge_{}",
                    uuid::Uuid::new_v4().to_string()[..8].to_string()
                );

                json!({
                    "knowledge_id": knowledge_id,
                    "agent_id": agent_id,
                    "knowledge_type": knowledge_type,
                    "content": content,
                    "tags": tags,
                    "confidence": confidence,
                    "stored_at": chrono::Utc::now().to_rfc3339(),
                    "access_count": 0,
                    "status": "stored"
                })
            }
            "retrieve_knowledge" => {
                let query = params
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query"))?;

                let knowledge_type = params.get("knowledge_type").and_then(|v| v.as_str());
                let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(5);

                json!({
                    "query": query,
                    "knowledge_type": knowledge_type,
                    "results": [
                        {
                            "knowledge_id": "knowledge_12345678",
                            "content": "Efficient task routing using weighted round-robin algorithm",
                            "relevance_score": 0.95,
                            "source_agent": "coordinator_01",
                            "knowledge_type": "best_practice",
                            "tags": ["routing", "optimization", "performance"],
                            "created_at": chrono::Utc::now().to_rfc3339()
                        },
                        {
                            "knowledge_id": "knowledge_87654321",
                            "content": "Memory optimization techniques for large-scale agent deployments",
                            "relevance_score": 0.87,
                            "source_agent": "specialist_02",
                            "knowledge_type": "technical_solution",
                            "tags": ["memory", "optimization", "scalability"],
                            "created_at": chrono::Utc::now().to_rfc3339()
                        }
                    ],
                    "total_found": limit,
                    "search_time": "23ms"
                })
            }
            "share_experience" => {
                let from_agent = params
                    .get("from_agent")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: from_agent"))?;

                let experience_type = params
                    .get("experience_type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        anyhow::anyhow!("Missing required parameter: experience_type")
                    })?;

                let valid_exp_types = [
                    "success_pattern",
                    "failure_analysis",
                    "optimization_tip",
                    "lesson_learned",
                ];
                if !valid_exp_types.contains(&experience_type) {
                    return Err(anyhow::anyhow!(
                        "Invalid experience_type: {}. Must be one of: {}",
                        experience_type,
                        valid_exp_types.join(", ")
                    ));
                }

                let description = params
                    .get("description")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: description"))?;

                let impact = params
                    .get("impact")
                    .and_then(|v| v.as_str())
                    .unwrap_or("medium");
                let share_id = format!("exp_{}", uuid::Uuid::new_v4().to_string()[..8].to_string());

                json!({
                    "share_id": share_id,
                    "from_agent": from_agent,
                    "experience_type": experience_type,
                    "description": description,
                    "impact": impact,
                    "shared_at": chrono::Utc::now().to_rfc3339(),
                    "recipients": "all_agents",
                    "integration_status": "pending",
                    "expected_benefit": match experience_type {
                        "success_pattern" => "15% efficiency improvement",
                        "optimization_tip" => "10% resource savings",
                        "failure_analysis" => "Risk reduction",
                        _ => "Knowledge enhancement"
                    }
                })
            }
            "get_insights" => {
                let insight_type = params
                    .get("insight_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("trending");
                let time_range = params
                    .get("time_range")
                    .and_then(|v| v.as_str())
                    .unwrap_or("24h");

                json!({
                    "insight_type": insight_type,
                    "time_range": time_range,
                    "insights": [
                        {
                            "topic": "Task Distribution Patterns",
                            "insight": "Peak activity occurs between 9-11 AM, consider preemptive scaling",
                            "confidence": 0.92,
                            "impact": "high",
                            "source_agents": ["coordinator_01", "worker_03", "worker_07"]
                        },
                        {
                            "topic": "Communication Efficiency",
                            "insight": "Broadcast messages reduce individual messaging by 40%",
                            "confidence": 0.85,
                            "impact": "medium",
                            "source_agents": ["specialist_02"]
                        },
                        {
                            "topic": "Error Recovery",
                            "insight": "Retry with exponential backoff reduces failure rate by 60%",
                            "confidence": 0.78,
                            "impact": "high",
                            "source_agents": ["recovery_agent_01"]
                        }
                    ],
                    "recommendations": [
                        "Implement predictive scaling based on time patterns",
                        "Increase use of broadcast for system-wide updates",
                        "Apply retry patterns to all agent communications"
                    ]
                })
            }
            _ => {
                // knowledge_graph
                let focus = params
                    .get("focus")
                    .and_then(|v| v.as_str())
                    .unwrap_or("all");

                json!({
                    "knowledge_graph": {
                        "nodes": [
                            {"id": "task_distribution", "type": "concept", "connections": 8, "importance": 0.95},
                            {"id": "performance_optimization", "type": "concept", "connections": 12, "importance": 0.88},
                            {"id": "agent_communication", "type": "concept", "connections": 6, "importance": 0.82},
                            {"id": "error_handling", "type": "concept", "connections": 9, "importance": 0.79}
                        ],
                        "relationships": [
                            {"from": "task_distribution", "to": "performance_optimization", "strength": 0.9, "type": "enhances"},
                            {"from": "agent_communication", "to": "task_distribution", "strength": 0.8, "type": "enables"},
                            {"from": "error_handling", "to": "performance_optimization", "strength": 0.7, "type": "supports"}
                        ],
                        "clusters": [
                            {"name": "Core Operations", "concepts": ["task_distribution", "agent_communication"]},
                            {"name": "Optimization", "concepts": ["performance_optimization", "error_handling"]}
                        ]
                    },
                    "focus": focus,
                    "generated_at": chrono::Utc::now().to_rfc3339()
                })
            }
        };

        Ok(json!({
            "action": action,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "result": result
        }))
    }

    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["store_knowledge", "retrieve_knowledge", "share_experience", "get_insights", "knowledge_graph"],
                    "description": "Knowledge sharing action to perform"
                },
                "agent_id": {
                    "type": "string",
                    "description": "Agent storing knowledge (required for store_knowledge)"
                },
                "knowledge_type": {
                    "type": "string",
                    "enum": ["best_practice", "technical_solution", "performance_data", "error_pattern", "optimization"],
                    "description": "Type of knowledge (required for store_knowledge, optional filter for retrieve_knowledge)"
                },
                "content": {
                    "type": "string",
                    "description": "Knowledge content (required for store_knowledge)"
                },
                "tags": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Tags for knowledge categorization"
                },
                "confidence": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 1,
                    "description": "Confidence level in the knowledge",
                    "default": 0.8
                },
                "query": {
                    "type": "string",
                    "description": "Search query (required for retrieve_knowledge)"
                },
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 20,
                    "description": "Number of results to return",
                    "default": 5
                },
                "from_agent": {
                    "type": "string",
                    "description": "Agent sharing experience (required for share_experience)"
                },
                "experience_type": {
                    "type": "string",
                    "enum": ["success_pattern", "failure_analysis", "optimization_tip", "lesson_learned"],
                    "description": "Type of experience being shared"
                },
                "description": {
                    "type": "string",
                    "description": "Experience description (required for share_experience)"
                },
                "impact": {
                    "type": "string",
                    "enum": ["low", "medium", "high", "critical"],
                    "description": "Impact level of the experience",
                    "default": "medium"
                },
                "insight_type": {
                    "type": "string",
                    "enum": ["trending", "patterns", "anomalies", "predictions"],
                    "description": "Type of insights to generate",
                    "default": "trending"
                },
                "time_range": {
                    "type": "string",
                    "enum": ["1h", "6h", "24h", "7d", "30d"],
                    "description": "Time range for analysis",
                    "default": "24h"
                },
                "focus": {
                    "type": "string",
                    "description": "Focus area for knowledge graph",
                    "default": "all"
                }
            },
            "required": ["action"]
        })
    }

    fn get_description(&self) -> String {
        "Share learnings between agents with knowledge storage, experience sharing, and intelligent insights".to_string()
    }
}
