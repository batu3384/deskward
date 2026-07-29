//! Framed read/write helpers for session TCP.

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::protocol::{decode_frame, encode_frame, Message};
use crate::Result;

pub async fn write_message(stream: &mut tokio::net::TcpStream, msg: &Message) -> Result<()> {
    let frame = encode_frame(msg)?;
    stream.write_all(&frame).await?;
    Ok(())
}

pub async fn read_message(stream: &mut tokio::net::TcpStream) -> Result<Message> {
    let mut header = [0u8; 4];
    stream.read_exact(&mut header).await?;
    let len = u32::from_be_bytes(header) as usize;
    if len > crate::admin_auth::MAX_FRAME_BYTES {
        return Err(crate::Error::Protocol("frame too large".into()));
    }
    let mut body = vec![0u8; len];
    stream.read_exact(&mut body).await?;
    let mut frame = Vec::with_capacity(4 + len);
    frame.extend_from_slice(&header);
    frame.extend_from_slice(&body);
    let (msg, _) = decode_frame(&frame)?;
    Ok(msg)
}
