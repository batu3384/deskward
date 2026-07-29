//! Session lifecycle (Faz 0 handshake state machine).

use crate::crypto::{sign_handshake, verify_handshake, Identity};
use crate::protocol::Message;
use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Idle,
    HelloSent,
    Established,
    Closed,
}

pub struct Session {
    pub peer_id: String,
    pub state: SessionState,
    identity: Identity,
    our_nonce: [u8; 32],
    peer_public: Option<[u8; 32]>,
}

impl Session {
    pub fn new(peer_id: impl Into<String>, identity: Identity) -> Self {
        Self {
            peer_id: peer_id.into(),
            state: SessionState::Idle,
            identity,
            our_nonce: rand::random(),
            peer_public: None,
        }
    }

    pub fn begin_handshake(&mut self) -> Message {
        self.state = SessionState::HelloSent;
        Message::HandshakeHello {
            peer_id: self.peer_id.clone(),
            nonce: self.our_nonce,
        }
    }

    pub fn on_hello(&mut self, _peer_id: String, peer_nonce: [u8; 32], peer_public: [u8; 32]) -> Result<Message> {
        self.peer_public = Some(peer_public);
        let sig = sign_handshake(&self.identity, &peer_nonce, &self.our_nonce);
        self.state = SessionState::Established;
        Ok(Message::HandshakeAck {
            peer_id: self.peer_id.clone(),
            nonce: self.our_nonce,
            signature: sig,
        })
    }

    pub fn on_ack(
        &mut self,
        responder_nonce: [u8; 32],
        signature: &[u8],
        peer_public: [u8; 32],
    ) -> Result<()> {
        if self.state != SessionState::HelloSent {
            return Err(Error::HandshakeFailed);
        }
        // Responder signed: initiator_nonce || responder_nonce
        verify_handshake(&peer_public, &self.our_nonce, &responder_nonce, signature)?;
        self.peer_public = Some(peer_public);
        self.state = SessionState::Established;
        Ok(())
    }

    pub fn close(&mut self) -> Message {
        self.state = SessionState::Closed;
        Message::SessionClose {
            session_id: self.peer_id.clone(),
        }
    }

    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.identity.public_key_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_handshake() {
        let id_a = Identity::generate();
        let id_b = Identity::generate();
        let mut sa = Session::new("a", id_a);
        let mut sb = Session::new("b", id_b);

        let hello = sa.begin_handshake();
        let Message::HandshakeHello { peer_id, nonce } = hello else {
            panic!("expected hello");
        };
        let ack = sb.on_hello(peer_id, nonce, sa.public_key_bytes()).unwrap();
        let Message::HandshakeAck {
            nonce: nb,
            signature,
            ..
        } = ack
        else {
            panic!("expected ack");
        };
        sa.on_ack(nb, &signature, sb.public_key_bytes()).unwrap();
        assert_eq!(sa.state, SessionState::Established);
        assert_eq!(sb.state, SessionState::Established);
    }
}
