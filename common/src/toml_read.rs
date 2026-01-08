use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SparkFile {
    pub app: AppSection,
    pub build: Option<BuildSection>,
    pub run: Option<RunSection>,
    pub env: Option<std::collections::HashMap<String, String>>,
    pub web: Option<WebSection>,
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
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSection {
    pub domain: String,       // например "mysite.local" или "mysite.com"
    pub root: Option<String>, // папка, где лежит index.html (например "dist" или ".")
}
