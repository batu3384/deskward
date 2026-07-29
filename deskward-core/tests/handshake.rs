//! Integration: end-to-end session handshake between two identities.

use deskward_core::crypto::Identity;
use deskward_core::session::{Session, SessionState};

#[test]
fn two_peer_handshake_establishes_session() {
    let id_host = Identity::generate();
    let id_ctrl = Identity::generate();

    let mut host = Session::new("mac-host", id_host);
    let mut ctrl = Session::new("win-ctrl", id_ctrl);

    let hello = ctrl.begin_handshake();
    let ack = host.on_hello(&hello).unwrap();
    ctrl.on_ack(&ack).unwrap();

    assert_eq!(host.state, SessionState::Established);
    assert_eq!(ctrl.state, SessionState::Established);
}
