use anyhow::Result;
use tracing::{error, info};

pub async fn check_health(url: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client.get(url).timeout(std::time::Duration::from_secs(10)).send().await?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("HTTP {}", resp.status()))
    }
}

pub async fn start_health_monitor(config: &super::super::config::SparkFile, app_name: &str) {
    if let Some(url) = &config.health.as_ref().map(|h| h.url.clone()) {
        let app_name = app_name.to_string();
        let url = url.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            if let Err(e) = check_health(&url).await {
                error!("Health check failed for {}: {}", app_name, e);
            } else {
                info!("Health check passed for {}", app_name);
            }
        });
    }
}