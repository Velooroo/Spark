use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// ============================================================================
// NETWORK PROTOCOL - LENGTH-PREFIXED MESSAGE FORMAT
// ============================================================================
//
// All TCP messages use the following format:
// [4 bytes: length (u32, big-endian)][N bytes: data]
//
// This allows the receiver to know exactly how many bytes to read,
// preventing message boundary issues in TCP streams.
//
// Example:
//   Message: "hello"
//   Encoded: [0x00, 0x00, 0x00, 0x05, 'h', 'e', 'l', 'l', 'o']
//            └────────┬────────┘  └────────┬─────────────┘
//                  length=5              payload
// ============================================================================

/// Sends a length-prefixed message over TCP stream
///
/// The message is sent in two parts:
/// 1. A 4-byte header containing the message length as u32 (big-endian)
/// 2. The actual message data
///
/// # Arguments
/// * `stream` - Mutable reference to an open TCP stream
/// * `data`   - Message payload as byte slice
///
/// # Returns
/// - `Ok(())` if the entire message was sent successfully
/// - `Err` if network I/O fails
///
/// # Example
/// ```
/// let mut stream = TcpStream::connect("127.0.0.1:7530").await?;
/// send_message(&mut stream, b"Hello, daemon!").await?;
/// ```
pub async fn send_message<S>(stream: &mut S, data: &[u8]) -> Result<()>
where
    S: AsyncWriteExt + Unpin,
{
    let len = (data.len() as u32).to_be_bytes();
    stream.write_all(&len).await?;
    stream.write_all(data).await?;
    Ok(())
}

/// Receives a length-prefixed message from TCP stream
///
/// This function reads a message in two steps:
/// 1. Read 4-byte header to determine message length
/// 2. Read exactly that many bytes as the payload
///
/// # Arguments
/// * `stream` - Mutable reference to an open TCP stream
///
/// # Returns
/// - `Ok(Vec<u8>)` containing the received message
/// - `Err` if network I/O fails or connection closes unexpectedly
///
/// # Errors
/// - Returns error if connection is closed before full message is received
/// - Returns error if received length is unreasonably large (potential attack)
///
/// # Example
/// ```
/// let mut stream = TcpStream::connect("127.0.0.1:7530").await?;
/// let message = recv_message(&mut stream).await?;
/// let text = String::from_utf8_lossy(&message);
/// println!("Received: {}", text);
/// ```
pub async fn recv_message<S>(stream: &mut S) -> Result<Vec<u8>>
where
    S: AsyncReadExt + Unpin,
{
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    Ok(buf)
}
