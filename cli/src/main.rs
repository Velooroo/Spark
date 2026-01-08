use clap::{Parser, Subcommand};
use common::{CommandConfig, execute_command};
use std::env;

#[derive(Subcommand, Debug)]
enum Commands {
    /// Deploy a repository to a device
    Deploy {
        /// Repository (e.g. kazilsky/test)
        #[arg(short, long)]
        repo: String,
    },

    /// Discover "Sparkle" devices in local network
    Discover,
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

    /// Forge username (or set SPARK_USER env)
    #[arg(long, global = true)]
    user: Option<String>,

    /// Forge password (or set SPARK_PASS env)
    #[arg(long, global = true)]
    pass: Option<String>,

    /// Boolean parameter for understand, needable use github or not
    #[arg(long, conflicts_with = "forge")]
    github: bool,
}

#[tokio::main]
async fn main() {
    let cli = CLI::parse();

    // Читаем из env, если не передано через аргументы
    let user = cli.user.or_else(|| env::var("SPARK_USER").ok());
    let pass = cli.pass.or_else(|| env::var("SPARK_PASS").ok());

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
        repo: None, // Заполним ниже
        forge: Some(forge_url),
        apps_dir: None,
    };

    let result = match cli.command {
        Commands::Deploy { repo } => {
            let mut cfg = config;
            cfg.repo = Some(repo);
            execute_command("cli", "deploy", cfg).await
        }

        Commands::Discover => execute_command("cli", "discover", config).await,
    };

    if let Err(e) = result {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
}
