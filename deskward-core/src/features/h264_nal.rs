//! H.264 Annex-B NAL helpers (no platform deps).

/// Split Annex-B bitstream into NAL payloads (without start codes).
pub fn split_annex_b(data: &[u8]) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < data.len() {
        let start = if data[i..].starts_with(&[0, 0, 0, 1]) {
            i += 4;
            i
        } else if data[i..].starts_with(&[0, 0, 1]) {
            i += 3;
            i
        } else {
            i += 1;
            continue;
        };
        let mut end = start;
        while end < data.len() {
            if data[end..].starts_with(&[0, 0, 0, 1])
                || (end + 2 < data.len() && data[end..].starts_with(&[0, 0, 1]))
            {
                break;
            }
            end += 1;
        }
        if end > start {
            out.push(data[start..end].to_vec());
        }
        i = end;
    }
    out
}

pub fn nal_type(nal: &[u8]) -> Option<u8> {
    nal.first().map(|b| b & 0x1f)
}

/// Extract first SPS (7) and PPS (8) from Annex-B stream.
pub fn extract_parameter_sets(data: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    let mut sps = None;
    let mut pps = None;
    for nal in split_annex_b(data) {
        let Some(t) = nal_type(&nal) else {
            continue;
        };
        match t {
            7 => {
                sps.get_or_insert(nal);
            }
            8 => {
                pps.get_or_insert(nal);
            }
            _ => {}
        }
    }
    Some((sps?, pps?))
}

/// Length-prefixed NALs (AVCC) for VideoToolbox sample buffers.
pub fn annex_b_to_avcc(nals: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    for nal in nals {
        let t = nal_type(nal).unwrap_or(0);
        if t == 7 || t == 8 {
            continue;
        }
        let len = nal.len() as u32;
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(nal);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_start_codes() {
        let data = [0, 0, 0, 1, 0x67, 0x42, 0, 0, 1, 0x68, 0xce];
        let nals = split_annex_b(&data);
        assert_eq!(nals.len(), 2);
        assert_eq!(nal_type(&nals[0]), Some(7));
        assert_eq!(nal_type(&nals[1]), Some(8));
    }
}
