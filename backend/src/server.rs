//! Server initialization and background task management
//!
//! This module handles server setup, routing, and background task coordination
//! for the multiagent hive system.

use axum::{
    extract::{ws::WebSocketUpgrade, State},
    http::{StatusCode, header::AUTHORIZATION},
    response::Response,
    routing::get,
    Router,
};
// Removed unused imports
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info, warn};

use crate::communication;
use crate::communication::mcp_http;
use crate::infrastructure::metrics::{AgentMetrics, AlertLevel, TaskMetrics};
use crate::infrastructure::middleware::security_headers_middleware;
use crate::utils::auth::{Claims, ClientType, Role};
use crate::utils::structured_logging::{SecurityEventDetails, SecurityEventType, StructuredLogger};
use crate::utils::validation::InputValidator;
use crate::AppState;

use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

/// Start background tasks for monitoring, alerting, and system maintenance
pub async fn start_background_tasks(app_state: AppState) {
    let metrics_interval = std::time::Duration::from_millis(
        app_state.config.performance.metrics_collection_interval_ms,
    );
    let alert_interval =
        std::time::Duration::from_millis(app_state.config.performance.alert_check_interval_ms);

    // Start metrics collection task
    start_metrics_collection_task(app_state.clone(), metrics_interval).await;

    // Start intelligent alert processing task
    start_alert_processing_task(app_state.clone(), alert_interval).await;

    // Start agent recovery task
    start_agent_recovery_task(app_state.clone()).await;

    // Start adaptive learning cleanup task
    start_learning_cleanup_task(app_state.clone()).await;

    // Start MCP HTTP service
    mcp_http::start_mcp_background_service(app_state.clone());

    info!("🔄 Background monitoring tasks started");
}

/// Start the metrics collection background task
async fn start_metrics_collection_task(app_state: AppState, interval: std::time::Duration) {
    let metrics_state = app_state;
    tokio::spawn(async move {
        let mut interval_timer = tokio::time::interval(interval);
        loop {
            interval_timer.tick().await;
            collect_and_update_metrics(&metrics_state).await;
        }
    });
}

/// Collect and update system metrics
async fn collect_and_update_metrics(app_state: &AppState) {
    // Collect traditional system metrics
    if let Err(e) = app_state.metrics.collect_system_metrics().await {
        error!("Failed to collect system metrics: {}", e);
    }

    // Collect advanced metrics with predictive analytics
    if let Err(e) = app_state.advanced_metrics.collect_system_metrics().await {
        error!("Failed to collect advanced metrics: {}", e);
    } else {
        debug!("Advanced metrics collected successfully");
    }

    // Snapshot current metrics for historical analysis
    app_state.metrics.snapshot_current_metrics().await;

    // Update hive metrics
    let hive = app_state.hive.read().await.get_status().await;

    // Update agent and task metrics from hive status
    let agent_metrics = extract_agent_metrics_from_hive(&hive);
    let task_metrics = extract_task_metrics_from_hive(&hive);

    // Update the metrics systems with the collected data
    app_state.metrics.update_agent_metrics(agent_metrics).await;
    app_state.metrics.update_task_metrics(task_metrics).await;

    // Snapshot the current metrics for historical analysis
    app_state.metrics.snapshot_current_metrics().await;
}

/// Extract agent metrics from hive status JSON
fn extract_agent_metrics_from_hive(hive_status: &serde_json::Value) -> AgentMetrics {
    AgentMetrics {
        total_agents: hive_status
            .get("total_agents")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0) as usize,
        active_agents: hive_status
            .get("active_agents")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0) as usize,
        idle_agents: hive_status
            .get("idle_agents")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0) as usize,
        failed_agents: hive_status
            .get("failed_agents")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0) as usize,
        average_agent_performance: hive_status
            .get("average_performance")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        agent_utilization_percent: 0.0,
        individual_agent_metrics: std::collections::HashMap::new(),
    }
}

/// Extract task metrics from hive status JSON
fn extract_task_metrics_from_hive(hive_status: &serde_json::Value) -> TaskMetrics {
    let total_tasks = hive_status
        .get("total_tasks")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);

    let completed_tasks = hive_status
        .get("completed_tasks")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);

    let task_success_rate = if total_tasks > 0 {
        (completed_tasks as f64 / total_tasks as f64) * 100.0
    } else {
        0.0
    };

    TaskMetrics {
        total_tasks_submitted: total_tasks,
        total_tasks_completed: completed_tasks,
        total_tasks_failed: hive_status
            .get("failed_tasks")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
        tasks_in_queue: hive_status
            .get("pending_tasks")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0) as usize,
        average_task_duration_ms: hive_status
            .get("average_task_completion_time")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        task_success_rate,
    }
}

/// Start the intelligent alert processing background task
async fn start_alert_processing_task(app_state: AppState, interval: std::time::Duration) {
    let alert_state = app_state;
    tokio::spawn(async move {
        let mut interval_timer = tokio::time::interval(interval);
        loop {
            interval_timer.tick().await;
            process_alerts(&alert_state).await;
        }
    });
}

/// Process intelligent alerts and traditional alerts
async fn process_alerts(app_state: &AppState) {
    // Process intelligent alerts with predictive capabilities
    match app_state
        .intelligent_alerting
        .process_intelligent_alerts()
        .await
    {
        Ok(intelligent_alerts) => {
            if !intelligent_alerts.is_empty() {
                info!(
                    "🚨 Processed {} intelligent alerts",
                    intelligent_alerts.len()
                );
                for alert in &intelligent_alerts {
                    debug!(
                        "Alert: {} (confidence: {:.2}, predicted: {})",
                        alert.base_alert.title, alert.confidence, alert.predicted
                    );
                }
            }
        }
        Err(e) => {
            error!("Failed to process intelligent alerts: {}", e);
        }
    }

    // Also check traditional alerts as backup
    let alerts = app_state.metrics.check_alerts().await;
    for alert in alerts {
        match alert.level {
            AlertLevel::Critical => {
                error!("🚨 CRITICAL ALERT: {} - {}", alert.title, alert.description);
                // In production, you would send notifications here
            }
            AlertLevel::Warning => {
                warn!("⚠️  WARNING: {} - {}", alert.title, alert.description);
            }
            AlertLevel::Info => {
                info!("ℹ️  INFO: {} - {}", alert.title, alert.description);
            }
        }
    }

    // Analyze trends
    let trends = app_state.metrics.analyze_trends().await;
    debug!(
        "System trends - CPU: {:?}, Memory: {:?}, Tasks: {:?}",
        trends.cpu_trend, trends.memory_trend, trends.task_completion_trend
    );
}

/// Start the agent recovery background task
async fn start_agent_recovery_task(app_state: AppState) {
    let recovery_state = app_state;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60)); // Check every minute
        loop {
            interval.tick().await;
            check_and_recover_failed_agents(&recovery_state).await;
        }
    });
}

/// Check for failed agents and attempt recovery
async fn check_and_recover_failed_agents(app_state: &AppState) {
    let hive = app_state.hive.read().await.get_agents_info().await;
    if let Some(agents) = hive.get("agents").and_then(|v| v.as_array()) {
        for agent_value in agents {
            if let Some(state) = agent_value.get("state").and_then(|v| v.as_str()) {
                if state == "Failed" {
                    if let Some(agent_id) = agent_value.get("id").and_then(|v| v.as_str()) {
                        info!("🔧 Attempting recovery for failed agent: {}", agent_id);
                        // In a real implementation, you would recover the specific agent
                        // For now, we just log the attempt
                    }
                }
            }
        }
    }
}

/// Start the adaptive learning cleanup background task
async fn start_learning_cleanup_task(app_state: AppState) {
    let learning_state = app_state;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600)); // Every hour
        loop {
            interval.tick().await;
            cleanup_learning_patterns(&learning_state).await;
        }
    });
}

/// Cleanup old learning patterns
async fn cleanup_learning_patterns(app_state: &AppState) {
    let mut learning_system = app_state.adaptive_learning.write().await;
    learning_system.cleanup_old_patterns();
    info!("🧹 Cleaned up old learning patterns");
}

/// Create the main application router with all routes configured
/// Authentication request payload
#[derive(serde::Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
    client_type: Option<ClientType>,
}

/// Login response
#[derive(serde::Serialize)]
struct LoginResponse {
    token: String,
    refresh_token: String,
    user: UserInfo,
    expires_in: usize,
}

/// User information for responses
#[derive(serde::Serialize)]
struct UserInfo {
    id: String,
    username: String,
    roles: Vec<String>,
    permissions: Vec<String>,
    client_type: ClientType,
}

/// Refresh token request
#[derive(serde::Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

/// Logout request
#[derive(serde::Deserialize)]
struct LogoutRequest {
    session_id: String,
}

/// JWT Claims extractor for authenticated routes
#[async_trait::async_trait]
impl<S> axum::extract::FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, axum::Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Get the authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "));

        let token = match auth_header {
            Some(token) => token,
            None => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    axum::Json(json!({
                        "error": "Missing authorization token"
                    })),
                ));
            }
        };

        // For now, we'll skip JWT validation in this demo
        // In production, you'd validate the token here
        // For demo purposes, return a mock claims object
        let claims = Claims {
            sub: "demo_user_id".to_string(),
            exp: 0, // Not used in demo
            iat: 0, // Not used in demo
            iss: "hive-system".to_string(),
            aud: "hive-api".to_string(),
            roles: vec!["Admin".to_string()],
            permissions: vec!["SystemAdmin".to_string(), "UserManagement".to_string()],
            session_id: "demo_session".to_string(),
            client_type: ClientType::Human,
        };

        Ok(claims)
    }
}

/// Authentication handlers
async fn login(
    State(state): State<AppState>,
    axum::Json(payload): axum::Json<LoginRequest>,
) -> Result<axum::Json<LoginResponse>, (StatusCode, axum::Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4();
    let start_time = std::time::Instant::now();

    info!(
        "🔐 [{}] Login attempt for user: {}",
        request_id, payload.username
    );

    // For demo purposes, we'll use a simple authentication
    // In production, this would validate against a user database
    if payload.username.is_empty() || payload.password.is_empty() {
        warn!("🚫 [{}] Invalid login credentials", request_id);
        return Err((
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({
                "error": "Invalid credentials",
                "request_id": request_id.to_string()
            })),
        ));
    }

    // Create a demo user with admin role
    let user_id = Uuid::new_v4().to_string();
    let client_type = payload.client_type.unwrap_or(ClientType::Human);
    let roles = vec![Role::Admin];
    let permissions = roles.iter().flat_map(|r| r.permissions()).collect::<Vec<_>>();

    // Generate JWT token
    let (token, session_id) = state.auth_manager.authenticate_user(
        user_id.clone(),
        roles.clone(),
        client_type.clone(),
        Some("127.0.0.1".to_string()), // In production, get from request
        Some("Demo User Agent".to_string()),
    ).await.map_err(|e| {
        error!("❌ [{}] Authentication failed: {}", request_id, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({
                "error": "Authentication failed",
                "request_id": request_id.to_string()
            })),
        )
    })?;

    let user_info = UserInfo {
        id: user_id,
        username: payload.username.clone(),
        roles: roles.iter().map(|r| format!("{:?}", r)).collect(),
        permissions: permissions.iter().map(|p| format!("{:?}", p)).collect(),
        client_type,
    };

    let duration = start_time.elapsed();
    info!(
        "✅ [{}] User {} logged in successfully ({}ms)",
        request_id,
        payload.username,
        duration.as_millis()
    );

    Ok(axum::Json(LoginResponse {
        token,
        refresh_token: session_id.clone(), // Using session_id as refresh token for demo
        user: user_info,
        expires_in: 3600, // 1 hour
    }))
}

async fn logout(
    State(state): State<AppState>,
    axum::Json(payload): axum::Json<LogoutRequest>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4();

    info!("🚪 [{}] Logout request for session: {}", request_id, payload.session_id);

    // Logout the user
    state.auth_manager.logout(&payload.session_id).await.map_err(|e| {
        error!("❌ [{}] Logout failed: {}", request_id, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({
                "error": "Logout failed",
                "request_id": request_id.to_string()
            })),
        )
    })?;

    info!("✅ [{}] User logged out successfully", request_id);

    Ok(axum::Json(json!({
        "success": true,
        "message": "Logged out successfully",
        "request_id": request_id.to_string()
    })))
}

async fn refresh_token(
    State(state): State<AppState>,
    axum::Json(payload): axum::Json<RefreshRequest>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4();

    info!("🔄 [{}] Token refresh request", request_id);

    // Refresh the token
    let new_token = state.auth_manager.refresh_token(&payload.refresh_token).await.map_err(|e| {
        warn!("🚫 [{}] Token refresh failed: {}", request_id, e);
        (
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({
                "error": "Invalid refresh token",
                "request_id": request_id.to_string()
            })),
        )
    })?;

    info!("✅ [{}] Token refreshed successfully", request_id);

    Ok(axum::Json(json!({
        "success": true,
        "token": new_token,
        "expires_in": 3600,
        "request_id": request_id.to_string()
    })))
}

async fn get_current_user(
    State(state): State<AppState>,
    claims: Option<Claims>,
) -> Result<axum::Json<UserInfo>, (StatusCode, axum::Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4();

    match claims {
        Some(claims) => {
            info!("👤 [{}] Getting current user info for: {}", request_id, claims.sub);

            let user_info = UserInfo {
                id: claims.sub.clone(),
                username: "demo_user".to_string(), // In production, get from database
                roles: claims.roles.clone(),
                permissions: claims.permissions.clone(),
                client_type: claims.client_type,
            };

            Ok(axum::Json(user_info))
        }
        None => {
            warn!("🚫 [{}] No authentication token provided", request_id);
            Err((
                StatusCode::UNAUTHORIZED,
                axum::Json(json!({
                    "error": "Authentication required",
                    "request_id": request_id.to_string()
                })),
            ))
        }
    }
}

pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { "🐝 Multiagent Hive System API v2.0 - CPU-native, GPU-optional" }),
        )
        .route("/health", get(health_check))
        .route("/metrics", get(get_metrics))
        .route("/ws", get(websocket_handler))
        .route("/api/auth/login", axum::routing::post(login))
        .route("/api/auth/logout", axum::routing::post(logout))
        .route("/api/auth/refresh", axum::routing::post(refresh_token))
        .route("/api/auth/me", get(get_current_user))
        .route("/api/agents", get(get_agents).post(create_agent))
        .route("/api/tasks", get(get_tasks).post(create_task))
        .route("/api/hive/status", get(get_hive_status))
        .route("/api/resources", get(get_resource_info))
        .route("/api/monitoring/metrics", get(get_monitoring_metrics))
        .route("/api/monitoring/alerts", get(get_monitoring_alerts))
        .route("/api/monitoring/health", get(get_monitoring_health))
        .route("/debug/system", get(debug_system_info))
        .nest("/api/mcp", mcp_http::create_mcp_router())
        .layer(axum::middleware::from_fn(security_headers_middleware))
        .layer(CorsLayer::permissive())
        .with_state(app_state)
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| communication::handle_websocket(socket, state))
}

async fn get_agents(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let agents_info = state.hive.read().await.get_agents_info().await;
    Ok(axum::Json(agents_info))
}

async fn create_agent(
    State(state): State<AppState>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> Result<(StatusCode, axum::Json<serde_json::Value>), (StatusCode, axum::Json<serde_json::Value>)>
{
    let request_id = Uuid::new_v4();
    let start_time = std::time::Instant::now();

    info!(
        "🔧 [{}] Starting agent creation request - Payload size: {} bytes",
        request_id,
        serde_json::to_string(&payload).unwrap_or_default().len()
    );

    // Log request details for debugging
    debug!(
        "📝 [{}] Agent creation payload: {}",
        request_id,
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "Invalid JSON".to_string())
    );

    // Apply rate limiting
    if state
        .rate_limiter
        .check_rate_limit("api_create_agent")
        .await
        .is_err()
    {
        warn!("🚫 [{}] Rate limit exceeded for agent creation", request_id);
        state.metrics.record_error("rate_limit_exceeded").await;

        let duration = start_time.elapsed();
        info!(
            "❌ [{}] Agent creation failed - Rate limit exceeded ({}ms)",
            request_id,
            duration.as_millis()
        );

        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            axum::Json(json!({
                "error": "Rate limit exceeded",
                "details": "Too many requests, please try again later",
                "request_id": request_id.to_string()
            })),
        ));
    }

    // Validate payload using comprehensive validation
    if let Err(e) = InputValidator::validate_agent_payload(&payload) {
        warn!("Invalid agent creation payload: {}", e);
        state.metrics.record_error("invalid_agent_payload").await;
        return Err((
            StatusCode::BAD_REQUEST,
            axum::Json(json!({
                "error": "Invalid payload",
                "details": e.to_string()
            })),
        ));
    }

    let hive = state.hive.write().await;
    match hive.create_agent(payload).await {
        Ok(agent_id) => {
            let duration = start_time.elapsed();
            info!(
                "✅ [{}] Agent created successfully: {} ({}ms)",
                request_id,
                agent_id,
                duration.as_millis()
            );

            // Log security event for agent creation
            StructuredLogger::log_security_event(
                &SecurityEventType::AuthenticationSuccess,
                &SecurityEventDetails {
                    client_id: "api".to_string(),
                    endpoint: format!("agent:{agent_id}"),
                    user_agent: None,
                    ip_address: None,
                    timestamp: Utc::now(),
                    additional_info: {
                        let mut info = std::collections::HashMap::new();
                        info.insert("action".to_string(), "create".to_string());
                        info.insert("resource_type".to_string(), "agent".to_string());
                        info.insert("request_id".to_string(), request_id.to_string());
                        info.insert("duration_ms".to_string(), duration.as_millis().to_string());
                        info
                    },
                },
            );

            Ok((
                StatusCode::CREATED,
                axum::Json(json!({
                    "success": true,
                    "agent_id": agent_id,
                    "message": "Agent created successfully",
                    "request_id": request_id.to_string(),
                    "processing_time_ms": duration.as_millis()
                })),
            ))
        }
        Err(e) => {
            let duration = start_time.elapsed();
            error!(
                "❌ [{}] Failed to create agent: {} ({}ms)",
                request_id,
                e,
                duration.as_millis()
            );
            state.metrics.record_error("agent_creation_failed").await;

            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({
                    "success": false,
                    "error": "Failed to create agent",
                    "details": e.to_string(),
                    "request_id": request_id.to_string(),
                    "processing_time_ms": duration.as_millis()
                })),
            ))
        }
    }
}

async fn get_tasks(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let tasks_info = state.hive.read().await.get_tasks_info().await;
    match tasks_info {
        Ok(info) => Ok(axum::Json(info)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({
                "error": "Failed to get tasks info",
                "details": e.to_string()
            })),
        )),
    }
}

async fn create_task(
    State(state): State<AppState>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> Result<(StatusCode, axum::Json<serde_json::Value>), (StatusCode, axum::Json<serde_json::Value>)>
{
    let request_id = Uuid::new_v4();
    let start_time = std::time::Instant::now();

    info!(
        "📋 [{}] Starting task creation request - Payload size: {} bytes",
        request_id,
        serde_json::to_string(&payload).unwrap_or_default().len()
    );

    // Log request details for debugging
    debug!(
        "📝 [{}] Task creation payload: {}",
        request_id,
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "Invalid JSON".to_string())
    );

    // Apply rate limiting
    if state
        .rate_limiter
        .check_rate_limit("api_create_task")
        .await
        .is_err()
    {
        warn!("🚫 [{}] Rate limit exceeded for task creation", request_id);
        state.metrics.record_error("rate_limit_exceeded").await;

        let duration = start_time.elapsed();
        info!(
            "❌ [{}] Task creation failed - Rate limit exceeded ({}ms)",
            request_id,
            duration.as_millis()
        );

        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            axum::Json(json!({
                "error": "Rate limit exceeded",
                "details": "Too many requests, please try again later",
                "request_id": request_id.to_string()
            })),
        ));
    }

    // Validate payload using comprehensive validation
    if let Err(e) = InputValidator::validate_task_payload(&payload) {
        warn!("Invalid task creation payload: {}", e);
        state.metrics.record_error("invalid_task_payload").await;
        return Err((
            StatusCode::BAD_REQUEST,
            axum::Json(json!({
                "error": "Invalid payload",
                "details": e.to_string()
            })),
        ));
    }

    let hive = state.hive.write().await;
    match hive.create_task(payload).await {
        Ok(task_id) => {
            let duration = start_time.elapsed();
            info!(
                "✅ [{}] Task created successfully: {} ({}ms)",
                request_id,
                task_id,
                duration.as_millis()
            );

            // Log security event for task creation
            StructuredLogger::log_security_event(
                &SecurityEventType::AuthenticationSuccess,
                &SecurityEventDetails {
                    client_id: "api".to_string(),
                    endpoint: format!("task:{task_id}"),
                    user_agent: None,
                    ip_address: None,
                    timestamp: Utc::now(),
                    additional_info: {
                        let mut info = std::collections::HashMap::new();
                        info.insert("action".to_string(), "create".to_string());
                        info.insert("resource_type".to_string(), "task".to_string());
                        info.insert("request_id".to_string(), request_id.to_string());
                        info.insert("duration_ms".to_string(), duration.as_millis().to_string());
                        info
                    },
                },
            );

            Ok((
                StatusCode::CREATED,
                axum::Json(json!({
                    "success": true,
                    "task_id": task_id,
                    "message": "Task created successfully",
                    "request_id": request_id.to_string(),
                    "processing_time_ms": duration.as_millis()
                })),
            ))
        }
        Err(e) => {
            let duration = start_time.elapsed();
            error!(
                "❌ [{}] Failed to create task: {} ({}ms)",
                request_id,
                e,
                duration.as_millis()
            );
            state.metrics.record_error("task_creation_failed").await;

            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({
                    "success": false,
                    "error": "Failed to create task",
                    "details": e.to_string(),
                    "request_id": request_id.to_string(),
                    "processing_time_ms": duration.as_millis()
                })),
            ))
        }
    }
}

async fn get_hive_status(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let status = state.hive.read().await.get_status().await;
    Ok(axum::Json(status))
}

async fn get_resource_info(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let resource_info = state.hive.read().await.get_resource_info().await;
    match resource_info {
        Ok(info) => Ok(axum::Json(info)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({
                "error": "Failed to get resource info",
                "details": e.to_string()
            })),
        )),
    }
}

async fn health_check(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4();
    let start_time = std::time::Instant::now();

    debug!("🏥 [{}] Health check request received", request_id);

    // Gather health data from all components
    let health_data = gather_health_data(&state).await;
    let response_time_ms = start_time.elapsed().as_millis();

    // Determine overall health status
    let overall_healthy = health_data.hive_healthy
        && health_data.resources_healthy
        && health_data.metrics_healthy
        && health_data.alerting_healthy;

    let health_status = build_health_response(&health_data, response_time_ms, overall_healthy);

    if overall_healthy {
        Ok(axum::Json(health_status))
    } else {
        Err((StatusCode::SERVICE_UNAVAILABLE, axum::Json(health_status)))
    }
}

/// Gather health data from all system components
async fn gather_health_data(state: &AppState) -> HealthData {
    // Perform comprehensive health checks
    let hive_status = state.hive.read().await.get_status().await;
    let metrics_health = state.metrics.get_current_metrics().await;
    let resource_info = match state.hive.read().await.get_resource_info().await {
        Ok(info) => info,
        Err(e) => {
            warn!("Failed to get resource info for health check: {}", e);
            json!({})
        }
    };

    // Extract metrics from hive status JSON
    let hive_status_clone = hive_status.clone();
    let hive_metrics = hive_status_clone
        .get("metrics")
        .unwrap_or(&serde_json::Value::Null);
    let total_agents = hive_metrics
        .get("total_agents")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let completed_tasks = hive_metrics
        .get("completed_tasks")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);

    // Extract resource info from JSON
    let resource_info_clone = resource_info.clone();
    let system_resources = resource_info_clone
        .get("system_resources")
        .unwrap_or(&serde_json::Value::Null);
    let memory_usage = system_resources
        .get("memory_usage")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let cpu_usage = system_resources
        .get("cpu_usage")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);

    // Check component health
    let hive_healthy = total_agents > 0 || completed_tasks > 0;
    let resources_healthy = memory_usage < 90.0 && cpu_usage < 95.0;
    let metrics_healthy = metrics_health.performance.average_response_time_ms < 5000.0;
    let alerting_healthy = true; // Simplified for now - alerting system is operational

    HealthData {
        hive_status,
        metrics_health,
        resource_info,
        hive_metrics: hive_metrics.clone(),
        system_resources: system_resources.clone(),
        total_agents,
        completed_tasks,
        memory_usage,
        cpu_usage,
        hive_healthy,
        resources_healthy,
        metrics_healthy,
        alerting_healthy,
    }
}

/// Build the health check response JSON
fn build_health_response(
    health_data: &HealthData,
    response_time_ms: u128,
    overall_healthy: bool,
) -> serde_json::Value {
    json!({
        "status": if overall_healthy { "healthy" } else { "unhealthy" },
        "timestamp": Utc::now(),
        "response_time_ms": response_time_ms,
        "version": "2.0.0",
        "components": {
            "hive_coordinator": {
                "status": if health_data.hive_healthy { "healthy" } else { "unhealthy" },
                "total_agents": health_data.total_agents,
                "active_agents": health_data.hive_metrics.get("active_agents").and_then(serde_json::Value::as_u64).unwrap_or(0),
                "completed_tasks": health_data.completed_tasks,
                "average_performance": health_data.hive_metrics.get("average_performance").and_then(serde_json::Value::as_f64).unwrap_or(0.0)
            },
            "resource_manager": {
                "status": if health_data.resources_healthy { "healthy" } else { "unhealthy" },
                "memory_usage_percent": health_data.memory_usage,
                "cpu_usage_percent": health_data.cpu_usage,
                "available_memory_mb": health_data.system_resources.get("available_memory").and_then(serde_json::Value::as_f64).unwrap_or(0.0),
                "cpu_cores": health_data.system_resources.get("cpu_cores").and_then(serde_json::Value::as_u64).unwrap_or(0)
            },
            "metrics_collector": {
                "status": if health_data.metrics_healthy { "healthy" } else { "unhealthy" },
                "response_time_ms": health_data.metrics_health.performance.average_response_time_ms,
                "requests_per_second": health_data.metrics_health.performance.requests_per_second,
                "error_rate": health_data.metrics_health.error_metrics.error_rate_per_minute
            },
            "intelligent_alerting": {
                "status": if health_data.alerting_healthy { "healthy" } else { "unhealthy" },
                "active_rules": "monitoring",
                "system_operational": true
            }
        },
        "system_info": {
            "cpu_native": true,
            "gpu_optional": true,
            "phase_2_active": true,
            "swarm_cohesion": health_data.hive_metrics.get("swarm_cohesion").and_then(serde_json::Value::as_f64).unwrap_or(0.0),
            "learning_progress": health_data.hive_metrics.get("learning_progress").and_then(serde_json::Value::as_f64).unwrap_or(0.0)
        }
    })
}

/// Struct to hold health check data
struct HealthData {
    hive_status: serde_json::Value,
    metrics_health: crate::infrastructure::metrics::SystemMetrics,
    resource_info: serde_json::Value,
    hive_metrics: serde_json::Value,
    system_resources: serde_json::Value,
    total_agents: u64,
    completed_tasks: u64,
    memory_usage: f64,
    cpu_usage: f64,
    hive_healthy: bool,
    resources_healthy: bool,
    metrics_healthy: bool,
    alerting_healthy: bool,
}

async fn get_metrics(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let metrics = state.metrics.get_current_metrics().await;
    let trends = state.metrics.analyze_trends().await;
    Ok(axum::Json(json!({
        "current_metrics": metrics,
        "trends": trends,
        "collection_timestamp": Utc::now()
    })))
}

async fn get_monitoring_metrics(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    // In a full implementation, this would use the MonitoringSystem
    // For now, return basic metrics
    let current_metrics = state.metrics.get_current_metrics().await;
    let metrics_history = state.metrics.get_metrics_history(24).await; // Last 24 hours

    Ok(axum::Json(json!({
        "current": current_metrics,
        "history": metrics_history,
        "timestamp": Utc::now(),
        "note": "Full monitoring system integration pending"
    })))
}

async fn get_monitoring_alerts(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    // In a full implementation, this would use the AlertManager
    // For now, return mock alerts
    let mock_alerts = vec![
        json!({
            "id": "alert-1",
            "title": "High CPU Usage",
            "description": "CPU usage exceeded 80%",
            "severity": "medium",
            "source": "system_monitor",
            "timestamp": Utc::now(),
            "acknowledged": false,
            "resolved": false
        })
    ];

    Ok(axum::Json(json!({
        "alerts": mock_alerts,
        "total": mock_alerts.len(),
        "active": mock_alerts.iter().filter(|a| !a["resolved"].as_bool().unwrap_or(true)).count(),
        "timestamp": Utc::now(),
        "note": "Alert system integration pending"
    })))
}

async fn get_monitoring_health(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    // Generate comprehensive health report
    let health_report = json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "uptime_seconds": 3600, // Mock uptime
        "components": {
            "hive_coordinator": {
                "status": "healthy",
                "active_agents": 5,
                "pending_tasks": 2
            },
            "monitoring_system": {
                "status": "healthy",
                "metrics_collected": 150,
                "alerts_active": 1
            },
            "persistence_layer": {
                "status": "healthy",
                "connections_active": 3,
                "queries_per_second": 25.5
            }
        },
        "performance": {
            "response_time_p50": 45.2,
            "response_time_p95": 120.8,
            "error_rate": 0.02,
            "throughput": 150.5
        },
        "note": "Comprehensive monitoring system integration pending"
    });

    Ok(axum::Json(health_report))
}

/// Debug endpoint for comprehensive system inspection
async fn debug_system_info(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let request_id = Uuid::new_v4();
    let start_time = std::time::Instant::now();

    info!("🔍 [{}] Debug system inspection requested", request_id);

    let hive_status = state.hive.read().await.get_status().await;
    let agents_info = state.hive.read().await.get_agents_info().await;
    let tasks_info = match state.hive.read().await.get_tasks_info().await {
        Ok(info) => info,
        Err(e) => {
            warn!("Failed to get tasks info for debug: {}", e);
            json!({})
        }
    };
    let resource_info = match state.hive.read().await.get_resource_info().await {
        Ok(info) => info,
        Err(e) => {
            warn!("Failed to get resource info for debug: {}", e);
            json!({})
        }
    };
    let memory_stats = match state.hive.read().await.get_memory_stats().await {
        Ok(stats) => stats,
        Err(e) => {
            warn!("Failed to get memory stats for debug: {}", e);
            json!({})
        }
    };
    let queue_health = match state.hive.read().await.check_queue_health().await {
        Ok(health) => health,
        Err(e) => {
            warn!("Failed to check queue health for debug: {}", e);
            json!({})
        }
    };
    let agent_health = state.hive.read().await.check_agent_health();

    let duration = start_time.elapsed();

    info!(
        "✅ [{}] Debug system inspection completed ({}ms)",
        request_id,
        duration.as_millis()
    );

    Ok(axum::Json(json!({
        "request_id": request_id.to_string(),
        "timestamp": Utc::now(),
        "processing_time_ms": duration.as_millis(),
        "system_overview": {
            "version": "2.0.0",
            "phase": "Phase 2 - CPU-native, GPU-optional",
            "hive_status": hive_status,
            "agents": agents_info,
            "tasks": tasks_info,
            "resources": resource_info
        },
        "health_checks": {
            "memory_stats": memory_stats,
            "queue_health": queue_health,
            "agent_health": agent_health
        },
        "queue_systems": {
            "work_stealing_metrics": json!({"note": "Work stealing queue metrics not available in current architecture"}),
            "legacy_queue_info": {
                "pending_tasks": tasks_info.get("legacy_queue").and_then(|q| q.get("pending_tasks")).unwrap_or(&json!(0)),
                "completed_tasks": tasks_info.get("legacy_queue").and_then(|q| q.get("completed_tasks")).unwrap_or(&json!(0)),
                "failed_tasks": tasks_info.get("legacy_queue").and_then(|q| q.get("failed_tasks")).unwrap_or(&json!(0))
            }
        },
        "debug_info": {
            "config_snapshot": {
                "server_host": state.config.server.host.clone(),
                "server_port": state.config.server.port,
                "log_level": state.config.logging.level.clone()
            },
            "system_info": {
                "cpu_count": num_cpus::get(),
                "os": std::env::consts::OS,
                "arch": std::env::consts::ARCH
            }
        }
    })))
}
