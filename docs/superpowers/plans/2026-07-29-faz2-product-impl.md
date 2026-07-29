# Faz 2 — Ürün iskeleti

**Goal:** Windows host parity; clipboard + file transfer over E2EE session; tray stub.

## Tasks

1. `deskward-host-windows` — gate, session, xcap capture, enigo input (macOS mirror)
2. Protocol: `ClipboardPush`, `FileOfferMsg`, `FileChunkMsg`, `FileComplete`
3. `session_runtime` — clipboard apply + file receive to temp dir
4. Flutter: clipboard toggle in session, tray stub doc

## Env flags

- `DESKWARD_CLIPBOARD=1` — enable clipboard relay on host
- `DESKWARD_FORCE_LISTEN=1` — dev bypass gate
