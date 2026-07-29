//! Secure session channel over TCP (Noise + framed messages).

use tokio::net::TcpStream;

use crate::io_framed::{read_message, write_message};
use crate::protocol::Message;
use crate::secure::SecureTransport;
use crate::{Error, Result};

async fn write_noise_packet(stream: &mut TcpStream, payload: &[u8]) -> Result<()> {
    write_message(
        stream,
        &Message::NoisePacket {
            payload: payload.to_vec(),
        },
    )
    .await
}

async fn read_noise_packet(stream: &mut TcpStream) -> Result<Vec<u8>> {
    match read_message(stream).await? {
        Message::NoisePacket { payload } => Ok(payload),
        other => Err(Error::Protocol(format!("expected NoisePacket, got {other:?}"))),
    }
}

pub async fn secure_handshake_initiator(stream: &mut TcpStream) -> Result<SecureTransport> {
    let mut hs = crate::secure::build_handshake_state(true)?;
    let mut buf = [0u8; 65535];

    let len = hs.write_message(&[], &mut buf).map_err(map_snow)?;
    write_noise_packet(stream, &buf[..len]).await?;

    let msg2 = read_noise_packet(stream).await?;
    hs.read_message(&msg2, &mut buf).map_err(map_snow)?;

    let len = hs.write_message(&[], &mut buf).map_err(map_snow)?;
    write_noise_packet(stream, &buf[..len]).await?;

    Ok(SecureTransport {
        transport: hs.into_transport_mode().map_err(map_snow)?,
    })
}

pub async fn secure_handshake_responder(stream: &mut TcpStream) -> Result<SecureTransport> {
    let mut hs = crate::secure::build_handshake_state(false)?;
    let mut buf = [0u8; 65535];

    let msg1 = read_noise_packet(stream).await?;
    hs.read_message(&msg1, &mut buf).map_err(map_snow)?;

    let len = hs.write_message(&[], &mut buf).map_err(map_snow)?;
    write_noise_packet(stream, &buf[..len]).await?;

    let msg3 = read_noise_packet(stream).await?;
    hs.read_message(&msg3, &mut buf).map_err(map_snow)?;

    Ok(SecureTransport {
        transport: hs.into_transport_mode().map_err(map_snow)?,
    })
}

pub async fn write_secure(
    stream: &mut TcpStream,
    secure: &mut SecureTransport,
    msg: &Message,
) -> Result<()> {
    let plain = serde_json::to_vec(msg)?;
    let cipher = secure.encrypt(&plain)?;
    write_message(
        stream,
        &Message::EncryptedFrame {
            payload: cipher,
        },
    )
    .await
}

pub async fn read_secure(stream: &mut TcpStream, secure: &mut SecureTransport) -> Result<Message> {
    let frame = read_message(stream).await?;
    let Message::EncryptedFrame { payload } = frame else {
        return Err(Error::Protocol("expected EncryptedFrame".into()));
    };
    let plain = secure.decrypt(&payload)?;
    let msg: Message = serde_json::from_slice(&plain)?;
    Ok(msg)
}

fn map_snow(e: snow::Error) -> Error {
    Error::Crypto(e.to_string())
}
