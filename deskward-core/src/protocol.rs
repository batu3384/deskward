use serde::{Deserialize, Serialize};

/// Deskward wire protocol v0 message types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    Register {
        peer_id: String,
        endpoint: String,
    },
    Heartbeat {
        peer_id: String,
    },
    PunchRequest {
        from: String,
        to: String,
    },
    PunchResponse {
        from: String,
        to: String,
        endpoint: String,
    },
    RelayAllocate {
        session_id: String,
    },
    RelayReady {
        session_id: String,
        relay_port: u16,
    },
    SessionOpen {
        session_id: String,
        initiator: String,
    },
    SessionClose {
        session_id: String,
    },
    HandshakeHello {
        peer_id: String,
        nonce: [u8; 32],
    },
    HandshakeAck {
        peer_id: String,
        nonce: [u8; 32],
        signature: Vec<u8>,
    },
}

/// Encode as 4-byte big-endian length + JSON payload.
pub fn encode_frame(msg: &Message) -> crate::Result<Vec<u8>> {
    let body = serde_json::to_vec(msg)?;
    if body.len() > u32::MAX as usize {
        return Err(crate::Error::Protocol("frame too large".into()));
    }
    let len = body.len() as u32;
    let mut out = Vec::with_capacity(4 + body.len());
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(&body);
    Ok(out)
}

/// Decode one frame; returns (message, bytes consumed).
pub fn decode_frame(buf: &[u8]) -> crate::Result<(Message, usize)> {
    if buf.len() < 4 {
        return Err(crate::Error::Protocol("buffer too short".into()));
    }
    let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    if buf.len() < 4 + len {
        return Err(crate::Error::Protocol("incomplete frame".into()));
    }
    let msg: Message = serde_json::from_slice(&buf[4..4 + len])?;
    Ok((msg, 4 + len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_register() {
        let msg = Message::Register {
            peer_id: "desk-001".into(),
            endpoint: "192.168.1.10:29115".into(),
        };
        let frame = encode_frame(&msg).unwrap();
        let (decoded, n) = decode_frame(&frame).unwrap();
        assert_eq!(n, frame.len());
        assert_eq!(decoded, msg);
    }

    #[test]
    fn incomplete_frame_errors() {
        let err = decode_frame(&[0, 0, 0, 10, 1, 2]);
        assert!(err.is_err());
    }
}
