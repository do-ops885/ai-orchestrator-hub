//! MCP Server for Multiagent Hive System
//!
//! Standalone Model Context Protocol server implementation
//! Supports both stdin/stdout and HTTP modes

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use multiagent_hive::{
    communication::mcp::{HiveMCPServer, MCPError, MCPRequest, MCPResponse},
    communication::mcp_http,
    HiveCoordinator,
};
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{error, info, Level};

/// Standalone MCP Server for Multiagent Hive System
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    info!("Command line arguments: {:?}", args);
    let mode = if args.len() > 1 && args[1] == "--http" {
        "http"
    } else {
        "stdio"
    };

    info!("Starting Multiagent Hive MCP Server (mode: {})", mode);

    // Initialize the hive coordinator
    let hive = Arc::new(RwLock::new(HiveCoordinator::new().await?));

    match mode {
        "http" => run_http_server(hive).await,
        _ => run_stdio_server(hive).await,
    }
}

/// Run MCP server in HTTP mode
async fn run_http_server(hive: Arc<RwLock<HiveCoordinator>>) -> Result<()> {
    // Create shared MCP server instance (Phase 1.1 optimization)
    let mcp_server = Arc::new(HiveMCPServer::new(Arc::clone(&hive)));

    // Enhanced app state with shared server instance
    #[derive(Clone)]
    struct MCPAppState {
        hive: Arc<RwLock<HiveCoordinator>>,
        mcp_server: Arc<HiveMCPServer>,  // SHARED INSTANCE
    }

    let app_state = MCPAppState { 
        hive,
        mcp_server: Arc::clone(&mcp_server)
    };

    // Create router with optimized handler
    let app = Router::new()
        .route("/", post(move |state: axum::extract::State<MCPAppState>, Json(request): Json<MCPRequest>| async move {
            // Use shared server instance instead of creating new one per request
            let response = state.mcp_server.handle_request(request).await;
            Ok::<Json<MCPResponse>, (StatusCode, Json<Value>)>(Json(response))
        }))
        .route("/health", get(|| async {
            Json(serde_json::json!({
                "status": "healthy",
                "service": "mcp-http-standalone",
                "mode": "http",
                "hive_connected": true
            }))
        }))
        .with_state(app_state);

    let listener = TcpListener::bind("0.0.0.0:3002").await?;
    info!("MCP HTTP Server listening on http://0.0.0.0:3002");
    info!("Available tools: create_swarm_agent, assign_swarm_task, analyze_with_nlp, get_swarm_status, coordinate_agents, and more");

    axum::serve(listener, app).await?;

    Ok(())
}

/// Run MCP server in stdio mode (original implementation)
async fn run_stdio_server(hive: Arc<RwLock<HiveCoordinator>>) -> Result<()> {
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
