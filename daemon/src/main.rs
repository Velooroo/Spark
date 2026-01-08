mod config;
mod deploy;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DeployMessage {
    pub repo: String,
    pub forge: String,
    pub auth_user: Option<String>,
    pub auth_password: Option<String>,
    pub auto_health: bool,
}

use common::{CommandConfig, execute_command};
use deploy::handler::handle_deploy_request;
use tracing::{error};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Sparkle Daemon starting...");

    // Clean setup
    let config = CommandConfig {
        port: Some(7530),
        ..Default::default() // Rest with defaults
    };

    if let Err(e) = execute_command("daemon", "start", config).await {
        error!("Daemon crashed: {}", e);
    }
}
