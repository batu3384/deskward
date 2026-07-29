import 'device_role.dart';

enum AppPlatform { macOs, windows, ios }

enum CheckId {
  tailscaleInstalled,
  tailscaleRunning,
  tailscaleSelfVisible,
  screenRecording,
  accessibility,
  launchAtLogin,
  unattendedPassword,
  hostListeningArmed,
  peerVisible,
}

enum CheckStatus { pending, actionNeeded, done }

class CheckItem {
  const CheckItem({
    required this.id,
    required this.title,
    required this.reason,
    required this.status,
    this.ctaLabel,
    this.ctaUrl,
    this.optional = false,
  });

  final CheckId id;
  final String title;
  final String reason;
  final CheckStatus status;
  final String? ctaLabel;
  final String? ctaUrl;
  final bool optional;

  CheckItem copyWith({CheckStatus? status}) => CheckItem(
        id: id,
        title: title,
        reason: reason,
        status: status ?? this.status,
        ctaLabel: ctaLabel,
        ctaUrl: ctaUrl,
        optional: optional,
      );
}

class TailscalePeer {
  const TailscalePeer({
    required this.name,
    required this.ipv4,
    required this.online,
    required this.os,
  });

  final String name;
  final String ipv4;
  final bool online;
  final String os;
}

List<CheckId> requiredCheckIds(DeviceRole role, AppPlatform platform) {
  final ids = <CheckId>[];

  void addHost() {
    if (platform == AppPlatform.ios) return;
    ids.addAll([
      CheckId.tailscaleInstalled,
      CheckId.tailscaleRunning,
      CheckId.tailscaleSelfVisible,
      CheckId.screenRecording,
      CheckId.accessibility,
      CheckId.unattendedPassword,
      CheckId.hostListeningArmed,
    ]);
  }

  void addController() {
    ids.addAll([
      CheckId.tailscaleInstalled,
      CheckId.tailscaleRunning,
      CheckId.peerVisible,
    ]);
  }

  if (role.allowsHost) addHost();
  if (role.allowsController) addController();

  return ids.toSet().toList();
}

bool isSetupComplete(
  DeviceRole role,
  AppPlatform platform,
  Map<CheckId, CheckStatus> statuses,
) {
  for (final id in requiredCheckIds(role, platform)) {
    if (statuses[id] != CheckStatus.done) return false;
  }
  return true;
}

String checkTitle(CheckId id) {
  switch (id) {
    case CheckId.tailscaleInstalled:
      return 'Tailscale kurulu';
    case CheckId.tailscaleRunning:
      return 'Tailscale çevrimiçi';
    case CheckId.tailscaleSelfVisible:
      return 'Bu cihaz tailnet\'te görünür';
    case CheckId.screenRecording:
      return 'Ekran Kaydı izni';
    case CheckId.accessibility:
      return 'Erişilebilirlik izni';
    case CheckId.launchAtLogin:
      return 'Giriş öğesi (opsiyonel)';
    case CheckId.unattendedPassword:
      return 'Kalıcı erişim şifresi';
    case CheckId.hostListeningArmed:
      return 'Host dinleme açık';
    case CheckId.peerVisible:
      return 'Uzak cihaz görünür';
  }
}

String checkReason(CheckId id) {
  switch (id) {
    case CheckId.tailscaleInstalled:
      return 'Tailscale olmadan uzak cihazlara erişemezsin';
    case CheckId.tailscaleRunning:
      return 'Tailscale oturumu açık ve çalışıyor olmalı';
    case CheckId.tailscaleSelfVisible:
      return 'Diğer cihazlar seni tailnet\'te görmeli';
    case CheckId.screenRecording:
      return 'Ekran paylaşımı için macOS izni gerekli';
    case CheckId.accessibility:
      return 'Uzak fare/klavye için izin gerekli';
    case CheckId.launchAtLogin:
      return 'Mac açılınca hazır olsun';
    case CheckId.unattendedPassword:
      return 'Katılımsız erişim için güçlü şifre (≥12 karakter)';
    case CheckId.hostListeningArmed:
      return 'Home\'da "Erişilebilir" açık olmalı';
    case CheckId.peerVisible:
      return 'En az bir çevrimiçi cihaz tailnet\'te olmalı';
  }
}

String? checkCtaLabel(CheckId id) {
  switch (id) {
    case CheckId.tailscaleInstalled:
    case CheckId.tailscaleRunning:
      return 'Tailscale\'i aç / indir';
    case CheckId.screenRecording:
    case CheckId.accessibility:
      return 'Sistem Ayarları';
    case CheckId.unattendedPassword:
      return 'Şifre belirle';
    case CheckId.peerVisible:
      return 'Cihaz paylaşımı yardımı';
    default:
      return null;
  }
}

String? checkCtaUrl(CheckId id) {
  switch (id) {
    case CheckId.tailscaleInstalled:
    case CheckId.tailscaleRunning:
      return 'https://tailscale.com/download';
    case CheckId.screenRecording:
      return 'x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture';
    case CheckId.accessibility:
      return 'x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility';
    case CheckId.peerVisible:
      return 'https://tailscale.com/kb/1085/sharing-devices/';
    default:
      return null;
  }
}

bool mayListenAsHost(
  DeviceRole role,
  AppPlatform platform,
  Map<CheckId, CheckStatus> statuses,
  bool userArmed,
) {
  return role.allowsHost &&
      isSetupComplete(role, platform, statuses) &&
      userArmed &&
      statuses[CheckId.screenRecording] == CheckStatus.done &&
      statuses[CheckId.accessibility] == CheckStatus.done;
}
