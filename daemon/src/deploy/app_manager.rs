use super::super::config::SparkFile;
use crate::deploy::gateway::SharedGatewayState;
use std::process::Command;
use tracing::{info, warn};

pub async fn start_application(
    config: &SparkFile,
    app_dir: &str,
    gateway: SharedGatewayState,
) -> anyhow::Result<Option<u32>> {
    info!("Starting {}...", config.app.name);

    let pid = if let Some(web) = &config.web {
        info!("Registering static site: {}", web.domain);

        let root_path = format!("{}/{}", app_dir, web.root.as_ref().unwrap_or(&".".to_string()));

        gateway
            .write()
            .await
            .static_routes
            .insert(web.domain.clone(), root_path);

        info!("Site live at http://{}:8080", web.domain);
        None
    } else if let Some(run) = &config.run {
        info!("Executing: {}", run.command);

        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(&run.command).current_dir(app_dir);

        if let Some(isolation) = &config.isolation {
            match isolation.r#type.as_str() {
                "systemd" => {
                    cmd = Command::new("systemd-run");
                    cmd.arg("--user").arg("--scope").arg("sh").arg("-c").arg(&run.command).current_dir(app_dir);
                }
                "chroot" => {
                    cmd = Command::new("chroot");
                    cmd.arg(app_dir).arg("sh").arg("-c").arg(&run.command);
                }
                _ => {} // none
            }
        }

        let child = cmd.spawn()?;
        let pid = child.id();

        info!("Process started with PID {}", pid);
        Some(pid)
    } else {
        warn!("No [web] or [run] section found!");
        None
    };

    Ok(pid)
}