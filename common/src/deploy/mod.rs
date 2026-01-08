use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

mod gateway;
mod handler;

use crate::config::CommandConfig;
use crate::deploy::gateway::{GatewayRoutes, SharedGatewayState, run_http_gateway};
use crate::protocol::{recv_message, send_message};
use crate::tls::{accept_tls, connect_tls};
use tracing::{info, warn, error};

pub use handler::handle_deploy_request;

#[derive(Serialize, Deserialize, Debug)]
pub struct DeployMessage {
    pub repo: String,
    pub forge: String,
    pub auth_user: Option<String>,
    pub auth_password: Option<String>,
    pub auto_health: bool,
}

// ============================================================================
// DAEMON FUNCTIONS - TCP Server
// ============================================================================

pub async fn run_daemon_server(config: &CommandConfig) -> Result<()> {
    let port = config.port.unwrap_or(7530);
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    info!("Daemon listening on {}", addr);

    let gateway_state: SharedGatewayState = Arc::new(RwLock::new(GatewayRoutes::default()));

    let state_clone = gateway_state.clone();
    tokio::spawn(async move {
        if let Err(e) = run_http_gateway(state_clone).await {
            tracing::error!("Gateway crashed: {}", e);
        }
    });

    loop {
        let (tcp, addr) = listener.accept().await?;
        info!("Connection from {}", addr);

        let socket = match accept_tls(tcp).await {
            Ok(s) => s,
            Err(e) => {
                error!("TLS handshake failed: {}", e);
                continue;
            }
        };

        let state_for_handler = gateway_state.clone();
        let config_clone = (*config).clone();
        tokio::spawn(async move {
            handle_deploy_request(socket, state_for_handler, &config_clone).await;
        });
    }
}

// ============================================================================
// CLI FUNCTIONS - Deployment Client
// ============================================================================

pub async fn run_deploy(config: CommandConfig) -> Result<()> {
    let host = config.host.unwrap();
    let port = config.port.unwrap();

    let tcp = TcpStream::connect(format!("{}:{}", host, port)).await?;
    info!("Connected to {}:{}", host, port);

    let use_tls = is_local_network(&host);

    let msg = DeployMessage {
        repo: config.repo.unwrap(),
        forge: config.forge.unwrap(),
        auth_user: config.auth_user,
        auth_password: config.auth_password,
        auto_health: config.auto_health,
    };

    let json = serde_json::to_vec(&msg)?;

    if use_tls {
        info!("Using TLS for remote connection");
        let mut stream = connect_tls(tcp, &host).await?;

        send_message(&mut stream, &json).await?;
        info!("Deploy request sent");

        let response = recv_message(&mut stream).await?;
        let response_text = String::from_utf8_lossy(&response);
        info!("Response: {}", response_text);
    } else {
        info!("Using plain TCP for local network");
        let mut stream = tcp;

        send_message(&mut stream, &json).await?;
        info!("Deploy request sent");

        let response = recv_message(&mut stream).await?;
        let response_text = String::from_utf8_lossy(&response);
        info!("Response: {}", response_text);
    }

    Ok(())
}

fn is_local_network(host: &str) -> bool {
    host.starts_with("127.")        // localhost
    || host.starts_with("192.168.")
    || host.starts_with("10.")
    || host.starts_with("172.16.")
    || host.starts_with("172.17.")
    || host.starts_with("172.18.")
    || host.starts_with("172.19.")
    || host.starts_with("172.20.")
    || host.starts_with("172.21.")
    || host.starts_with("172.22.")
    || host.starts_with("172.23.")
    || host.starts_with("172.24.")
    || host.starts_with("172.25.")
    || host.starts_with("172.26.")
    || host.starts_with("172.27.")
    || host.starts_with("172.28.")
    || host.starts_with("172.29.")
    || host.starts_with("172.30.")
    || host.starts_with("172.31.")
    || host == "localhost"
}
