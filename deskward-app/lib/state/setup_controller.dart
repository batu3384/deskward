import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../ffi/deskward_ffi.dart';
import '../models/check_item.dart';
import '../models/device_role.dart';

class SetupController extends ChangeNotifier {
  SetupController();

  static const _roleKey = 'deskward_role';
  static const _passwordHashKey = 'deskward_password_hash';
  static const _userArmedKey = 'deskward_user_armed';
  static const _codecKey = 'deskward_codec';

  DeviceRole? _role;
  AppPlatform _platform = AppPlatform.macOs;
  final Map<CheckId, CheckStatus> _statuses = {};
  bool _userArmed = false;
  String? _passwordHash;
  bool _tailscaleRunning = false;
  bool _tailscaleInstalled = false;
  String? _selfName;
  String? _selfIpv4;
  List<TailscalePeer> _peers = [];
  Timer? _pollTimer;
  SharedPreferences? _prefs;
  bool _ffiAvailable = false;
  CodecOption _codec = CodecOption.jpeg;

  DeviceRole? get role => _role;
  AppPlatform get platform => _platform;
  bool get userArmed => _userArmed;
  List<TailscalePeer> get peers => List.unmodifiable(_peers);
  bool get tailscaleRunning => _tailscaleRunning;
  bool get ffiAvailable => _ffiAvailable;
  CodecOption get codec => _codec;

  bool get isSetupComplete {
    if (_role == null) return false;
    return isSetupCompleteFor(_role!, _platform, _statuses);
  }

  bool get mayListenAsHost {
    if (_role == null) return false;
    return mayListenAsHostFor(_role!, _platform, _statuses, _userArmed);
  }

  Future<void> init() async {
    _platform = _detectPlatform();
    _ffiAvailable = !kIsWeb && DeskwardFfi.isAvailable;
    _prefs = await SharedPreferences.getInstance();
    _role = DeviceRole.fromStorage(_prefs!.getString(_roleKey));
    _userArmed = _prefs!.getBool(_userArmedKey) ?? false;
    _passwordHash = _prefs!.getString(_passwordHashKey);
    _codec = CodecOption.fromStorage(_prefs!.getString(_codecKey));
    _poll();
    _pollTimer = Timer.periodic(const Duration(seconds: 2), (_) => _poll());
    notifyListeners();
  }

  AppPlatform _detectPlatform() {
    if (kIsWeb) return AppPlatform.macOs;
    if (Platform.isIOS) return AppPlatform.ios;
    if (Platform.isMacOS) return AppPlatform.macOs;
    if (Platform.isWindows) return AppPlatform.windows;
    return AppPlatform.macOs;
  }

  Future<void> setRole(DeviceRole role) async {
    _role = role;
    await _prefs?.setString(_roleKey, role.storageValue);
    _refreshStatuses();
    await _persistHostBridge();
    notifyListeners();
  }

  Future<void> setUserArmed(bool armed) async {
    _userArmed = armed;
    await _prefs?.setBool(_userArmedKey, armed);
    _refreshStatuses();
    await _persistHostBridge();
    notifyListeners();
  }

  Future<void> setCodec(CodecOption codec) async {
    _codec = codec;
    await _prefs?.setString(_codecKey, codec.storageValue);
    await _persistHostBridge();
    notifyListeners();
  }

  Future<bool> setPassword(String plain) async {
    if (plain.length < 12) return false;
    if (_ffiAvailable) {
      final hash = DeskwardFfi.instance.hashPassword(plain);
      if (hash == null) return false;
      _passwordHash = hash;
    } else {
      _passwordHash = 'argon2id:stub:${plain.hashCode}';
    }
    await _prefs?.setString(_passwordHashKey, _passwordHash!);
    _refreshStatuses();
    await _persistHostBridge();
    notifyListeners();
    return true;
  }

  ConnectResponse connectToPeer(String peerName, String password) {
    if (!_ffiAvailable) {
      return ConnectResponse.failure(ConnectResult.unknown);
    }
    return DeskwardFfi.instance.connect(peerName, password);
  }

  List<CheckItem> get checklistItems {
    if (_role == null) return [];
    final ids = requiredCheckIds(_role!, _platform);
    final items = ids
        .map(
          (id) => CheckItem(
            id: id,
            title: checkTitle(id),
            reason: checkReason(id),
            status: _statuses[id] ?? CheckStatus.pending,
            ctaLabel: checkCtaLabel(id),
            ctaUrl: checkCtaUrl(id),
          ),
        )
        .toList();

    if (_role!.allowsHost && _platform != AppPlatform.ios) {
      items.add(
        CheckItem(
          id: CheckId.launchAtLogin,
          title: checkTitle(CheckId.launchAtLogin),
          reason: checkReason(CheckId.launchAtLogin),
          status: CheckStatus.pending,
          optional: true,
        ),
      );
    }
    return items;
  }

  int get completedCount => checklistItems
      .where((i) => !i.optional && i.status == CheckStatus.done)
      .length;

  int get requiredCount =>
      checklistItems.where((i) => !i.optional).length;

  void _poll() {
    if (_ffiAvailable) {
      _pollFfi();
    } else {
      _pollStub();
    }
    _refreshStatuses();
    notifyListeners();
  }

  void _pollFfi() {
    try {
      final ts = DeskwardFfi.instance.pollTailscale();
      _tailscaleInstalled = ts.installed;
      _tailscaleRunning = ts.running;
      _selfName = ts.selfName;
      _selfIpv4 = ts.selfIpv4;
      _peers = ts.peers;
    } catch (_) {
      _pollStub();
    }
  }

  void _pollStub() {
    _tailscaleInstalled = true;
    _tailscaleRunning = true;
    _selfName = 'ev-mac';
    _selfIpv4 = '100.64.0.2';
    _peers = const [
      TailscalePeer(
        name: 'ofis-win',
        ipv4: '100.64.0.3',
        online: true,
        os: 'Windows',
      ),
    ];
  }

  void _refreshStatuses() {
    if (_role == null) return;

    void set(CheckId id, CheckStatus s) => _statuses[id] = s;

    set(
      CheckId.tailscaleInstalled,
      _tailscaleInstalled ? CheckStatus.done : CheckStatus.actionNeeded,
    );
    set(
      CheckId.tailscaleRunning,
      _tailscaleRunning ? CheckStatus.done : CheckStatus.actionNeeded,
    );
    set(
      CheckId.tailscaleSelfVisible,
      _selfName != null && _selfIpv4 != null
          ? CheckStatus.done
          : CheckStatus.actionNeeded,
    );
    set(
      CheckId.peerVisible,
      _peers.any((p) => p.online)
          ? CheckStatus.done
          : CheckStatus.actionNeeded,
    );

    if (_role!.allowsHost && _platform != AppPlatform.ios) {
      final perms = _ffiAvailable
          ? DeskwardFfi.instance.pollPermissions()
          : (screenRecording: false, accessibility: false);
      set(
        CheckId.screenRecording,
        perms.screenRecording ? CheckStatus.done : CheckStatus.actionNeeded,
      );
      set(
        CheckId.accessibility,
        perms.accessibility ? CheckStatus.done : CheckStatus.actionNeeded,
      );
      set(
        CheckId.unattendedPassword,
        _passwordHash != null ? CheckStatus.done : CheckStatus.actionNeeded,
      );
      set(
        CheckId.hostListeningArmed,
        _userArmed ? CheckStatus.done : CheckStatus.pending,
      );
    }
    unawaited(_persistHostBridge());
  }

  Future<void> _persistHostBridge() async {
    if (!Platform.isMacOS || _role == null || !_role!.allowsHost) return;
    final home = Platform.environment['HOME'];
    if (home == null) return;

    final checks = <String, String>{};
    for (final id in requiredCheckIds(_role!, _platform)) {
      checks[_checkIdJson(id)] = _checkStatusJson(_statuses[id] ?? CheckStatus.pending);
    }

    final payload = {
      'snapshot': {
        'role': _role!.storageValue,
        'platform': 'mac_os',
        'checks': checks,
      },
      'user_armed': _userArmed,
      'password_hash': _passwordHash,
      'codec': _codec.storageValue,
    };

    final dir = Directory('$home/.deskward');
    if (!dir.existsSync()) {
      await dir.create(recursive: true);
    }
    await File('${dir.path}/setup.json').writeAsString(
      const JsonEncoder.withIndent('  ').convert(payload),
    );
  }

  String _checkIdJson(CheckId id) => switch (id) {
        CheckId.tailscaleInstalled => 'tailscale_installed',
        CheckId.tailscaleRunning => 'tailscale_running',
        CheckId.tailscaleSelfVisible => 'tailscale_self_visible',
        CheckId.screenRecording => 'screen_recording',
        CheckId.accessibility => 'accessibility',
        CheckId.launchAtLogin => 'launch_at_login',
        CheckId.unattendedPassword => 'unattended_password',
        CheckId.hostListeningArmed => 'host_listening_armed',
        CheckId.peerVisible => 'peer_visible',
      };

  String _checkStatusJson(CheckStatus status) => switch (status) {
        CheckStatus.pending => 'pending',
        CheckStatus.actionNeeded => 'action_needed',
        CheckStatus.done => 'done',
      };

  @override
  void dispose() {
    _pollTimer?.cancel();
    super.dispose();
  }
}

enum CodecOption {
  jpeg('jpeg', 'JPEG', 'Varsayılan — düşük gecikme'),
  h264('h264', 'H.264 (yazılım)', 'OpenH264 — tüm platformlar'),
  h264Hw('h264-hw', 'H.264 (donanım)', 'VideoToolbox — yalnızca macOS host');

  const CodecOption(this.storageValue, this.label, this.hint);

  final String storageValue;
  final String label;
  final String hint;

  static CodecOption fromStorage(String? raw) => switch (raw) {
        'h264-hw' => CodecOption.h264Hw,
        'h264' => CodecOption.h264,
        _ => CodecOption.jpeg,
      };
}

bool isSetupCompleteFor(
  DeviceRole role,
  AppPlatform platform,
  Map<CheckId, CheckStatus> statuses,
) {
  for (final id in requiredCheckIds(role, platform)) {
    if (statuses[id] != CheckStatus.done) return false;
  }
  return true;
}

bool mayListenAsHostFor(
  DeviceRole role,
  AppPlatform platform,
  Map<CheckId, CheckStatus> statuses,
  bool userArmed,
) {
  return role.allowsHost &&
      isSetupCompleteFor(role, platform, statuses) &&
      userArmed &&
      statuses[CheckId.screenRecording] == CheckStatus.done &&
      statuses[CheckId.accessibility] == CheckStatus.done;
}
