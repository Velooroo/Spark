use anyhow::Result;

mod config;
mod deploy;
mod discovery;
mod protocol;
mod tls;
mod toml_read;

pub use config::CommandConfig;

use deploy::{run_daemon_server, run_deploy};
use discovery::{run_discovery_client, run_discovery_server};

pub async fn execute_command(
    client_type: &str,
    command: &str,
    config: CommandConfig,
) -> Result<()> {
    // <-- Изменено с Box<dyn Error>
    match (client_type, command) {
        ("cli", "deploy") => run_deploy(config).await,

        ("cli", "discover") => run_discovery_client().await,

        ("daemon", "start") => {
            let port = config.port.unwrap_or(7530);
            let tcp = run_daemon_server(port);
            let udp = run_discovery_server(7001);
            let _ = tokio::join!(tcp, udp);
            Ok(())
        }

        _ => {
            println!("Unknown command");
            Ok(())
        }
    }
}
