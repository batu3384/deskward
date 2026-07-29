# Faz 1 — Kişisel döngü Implementation Plan

> **Goal:** macOS host + Windows/iOS controller; telefon veya PC'den ev Mac'e kontrol.

**Architecture:** `deskward-host-macos` ScreenCaptureKit + CGEvent; `deskward-app` FFI to `deskward-core`; Pi `deploy/pi`.

**Tech Stack:** Rust, Flutter, VideoToolbox/H264, Pi Docker.

## Global Constraints

- Ports 29115–29117.
- Protokol v0 + session E2EE hedefi (Noise Faz 1.1).
- iOS controller: touch ≥44px.

---

### Task 1: macOS capture stub

**Files:** `deskward-host-macos/src/capture.rs`, `main.rs`

- [ ] `MacScreenCapture` implements `ScreenCapture` (stub returns None until SCK wired)
- [ ] `cargo build -p deskward-host-macos`

### Task 2: macOS input stub

**Files:** `deskward-host-macos/src/input.rs`

- [ ] `MacInputInjector` implements `InputInjector` (log-only stub)

### Task 3: Host daemon register to id

**Files:** `deskward-host-macos/src/agent.rs`

- [ ] TCP connect deskward-id, `Register` + heartbeat loop

### Task 4: Flutter FFI bridge

**Files:** `deskward-app/rust/` (flutter_rust_bridge or manual cdylib — Faz 1.2)

- [ ] `deskward-core` cdylib export `deskward_connect(peer_id)`

### Task 5: Pi deploy verify

- [ ] `docker compose build` on Pi or local
- [ ] Manual: two `deskward-id` clients punch test

### Success

LAN'da Mac host kayıtlı; Win client punch + handshake PASS; video path stub loglanır.
