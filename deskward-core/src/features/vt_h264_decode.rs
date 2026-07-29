//! VideoToolbox hardware H.264 decode (macOS / iOS).

use std::sync::{Arc, Mutex};

use apple_cf::cm::{CMBlockBuffer, CMFormatDescription, CMSampleBuffer, CMSampleTimingInfo, CMTime};
use apple_cf::cv::CVPixelBufferLockFlags;
use apple_cf::raw::{
    kCFAllocatorDefault, CMBlockBufferRef, CMFormatDescriptionRef,
    CMSampleBufferCreateReady, CMSampleBufferRef,
    CMVideoFormatDescriptionCreateFromH264ParameterSets,
};
use videotoolbox::decompression::{DecompressionSession, DecodedFrame};
use videotoolbox::error::VTError;

use crate::features::frame_codec::encode_jpeg;
use crate::features::h264_nal::{annex_b_to_avcc, extract_parameter_sets, split_annex_b};
use crate::Result;

pub struct VtH264Decoder {
    session: Option<DecompressionSession>,
    format: Option<CMFormatDescription>,
    pts: i64,
    slot: Arc<Mutex<Option<Vec<u8>>>>,
}

impl VtH264Decoder {
    pub fn new() -> Self {
        Self {
            session: None,
            format: None,
            pts: 0,
            slot: Arc::new(Mutex::new(None)),
        }
    }

    pub fn decode(&mut self, annex_b: &[u8]) -> Result<Vec<u8>> {
        if self.session.is_none() {
            let (sps, pps) = extract_parameter_sets(annex_b)
                .ok_or_else(|| crate::Error::Protocol("h264 missing sps/pps".into()))?;
            self.init_session(&sps, &pps)?;
        }
        let nals = split_annex_b(annex_b);
        let avcc = annex_b_to_avcc(&nals);
        if avcc.is_empty() {
            return Err(crate::Error::Protocol("h264 empty avcc frame".into()));
        }
        let block = CMBlockBuffer::create(&avcc)
            .ok_or_else(|| crate::Error::Protocol("CMBlockBuffer create failed".into()))?;
        let format = self
            .format
            .as_ref()
            .ok_or_else(|| crate::Error::Protocol("vt format missing".into()))?;
        let sample = create_sample_buffer(&block, format, self.pts)?;
        self.pts += 1;
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| crate::Error::Protocol("vt session missing".into()))?;
        {
            let mut guard = self.slot.lock().unwrap();
            *guard = None;
        }
        session
            .decode(&sample)
            .map_err(map_vt)?;
        session.wait_for_async_frames().map_err(map_vt)?;
        let jpeg = self
            .slot
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| crate::Error::Protocol("vt decode produced no frame".into()))?;
        Ok(jpeg)
    }

    fn init_session(&mut self, sps: &[u8], pps: &[u8]) -> Result<()> {
        let format = create_h264_format_description(sps, pps)?;
        let slot = self.slot.clone();
        let session = DecompressionSession::new(&format, move |frame: DecodedFrame| {
            if let Ok(jpeg) = pixel_buffer_to_jpeg(frame) {
                if let Ok(mut g) = slot.lock() {
                    *g = Some(jpeg);
                }
            }
        })
        .map_err(map_vt)?;
        self.format = Some(format);
        self.session = Some(session);
        Ok(())
    }
}

fn create_h264_format_description(sps: &[u8], pps: &[u8]) -> Result<CMFormatDescription> {
    let pointers = [sps.as_ptr(), pps.as_ptr()];
    let sizes = [sps.len(), pps.len()];
    let mut out: CMFormatDescriptionRef = std::ptr::null_mut();
    let status = unsafe {
        CMVideoFormatDescriptionCreateFromH264ParameterSets(
            kCFAllocatorDefault,
            2,
            pointers.as_ptr(),
            sizes.as_ptr(),
            4,
            &mut out,
        )
    };
    if status != 0 || out.is_null() {
        return Err(crate::Error::Protocol(format!(
            "CMVideoFormatDescriptionCreateFromH264ParameterSets: {status}"
        )));
    }
    unsafe { CMFormatDescription::from_raw(out as *mut std::ffi::c_void) }
        .ok_or_else(|| crate::Error::Protocol("format description null".into()))
}

fn cm_timing_raw(timing: CMSampleTimingInfo) -> apple_cf::raw::CMSampleTimingInfo {
    // ponytail: cm/raw CMSampleTimingInfo share repr(C) field layout
    unsafe { std::mem::transmute(timing) }
}

fn create_sample_buffer(
    block: &CMBlockBuffer,
    format: &CMFormatDescription,
    pts: i64,
) -> Result<CMSampleBuffer> {
    let timing = cm_timing_raw(CMSampleTimingInfo::with_times(
        CMTime::new(1, 600),
        CMTime::new(pts, 600),
        CMTime::INVALID,
    ));
    let size = block.data_length();
    let mut out: CMSampleBufferRef = std::ptr::null_mut();
    let status = unsafe {
        CMSampleBufferCreateReady(
            kCFAllocatorDefault,
            block.as_ptr() as CMBlockBufferRef,
            format.as_ptr() as CMFormatDescriptionRef,
            1,
            1,
            &timing,
            1,
            &size,
            &mut out,
        )
    };
    if status != 0 || out.is_null() {
        return Err(crate::Error::Protocol(format!(
            "CMSampleBufferCreateReady: {status}"
        )));
    }
    unsafe { CMSampleBuffer::from_raw_retained(out.cast::<std::ffi::c_void>()) }
        .ok_or_else(|| crate::Error::Protocol("sample buffer null".into()))
}

fn pixel_buffer_to_jpeg(frame: DecodedFrame) -> Result<Vec<u8>> {
    let Some(pb) = frame.image_buffer else {
        return Err(crate::Error::Protocol("vt dropped frame".into()));
    };
    let guard = pb
        .lock(CVPixelBufferLockFlags::READ_ONLY)
        .map_err(|e| crate::Error::Protocol(format!("cv lock: {e}")))?;
    let w = guard.width() as u32;
    let h = guard.height() as u32;
    let stride = guard.bytes_per_row();
    let base = guard.base_address();
    if base.is_null() {
        return Err(crate::Error::Protocol("cv base null".into()));
    }
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    unsafe {
        for row in 0..h as usize {
            let src = base.add(row * stride);
            let dst = (row * w as usize) * 4;
            for col in 0..w as usize {
                let si = col * 4;
                let di = dst + col * 4;
                // BGRA → RGBA
                rgba[di] = *src.add(si + 2);
                rgba[di + 1] = *src.add(si + 1);
                rgba[di + 2] = *src.add(si);
                rgba[di + 3] = *src.add(si + 3);
            }
        }
    }
    encode_jpeg(&rgba, w, h).map(|f| f.data)
}

fn map_vt(e: VTError) -> crate::Error {
    crate::Error::Protocol(format!("vt: {e:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::frame_codec::{encode_rgba, CodecProfile};

    fn checker_rgba(w: u32, h: u32) -> Vec<u8> {
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                let v = ((x + y) % 255) as u8;
                rgba[i] = v;
                rgba[i + 1] = v / 2;
                rgba[i + 2] = 255 - v;
                rgba[i + 3] = 255;
            }
        }
        rgba
    }

    #[test]
    fn vt_decode_openh264_bitstream() {
        let rgba = checker_rgba(128, 128);
        let frame = encode_rgba(&rgba, 128, 128, CodecProfile::H264).unwrap();
        let mut dec = VtH264Decoder::new();
        let jpeg = dec.decode(&frame.data).expect("vt decode");
        assert!(jpeg.starts_with(&[0xFF, 0xD8]));
    }
}
