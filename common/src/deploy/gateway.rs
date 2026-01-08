use anyhow::Result;
use axum::{
    Router,
    body::Body,
    extract::{Host, State},
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower::ServiceExt;
use tower_http::services::ServeDir;

// ============================================================================
// STATE
// ============================================================================

#[derive(Debug, Default)]
pub struct GatewayRoutes {
    // domain -> path (e.g. "mysite.local" -> "/home/user/.spark/apps/site")
    pub static_routes: HashMap<String, String>,

    // domain -> port (e.g. "api.local" -> 3000)
    pub proxy_routes: HashMap<String, u16>,
}

pub type SharedGatewayState = Arc<RwLock<GatewayRoutes>>;

// ============================================================================
// SERVER
// ============================================================================

pub async fn run_http_gateway(state: SharedGatewayState) -> Result<()> {
    let app = Router::new()
        .fallback(handle_request) // Ловим все запросы
        .with_state(state);

    let addr = "0.0.0.0:80";
    let listener = TcpListener::bind(addr).await?;
    println!("🌍 [Gateway] HTTP Gateway listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

// ============================================================================
// HANDLER
// ============================================================================

async fn handle_request(
    State(state): State<SharedGatewayState>,
    Host(mut hostname): Host,
    req: Request<Body>,
) -> Response {
    // Убираем порт из хоста если есть (mysite.local:8080 -> mysite.local)
    if let Some(idx) = hostname.find(':') {
        hostname = hostname[..idx].to_string();
    }

    let state = state.read().await;

    // 1. Static Site
    if let Some(path) = state.static_routes.get(&hostname) {
        println!("🌍 [Gateway] Serving static for {}: {}", hostname, path);
        // ServeDir сам обрабатывает index.html, 404 и т.д.
        return match ServeDir::new(path).oneshot(req).await {
            Ok(res) => res.into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Static serve error: {}", err),
            )
                .into_response(),
        };
    }

    // 2. Reverse Proxy (пока заглушка)
    if let Some(port) = state.proxy_routes.get(&hostname) {
        return (
            StatusCode::OK,
            format!("Proxying to localhost:{} (not implemented yet)", port),
        )
            .into_response();
    }

    // 3. Not Found
    (StatusCode::NOT_FOUND, "Domain not configured in Spark").into_response()
}
