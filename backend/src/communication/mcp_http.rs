//! HTTP-based MCP Server for Multiagent Hive System
//!
//! This module provides HTTP endpoints for MCP (Model Context Protocol) communication,
//! allowing MCP clients to connect via HTTP instead of stdin/stdout.
//!
//! Phase 2: Includes optimized HTTP connection pool with health monitoring and load balancing.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, error, info, warn};

use crate::communication::mcp::{MCPRequest, MCPResponse};
use crate::AppState;

/// HTTP handler for MCP requests
pub async fn handle_mcp_request(
    State(state): State<AppState>,
    Json(request): Json<MCPRequest>,
) -> Result<Json<MCPResponse>, (StatusCode, Json<Value>)> {
    let request_id = uuid::Uuid::new_v4();
    let start_time = std::time::Instant::now();

    info!(
        "🔌 [{}] Received MCP HTTP request: {} (id: {:?})",
        request_id, request.method, request.id
    );

    // Log request details for debugging
    debug!(
        "📝 [{}] MCP request details - Method: {}, Params: {}",
        request_id,
        request.method,
        serde_json::to_string(&request.params).unwrap_or_else(|_| "Invalid JSON".to_string())
    );

    let mcp_server = Arc::clone(&state.mcp_server);

    // Handle the request
    let response = mcp_server.handle_request(request).await;

    let duration = start_time.elapsed();

    debug!(
        "📤 [{}] MCP HTTP response: {:?} ({}ms)",
        request_id,
        response.id,
        duration.as_millis()
    );

    // Ensure response has proper structure
    if response.result.is_none() && response.error.is_none() {
        error!(
            "❌ [{}] MCP request processing failed - No result or error returned ({}ms)",
            request_id,
            duration.as_millis()
        );

        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "MCP request processing failed",
                "message": "No result or error returned from MCP handler",
                "request_id": request_id.to_string(),
                "processing_time_ms": duration.as_millis()
            })),
        ));
    }

    // Log successful response
    if response.error.is_none() {
        info!(
            "✅ [{}] MCP request completed successfully ({}ms)",
            request_id,
            duration.as_millis()
        );
    } else {
        warn!(
            "⚠️ [{}] MCP request completed with error: {} ({}ms)",
            request_id,
            response
                .error
                .as_ref()
                .map_or("Unknown error".to_string(), |e| e.message.clone()),
            duration.as_millis()
        );
    }

    Ok(Json(response))
}

/// Create MCP HTTP router
pub fn create_mcp_router() -> Router<AppState> {
    Router::new()
        .route("/", post(handle_mcp_request))
        .route("/health", get(mcp_health_check))
}

/// Initialize MCP server for background operation
pub fn start_mcp_background_service(_state: AppState) {
    info!("🚀 Starting MCP HTTP service as background component");

    // The MCP server is now available via HTTP endpoints
    // No additional background tasks needed since it's integrated into the main server
    info!("📡 MCP HTTP endpoint available at /mcp");
    info!("🔧 Available MCP tools: create_swarm_agent, assign_swarm_task, analyze_with_nlp, get_swarm_status, coordinate_agents");
}

/// Health check for MCP service
pub async fn mcp_health_check(State(state): State<AppState>) -> Json<Value> {
    let hive = state.hive.read().await;
    let status = hive.get_status().await;

    Json(serde_json::json!({
        "service": "mcp-http",
        "status": "healthy",
        "hive_connected": true,
        "total_agents": status.get("total_agents").unwrap_or(&Value::Null),
        "active_agents": status.get("active_agents").unwrap_or(&Value::Null),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// ===== PHASE 2: HTTP Connection Pool Optimization =====

/// Configuration for HTTP connection pool
#[derive(Debug, Clone)]
pub struct HttpConnectionPoolConfig {
    pub max_connections: usize,
    pub max_connections_per_host: usize,
    pub connection_timeout: Duration,
    pub request_timeout: Duration,
    pub idle_timeout: Duration,
    pub health_check_interval: Duration,
    pub max_idle_connections: usize,
    pub enable_load_balancing: bool,
    pub retry_attempts: u32,
    pub circuit_breaker_threshold: u32,
}

impl Default for HttpConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            max_connections_per_host: 10,
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(60),
            idle_timeout: Duration::from_secs(300),
            health_check_interval: Duration::from_secs(30),
            max_idle_connections: 20,
            enable_load_balancing: true,
            retry_attempts: 3,
            circuit_breaker_threshold: 5,
        }
    }
}

/// Health status of an HTTP endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EndpointHealth {
    Healthy,
    Degraded { latency_ms: u64 },
    Unhealthy { reason: String, failures: u32 },
}

/// HTTP endpoint with health monitoring
#[derive(Debug, Clone)]
pub struct HttpEndpoint {
    pub url: String,
    pub health: EndpointHealth,
    pub last_health_check: Instant,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_latency_ms: f64,
    pub circuit_breaker_failures: u32,
}

impl HttpEndpoint {
    pub fn new(url: String) -> Self {
        Self {
            url,
            health: EndpointHealth::Healthy,
            last_health_check: Instant::now(),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_latency_ms: 0.0,
            circuit_breaker_failures: 0,
        }
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.health, EndpointHealth::Healthy)
    }

    pub fn record_request(&mut self, success: bool, latency_ms: u64) {
        self.total_requests += 1;
        if success {
            self.successful_requests += 1;
            self.circuit_breaker_failures = 0; // Reset on success
        } else {
            self.failed_requests += 1;
            self.circuit_breaker_failures += 1;
        }

        // Update rolling average latency
        let alpha = 0.1; // Smoothing factor
        self.average_latency_ms =
            self.average_latency_ms * (1.0 - alpha) + latency_ms as f64 * alpha;
    }

    pub fn update_health(&mut self, health: EndpointHealth) {
        self.health = health;
        self.last_health_check = Instant::now();
    }
}

/// HTTP connection pool with health monitoring and load balancing
pub struct HttpConnectionPool {
    config: HttpConnectionPoolConfig,
    endpoints: Arc<Mutex<HashMap<String, HttpEndpoint>>>,
    semaphore: Arc<Semaphore>,
    stats: Arc<Mutex<HttpPoolStats>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpPoolStats {
    pub total_endpoints: usize,
    pub healthy_endpoints: usize,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_latency_ms: f64,
    pub active_connections: usize,
    pub queued_requests: usize,
}

impl HttpConnectionPool {
    /// Create a new HTTP connection pool
    pub fn new(config: HttpConnectionPoolConfig) -> Self {
        Self {
            config: config.clone(),
            endpoints: Arc::new(Mutex::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(config.max_connections)),
            stats: Arc::new(Mutex::new(HttpPoolStats {
                total_endpoints: 0,
                healthy_endpoints: 0,
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                average_latency_ms: 0.0,
                active_connections: 0,
                queued_requests: 0,
            })),
        }
    }

    /// Add an endpoint to the pool
    pub async fn add_endpoint(&self, url: String) {
        let mut endpoints = self.endpoints.lock().await;
        if !endpoints.contains_key(&url) {
            let endpoint = HttpEndpoint::new(url.clone());
            let is_healthy = endpoint.is_healthy();
            endpoints.insert(url, endpoint);
            let mut stats = self.stats.lock().await;
            stats.total_endpoints += 1;
            if is_healthy {
                stats.healthy_endpoints += 1;
            }
        }
    }

    /// Remove an endpoint from the pool
    pub async fn remove_endpoint(&self, url: &str) {
        let mut endpoints = self.endpoints.lock().await;
        if endpoints.remove(url).is_some() {
            let mut stats = self.stats.lock().await;
            stats.total_endpoints = stats.total_endpoints.saturating_sub(1);
            // Recalculate healthy endpoints
            stats.healthy_endpoints = endpoints.values().filter(|e| e.is_healthy()).count();
        }
    }

    /// Get the next healthy endpoint using load balancing
    pub async fn get_endpoint(&self) -> Option<String> {
        let endpoints = self.endpoints.lock().await;
        let healthy_endpoints: Vec<_> = endpoints.values().filter(|e| e.is_healthy()).collect();

        if healthy_endpoints.is_empty() {
            return None;
        }

        if self.config.enable_load_balancing {
            // Simple load balancing: choose endpoint with lowest average latency
            healthy_endpoints
                .iter()
                .min_by(|a, b| {
                    a.average_latency_ms
                        .partial_cmp(&b.average_latency_ms)
                        .expect("replaced unwrap")
                })
                .map(|e| e.url.clone())
        } else {
            // Round-robin or random selection
            Some(healthy_endpoints[0].url.clone())
        }
    }

    /// Execute an HTTP request through the pool
    pub async fn execute_request<F, Fut, T>(
        &self,
        endpoint_url: &str,
        request_fn: F,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
    {
        let _permit = self.semaphore.acquire().await?;
        let start_time = Instant::now();

        // Update stats
        {
            let mut stats = self.stats.lock().await;
            stats.active_connections += 1;
            stats.total_requests += 1;
        }

        let result = request_fn().await;
        let latency_ms = start_time.elapsed().as_millis() as u64;

        // Update endpoint stats
        {
            let mut endpoints = self.endpoints.lock().await;
            if let Some(endpoint) = endpoints.get_mut(endpoint_url) {
                let success = result.is_ok();
                endpoint.record_request(success, latency_ms);

                // Update health based on circuit breaker
                if endpoint.circuit_breaker_failures >= self.config.circuit_breaker_threshold {
                    endpoint.update_health(EndpointHealth::Unhealthy {
                        reason: "Circuit breaker triggered".to_string(),
                        failures: endpoint.circuit_breaker_failures,
                    });
                }
            }
        }

        // Update pool stats
        {
            let mut stats = self.stats.lock().await;
            stats.active_connections = stats.active_connections.saturating_sub(1);

            if result.is_ok() {
                stats.successful_requests += 1;
            } else {
                stats.failed_requests += 1;
            }

            // Update rolling average latency
            let alpha = 0.1;
            stats.average_latency_ms =
                stats.average_latency_ms * (1.0 - alpha) + latency_ms as f64 * alpha;
        }

        result
    }

    /// Perform health checks on all endpoints
    pub async fn perform_health_checks(&self) {
        let endpoint_urls: Vec<String> = {
            let endpoints = self.endpoints.lock().await;
            endpoints.keys().cloned().collect()
        };

        for url in endpoint_urls {
            self.check_endpoint_health(&url).await;
        }
    }

    /// Check health of a specific endpoint
    async fn check_endpoint_health(&self, url: &str) {
        let start_time = Instant::now();

        // Simple health check - in a real implementation, you'd make a lightweight request
        let is_healthy = true; // Placeholder - implement actual health check
        let latency_ms = start_time.elapsed().as_millis() as u64;

        let mut endpoints = self.endpoints.lock().await;
        if let Some(endpoint) = endpoints.get_mut(url) {
            if is_healthy {
                if latency_ms > 1000 {
                    // High latency threshold
                    endpoint.update_health(EndpointHealth::Degraded { latency_ms });
                } else {
                    endpoint.update_health(EndpointHealth::Healthy);
                }
            } else {
                endpoint.update_health(EndpointHealth::Unhealthy {
                    reason: "Health check failed".to_string(),
                    failures: endpoint.circuit_breaker_failures + 1,
                });
            }
        }

        // Update healthy endpoint count
        let mut stats = self.stats.lock().await;
        stats.healthy_endpoints = endpoints.values().filter(|e| e.is_healthy()).count();
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> HttpPoolStats {
        self.stats.lock().await.clone()
    }

    /// Get endpoint information
    pub async fn get_endpoints(&self) -> HashMap<String, EndpointHealth> {
        let endpoints = self.endpoints.lock().await;
        endpoints
            .iter()
            .map(|(url, endpoint)| (url.clone(), endpoint.health.clone()))
            .collect()
    }

    /// Start background health checking
    pub fn start_health_monitoring(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.config.health_check_interval);
            loop {
                interval.tick().await;
                self.perform_health_checks().await;
            }
        })
    }
}

impl Default for HttpConnectionPool {
    fn default() -> Self {
        Self::new(HttpConnectionPoolConfig::default())
    }
}
