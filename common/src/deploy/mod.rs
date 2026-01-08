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

pub use handler::handle_deploy_request;

#[derive(Serialize, Deserialize, Debug)]
pub struct DeployMessage {
    pub repo: String,
    pub forge: String,
    pub auth_user: Option<String>,
    pub auth_password: Option<String>,
}

// ============================================================================
// DAEMON FUNCTIONS - TCP Server
// ============================================================================

pub async fn run_daemon_server(port: u16) -> Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    println!("🔥 [Daemon] Listening on {}", addr);

    let gateway_state: SharedGatewayState = Arc::new(RwLock::new(GatewayRoutes::default()));

    let state_clone = gateway_state.clone();
    tokio::spawn(async move {
        if let Err(e) = run_http_gateway(state_clone).await {
            eprintln!("❌ Gateway crashed: {}", e);
        }
    });

    loop {
        let (tcp, addr) = listener.accept().await?;
        println!("🔌 [Daemon] Connection from {}", addr);

        let socket = match accept_tls(tcp).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("❌ TLS handshake failed: {}", e);
                continue;
            }
        };

        let state_for_handler = gateway_state.clone();
        tokio::spawn(handle_deploy_request(socket, state_for_handler));
    }
}

// ============================================================================
// CLI FUNCTIONS - Deployment Client
// ============================================================================

pub async fn run_deploy(config: CommandConfig) -> Result<()> {
    let host = config.host.unwrap();
    let port = config.port.unwrap();

    let tcp = TcpStream::connect(format!("{}:{}", host, port)).await?;

    // Проверяем: локальная сеть или интернет?
    let use_tls = is_local_network(&host);

    let msg = DeployMessage {
        repo: config.repo.unwrap(),
        forge: config.forge.unwrap(),
        auth_user: config.auth_user,
        auth_password: config.auth_password,
    };

    let json = serde_json::to_vec(&msg)?;

    // Отправляем через TLS или обычный TCP
    if use_tls {
        println!("🔒 Using TLS (remote connection)");
        let mut stream = connect_tls(tcp, &host).await?;

        send_message(&mut stream, &json).await?;
        println!("✅ [CLI] Deploy request sent!");

        let response = recv_message(&mut stream).await?;
        let response_text = String::from_utf8_lossy(&response);
        println!("📬 [Daemon response]: {}", response_text);
    } else {
        println!("🔓 No TLS (local network)");
        let mut stream = tcp;

        send_message(&mut stream, &json).await?;
        println!("✅ [CLI] Deploy request sent!");

        let response = recv_message(&mut stream).await?;
        let response_text = String::from_utf8_lossy(&response);
        println!("📬 [Daemon response]: {}", response_text);
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
