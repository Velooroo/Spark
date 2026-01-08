use super::DeployMessage;
use crate::deploy::gateway::SharedGatewayState;
use crate::protocol::{recv_message, send_message};
use crate::toml_read::SparkFile;
use anyhow::Result;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tar::Archive;
use tokio::io::{AsyncRead, AsyncWrite};

// ============================================================================
// DAEMON INTERNAL - Request Handler (GENERIC)
// ============================================================================

pub async fn handle_deploy_request<S>(mut socket: S, gateway: SharedGatewayState)
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let msg = match read_deploy_message(&mut socket).await {
        Ok(m) => m,
        Err(e) => {
            eprintln!("❌ Failed to read message: {}", e);
            return;
        }
    };

    println!("📦 [Daemon] Deploying {}", msg.repo);

    let bytes = match download_archive(&msg).await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("❌ Download failed: {}", e);
            let _ = send_error(&mut socket, "Download failed").await;
            return;
        }
    };

    let app_dir = match save_and_extract(&msg.repo, &bytes).await {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("❌ Save failed: {}", e);
            let _ = send_error(&mut socket, "Save failed").await;
            return;
        }
    };

    let started = match start_application(&app_dir, gateway).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("X Start failed: {}", e);
            let _ = send_error(
                &mut socket,
                format!("Start application error: {}", e).trim(),
            )
            .await;
            return;
        }
    };

    let response = format!("✅ Deployed to {}", app_dir);
    let _ = send_response(&mut socket, &response).await;
}

// ============================================================================
// DAEMON INTERNAL - Helper Functions (ALL GENERIC)
// ============================================================================

async fn read_deploy_message<S>(socket: &mut S) -> Result<DeployMessage>
where
    S: AsyncRead + Unpin,
{
    let data = recv_message(socket).await?;
    let msg: DeployMessage = serde_json::from_slice(&data)?;
    Ok(msg)
}

async fn download_archive(msg: &DeployMessage) -> Result<Vec<u8>> {
    let url = if msg.forge == "github" {
        format!("https://api.github.com/repos/{}/tarball/main", msg.repo)
    } else {
        format!("{}/{}/archive", msg.forge, msg.repo)
    };

    println!("⬇️  [Daemon] Downloading {}...", url);

    let client = reqwest::Client::new();
    let mut req = client.get(&url).header("User-Agent", "Spark-Deploy-Agent");

    if let (Some(user), Some(pass)) = (&msg.auth_user, &msg.auth_password) {
        if msg.forge == "github" {
            req = req.bearer_auth(pass); // pass здесь это Personal Access Token
        } else {
            req = req.basic_auth(user, Some(pass));
        }
    }

    let response = req.send().await?;

    if !response.status().is_success() {
        anyhow::bail!("HTTP {}", response.status());
    }

    let bytes = response.bytes().await?;
    println!("✅ [Daemon] Downloaded {} bytes", bytes.len());

    Ok(bytes.to_vec())
}

async fn save_and_extract(repo: &str, data: &[u8]) -> Result<String> {
    let home = std::env::var("HOME").unwrap_or("/tmp".to_string());
    let base_dir = std::env::var("SPARK_APPS_DIR").unwrap_or(format!("{}/.spark/apps", home));

    let app_name = repo.replace("/", "_");
    let app_dir = format!("{}/{}", base_dir, app_name);

    std::fs::create_dir_all(&app_dir)?;

    let archive_path = format!("{}/{}.tar.gz", app_dir, app_name);
    let mut file = File::create(&archive_path)?;
    file.write_all(data)?;

    // println!("💾 [Daemon] Saved to {}", archive_path);

    // Disarchive
    disarchive_and_delete_archive(&archive_path, &app_dir).await?;

    println!("💾 [Daemon] Saved to {}", &app_dir);

    Ok(app_dir)
}

async fn disarchive_and_delete_archive(archive_path: &String, app_dir: &String) -> Result<()> {
    let file = std::fs::File::open(&archive_path)?;
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);

    archive.unpack(app_dir)?;
    println!("📦 [Daemon] Extracted to {}", app_dir);

    std::fs::remove_file(&archive_path)?;

    Ok(())
}

async fn start_application(app_dir: &String, gateway: SharedGatewayState) -> Result<()> {
    let config_path = format!("{}/spark.toml", app_dir);
    let content =
        std::fs::read_to_string(&config_path).map_err(|_| anyhow::anyhow!("spark.toml missing"))?;

    let config: SparkFile = toml::from_str(&content)?;

    println!("{:?}", config);

    println!("🚀 [Daemon] Starting {}...", config.app.name);

    // 2. Билд (если надо)
    if let Some(build) = config.build {
        println!("🔨 Building: {}", build.command);
        let status = Command::new("sh")
            .arg("-c")
            .arg(build.command)
            .current_dir(app_dir)
            .status()?;

        if !status.success() {
            return Err(anyhow::anyhow!("Build failed"));
        }
    }

    if let Some(web) = config.web {
        // === ВАРИАНТ 1: Статический сайт (через Gateway) ===
        println!("🌍 [Daemon] Registering static site: {}", web.domain);

        let root_path = format!("{}/{}", app_dir, web.root.unwrap_or(".".to_string()));

        // Добавляем в роутер (в память)
        gateway
            .write()
            .await
            .static_routes
            .insert(web.domain.clone(), root_path);

        println!("✅ Site is live at http://{}:8080", web.domain);
    } else if let Some(run) = config.run {
        // === ВАРИАНТ 2: Обычный процесс (скрипт/бинарник) ===
        println!("▶️ Executing: {}", run.command);

        Command::new("sh")
            .arg("-c")
            .arg(run.command)
            .current_dir(app_dir)
            .spawn()?;

        println!("✅ Process started in background");
    } else {
        println!("⚠️ No [web] or [run] section found!");
    }
    Ok(())
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
