# Spark Configuration Guide

## Basic Structure

Every `spark.toml` needs at least:

```toml
[app]
name = "your-app-name"
version = "1.0.0"
```

## For Different App Types

### 1. Static Website
```toml
[app]
name = "my-site"
version = "1.0.0"

[web]
domain = "mysite.local"
root = "./build"
```

### 2. Backend Service
```toml
[app]
name = "my-api"
version = "1.0.0"

[run]
command = "node server.js"
port = 3000
```

### 3. App with Build Step
```toml
[app]
name = "my-rust-app"
version = "1.0.0"

[build]
command = "cargo build --release"

[run]
command = "./target/release/my-app"
port = 8080
```

## Optional Sections

### Health Checks
```toml
[health]
url = "http://localhost:3000/health"
timeout = 30
```

### Process Isolation
```toml
[isolation]
type = "systemd"  # or "chroot" or "none"
```

## Quick Copy-Paste Templates

- **Node.js API**: Copy `examples/nodejs-api.toml`
- **Python App**: Copy `examples/python-api.toml`
- **Static Site**: Copy `examples/website.toml`

## Tips

- `name` should be unique across your deployments
- `port` is optional but helps with health checks
- `domain` in `[web]` creates a virtual host
- All sections except `[app]` are optional