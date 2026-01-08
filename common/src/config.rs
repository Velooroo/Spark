// ============================================================================
// CONFIGURATION STRUCTURE
// ============================================================================
//
// CommandConfig is the central configuration structure used by both
// Spark CLI and Sparkle Daemon. It holds all parameters needed for
// deployment operations, discovery, and network communication.
//
// This struct uses Option<T> for most fields to support:
// - CLI argument parsing (fields may be omitted)
// - Environment variable fallbacks
// - Flexible configuration sources
// ============================================================================

/// Config for CLI and daemon commands
#[derive(Clone)]
pub struct CommandConfig {
    // ========================================================================
    // Authentication Configuration
    // ========================================================================
    /// Username for Forge auth
    pub auth_user: Option<String>,

    /// Password for Forge auth
    pub auth_password: Option<String>,

    // ========================================================================
    // Network Configuration
    // ========================================================================
    /// Target daemon host/IP
    pub host: Option<String>,

    /// Target daemon port
    pub port: Option<u16>,

    // ========================================================================
    // Repository Configuration
    // ========================================================================
    /// Repository to deploy
    pub repo: Option<String>,

    /// Forge server URL
    pub forge: Option<String>,

    // ========================================================================
    // Deployment Configuration
    // ========================================================================
    /// Custom apps directory
    pub apps_dir: Option<String>,

    /// Auto-add health check if missing
    pub auto_health: bool,
}

// ============================================================================
// DEFAULT IMPLEMENTATION
// ============================================================================

impl Default for CommandConfig {
    fn default() -> Self {
        Self {
            auth_user: None,
            auth_password: None,
            host: None,
            port: Some(7530),
            repo: None,
            forge: Some("http://localhost:8080".to_string()),
            apps_dir: None,
            auto_health: false,
        }
    }
}
