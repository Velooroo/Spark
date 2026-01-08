# ⚡ Spark

[![Release](https://img.shields.io/github/v/release/Velooroo/spark)](https://github.com/Velooroo/Spark/releases)
[![License](https://img.shields.io/github/license/Velooroo/Spark)](LICENSE)

**The decentralized deployment toolchain for the edge.**

Spark replaces complex Ansible setups and Docker registries with a lightweight, single-binary ecosystem designed for IoT devices, home labs, and cyberdecks.

> **Zero Dependencies.** No Python, no Node.js, no Docker required on the target.

## ✨ Features

- **Push-to-Deploy:** Ship code from your laptop to Raspberry Pi in seconds.
- **Auto-Discovery:** Finds devices in your local network via UDP broadcast.
- **Built-in Gateway:** Automatic reverse proxy (Virtual Hosts) for web apps.
- **Secure:** TLS encryption for all data transfers.
- **Universal:** Supports GitHub, GitLab, and self-hosted Forge servers.

---

## 🚀 Quick Start

### 1. Install (Developer & Server)
Download the binary from [Releases](https://github.com/Velooroo/Spark/releases) or build from source:

```bash
cargo install --path .
```

### 2. Setup the Target Device (Daemon)
Run this on your Raspberry Pi / VPS:

```bash
# Starts TCP listener (7530) and UDP discovery (7001)
sparkle
```

### 3. Configure Your App
Create `spark.toml` in your project root. See `examples/` for templates:

```toml
[app]
name = "my-app"
version = "1.0.0"

[run]
command = "node server.js"
port = 3000

auto_health = true
```

### 4. Deploy from your Machine (CLI)

```bash
# Deploy to auto-discovered device
spark deploy --repo user/my-project

# Or specify target
spark deploy --repo user/my-project --host 192.168.1.100 --port 7530
```

### 5. Manage Apps

```bash
# Control running apps
spark start my-app
spark stop my-app
spark restart my-app
spark rollback my-app

# Deploy with overrides
spark deploy --repo user/my-app --auto-health  # Force health check
spark deploy --repo user/my-app --isolation systemd  # Override isolation
```

### 2. Setup the Target Device (Daemon)
Run this on your Raspberry Pi / VPS:

```bash
# Starts TCP listener (7530) and UDP discovery (7001)
sparkle
```

<img width="358" height="67" alt="image" src="https://github.com/user-attachments/assets/dc4686fb-bd04-4f70-ac00-7de54a8ed4f9" />


### 3. Deploy from your Machine (CLI)

Add a `spark.toml` to your project and deploy:

```bash
# Auto-discover device and deploy
spark deploy --repo user/my-project

# Add automatic health check if app doesn't have one
spark deploy --repo user/my-project --auto-health
```

 <img width="791" height="207" alt="image" src="https://github.com/user-attachments/assets/7ca91006-224f-433b-8c3c-027af7427903" />

### 5. Manage Apps

```bash
# Control running apps
spark start my-app
spark stop my-app
spark restart my-app
spark rollback my-app
```

## 🧪 Testing Guide

### Local Testing (No Hardware Needed)

1. **Build Spark**:
   ```bash
   cargo build --release
   ```

2. **Start Daemon Locally**:
   ```bash
   ./target/release/sparkle
   ```

3. **Create Test App**:
   ```bash
   mkdir test-app && cd test-app
   echo 'console.log("Hello from Spark!"); setInterval(() => {}, 1000);' > server.js
   echo '{"name": "test-app", "version": "1.0.0", "scripts": {"start": "node server.js"}}' > package.json
   ```

4. **Create spark.toml**:
   ```toml
   [app]
   name = "test-app"
   version = "1.0.0"

   [run]
   command = "node server.js"
   port = 3000

   auto_health = true
   ```

5. **Deploy**:
   ```bash
   # In another terminal
   spark deploy --repo test-app --host 127.0.0.1
   ```

6. **Test Management**:
   ```bash
   spark start test-app
   spark stop test-app
   curl http://localhost:3000/health  # Should work with auto-health
   ```

### Remote Testing

1. Copy `spark` and `sparkle` binaries to your target device
2. Run `sparkle` on device
3. Use `spark deploy --repo your-repo --host device-ip` from dev machine

### Troubleshooting

- **Connection Issues**: Check firewall, ensure ports 7530 (TCP) and 7001 (UDP) are open
- **Health Checks Fail**: Verify your app listens on the specified port
- **Docker DB Issues**: Ensure Docker is installed and running on target device
- **Permission Errors**: Run daemon with appropriate permissions for systemd isolation

---

## 🛠 Configuration (`spark.toml`)

Place this file in the root of your project. See `examples/` for ready-to-use templates.

**CLI flags override TOML settings** - you can use either or both.

### Quick Reference:
- `[app]` - Required: name and version of your app
- `[build]` - Optional: commands to build/compile your app
- `[run]` - For backend services: command and port
- `[web]` - For static sites: domain and root folder
- `[health]` - Optional: custom health check endpoint
- `auto_health` - Optional: auto health check (boolean)
- `[isolation]` - Optional: process isolation (systemd/chroot/none)

### Simple Web App

```toml
[app]
name = "my-web-app"
version = "1.0.0"

[web]
domain = "myapp.local"  # Access at http://myapp.local:8080
root = "./dist"         # Folder with index.html
```

### Backend Service

```toml
[app]
name = "api-server"
version = "1.0.0"

[build]                 # Optional: build before run
command = "npm run build"

[run]
command = "node server.js"
port = 3000             # Used for auto-health if --auto-health flag

[health]                # Optional: custom health check
url = "http://localhost:3000/health"
timeout = 10

# OR: auto health check (alternative to [health])
auto_health = true     # Auto-check main port if no custom [health]

[isolation]             # Optional: process isolation
type = "systemd"       # systemd, chroot, none

[database]             # Optional: auto database setup
type = "postgres"
name = "myapp_db"
user = "dbuser"
password = "secret"
port = 5432
preseed = "./db/init.sql"

[storage]              # Optional: storage setup
type = "s3"
bucket = "mybucket"
endpoint = "https://s3.amazonaws.com"

[hooks]                # Optional: custom scripts
pre_deploy = "sh ./scripts/pre.sh"
post_deploy = "sh ./scripts/post.sh"

[notify]               # Optional: notifications
on_success = ["mailto:admin@example.com"]
on_fail = ["telegram:@mychannel"]
```

**More examples:** Check `examples/` folder for Node.js, Python, and static site configs.

## 🛠 Advanced Features

Spark supports advanced deployment configurations for complex applications:

### Database Auto-Setup
Automatically deploy and configure databases using Docker:

```toml
[database]
type = "postgres"      # postgres, mysql, sqlite
name = "myapp_db"
user = "dbuser"
password = "secret"
port = 5432
preseed = "./db/init.sql"  # Optional: SQL file to execute after setup
```

Spark will:
- Pull and run the appropriate Docker container
- Configure environment variables
- Wait for database to be ready
- Execute preseed SQL if provided
- Make database accessible on specified port

### Storage Integration
Integrate with S3-compatible storage:

```toml
[storage]
type = "s3"            # s3, minio
bucket = "mybucket"
endpoint = "https://s3.amazonaws.com"
access_key = "AKIA..."
secret_key = "..."
```

### Deployment Hooks
Run custom scripts before/after deployment:

```toml
[hooks]
pre_deploy = "sh ./scripts/backup.sh"
post_deploy = "sh ./scripts/notify.sh"
```

### Notifications
Get notified on deployment success/failure:

```toml
[notify]
on_success = ["mailto:admin@example.com", "telegram:@mychannel"]
on_fail = ["mailto:devops@example.com"]
```

### Resource Limits
Set memory/CPU limits:

```toml
[resource_limits]
memory = "256MB"
cpu = "0.5"
timeout = "300s"
```

### Secrets Management
Inject secrets from environment:

```toml
[secrets]
API_KEY = "env:MY_API_KEY"
DB_PASS = "env:DATABASE_PASSWORD"
```

### Deployment Strategies
Use canary or blue-green deployments:

```toml
[strategy]
type = "canary"        # canary, bluegreen, rolling
percent = 20
wait_time = "60s"
```

### Backend Service (Python/Rust/Node)

```toml
[app]
name = "sensor-api"
version = "0.1.0"

[build]
command = "cargo build --release"

[run]
command = "./target/release/sensor-api"
port = 3000
```

---

## 📦 How it works

1.  **Spark CLI** packs your repository (from GitHub or Forge).
2.  Connects to **Sparkle Daemon** via TLS-encrypted TCP.
3.  **Daemon** downloads, extracts, and builds the application.
4.  **Daemon** launches the process or registers the domain in the internal Gateway.

## 🔐 Security

Spark uses **TLS** for all data transfer.
- **Local Network:** Uses self-signed certificates automatically.
- **Internet/Production:** Supports Let's Encrypt / Custom certificates via env vars:
  - `SPARK_TLS_CERT_FILE`
  - `SPARK_TLS_KEY_FILE`

---

## License

Apache-2.0 license © [VeloroLABS](https://github.com/Velooroo)
