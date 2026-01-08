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

#[derive(Debug, Default)]
pub struct GatewayRoutes {
    pub static_routes: HashMap<String, String>,
    pub proxy_routes: HashMap<String, u16>,
}

pub type SharedGatewayState = Arc<RwLock<GatewayRoutes>>;

pub async fn run_http_gateway(state: SharedGatewayState) -> Result<()> {
    let app = Router::new()
        .fallback(handle_request)
        .with_state(state);

    let addr = "0.0.0.0:80";
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("HTTP Gateway listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle_request(
    State(state): State<SharedGatewayState>,
    Host(mut hostname): Host,
    req: Request<Body>,
) -> Response {
    if let Some(idx) = hostname.find(':') {
        hostname = hostname[..idx].to_string();
    }

    let state = state.read().await;

    if let Some(path) = state.static_routes.get(&hostname) {
        tracing::info!("Serving static for {}: {}", hostname, path);
        return match ServeDir::new(path).oneshot(req).await {
            Ok(res) => res.into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Static serve error: {}", err),
            )
                .into_response(),
        };
    }

    if let Some(port) = state.proxy_routes.get(&hostname) {
        return (
            StatusCode::OK,
            format!("Proxying to localhost:{} (not implemented yet)", port),
        )
            .into_response();
    }

    (StatusCode::NOT_FOUND, "Domain not configured in Spark").into_response()
}