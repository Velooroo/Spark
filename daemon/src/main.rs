use common::{CommandConfig, execute_command};

#[tokio::main]
async fn main() {
    println!("🔥 Sparkle Daemon starting...");

    // Чисто, без костылей
    let config = CommandConfig {
        port: Some(7530),
        ..Default::default() // Остальное с дефолта
    };

    if let Err(e) = execute_command("daemon", "start", config).await {
        eprintln!("💀 Daemon crashed: {}", e);
    }
}
