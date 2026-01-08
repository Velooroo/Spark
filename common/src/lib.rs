use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::error;

mod config;
mod deploy;
mod discovery;
pub mod protocol;
mod tls;

pub use config::CommandConfig;

use deploy::{run_daemon_server, run_deploy};
use discovery::{run_discovery_client, run_discovery_server};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppState {
    pub name: String,
    pub version: String,
    pub status: String, // running, stopped, failed
    pub pid: Option<u32>,
    pub port: Option<u16>,
    pub health_url: Option<String>,
    pub isolation: Option<String>,
}

pub fn save_app_state(app_dir: &str, state: &AppState) -> Result<()> {
    let state_path = format!("{}/state.toml", app_dir);
    let content = toml::to_string(state)?;
    fs::write(state_path, content)?;
    Ok(())
}

pub fn load_app_state(app_dir: &str) -> Result<Option<AppState>> {
    let state_path = format!("{}/state.toml", app_dir);
    if Path::new(&state_path).exists() {
        let content = fs::read_to_string(state_path)?;
        let state: AppState = toml::from_str(&content)?;
        Ok(Some(state))
    } else {
        Ok(None)
    }
}

pub async fn execute_command(
    _client_type: &str,
    command: &str,
    config: CommandConfig,
) -> Result<()> {
    match command {
        "deploy" => deploy::run_deploy(config).await,
        "discover" => discovery::run_discovery_client().await,
        "start" => {
            // Daemon start logic here
            deploy::run_daemon_server(&config).await
        }
        _ => {
            error!("Unknown command");
            Ok(())
        }
    }
}
