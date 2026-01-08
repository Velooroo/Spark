use super::super::config::SparkFile;
use anyhow::Result;

pub fn load_spark_config(app_dir: &str) -> Result<SparkFile> {
    let config_path = format!("{}/spark.toml", app_dir);
    let content =
        std::fs::read_to_string(&config_path).map_err(|_| anyhow::anyhow!("spark.toml missing"))?;

    let config: SparkFile = toml::from_str(&content)?;
    Ok(config)
}
