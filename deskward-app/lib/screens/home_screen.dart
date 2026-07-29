import 'package:flutter/material.dart';

import '../ffi/deskward_ffi.dart';
import '../state/setup_controller.dart';
import '../theme/deskward_theme.dart';
import '../widgets/device_tile.dart';
import 'checklist_screen.dart';
import 'session_screen.dart';
import 'settings_screen.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key, required this.setup});

  final SetupController setup;

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  @override
  void initState() {
    super.initState();
    widget.setup.addListener(_onSetupChanged);
    WidgetsBinding.instance.addPostFrameCallback((_) => _guardSetup());
  }

  @override
  void dispose() {
    widget.setup.removeListener(_onSetupChanged);
    super.dispose();
  }

  void _onSetupChanged() => setState(() {});

  void _guardSetup() {
    if (!widget.setup.isSetupComplete && mounted) {
      Navigator.of(context).pushReplacement(
        MaterialPageRoute(builder: (_) => ChecklistScreen(setup: widget.setup)),
      );
    }
  }

  Future<void> _connectTo(TailscalePeer peer) async {
    final password = await _askPassword(context);
    if (password == null || !mounted) return;

    if (!widget.setup.ffiAvailable) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('FFI yok — cargo build -p deskward-ffi çalıştır'),
        ),
      );
      return;
    }

    final response = widget.setup.connectToPeer(peer.name, password);
    if (!mounted) return;

    if (response.isOk && response.sessionId != null) {
      Navigator.of(context).push(
        MaterialPageRoute(
          builder: (_) => SessionScreen(
            peerName: peer.name,
            sessionId: response.sessionId!,
          ),
        ),
      );
    } else {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(response.error?.message ?? 'Bağlantı hatası')),
      );
    }
  }

  Future<String?> _askPassword(BuildContext context) async {
    final controller = TextEditingController();
    final ok = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('Host şifresi'),
        content: TextField(
          controller: controller,
          obscureText: true,
          decoration: const InputDecoration(
            labelText: 'Şifre',
            border: OutlineInputBorder(),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('İptal'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: const Text('Bağlan'),
          ),
        ],
      ),
    );
    if (ok != true) {
      controller.dispose();
      return null;
    }
    final value = controller.text;
    controller.dispose();
    return value;
  }

  @override
  Widget build(BuildContext context) {
    if (!widget.setup.isSetupComplete) {
      return const Scaffold(body: Center(child: CircularProgressIndicator()));
    }

    final peers = widget.setup.peers.where((p) => p.online).toList();
    final role = widget.setup.role;
    final showHostControls = role?.allowsHost ?? false;

    return Scaffold(
      appBar: AppBar(
        actions: [
          IconButton(
            tooltip: 'Ayarlar',
            icon: const Icon(Icons.settings_outlined),
            onPressed: () {
              Navigator.of(context).push(
                MaterialPageRoute(
                  builder: (_) => SettingsScreen(setup: widget.setup),
                ),
              );
            },
          ),
        ],
      ),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 32),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Text(
                'Deskward',
                style: Theme.of(context).textTheme.displaySmall?.copyWith(
                      fontWeight: FontWeight.w700,
                      letterSpacing: -1.2,
                    ),
              ),
              const SizedBox(height: 8),
              _StatusChip(
                running: widget.setup.tailscaleRunning,
                ffi: widget.setup.ffiAvailable,
              ),
              if (showHostControls) ...[
                const SizedBox(height: 16),
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  title: const Text('Erişilebilir'),
                  subtitle: Text(
                    widget.setup.mayListenAsHost
                        ? 'Host dinliyor (tailnet · :29118)'
                        : 'İzinler veya şifre eksik — dinleme kapalı',
                  ),
                  value: widget.setup.userArmed,
                  activeThumbColor: DeskwardTheme.accent,
                  onChanged: (v) => widget.setup.setUserArmed(v),
                ),
              ],
              const SizedBox(height: 24),
              Text(
                'Cihazlar',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 12),
              Expanded(
                child: peers.isEmpty
                    ? Center(
                        child: Text(
                          'Tailscale\'de çevrimiçi cihaz yok.\nCihaz paylaşımı veya davet gerekli.',
                          textAlign: TextAlign.center,
                          style: Theme.of(context).textTheme.bodyMedium,
                        ),
                      )
                    : ListView.separated(
                        itemCount: peers.length,
                        separatorBuilder: (_, __) => const SizedBox(height: 8),
                        itemBuilder: (context, i) => DeviceTile(
                          peer: peers[i],
                          onConnect: () => _connectTo(peers[i]),
                        ),
                      ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _StatusChip extends StatelessWidget {
  const _StatusChip({required this.running, required this.ffi});

  final bool running;
  final bool ffi;

  @override
  Widget build(BuildContext context) {
    final label = running
        ? (ffi ? 'Tailscale · Çevrimiçi' : 'Tailscale · Mock')
        : 'Tailscale · Çevrimdışı';
    return Align(
      alignment: Alignment.centerLeft,
      child: Chip(
        avatar: Icon(
          Icons.circle,
          size: 12,
          color: running ? DeskwardTheme.accent : Colors.grey,
        ),
        label: Text(label),
      ),
    );
  }
}
