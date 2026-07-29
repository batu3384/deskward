# Faz 2 — Ürün iskeleti Implementation Plan

> **Goal:** Windows host; dosya/clipboard; tray servis; Win↔Mac çift yön.

**Architecture:** `deskward-host-windows` DXGI capture; `deskward-core::features` clipboard + file transfer frames.

## Tasks

### Task 1: Windows host crate

**Files:** `deskward-host-windows/`

- [ ] `WinScreenCapture` stub (DXGI/WGC trait impl)
- [ ] `WinInputInjector` stub (SendInput)

### Task 2: Clipboard sync

**Files:** `deskward-core/src/features/clipboard.rs`

- [ ] `ClipboardMessage` protocol variant
- [ ] `ClipboardSync` trait

### Task 3: File transfer

**Files:** `deskward-core/src/features/file_transfer.rs`

- [ ] Chunked `FileOffer` / `FileChunk` / `FileComplete` messages

### Task 4: Tray + service

**Files:** `deskward-app` platform channels

- [ ] macOS/Win menu bar tray stub
- [ ] Launch agent at login (document in README)

### Success

Win host registers; Mac controller connects; clipboard text roundtrip test PASS.
