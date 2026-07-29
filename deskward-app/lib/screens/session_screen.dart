import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../ffi/deskward_ffi.dart';
import '../theme/deskward_theme.dart';

/// Remote session — JPEG frame poll + touch pointer injection.
class SessionScreen extends StatefulWidget {
  const SessionScreen({
    super.key,
    required this.peerName,
    required this.sessionId,
  });

  final String peerName;
  final int sessionId;

  @override
  State<SessionScreen> createState() => _SessionScreenState();
}

class _SessionScreenState extends State<SessionScreen> {
  Uint8List? _frameBytes;
  Timer? _pollTimer;
  Size? _remoteSize;
  String _perfLabel = '';

  @override
  void initState() {
    super.initState();
    _pollTimer = Timer.periodic(const Duration(milliseconds: 66), (_) => _poll());
  }

  @override
  void dispose() {
    _pollTimer?.cancel();
    if (DeskwardFfi.isAvailable) {
      DeskwardFfi.instance.disconnect(widget.sessionId);
    }
    super.dispose();
  }

  void _poll() {
    if (!DeskwardFfi.isAvailable) return;
    final frame = DeskwardFfi.instance.pollFrame(widget.sessionId);
    final metrics = DeskwardFfi.instance.pollMetrics(widget.sessionId);
    if (!mounted) return;
    if (frame != null) {
      setState(() {
        _frameBytes = frame.bytes;
        _remoteSize = Size(frame.width.toDouble(), frame.height.toDouble());
      });
    }
    if (metrics != null) {
      setState(() => _perfLabel = metrics.kbpsLabel);
    }
  }

  Future<void> _sendClipboardFromSystem() async {
    final data = await Clipboard.getData('text/plain');
    final text = data?.text;
    if (text == null || text.isEmpty) return;
    if (!DeskwardFfi.isAvailable) return;
    DeskwardFfi.instance.sendClipboard(widget.sessionId, text);
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Pano host\'a gönderildi')),
    );
  }

  void _sendPointer(Offset local, Size viewSize, bool pressed) {
    if (!DeskwardFfi.isAvailable || _remoteSize == null) return;
    final remote = _remoteSize!;
    final x = (local.dx / viewSize.width) * remote.width;
    final y = (local.dy / viewSize.height) * remote.height;
    DeskwardFfi.instance.sendPointer(widget.sessionId, x, y, pressed);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.black,
      appBar: AppBar(
        backgroundColor: Colors.black,
        foregroundColor: DeskwardTheme.inkDark,
        title: Text(widget.peerName),
        actions: [
          IconButton(
            tooltip: 'Panodan gönder',
            icon: const Icon(Icons.content_paste),
            iconSize: 24,
            padding: const EdgeInsets.all(12),
            constraints: const BoxConstraints(minWidth: 48, minHeight: 48),
            onPressed: _sendClipboardFromSystem,
          ),
          Padding(
            padding: const EdgeInsets.only(right: 8),
            child: Center(
              child: Text(
                _perfLabel,
                style: Theme.of(context).textTheme.labelSmall?.copyWith(
                      color: DeskwardTheme.inkDark.withValues(alpha: 0.8),
                    ),
              ),
            ),
          ),
          Padding(
            padding: const EdgeInsets.only(right: 12),
            child: Chip(
              label: const Text('E2EE'),
              avatar: Icon(Icons.lock, size: 14, color: DeskwardTheme.accent),
            ),
          ),
        ],
      ),
      body: LayoutBuilder(
        builder: (context, constraints) {
          final viewSize = Size(constraints.maxWidth, constraints.maxHeight);
          return GestureDetector(
            behavior: HitTestBehavior.opaque,
            onPanDown: (d) => _sendPointer(d.localPosition, viewSize, true),
            onPanUpdate: (d) => _sendPointer(d.localPosition, viewSize, true),
            onPanEnd: (_) => _sendPointer(Offset.zero, viewSize, false),
            child: Center(
              child: _frameBytes == null
                  ? Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        const CircularProgressIndicator(color: DeskwardTheme.accent),
                        const SizedBox(height: 16),
                        Text(
                          'Video bekleniyor…',
                          style: Theme.of(context)
                              .textTheme
                              .bodyMedium
                              ?.copyWith(color: DeskwardTheme.inkDark),
                        ),
                      ],
                    )
                  : InteractiveViewer(
                      minScale: 0.5,
                      maxScale: 3,
                      child: Image.memory(
                        _frameBytes!,
                        fit: BoxFit.contain,
                        gaplessPlayback: true,
                      ),
                    ),
            ),
          );
        },
      ),
    );
  }
}
