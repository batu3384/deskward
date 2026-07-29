# Kod 3 — Session over Tailscale + FFI

**Goal:** Gerçek Tailscale LocalAPI, Tailscale IP üzerinden oturum, şifre doğrulama, Flutter FFI.

**Spec:** `2026-07-29-tailscale-onboarding-security-design.md` §9 Kod 3

## Tasks

1. Protocol: `public_key` handshake wire + `SessionAuth` / `SessionAuthResult`
2. `deskward-core::connect` — client TCP `100.x:29118`, handshake, auth
3. `deskward-core::tailscale::system` — `tailscale-localapi` wrapper
4. `deskward-host-macos::session` — tailnet-only listener
5. `deskward-ffi` — cdylib: status, permissions, connect, hash_password
6. Flutter — `dart:ffi` bindings, SetupController real poll, connect + password dialog

## Port

| Servis | TCP |
|--------|-----|
| deskward-session | 29118 |

## Deferred

- Video stream, ScreenCaptureKit, Noise E2EE payload
