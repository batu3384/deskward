//! H.264 decode for session display path.

use openh264::decoder::DecodedYUV;
use openh264::decoder::Decoder;
use openh264::formats::YUVSource;

use crate::features::frame_codec::encode_jpeg;
use crate::media::{Codec, VideoFrame};
use crate::Result;

#[cfg(any(target_os = "macos", target_os = "ios"))]
use crate::features::vt_h264_decode::VtH264Decoder;

pub struct H264DecoderState {
    backend: DecoderBackend,
}

enum DecoderBackend {
    OpenH264(Decoder),
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    VideoToolbox(VtH264Decoder),
}

impl Default for H264DecoderState {
    fn default() -> Self {
        Self {
            backend: DecoderBackend::new(),
        }
    }
}

impl DecoderBackend {
    fn new() -> Self {
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        if vt_decode_enabled() {
            return Self::VideoToolbox(VtH264Decoder::new());
        }
        Self::OpenH264(Decoder::new().expect("openh264 decoder"))
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn vt_decode_enabled() -> bool {
    !matches!(
        std::env::var("DESKWARD_VT_DECODE").as_deref(),
        Ok("0") | Ok("false")
    )
}

impl H264DecoderState {
    pub fn backend_name(&self) -> &'static str {
        match self.backend {
            DecoderBackend::OpenH264(_) => "openh264",
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            DecoderBackend::VideoToolbox(_) => "videotoolbox",
        }
    }

    pub fn decode_for_display(&mut self, frame: &VideoFrame) -> Result<Vec<u8>> {
        match frame.codec {
            Codec::Jpeg | Codec::Raw => Ok(frame.data.clone()),
            Codec::H264 => self.decode_h264(&frame.data),
            _ => Err(crate::Error::Protocol(format!(
                "unsupported codec {:?}",
                frame.codec
            ))),
        }
    }

    fn decode_h264(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        match &mut self.backend {
            DecoderBackend::OpenH264(dec) => decode_openh264(dec, data),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            DecoderBackend::VideoToolbox(vt) => match vt.decode(data) {
                Ok(jpeg) => Ok(jpeg),
                Err(_) => {
                    self.backend = DecoderBackend::OpenH264(Decoder::new().map_err(|e| {
                        crate::Error::Protocol(format!("openh264: {e:?}"))
                    })?);
                    self.decode_h264(data)
                }
            },
        }
    }
}

fn decode_openh264(dec: &mut Decoder, data: &[u8]) -> Result<Vec<u8>> {
    let yuv = dec
        .decode(data)
        .map_err(|e| crate::Error::Protocol(format!("h264: {e:?}")))?;
    let Some(yuv) = yuv else {
        return Err(crate::Error::Protocol("h264 decode produced no frame".into()));
    };
    yuv_to_jpeg(&yuv)
}

fn yuv_to_jpeg(yuv: &DecodedYUV<'_>) -> Result<Vec<u8>> {
    let (w, h) = yuv.dimensions();
    let mut rgba = vec![0u8; w * h * 4];
    yuv.write_rgba8(&mut rgba);
    encode_jpeg(&rgba, w as u32, h as u32).map(|f| f.data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::frame_codec::{encode_rgba, CodecProfile};

    #[test]
    fn h264_display_roundtrip() {
        let mut rgba = vec![0u8; 64 * 64 * 4];
        for px in rgba.chunks_exact_mut(4) {
            px[0] = 10;
            px[1] = 20;
            px[2] = 30;
            px[3] = 255;
        }
        let frame = encode_rgba(&rgba, 64, 64, CodecProfile::H264).unwrap();
        let mut dec = H264DecoderState::default();
        let jpeg = dec.decode_for_display(&frame).unwrap();
        assert!(jpeg.starts_with(&[0xFF, 0xD8]));
    }
}
