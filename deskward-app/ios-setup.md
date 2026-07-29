# Deskward iOS controller bootstrap

Flutter SDK gerekli. Proje kökünde:

```bash
cd deskward-app
flutter create . --platforms=ios
```

## Info.plist ekleri

`ios/Runner/Info.plist` içine Tailscale / yerel ağ için:

```xml
<key>NSLocalNetworkUsageDescription</key>
<string>Deskward uzak masaüstü oturumu için yerel ağ erişimi gerekir.</string>
<key>NSBonjourServices</key>
<array>
  <string>_deskward._tcp</string>
</array>
```

## Build

```bash
cargo build -p deskward-ffi --release
flutter build ios --release
```

Touch hedefleri session ekranında ≥44px (platform design).
