import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:typed_data';

import 'package:ffi/ffi.dart';

import '../models/check_item.dart';

typedef _FreeStringNative = Void Function(Pointer<Utf8>);
typedef _FreeString = void Function(Pointer<Utf8>);

typedef _TailscaleStatusNative = Pointer<Utf8> Function();
typedef _TailscaleStatus = Pointer<Utf8> Function();

typedef _PermissionsStatusNative = Pointer<Utf8> Function();
typedef _PermissionsStatus = Pointer<Utf8> Function();

typedef _HashPasswordNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _HashPassword = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _ConnectNative = Int64 Function(Pointer<Utf8>, Pointer<Utf8>);
typedef _Connect = int Function(Pointer<Utf8>, Pointer<Utf8>);

typedef _PollFrameNative = Pointer<Utf8> Function(Uint64);
typedef _PollFrame = Pointer<Utf8> Function(int);

typedef _SessionPointerNative = Int32 Function(Uint64, Double, Double, Bool);
typedef _SessionPointer = int Function(int, double, double, bool);

typedef _SessionDisconnectNative = Int32 Function(Uint64);
typedef _SessionDisconnect = int Function(int);

typedef _SessionClipboardNative = Int32 Function(Uint64, Pointer<Utf8>);
typedef _SessionClipboard = int Function(int, Pointer<Utf8>);

typedef _SessionMetricsNative = Pointer<Utf8> Function(Uint64);
typedef _SessionMetrics = Pointer<Utf8> Function(int);

class DeskwardFfi {
  DeskwardFfi._(this._lib);

  static DeskwardFfi? _instance;

  static DeskwardFfi get instance {
    _instance ??= DeskwardFfi._(_openLib());
    return _instance!;
  }

  static bool get isAvailable => _tryOpenLib() != null;

  final DynamicLibrary _lib;
  late final _FreeString _freeString = _lib
      .lookup<NativeFunction<_FreeStringNative>>('deskward_free_string')
      .asFunction();
  late final _TailscaleStatus _tailscaleStatus = _lib
      .lookup<NativeFunction<_TailscaleStatusNative>>('deskward_tailscale_status')
      .asFunction();
  late final _PermissionsStatus _permissionsStatus = _lib
      .lookup<NativeFunction<_PermissionsStatusNative>>(
        'deskward_permissions_status',
      )
      .asFunction();
  late final _HashPassword _hashPassword = _lib
      .lookup<NativeFunction<_HashPasswordNative>>('deskward_hash_password')
      .asFunction();
  late final _Connect _connect = _lib
      .lookup<NativeFunction<_ConnectNative>>('deskward_connect')
      .asFunction();
  late final _PollFrame _pollFrame = _lib
      .lookup<NativeFunction<_PollFrameNative>>('deskward_session_poll_frame')
      .asFunction();
  late final _SessionPointer _sessionPointer = _lib
      .lookup<NativeFunction<_SessionPointerNative>>('deskward_session_pointer')
      .asFunction();
  late final _SessionDisconnect _sessionDisconnect = _lib
      .lookup<NativeFunction<_SessionDisconnectNative>>(
        'deskward_session_disconnect',
      )
      .asFunction();
  late final _SessionClipboard _sessionClipboard = _lib
      .lookup<NativeFunction<_SessionClipboardNative>>(
        'deskward_session_send_clipboard',
      )
      .asFunction();
  late final _SessionMetrics _sessionMetrics = _lib
      .lookup<NativeFunction<_SessionMetricsNative>>(
        'deskward_session_metrics',
      )
      .asFunction();

  static DynamicLibrary? _tryOpenLib() {
    try {
      if (Platform.isMacOS) {
        const paths = [
          'libdeskward_ffi.dylib',
          '../target/debug/libdeskward_ffi.dylib',
          '../target/release/libdeskward_ffi.dylib',
        ];
        for (final path in paths) {
          try {
            return DynamicLibrary.open(path);
          } catch (_) {}
        }
      }
      return null;
    } catch (_) {
      return null;
    }
  }

  static DynamicLibrary _openLib() {
    final lib = _tryOpenLib();
    if (lib == null) {
      throw StateError(
        'deskward FFI library not found — run: cargo build -p deskward-ffi',
      );
    }
    return lib;
  }

  String? _takeString(Pointer<Utf8> ptr) {
    if (ptr == nullptr) return null;
    try {
      return ptr.toDartString();
    } finally {
      _freeString(ptr);
    }
  }

  TailscalePollResult pollTailscale() {
    final json = _takeString(_tailscaleStatus());
    if (json == null) return TailscalePollResult.empty();
    final map = jsonDecode(json) as Map<String, dynamic>;
    final peers = (map['peers'] as List<dynamic>? ?? [])
        .map(
          (p) => TailscalePeer(
            name: p['name'] as String? ?? '',
            ipv4: p['ipv4'] as String? ?? '',
            online: p['online'] as bool? ?? false,
            os: p['os'] as String? ?? 'unknown',
          ),
        )
        .toList();
    return TailscalePollResult(
      installed: map['installed'] as bool? ?? false,
      running: map['running'] as bool? ?? false,
      selfName: map['self_name'] as String?,
      selfIpv4: map['self_ipv4'] as String?,
      peers: peers,
    );
  }

  ({bool screenRecording, bool accessibility}) pollPermissions() {
    final json = _takeString(_permissionsStatus());
    if (json == null) {
      return (screenRecording: false, accessibility: false);
    }
    final map = jsonDecode(json) as Map<String, dynamic>;
    return (
      screenRecording: map['screen_recording'] as bool? ?? false,
      accessibility: map['accessibility'] as bool? ?? false,
    );
  }

  String? hashPassword(String plain) {
    final ptr = plain.toNativeUtf8();
    try {
      return _takeString(_hashPassword(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  ConnectResponse connect(String peerName, String password) {
    final peerPtr = peerName.toNativeUtf8();
    final passPtr = password.toNativeUtf8();
    try {
      final code = _connect(peerPtr, passPtr);
      if (code > 0) {
        return ConnectResponse.success(code);
      }
      return ConnectResponse.failure(ConnectResult.fromCode(code));
    } finally {
      malloc.free(peerPtr);
      malloc.free(passPtr);
    }
  }

  SessionFrame? pollFrame(int sessionId) {
    final json = _takeString(_pollFrame(sessionId));
    if (json == null) return null;
    final map = jsonDecode(json) as Map<String, dynamic>;
    final data = base64Decode(map['data_b64'] as String? ?? '');
    return SessionFrame(
      width: map['width'] as int? ?? 0,
      height: map['height'] as int? ?? 0,
      codec: map['codec'] as String? ?? 'jpeg',
      bytes: Uint8List.fromList(data),
    );
  }

  void sendPointer(int sessionId, double x, double y, bool pressed) {
    _sessionPointer(sessionId, x, y, pressed);
  }

  void sendClipboard(int sessionId, String text) {
    final ptr = text.toNativeUtf8();
    try {
      _sessionClipboard(sessionId, ptr);
    } finally {
      malloc.free(ptr);
    }
  }

  SessionMetrics? pollMetrics(int sessionId) {
    final json = _takeString(_sessionMetrics(sessionId));
    if (json == null) return null;
    final map = jsonDecode(json) as Map<String, dynamic>;
    return SessionMetrics(
      fps: (map['fps'] as num?)?.toDouble() ?? 0,
      framesReceived: map['frames_received'] as int? ?? 0,
      bytesReceived: map['bytes_received'] as int? ?? 0,
      decoder: map['decoder'] as String? ?? 'unknown',
    );
  }

  void disconnect(int sessionId) {
    _sessionDisconnect(sessionId);
  }
}

class SessionMetrics {
  const SessionMetrics({
    required this.fps,
    required this.framesReceived,
    required this.bytesReceived,
    required this.decoder,
  });

  final double fps;
  final int framesReceived;
  final int bytesReceived;
  final String decoder;

  String get kbpsLabel {
    final kb = bytesReceived / 1024;
    return '${fps.toStringAsFixed(1)} fps · ${kb.toStringAsFixed(0)} KB · $decoder';
  }
}

class SessionFrame {
  const SessionFrame({
    required this.width,
    required this.height,
    required this.codec,
    required this.bytes,
  });

  final int width;
  final int height;
  final String codec;
  final Uint8List bytes;
}

class ConnectResponse {
  const ConnectResponse._({this.sessionId, this.error});

  factory ConnectResponse.success(int sessionId) =>
      ConnectResponse._(sessionId: sessionId);

  factory ConnectResponse.failure(ConnectResult error) =>
      ConnectResponse._(error: error);

  final int? sessionId;
  final ConnectResult? error;

  bool get isOk => sessionId != null;
}

class TailscalePollResult {
  const TailscalePollResult({
    required this.installed,
    required this.running,
    this.selfName,
    this.selfIpv4,
    required this.peers,
  });

  factory TailscalePollResult.empty() => const TailscalePollResult(
        installed: false,
        running: false,
        peers: [],
      );

  final bool installed;
  final bool running;
  final String? selfName;
  final String? selfIpv4;
  final List<TailscalePeer> peers;
}

enum ConnectResult {
  ok,
  invalidPeer,
  invalidPassword,
  authFailed,
  peerNotFound,
  handshakeFailed,
  unknown;

  static ConnectResult fromCode(int code) => switch (code) {
        0 => ConnectResult.ok,
        -1 => ConnectResult.invalidPeer,
        -2 => ConnectResult.invalidPassword,
        -3 => ConnectResult.authFailed,
        -4 => ConnectResult.peerNotFound,
        -5 => ConnectResult.handshakeFailed,
        _ => ConnectResult.unknown,
      };

  String get message => switch (this) {
        ConnectResult.ok => 'Bağlandı',
        ConnectResult.invalidPeer => 'Geçersiz cihaz adı',
        ConnectResult.invalidPassword => 'Şifre gerekli',
        ConnectResult.authFailed => 'Yanlış şifre',
        ConnectResult.peerNotFound => 'Cihaz bulunamadı veya çevrimdışı',
        ConnectResult.handshakeFailed => 'Güvenlik el sıkışması başarısız',
        ConnectResult.unknown => 'Bağlantı hatası',
      };
}
