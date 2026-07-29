//! Noise Protocol XX transport (ChaCha20-Poly1305).

use snow::{Builder, TransportState};

use crate::{Error, Result};

const PATTERN: &str = "Noise_XX_25519_ChaChaPoly_SHA256";
const MAX_MSG: usize = 65535;

/// AEAD transport after Noise XX handshake completes.
pub struct SecureTransport {
    pub(crate) transport: TransportState,
}

impl SecureTransport {
    pub fn encrypt(&mut self, plain: &[u8]) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; plain.len() + 32];
        let len = self
            .transport
            .write_message(plain, &mut buf)
            .map_err(map_snow)?;
        buf.truncate(len);
        Ok(buf)
    }

    pub fn decrypt(&mut self, cipher: &[u8]) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; cipher.len() + 32];
        let len = self
            .transport
            .read_message(cipher, &mut buf)
            .map_err(map_snow)?;
        buf.truncate(len);
        Ok(buf)
    }
}

/// Controller (initiator) — 3-message Noise XX.
pub fn noise_initiator_handshake(
    mut write: impl FnMut(&[u8]) -> Result<()>,
    mut read: impl FnMut() -> Result<Vec<u8>>,
) -> Result<SecureTransport> {
    let mut hs = build_handshake_state(true)?;
    let mut buf = [0u8; MAX_MSG];

    let len = hs.write_message(&[], &mut buf).map_err(map_snow)?;
    write(&buf[..len])?;

    let msg2 = read()?;
    hs.read_message(&msg2, &mut buf).map_err(map_snow)?;

    let len = hs.write_message(&[], &mut buf).map_err(map_snow)?;
    write(&buf[..len])?;

    let transport = hs.into_transport_mode().map_err(map_snow)?;
    Ok(SecureTransport { transport })
}

/// Host (responder).
pub fn noise_responder_handshake(
    mut read: impl FnMut() -> Result<Vec<u8>>,
    mut write: impl FnMut(&[u8]) -> Result<()>,
) -> Result<SecureTransport> {
    let mut hs = build_handshake_state(false)?;
    let mut buf = [0u8; MAX_MSG];

    let msg1 = read()?;
    hs.read_message(&msg1, &mut buf).map_err(map_snow)?;

    let len = hs.write_message(&[], &mut buf).map_err(map_snow)?;
    write(&buf[..len])?;

    let msg3 = read()?;
    hs.read_message(&msg3, &mut buf).map_err(map_snow)?;

    let transport = hs.into_transport_mode().map_err(map_snow)?;
    Ok(SecureTransport { transport })
}

fn map_snow(e: snow::Error) -> Error {
    Error::Crypto(e.to_string())
}

pub(crate) fn build_handshake_state(initiator: bool) -> Result<snow::HandshakeState> {
    let params: snow::params::NoiseParams = PATTERN
        .parse::<snow::params::NoiseParams>()
        .map_err(|e| Error::Crypto(e.to_string()))?;
    let builder = Builder::new(params);
    let kp = builder.generate_keypair().map_err(map_snow)?;
    let builder = builder.local_private_key(&kp.private).map_err(map_snow)?;
    if initiator {
        builder.build_initiator().map_err(map_snow)
    } else {
        builder.build_responder().map_err(map_snow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[test]
    fn noise_xx_encrypt_roundtrip() {
        let mut initiator = build_handshake_state(true).expect("initiator");
        let mut responder = build_handshake_state(false).expect("responder");
        let mut buf = [0u8; MAX_MSG];
        let mut wire: VecDeque<Vec<u8>> = VecDeque::new();

        let len = initiator.write_message(&[], &mut buf).unwrap();
        wire.push_back(buf[..len].to_vec());

        let msg1 = wire.pop_front().unwrap();
        responder.read_message(&msg1, &mut buf).unwrap();

        let len = responder.write_message(&[], &mut buf).unwrap();
        wire.push_back(buf[..len].to_vec());

        let msg2 = wire.pop_front().unwrap();
        initiator.read_message(&msg2, &mut buf).unwrap();

        let len = initiator.write_message(&[], &mut buf).unwrap();
        wire.push_back(buf[..len].to_vec());

        let msg3 = wire.pop_front().unwrap();
        responder.read_message(&msg3, &mut buf).unwrap();

        let mut initiator = SecureTransport {
            transport: initiator.into_transport_mode().unwrap(),
        };
        let mut responder = SecureTransport {
            transport: responder.into_transport_mode().unwrap(),
        };

        let plain = b"deskward payload";
        let cipher = initiator.encrypt(plain).unwrap();
        let back = responder.decrypt(&cipher).unwrap();
        assert_eq!(back, plain);
    }
}
