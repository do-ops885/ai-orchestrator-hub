//! MCP Server for Multiagent Hive System
//! 
//! Standalone Model Context Protocol server implementation

use anyhow::Result;
use multiagent_hive::{
    communication::mcp::{HiveMCPServer, MCPError, MCPRequest, MCPResponse},
    HiveCoordinator,
};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::RwLock;
use tracing::{error, info, Level};

/// Standalone MCP Server for Multiagent Hive Systemcon
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Multiagent Hive MCP Server");

    // Initialize the hive coordinator
    let hive = Arc::new(RwLock::new(HiveCoordinator::new().await?));
    let mcp_server = HiveMCPServer::new(hive);

    info!("MCP Server ready - listening on stdin/stdout");
    info!(
        "Available tools: create_swarm_agent, assign_swarm_task, analyze_with_nlp, get_swarm_status, coordinate_agents"
    );

    // MCP protocol uses stdin/stdout for communication
    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<MCPRequest>(&line) {
            Ok(request) => {
                let response = mcp_server.handle_request(request).await;
                let response_json = serde_json::to_string(&response)?;
                stdout.write_all(response_json.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
            Err(e) => {
                error!("Invalid JSON-RPC request: {}", e);
                let error_response = MCPResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(MCPError {
                        code: -32700,
                        message: "Parse error".to_string(),
                        data: Some(serde_json::json!({"details": e.to_string()})),
                    }),
                };
                let error_json = serde_json::to_string(&error_response)?;
                stdout.write_all(error_json.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
        }
    }

    Ok(())
}
