use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SparkFile {
    pub app: AppSection,
    pub build: Option<BuildSection>,
    pub run: Option<RunSection>,
    pub env: Option<std::collections::HashMap<String, String>>,
    pub web: Option<WebSection>,
    pub health: Option<HealthSection>,
    pub isolation: Option<IsolationSection>,
    pub storage: Option<StorageSection>,
    pub database: Option<DatabaseSection>,
    pub notify: Option<NotifySection>,
    pub secrets: Option<SecretsSection>,
    pub resource_limits: Option<ResourceLimitsSection>,
    pub hooks: Option<HooksSection>,
    pub metrics: Option<MetricsSection>,
    pub strategy: Option<StrategySection>,
    pub auto_health: Option<bool>, // Auto-add health check if missing
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppSection {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildSection {
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunSection {
    pub command: String,
    pub port: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSection {
    pub domain: String,
    pub root: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthSection {
    pub url: String,
    pub timeout: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IsolationSection {
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageSection {
    pub r#type: String,
    pub bucket: Option<String>,
    pub endpoint: Option<String>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub size: Option<String>,
    pub mount: Option<String>,
    pub public: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseSection {
    pub r#type: String,
    pub name: Option<String>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub port: Option<u16>,
    pub preseed: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotifySection {
    pub on_success: Option<Vec<String>>,
    pub on_fail: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecretsSection {
    #[serde(flatten)]
    pub secrets: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceLimitsSection {
    pub memory: Option<String>,
    pub cpu: Option<String>,
    pub timeout: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HooksSection {
    pub pre_deploy: Option<String>,
    pub post_deploy: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsSection {
    pub pushgateway: Option<String>,
    pub collect: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StrategySection {
    pub r#type: String,
    pub percent: Option<u8>,
    pub wait_time: Option<String>,
}
