use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

use crate::{Error, Result};

/// Ed25519 device identity (Faz 0 handshake).
#[derive(Clone)]
pub struct Identity {
    signing: SigningKey,
    verifying: VerifyingKey,
}

impl Identity {
    pub fn generate() -> Self {
        let signing = SigningKey::generate(&mut OsRng);
        let verifying = signing.verifying_key();
        Self { signing, verifying }
    }

    pub fn from_signing_key(signing: SigningKey) -> Self {
        let verifying = signing.verifying_key();
        Self { signing, verifying }
    }

    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.verifying
    }

    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.verifying.to_bytes()
    }

    pub fn sign(&self, msg: &[u8]) -> Vec<u8> {
        self.signing.sign(msg).to_bytes().to_vec()
    }

    pub fn verify(public: &[u8; 32], msg: &[u8], sig: &[u8]) -> Result<()> {
        let vk = VerifyingKey::from_bytes(public)
            .map_err(|e| Error::Crypto(e.to_string()))?;
        let sig = Signature::from_slice(sig).map_err(|e| Error::Crypto(e.to_string()))?;
        vk.verify(msg, &sig)
            .map_err(|e| Error::Crypto(e.to_string()))
    }
}

/// Build HandshakeAck signature over peer nonce || our nonce.
pub fn sign_handshake(id: &Identity, peer_nonce: &[u8; 32], our_nonce: &[u8; 32]) -> Vec<u8> {
    let mut msg = Vec::with_capacity(64);
    msg.extend_from_slice(peer_nonce);
    msg.extend_from_slice(our_nonce);
    id.sign(&msg)
}

pub fn verify_handshake(
    peer_public: &[u8; 32],
    peer_nonce: &[u8; 32],
    our_nonce: &[u8; 32],
    signature: &[u8],
) -> Result<()> {
    let mut msg = Vec::with_capacity(64);
    msg.extend_from_slice(peer_nonce);
    msg.extend_from_slice(our_nonce);
    Identity::verify(peer_public, &msg, signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_verify_roundtrip() {
        let a = Identity::generate();
        let b = Identity::generate();
        let na = [1u8; 32];
        let nb = [2u8; 32];
        let sig = sign_handshake(&a, &nb, &na);
        verify_handshake(&a.public_key_bytes(), &nb, &na, &sig).unwrap();
        assert!(verify_handshake(&b.public_key_bytes(), &nb, &na, &sig).is_err());
    }
}
