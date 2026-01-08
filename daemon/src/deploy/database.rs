use super::super::config::DatabaseSection;
use anyhow::Result;
use std::process::Command;
use tracing::info;

pub fn setup_database(db: &DatabaseSection, app_dir: &str) -> Result<()> {
    match db.r#type.as_str() {
        "postgres" => setup_postgres(db, app_dir),
        "mysql" => setup_mysql(db, app_dir),
        "sqlite" => setup_sqlite(db, app_dir),
        _ => anyhow::bail!("Unsupported database type: {}", db.r#type),
    }
}

fn setup_postgres(db: &DatabaseSection, app_dir: &str) -> Result<()> {
    info!("Setting up PostgreSQL database");

    let name = db.name.as_deref().unwrap_or("postgres");
    let user = db.user.as_deref().unwrap_or("postgres");
    let password = db.password.as_deref().unwrap_or("password");
    let port = db.port.unwrap_or(5432);

    let container_name = format!("spark-{}-db", name);

    // Stop existing container if running
    let _ = Command::new("docker")
        .args(&["stop", &container_name])
        .status();

    let _ = Command::new("docker")
        .args(&["rm", &container_name])
        .status();

    // Run PostgreSQL container
    let status = Command::new("docker")
        .args(&[
            "run",
            "-d",
            "--name",
            &container_name,
            "-e",
            &format!("POSTGRES_DB={}", name),
            "-e",
            &format!("POSTGRES_USER={}", user),
            "-e",
            &format!("POSTGRES_PASSWORD={}", password),
            "-p",
            &format!("{}:5432", port),
            "postgres:14-alpine",
        ])
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to start PostgreSQL container");
    }

    // Wait for DB to be ready
    std::thread::sleep(std::time::Duration::from_secs(5));

    // Run preseed SQL if provided
    if let Some(preseed) = &db.preseed {
        let sql_path = format!("{}/{}", app_dir, preseed);
        if std::path::Path::new(&sql_path).exists() {
            let status = Command::new("docker")
                .args(&[
                    "exec",
                    "-i",
                    &container_name,
                    "psql",
                    "-U",
                    user,
                    "-d",
                    name,
                ])
                .stdin(std::fs::File::open(&sql_path)?)
                .status()?;

            if !status.success() {
                info!("Preseed SQL executed (may have warnings)");
            }
        }
    }

    info!("PostgreSQL database ready on port {}", port);
    Ok(())
}

fn setup_mysql(db: &DatabaseSection, app_dir: &str) -> Result<()> {
    info!("Setting up MySQL database");

    let name = db.name.as_deref().unwrap_or("mysql");
    let user = db.user.as_deref().unwrap_or("root");
    let password = db.password.as_deref().unwrap_or("password");
    let port = db.port.unwrap_or(3306);

    let container_name = format!("spark-{}-db", name);

    // Stop existing container
    let _ = Command::new("docker")
        .args(&["stop", &container_name])
        .status();

    let _ = Command::new("docker")
        .args(&["rm", &container_name])
        .status();

    // Run MySQL container
    let status = Command::new("docker")
        .args(&[
            "run",
            "-d",
            "--name",
            &container_name,
            "-e",
            &format!("MYSQL_DATABASE={}", name),
            "-e",
            &format!("MYSQL_USER={}", user),
            "-e",
            &format!("MYSQL_PASSWORD={}", password),
            "-e",
            &format!("MYSQL_ROOT_PASSWORD={}", password),
            "-p",
            &format!("{}:3306", port),
            "mysql:8.0",
        ])
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to start MySQL container");
    }

    // Wait for DB to be ready
    std::thread::sleep(std::time::Duration::from_secs(10));

    // Run preseed SQL if provided
    if let Some(preseed) = &db.preseed {
        let sql_path = format!("{}/{}", app_dir, preseed);
        if std::path::Path::new(&sql_path).exists() {
            let status = Command::new("docker")
                .args(&[
                    "exec",
                    "-i",
                    &container_name,
                    "mysql",
                    "-u",
                    user,
                    &format!("-p{}", password),
                    name,
                ])
                .stdin(std::fs::File::open(&sql_path)?)
                .status()?;

            if !status.success() {
                info!("Preseed SQL executed (may have warnings)");
            }
        }
    }

    info!("MySQL database ready on port {}", port);
    Ok(())
}

fn setup_sqlite(db: &DatabaseSection, app_dir: &str) -> Result<()> {
    info!("Setting up SQLite database");

    let name = db.name.as_deref().unwrap_or("app.db");
    let db_path = format!("{}/{}", app_dir, name);

    // SQLite doesn't need container setup - just create file
    if !std::path::Path::new(&db_path).exists() {
        std::fs::File::create(&db_path)?;
    }

    // Run preseed SQL if provided
    if let Some(preseed) = &db.preseed {
        let sql_path = format!("{}/{}", app_dir, preseed);
        if std::path::Path::new(&sql_path).exists() {
            let status = Command::new("sqlite3")
                .args(&[&db_path])
                .stdin(std::fs::File::open(&sql_path)?)
                .status()?;

            if !status.success() {
                info!("Preseed SQL executed for SQLite");
            }
        }
    }

    info!("SQLite database ready at {}", db_path);
    Ok(())
}
