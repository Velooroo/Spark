use anyhow::Result;
use tokio::net::UdpSocket;

// ============================================================================
// CLI FUNCTIONS
// ============================================================================

/// Discovers Sparkle daemons in the local network via UDP broadcast
///
/// This function sends a UDP broadcast message to all devices on the local
/// network and waits for the first daemon to respond with its address.
///
/// # Protocol
/// - Sends: "SPARK_DISCOVER" as broadcast to 255.255.255.255:7001
/// - Receives: "SPARK_HERE" from daemon with its IP address
///
/// # Returns
/// - `Ok(())` if at least one daemon was discovered
/// - `Err` if network operation fails
///
/// # Example
/// ```
/// // CLI usage: spark discover
/// run_discovery_client().await?;
/// ```
pub async fn run_discovery_client() -> Result<()> {
    // Bind to any available port on all network interfaces
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    // Enable broadcast mode for UDP socket
    socket.set_broadcast(true)?;

    println!("📡 [CLI] Broadcasting discovery...");

    // Send discovery message to all devices on port 7001
    socket
        .send_to(b"SPARK_DISCOVER", "255.255.255.255:7001")
        .await?;

    // Prepare buffer for response (1KB is enough for IP address)
    let mut buf = [0; 1024];

    // Wait for first response from any daemon
    let (_len, addr) = socket.recv_from(&mut buf).await?;

    println!("✅ [CLI] Found device at: {}", addr);
    Ok(())
}

// ============================================================================
// DAEMON FUNCTIONS
// ============================================================================

/// Listens for discovery requests from Spark CLI via UDP broadcast
///
/// This function runs indefinitely, responding to all discovery requests
/// from Spark CLI clients on the local network. It should be spawned as
/// a background task alongside the main TCP deployment server.
///
/// # Protocol
/// - Listens on: 0.0.0.0:<port> (default 7001)
/// - Receives: "SPARK_DISCOVER" from CLI clients
/// - Responds: "SPARK_HERE" back to the sender
///
/// # Arguments
/// * `port` - UDP port to listen on (typically 7001)
///
/// # Returns
/// - Never returns (runs in infinite loop)
/// - `Err` only if socket binding fails
///
/// # Example
/// ```
/// // Daemon usage: runs alongside TCP server
/// tokio::spawn(run_discovery_server(7001));
/// ```
pub async fn run_discovery_server(port: u16) -> Result<()> {
    // Bind to specified port on all network interfaces
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).await?;

    println!("👂 [Daemon] Listening for discovery on UDP {}", port);

    // Prepare buffer for incoming discovery messages
    let mut buf = [0; 1024];

    // Main discovery loop - runs forever
    loop {
        // Wait for incoming UDP packet
        let (len, addr) = socket.recv_from(&mut buf).await?;

        // Convert received bytes to string (lossy = replace invalid UTF-8)
        let msg = String::from_utf8_lossy(&buf[..len]);

        // Check if this is a valid discovery request
        if msg == "SPARK_DISCOVER" {
            println!("👋 [Daemon] Discovery ping from {}", addr);

            // Send acknowledgment back to the client
            socket.send_to(b"SPARK_HERE", addr).await?;
        }
    }
}
