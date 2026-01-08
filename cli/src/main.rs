use clap::{Parser, Subcommand};
use common::{AppState, CommandConfig, execute_command, load_app_state, save_app_state};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::{error, info};
use tracing_subscriber;

#[derive(Serialize, Deserialize)]
struct AuthConfig {
    user: Option<String>,
    pass: Option<String>,
    forge: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Deploy a repository to a device
    Deploy {
        /// Repository name
        #[arg(short, long)]
        repo: String,
    },

    /// Discover devices in local network
    Discover,

    /// Start a deployed application
    Start {
        /// Application name
        app: String,
    },

    /// Stop a deployed application
    Stop {
        /// Application name
        app: String,
    },

    /// Restart a deployed application
    Restart {
        /// Application name
        app: String,
    },

    /// Rollback an application to previous version
    Rollback {
        /// Application name
        app: String,
    },
}

#[derive(Parser, Debug)]
#[command(version, about = "Spark CLI - IoT Deployment Tool", long_about = None)]
struct CLI {
    #[command(subcommand)]
    command: Commands,

    /// Target host for deploy
    #[arg(long, default_value = "127.0.0.1", global = true)]
    host: String,

    /// Target port for deploy
    #[arg(long, default_value_t = 7530, global = true)]
    port: u16,

    /// Forge URL
    #[arg(long, default_value = "http://localhost:8080", global = true)]
    forge: String,

    /// Forge username
    #[arg(long, global = true)]
    user: Option<String>,

    /// Forge password
    #[arg(long, global = true)]
    pass: Option<String>,

    /// Use GitHub instead of custom forge
    #[arg(long, conflicts_with = "forge")]
    github: bool,

    /// Auto-add health check if app doesn't have one (uses main port)
    #[arg(long)]
    auto_health: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = CLI::parse();

    // Load saved auth
    let saved_auth = load_auth_config();

    // Read from args, env, or saved
    let user = cli
        .user
        .or(saved_auth.as_ref().and_then(|a| a.user.clone()))
        .or_else(|| env::var("SPARK_USER").ok());
    let pass = cli
        .pass
        .or(saved_auth.as_ref().and_then(|a| a.pass.clone()))
        .or_else(|| env::var("SPARK_PASS").ok());

    let forge_url = if cli.github {
        "github".to_string()
    } else {
        cli.forge
    };

    let config = CommandConfig {
        auth_user: user,
        auth_password: pass,
        host: Some(cli.host),
        port: Some(cli.port),
        repo: None, // Will fill below
        forge: Some(forge_url),
        apps_dir: None,
        auto_health: cli.auto_health,
    };

    let result = match cli.command {
        Commands::Deploy { repo } => {
            let mut cfg = config;
            cfg.repo = Some(repo);
            execute_command("cli", "deploy", cfg).await
        }

        Commands::Discover => execute_command("cli", "discover", config).await,

        Commands::Start { app } => manage_process("start", &app).await,

        Commands::Stop { app } => manage_process("stop", &app).await,

        Commands::Restart { app } => manage_process("restart", &app).await,

        Commands::Rollback { app } => rollback_app(&app).await,
    };

    if let Err(e) = result {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn manage_process(action: &str, app: &str) -> anyhow::Result<()> {
    let home = env::var("HOME").unwrap_or("/tmp".to_string());
    let app_dir = format!("{}/.spark/apps/{}", home, app);

    let mut state = load_app_state(&app_dir)?.ok_or_else(|| anyhow::anyhow!("App not found"))?;

    match action {
        "start" => {
            if state.status == "running" {
                info!("Already running");
                return Ok(());
            }
            state.status = "running".to_string();
            save_app_state(&app_dir, &state)?;
            info!("Started {}", app);
        }
        "stop" => {
            if let Some(pid) = state.pid {
                Command::new("kill").arg(pid.to_string()).status()?;
            }
            state.status = "stopped".to_string();
            save_app_state(&app_dir, &state)?;
            info!("Stopped {}", app);
        }
        "restart" => {
            if let Some(pid) = state.pid {
                Command::new("kill").arg(pid.to_string()).status()?;
            }
            state.status = "running".to_string();
            save_app_state(&app_dir, &state)?;
            info!("Restarted {}", app);
        }
        _ => return Err(anyhow::anyhow!("Unknown action")),
    }
    Ok(())
}

async fn rollback_app(app: &str) -> anyhow::Result<()> {
    let home = env::var("HOME").unwrap_or("/tmp".to_string());
    let app_dir = format!("{}/.spark/apps/{}", home, app);
    let versions_dir = format!("{}/versions", app_dir);

    let entries = std::fs::read_dir(&versions_dir)?;
    let mut backups: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    backups.sort_by_key(|e| e.path());

    if let Some(latest) = backups.last() {
        let current_link = format!("{}/current", app_dir);
        if std::path::Path::new(&current_link).exists() {
            std::fs::remove_file(&current_link)?;
        }
        std::os::unix::fs::symlink(latest.path(), &current_link)?;
        info!("Rolled back {}", app);
    } else {
        error!("No backups found");
    }
    Ok(())
}

fn load_auth_config() -> Option<AuthConfig> {
    let home = env::var("HOME").unwrap_or("/tmp".to_string());
    let auth_path = format!("{}/.spark/auth.toml", home);
    if Path::new(&auth_path).exists() {
        let content = fs::read_to_string(auth_path).ok()?;
        toml::from_str(&content).ok()
    } else {
        None
    }
}

fn save_auth_config(auth: &AuthConfig) {
    let home = env::var("HOME").unwrap_or("/tmp".to_string());
    let auth_path = format!("{}/.spark/auth.toml", home);
    if let Ok(content) = toml::to_string(auth) {
        let _ = fs::write(auth_path, content);
    }
}
