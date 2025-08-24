use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{info, warn};
use uuid::Uuid;

use crate::infrastructure::MetricsCollector;
use crate::utils::HiveConfig;

/// Request ID middleware for tracing
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();
    request.headers_mut().insert(
        "x-request-id",
        request_id
            .parse()
            .unwrap_or_else(|_| "unknown".parse().expect("Static string should parse")),
    );

    let response = next.run(request).await;
    response
}

/// Request logging and metrics middleware
pub async fn logging_middleware(
    State(metrics): State<std::sync::Arc<MetricsCollector>>,
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    info!(
        request_id = %request_id,
        method = %method,
        uri = %uri,
        "Request started"
    );

    let response = next.run(request).await;
    let duration = start.elapsed();
    let status = response.status();

    // Record metrics
    if let Err(e) = record_request_metrics(&metrics, &method.to_string(), status, duration).await {
        warn!("Failed to record request metrics: {}", e);
    }

    info!(
        request_id = request_id,
        method = %method,
        uri = %uri,
        status = %status,
        duration_ms = duration.as_millis(),
        "Request completed"
    );

    response
}

/// Rate limiting middleware
pub async fn rate_limiting_middleware(
    State(config): State<std::sync::Arc<HiveConfig>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Simple rate limiting based on IP (in production, use Redis or similar)
    let _client_ip = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    // For demo purposes, allow all requests
    // In production, implement proper rate limiting logic
    if config.server.max_connections > 0 {
        // Rate limiting logic would go here
    }

    Ok(next.run(request).await)
}

/// Security headers middleware
pub async fn security_headers_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();
    headers.insert("X-Content-Type-Options", "nosniff".parse().expect("Static string should parse"));
    headers.insert("X-Frame-Options", "DENY".parse().expect("Static string should parse"));
    headers.insert("X-XSS-Protection", "1; mode=block".parse().expect("Static string should parse"));
    headers.insert(
        "Referrer-Policy",
        "strict-origin-when-cross-origin".parse().expect("Static string should parse"),
    );
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
            .parse()
            .expect("Static string should parse"),
    );

    response
}

async fn record_request_metrics(
    metrics: &MetricsCollector,
    _method: &str,
    status: StatusCode,
    duration: std::time::Duration,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Update performance metrics
    let performance_metrics = crate::infrastructure::metrics::PerformanceMetrics {
        requests_per_second: 1.0, // This would be calculated over time
        average_response_time_ms: duration.as_millis() as f64,
        p95_response_time_ms: duration.as_millis() as f64, // Simplified
        p99_response_time_ms: duration.as_millis() as f64, // Simplified
        throughput_tasks_per_second: 0.0,                  // Would be calculated separately
    };

    metrics
        .update_performance_metrics(performance_metrics)
        .await;

    // Record errors if status indicates failure
    if status.is_client_error() || status.is_server_error() {
        let error_type = if status.is_client_error() {
            "client_error"
        } else {
            "server_error"
        };
        metrics.record_error(error_type).await;
    }

    Ok(())
}

/// CORS middleware with configurable origins
pub fn cors_middleware(config: &HiveConfig) -> tower_http::cors::CorsLayer {
    use axum::http::Method;
    use tower_http::cors::{Any, CorsLayer};

    let mut cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any)
        .expose_headers([axum::http::HeaderName::from_static("x-request-id")]);

    if config.server.cors_origins.is_empty()
        || config.server.cors_origins.contains(&"*".to_string())
    {
        cors = cors.allow_origin(Any);
    } else {
        for origin in &config.server.cors_origins {
            if let Ok(origin_header) = origin.parse::<axum::http::HeaderValue>() {
                cors = cors.allow_origin(origin_header);
            }
        }
    }

    cors
}
