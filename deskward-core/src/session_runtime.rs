//! Host/client session loops over secure channel.

use std::sync::Arc;
use std::time::Duration;

use crate::perf::SessionMetricsTracker;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::features::file_transfer::FileChunk;
use crate::features::h264_decode::H264DecoderState;
use crate::input::{InputInjector, KeyEvent, PointerEvent};
use crate::media::ScreenCapture;
use crate::protocol::Message;
use crate::secure::SecureTransport;
use crate::session_channel::{read_secure, write_secure};
use crate::session_handlers::HostHandlers;
use crate::{Error, Result};

const FRAME_INTERVAL: Duration = Duration::from_millis(100);

pub struct ClientSession {
    stream: Arc<Mutex<TcpStream>>,
    secure: Arc<Mutex<SecureTransport>>,
    pub latest_frame: Arc<Mutex<Option<FramePayload>>>,
    metrics: Arc<Mutex<SessionMetricsTracker>>,
    h264_decoder: Arc<Mutex<H264DecoderState>>,
}

impl Clone for ClientSession {
    fn clone(&self) -> Self {
        Self {
            stream: self.stream.clone(),
            secure: self.secure.clone(),
            latest_frame: self.latest_frame.clone(),
            metrics: self.metrics.clone(),
            h264_decoder: self.h264_decoder.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FramePayload {
    pub width: u32,
    pub height: u32,
    pub codec: String,
    pub data: Vec<u8>,
}

pub async fn run_host_session<C, I>(
    stream: TcpStream,
    secure: SecureTransport,
    mut capture: C,
    mut input: I,
    mut handlers: HostHandlers<'_>,
) -> Result<()>
where
    C: ScreenCapture + 'static,
    I: InputInjector + 'static,
{
    let stream = Arc::new(Mutex::new(stream));
    let secure = Arc::new(Mutex::new(secure));

    let mut interval = tokio::time::interval(FRAME_INTERVAL);
    loop {
        tokio::select! {
            read_result = async {
                let mut s = stream.lock().await;
                let mut sec = secure.lock().await;
                read_secure(&mut *s, &mut *sec).await
            } => {
                let msg = read_result?;
                match msg {
                    Message::InputPointer { x, y, button, pressed } => {
                        if pressed {
                            let _ = input.pointer_button(PointerEvent { x, y, button, pressed });
                        } else {
                            let _ = input.move_pointer(x, y);
                        }
                    }
                    Message::InputKey { keycode, pressed } => {
                        let ev = if pressed {
                            KeyEvent::Down { keycode }
                        } else {
                            KeyEvent::Up { keycode }
                        };
                        let _ = input.key(ev);
                    }
                    Message::ClipboardPush { mime, data } => {
                        if let Some(cb) = handlers.clipboard.as_deref_mut() {
                            let _ = cb.apply_remote(&crate::features::clipboard::ClipboardPayload {
                                mime,
                                data,
                            });
                        }
                    }
                    Message::FileOfferMsg {
                        path,
                        size,
                        session_id,
                    } => {
                        if let Some(files) = handlers.files.as_deref_mut() {
                            let _ = files.on_offer(&path, size, &session_id);
                        }
                    }
                    Message::FileChunkMsg {
                        session_id,
                        offset,
                        data,
                        final_chunk,
                    } => {
                        if let Some(files) = handlers.files.as_deref_mut() {
                            let chunk = FileChunk {
                                session_id,
                                offset,
                                data,
                                final_chunk,
                            };
                            let _ = files.on_chunk(&chunk);
                        }
                    }
                    Message::FileComplete { session_id } => {
                        if let Some(files) = handlers.files.as_deref_mut() {
                            let _ = files.on_complete(&session_id);
                        }
                    }
                    Message::SessionClose { .. } => break,
                    _ => {}
                }
            }
            _ = interval.tick() => {
                if let Ok(Some(frame)) = capture.capture_frame() {
                    if let Some(rec) = handlers.recording.as_deref_mut() {
                        let _ = rec.on_frame(&frame);
                    }
                    let msg = Message::MediaFrame {
                        width: frame.width,
                        height: frame.height,
                        codec: frame.codec.wire_name().into(),
                        keyframe: frame.keyframe,
                        data: frame.data,
                    };
                    let mut s = stream.lock().await;
                    let mut sec = secure.lock().await;
                    if write_secure(&mut *s, &mut *sec, &msg).await.is_err() {
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn start_client_session(
    stream: TcpStream,
    secure: SecureTransport,
) -> Result<ClientSession> {
    let stream = Arc::new(Mutex::new(stream));
    let secure = Arc::new(Mutex::new(secure));
    let latest_frame = Arc::new(Mutex::new(None::<FramePayload>));
    let metrics = Arc::new(Mutex::new(SessionMetricsTracker::default()));
    let h264_decoder = Arc::new(Mutex::new(H264DecoderState::default()));

    let stream_r = stream.clone();
    let secure_r = secure.clone();
    let frames = latest_frame.clone();
    let metrics_r = metrics.clone();
    let decoder_r = h264_decoder.clone();
    tokio::spawn(async move {
        loop {
            let mut s = stream_r.lock().await;
            let mut sec = secure_r.lock().await;
            match read_secure(&mut *s, &mut *sec).await {
                Ok(Message::MediaFrame {
                    width,
                    height,
                    codec,
                    data,
                    ..
                }) => {
                    metrics_r.lock().await.on_frame(data.len());
                    let video = crate::media::VideoFrame {
                        width,
                        height,
                        data: data.clone(),
                        codec: crate::media::Codec::from_wire(&codec),
                        keyframe: true,
                    };
                    let display = {
                        let mut dec = decoder_r.lock().await;
                        dec.decode_for_display(&video)
                    };
                    if let Ok(jpeg) = display {
                        let backend = {
                            let dec = decoder_r.lock().await;
                            dec.backend_name().to_string()
                        };
                        metrics_r.lock().await.set_decoder(&backend);
                        *frames.lock().await = Some(FramePayload {
                            width,
                            height,
                            codec: "jpeg".into(),
                            data: jpeg,
                        });
                    }
                }
                Ok(Message::SessionClose { .. }) | Err(_) => break,
                Ok(_) => {}
            }
        }
    });

    Ok(ClientSession {
        stream,
        secure,
        latest_frame,
        metrics,
        h264_decoder,
    })
}

impl ClientSession {
    pub async fn send_pointer(&self, x: f64, y: f64, pressed: bool) -> Result<()> {
        let msg = Message::InputPointer {
            x,
            y,
            button: 0,
            pressed,
        };
        let mut s = self.stream.lock().await;
        let mut sec = self.secure.lock().await;
        write_secure(&mut *s, &mut *sec, &msg).await
    }

    pub async fn send_clipboard_text(&self, text: &str) -> Result<()> {
        let msg = Message::ClipboardPush {
            mime: "text/plain".into(),
            data: text.as_bytes().to_vec(),
        };
        let mut s = self.stream.lock().await;
        let mut sec = self.secure.lock().await;
        write_secure(&mut *s, &mut *sec, &msg).await
    }

    pub async fn send_file_offer(&self, path: &str, size: u64, session_id: &str) -> Result<()> {
        let msg = Message::FileOfferMsg {
            path: path.into(),
            size,
            session_id: session_id.into(),
        };
        let mut s = self.stream.lock().await;
        let mut sec = self.secure.lock().await;
        write_secure(&mut *s, &mut *sec, &msg).await
    }

    pub async fn send_file_chunk(&self, chunk: FileChunk) -> Result<()> {
        let msg = Message::FileChunkMsg {
            session_id: chunk.session_id,
            offset: chunk.offset,
            data: chunk.data,
            final_chunk: chunk.final_chunk,
        };
        let mut s = self.stream.lock().await;
        let mut sec = self.secure.lock().await;
        write_secure(&mut *s, &mut *sec, &msg).await
    }

    pub async fn send_file_complete(&self, session_id: &str) -> Result<()> {
        let msg = Message::FileComplete {
            session_id: session_id.into(),
        };
        let mut s = self.stream.lock().await;
        let mut sec = self.secure.lock().await;
        write_secure(&mut *s, &mut *sec, &msg).await
    }

    pub async fn poll_frame(&self) -> Option<FramePayload> {
        self.latest_frame.lock().await.clone()
    }

    pub async fn metrics(&self) -> crate::perf::SessionMetrics {
        self.metrics.lock().await.snapshot()
    }

    pub async fn close(&self) -> Result<()> {
        let msg = Message::SessionClose {
            session_id: "deskward".into(),
        };
        let mut s = self.stream.lock().await;
        let mut sec = self.secure.lock().await;
        write_secure(&mut *s, &mut *sec, &msg).await?;
        s.shutdown().await.map_err(Error::Io)?;
        Ok(())
    }
}
