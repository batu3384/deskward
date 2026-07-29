# Deskward

Self-hosted remote desktop platform. Rust core + Flutter UI.

## Workspace

| Path | Role |
|------|------|
| `deskward-core` | Protocol, crypto, session, media/input traits |
| `deskward-id` | ID / rendezvous server (TCP 29115) |
| `deskward-relay` | Relay fallback (TCP 29117) |
| `deskward-app` | Flutter client (Win / macOS / iOS) |
| `deskward-host-macos` | macOS host agent (Faz 1) |
| `deskward-host-windows` | Windows host agent (Faz 2) |
| `deskward-console` | Admin API stub (Faz 3) |
| `deskward-ffi` | Rust cdylib for Flutter (Tailscale, connect, perms) |
| `deploy/pi` | Opsiyonel self-host iskelet (Pi yok — kullanılmıyor) |

## Quick start

```bash
# Build
cargo test
cargo build --release
cargo build -p deskward-ffi   # Flutter: libdeskward_ffi.dylib

# Run servers (two terminals)
./target/release/deskward-id
./target/release/deskward-relay
```

## Docs

- [Platform design](docs/superpowers/specs/2026-07-29-deskward-platform-design.md)
- [Tailscale onboarding & security](docs/superpowers/specs/2026-07-29-tailscale-onboarding-security-design.md)
- [Kod 3 session plan](docs/superpowers/plans/2026-07-29-kod3-session-tailscale.md)
- [Kod 4 media + E2EE plan](docs/superpowers/plans/2026-07-29-kod4-media-e2ee.md)
- [Faz 2 product plan](docs/superpowers/plans/2026-07-29-faz2-product-impl.md)
- [Faz 3 Pro plan](docs/superpowers/plans/2026-07-29-faz3-pro.md)
- [Faz 4 hardening plan](docs/superpowers/plans/2026-07-29-faz4-hardening.md)
- [Faz 5 media + iOS plan](docs/superpowers/plans/2026-07-29-faz5-media-ios.md)
- [Store release checklist](docs/store-release.md)

## Codec (Faz 5)

| Profil | Değer | Platform |
|--------|-------|----------|
| JPEG | `jpeg` | hepsi |
| H.264 yazılım | `h264` | hepsi (OpenH264) |
| H.264 donanım | `h264-hw` | macOS host (VideoToolbox) |

Ayarlar UI veya `DESKWARD_CODEC` / `setup.json` → `"codec"`.

Client H264 decode: Apple → VideoToolbox (fallback OpenH264); diğer platformlar OpenH264. Metrics overlay `decoder` alanı gösterir.

## iOS

`deskward-app/ios-setup.md` — `flutter create --platforms=ios` + Info.plist.
- [Threat model](docs/security/threat-model.md)

## Tray / arka plan (Faz 2 stub)

- macOS: `LaunchAgent` plist ile `deskward-host-macos` otomatik başlat (yakında)
- Windows: Görev Zamanlayıcı veya servis (yakında)
- Flutter menü çubuğu tray: `deskward-app/lib/services/tray_stub.dart` — platform channel Faz 2.1
- [Faz 0 plan](docs/superpowers/plans/2026-07-29-faz0-foundation.md)
- [Faz 1 plan](docs/superpowers/plans/2026-07-29-faz1-personal.md)
- [Design system](design-system/deskward/MASTER.md)

## License

MIT
