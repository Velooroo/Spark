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

/// Send message with length prefix over TCP
pub async fn send_message<S>(stream: &mut S, data: &[u8]) -> Result<()>
where
    S: AsyncWriteExt + Unpin,
{
    let len = (data.len() as u32).to_be_bytes();
    stream.write_all(&len).await?;
    stream.write_all(data).await?;
    Ok(())
}

/// Receive message with length prefix from TCP
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
