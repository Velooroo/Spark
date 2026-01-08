use super::super::config::SparkFile;
use super::{config_loader, hooks, database, health_monitor, app_manager, gateway::SharedGatewayState};
use common::protocol::{recv_message, send_message};
use common::{save_app_state, AppState, CommandConfig};
use super::super::DeployMessage;
use anyhow::Result;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tar::Archive;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{info, warn, error};

pub async fn handle_deploy_request<S>(mut socket: S, gateway: SharedGatewayState, config: &CommandConfig)
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let msg = match read_deploy_message(&mut socket).await {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to read message: {}", e);
            return;
        }
    };

    info!("Deploying {}", msg.repo);

    let bytes = match download_archive(&msg).await {
        Ok(b) => b,
        Err(e) => {
            error!("Download failed: {}", e);
            let _ = send_error(&mut socket, "Download failed").await;
            return;
        }
    };

    let app_dir = match save_and_extract(&msg.repo, &bytes).await {
        Ok(dir) => dir,
        Err(e) => {
            error!("Save failed: {}", e);
            let _ = send_error(&mut socket, "Save failed").await;
            return;
        }
    };

    let mut config = match config_loader::load_spark_config(&app_dir) {
        Ok(c) => c,
        Err(e) => {
            error!("Config load failed: {}", e);
            let _ = send_error(&mut socket, "Config load failed").await;
            return;
        }
    };

    // Auto-add health check if requested (CLI flag or TOML option) and not present
    let should_auto_health = msg.auto_health || config.auto_health.unwrap_or(false);
    if config.health.is_none() && should_auto_health {
        if let Some(port) = config.run.as_ref().and_then(|r| r.port) {
            config.health = Some(super::super::config::HealthSection {
                url: format!("http://localhost:{}", port),
                timeout: Some(30),
            });
            info!("Auto-added health check for port {}", port);
        }
    }

    hooks::run_pre_deploy_hooks(&config, &app_dir);

    let started = match app_manager::start_application(&config, &app_dir, gateway.clone()).await {
        Ok(s) => s,
        Err(e) => {
            error!("Start failed: {}", e);
            let _ = send_error(&mut socket, "Start failed").await;
            return;
        }
    };

    if let Some(db) = &config.database {
        if let Err(e) = database::setup_database(db, &app_dir) {
            error!("Database setup failed: {}", e);
        }
    }

    let state = AppState {
        name: config.app.name.clone(),
        version: config.app.version.clone(),
        status: "running".to_string(),
        pid: started,
        port: config.run.as_ref().and_then(|r| r.port),
        health_url: config.health.as_ref().map(|h| h.url.clone()),
        isolation: config.isolation.as_ref().map(|i| i.r#type.clone()),
    };
    if let Err(e) = save_app_state(&app_dir, &state) {
        error!("Failed to save state: {}", e);
    }

    health_monitor::start_health_monitor(&config, &config.app.name).await;

    hooks::run_post_deploy_hooks(&config, &app_dir);

    let response = format!("Deployed to {}", app_dir);
    info!("{}", response);
    let _ = send_response(&mut socket, &response).await;
}

async fn read_deploy_message<S>(socket: &mut S) -> Result<DeployMessage>
where
    S: AsyncRead + Unpin,
{
    let data: Vec<u8> = recv_message(socket).await?;
    let msg: DeployMessage = serde_json::from_slice(&data)?;
    Ok(msg)
}

async fn download_archive(msg: &DeployMessage) -> Result<Vec<u8>> {
    let url = if msg.forge == "github" {
        format!("https://api.github.com/repos/{}/tarball/main", msg.repo)
    } else {
        format!("{}/{}/archive", msg.forge, msg.repo)
    };

    info!("Downloading {}...", url);

    let client = reqwest::Client::new();
    let mut req = client.get(&url).header("User-Agent", "Spark-Deploy-Agent");

    if let (Some(user), Some(pass)) = (&msg.auth_user, &msg.auth_password) {
        if msg.forge == "github" {
            req = req.bearer_auth(pass);
        } else {
            req = req.basic_auth(user, Some(pass));
        }
    }

    let response = req.send().await?;

    if !response.status().is_success() {
        anyhow::bail!("HTTP {}", response.status());
    }

    let bytes = response.bytes().await?;
    info!("Downloaded {} bytes", bytes.len());

    Ok(bytes.to_vec())
}

async fn save_and_extract(repo: &str, data: &[u8]) -> Result<String> {
    let home = std::env::var("HOME").unwrap_or("/tmp".to_string());
    let base_dir = std::env::var("SPARK_APPS_DIR").unwrap_or(format!("{}/.spark/apps", home));

    let app_name = repo.replace("/", "_");
    let app_dir = format!("{}/{}", base_dir, app_name);

    std::fs::create_dir_all(&app_dir)?;

    // Backup current version for rollback
    let current_link = format!("{}/current", app_dir);
    if std::path::Path::new(&current_link).exists() {
        let backup_dir = format!("{}/versions/{}", app_dir, chrono::Utc::now().timestamp());
        std::fs::create_dir_all(&backup_dir)?;
        if let Ok(current_path) = std::fs::read_link(&current_link) {
            std::fs::rename(current_path, &backup_dir)?;
        }
    }

    let archive_path = format!("{}/{}.tar.gz", app_dir, app_name);
    let mut file = File::create(&archive_path)?;
    file.write_all(data)?;

    // Extract
    let extracted_dir = disarchive_and_delete_archive(&archive_path, &app_dir).await?;

    // Create current symlink
    let current_link = format!("{}/current", app_dir);
    if std::path::Path::new(&current_link).exists() {
        std::fs::remove_file(&current_link)?;
    }
    std::os::unix::fs::symlink(&extracted_dir, &current_link)?;

    info!("Saved to {}", &app_dir);

    Ok(app_dir)
}

async fn disarchive_and_delete_archive(archive_path: &String, app_dir: &String) -> Result<String> {
    let file = std::fs::File::open(&archive_path)?;
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);

    archive.unpack(app_dir)?;
    info!("Extracted to {}", app_dir);

    std::fs::remove_file(&archive_path)?;

    Ok(app_dir.clone())
}

async fn send_response<S>(socket: &mut S, msg: &str) -> Result<()>
where
    S: AsyncWrite + Unpin,
{
    send_message(socket, msg.as_bytes()).await
}

async fn send_error<S>(socket: &mut S, error: &str) -> Result<()>
where
    S: AsyncWrite + Unpin,
{
    let msg = format!("❌ Error: {}", error);
    send_message(socket, msg.as_bytes()).await
}