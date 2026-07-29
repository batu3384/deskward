# Kod 4 — Video + Noise E2EE + Session UI

**Goal:** Encrypted media/input over session; macOS capture + input; Flutter session view.

## Tasks

1. `deskward-core::secure` — Noise XX + AEAD transport
2. Protocol: `NoiseStep*`, `MediaFrame`, `InputPointer`, `InputKey`, `EncryptedFrame`
3. `deskward-core::session_channel` — secure read/write helpers
4. `deskward-host-macos` — xcap capture (JPEG), enigo input, host session loop
5. `deskward-ffi` — session handle, poll frame, send pointer
6. Flutter `SessionScreen` — frame poll + touch input

## Deferred

- VideoToolbox H264 hardware encode
- Full ScreenCaptureKit streaming API
- Windows controller decode
