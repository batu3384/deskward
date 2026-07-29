# Faz 5 — Media codec + iOS controller

> **Goal:** H264 encode path, client decode, codec settings, iOS platform scaffold.

## Tasks

1. `deskward-core/features/frame_codec` — JPEG + OpenH264 encode/decode
2. Host capture → `encode_rgba`; session wire uses real codec tag
3. Client decodes H264 → JPEG for Flutter `Image.memory`
4. `setup.json` + Settings UI — `codec: jpeg|h264`
5. `flutter create --platforms=ios` + Info.plist local network

## Env

- `DESKWARD_CODEC=jpeg|h264` (host fallback)
