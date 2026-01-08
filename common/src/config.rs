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

/// Configuration container for Spark CLI and Sparkle Daemon commands
///
/// This structure holds all parameters needed to execute deployment,
/// discovery, and other Spark operations. Most fields are optional to
/// allow flexible configuration from multiple sources (CLI args, env vars,
/// config files).
///
/// # Field Categories
///
/// ## Authentication (Forge access)
/// * `auth_user`     - Username for Forge HTTP Basic Auth
/// * `auth_password` - Password for Forge HTTP Basic Auth
///
/// ## Network (deployment target)
/// * `host` - Target daemon hostname/IP (e.g. "192.168.1.50")
/// * `port` - Target daemon TCP port (default: 7530)
///
/// ## Repository (what to deploy)
/// * `repo`  - Repository identifier (e.g. "kazilsky/myapp")
/// * `forge` - Forge server URL (e.g. "http://forge.local:8080")
///
/// ## Deployment (where to deploy)
/// * `apps_dir` - Custom application directory (overrides default)
///
/// # Usage Example (CLI)
/// ```rust
/// use std::env;
///
/// // Build config from CLI arguments and environment
/// let config = CommandConfig {
///     auth_user: Some("kazilsky".to_string()),
///     auth_password: env::var("SPARK_PASS").ok(),
///     host: Some("192.168.1.50".to_string()),
///     port: Some(7530),
///     repo: Some("kazilsky/myapp".to_string()),
///     forge: Some("http://localhost:8080".to_string()),
///     apps_dir: None, // Use default (~/.spark/apps)
/// };
///
/// run_deploy(config).await?;
/// ```
///
/// # Usage Example (Daemon)
/// ```rust
/// // Daemon only needs port configuration
/// let config = CommandConfig {
///     port: Some(7530),
///     ..Default::default()
/// };
///
/// run_daemon_server(config.port.unwrap()).await?;
/// ```
pub struct CommandConfig {
    // ========================================================================
    // Authentication Configuration
    // ========================================================================
    /// Username for Forge authentication
    ///
    /// Used for HTTP Basic Auth when downloading private repositories.
    /// If None, requests are sent without authentication (public repos only).
    ///
    /// # Sources (priority order)
    /// 1. CLI flag: `--user <username>`
    /// 2. Environment: `SPARK_USER`
    /// 3. Config file: `~/.spark/config.toml` (future)
    pub auth_user: Option<String>,

    /// Password for Forge authentication
    ///
    /// Used for HTTP Basic Auth when downloading private repositories.
    /// Should be kept secret - prefer environment variable over CLI flag.
    ///
    /// # Security Warning
    /// Passing passwords via CLI flags may expose them in shell history.
    /// Prefer using SPARK_PASS environment variable.
    ///
    /// # Sources (priority order)
    /// 1. CLI flag: `--pass <password>` (not recommended)
    /// 2. Environment: `SPARK_PASS` (recommended)
    /// 3. Config file: `~/.spark/config.toml` (future, encrypted)
    pub auth_password: Option<String>,

    // ========================================================================
    // Network Configuration
    // ========================================================================
    /// Target daemon hostname or IP address
    ///
    /// Used by CLI to specify which daemon to connect to for deployment.
    /// Ignored by daemon itself.
    ///
    /// # Examples
    /// - `"127.0.0.1"` - Local daemon (testing)
    /// - `"192.168.1.50"` - Remote daemon (production)
    /// - `"sparkle.local"` - mDNS hostname (future feature)
    ///
    /// # Default
    /// `"127.0.0.1"` if not specified
    pub host: Option<String>,

    /// Target daemon TCP port
    ///
    /// Used by both CLI (to connect) and daemon (to listen).
    ///
    /// # Default
    /// `7530` - Default Spark deployment port
    ///
    /// # Note
    /// UDP discovery always uses port 7001 (separate from this)
    pub port: Option<u16>,

    // ========================================================================
    // Repository Configuration
    // ========================================================================
    /// Repository identifier in "owner/name" format
    ///
    /// Specifies which repository to deploy. This is used to construct
    /// the Forge download URL: `{forge}/{repo}/archive`
    ///
    /// # Format
    /// `"owner/repository"` - Must contain exactly one forward slash
    ///
    /// # Examples
    /// - `"kazilsky/myapp"` - User repository
    /// - `"org/project"` - Organization repository
    ///
    /// # Required
    /// This field is mandatory for deployment operations.
    /// CLI will return error if not provided via `--repo` flag.
    pub repo: Option<String>,

    /// Forge server base URL
    ///
    /// Base URL of the Forge git server where repositories are hosted.
    /// Used to construct archive download URLs.
    ///
    /// # Format
    /// Must be a valid HTTP/HTTPS URL without trailing slash.
    ///
    /// # Examples
    /// - `"http://localhost:8080"` - Local development
    /// - `"https://forge.example.com"` - Production server
    ///
    /// # Default
    /// `"http://localhost:8080"` if not specified
    pub forge: Option<String>,

    // ========================================================================
    // Deployment Configuration
    // ========================================================================
    /// Custom application storage directory
    ///
    /// Overrides the default application storage location. Useful for:
    /// - Custom deployment paths (e.g. `/opt/apps`)
    /// - Testing (e.g. `/tmp/spark-test`)
    /// - Multi-tenant setups
    ///
    /// # Default Behavior (if None)
    /// Uses `$SPARK_APPS_DIR` environment variable, or falls back to:
    /// - `~/.spark/apps/` (user installations)
    /// - `/var/spark/apps/` (system-wide, future)
    ///
    /// # Examples
    /// - `Some("/opt/spark/apps".to_string())` - Custom system path
    /// - `Some("/tmp/test".to_string())` - Temporary testing
    /// - `None` - Use default location
    pub apps_dir: Option<String>,
}

// ============================================================================
// DEFAULT IMPLEMENTATION
// ============================================================================

impl Default for CommandConfig {
    /// Creates a CommandConfig with sensible defaults
    ///
    /// # Default Values
    /// - `auth_user`: None (no authentication)
    /// - `auth_password`: None (no authentication)
    /// - `host`: None (will default to "127.0.0.1" in CLI)
    /// - `port`: Some(7530) (standard Spark port)
    /// - `repo`: None (must be specified by user)
    /// - `forge`: Some("http://localhost:8080") (local Forge)
    /// - `apps_dir`: None (use standard location)
    ///
    /// # Example
    /// ```rust
    /// let config = CommandConfig::default();
    /// assert_eq!(config.port, Some(7530));
    /// assert_eq!(config.forge, Some("http://localhost:8080".to_string()));
    /// ```
    fn default() -> Self {
        Self {
            auth_user: None,
            auth_password: None,
            host: None,
            port: Some(7530),
            repo: None,
            forge: Some("http://localhost:8080".to_string()),
            apps_dir: None,
        }
    }
}

