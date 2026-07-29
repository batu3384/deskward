//! Property tests — decode must not panic on arbitrary input.

use deskward_core::protocol::decode_frame;
use proptest::prelude::*;

proptest! {
    #[test]
    fn decode_frame_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..8192)) {
        let _ = decode_frame(&bytes);
    }
}
