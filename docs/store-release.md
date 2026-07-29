# Deskward — Store release checklist

## macOS (notarization)

1. `cargo build --release -p deskward-host-macos -p deskward-ffi`
2. Flutter `flutter build macos --release`
3. Codesign host + app with Developer ID
4. `notarytool submit` + staple
5. Hardened runtime + entitlements: screen capture, accessibility, network client

## iOS (TestFlight)

1. Apple Developer account + App ID (`com.deskward.app`)
2. `flutter build ios --release`
3. Xcode Archive → Distribute → TestFlight
4. Privacy manifest: local network (Tailscale), no tracking

## Windows (optional Store)

1. `cargo build --release -p deskward-host-windows`
2. `flutter build windows --release`
3. MSIX packaging or classic installer + Authenticode sign

## Env / secrets

- Never ship `DESKWARD_FORCE_LISTEN=1` in production builds
- Pin `DESKWARD_SERVER_PUBKEY` for self-host ID
- Audit log path writable only by service account
