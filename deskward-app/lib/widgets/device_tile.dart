import 'package:flutter/material.dart';

import '../models/check_item.dart';
import '../theme/deskward_theme.dart';

class DeviceTile extends StatelessWidget {
  const DeviceTile({
    super.key,
    required this.peer,
    required this.onConnect,
  });

  final TailscalePeer peer;
  final VoidCallback onConnect;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: ListTile(
        leading: CircleAvatar(
          backgroundColor: peer.online
              ? DeskwardTheme.accent.withValues(alpha: 0.15)
              : Theme.of(context).colorScheme.surfaceContainerHighest,
          child: Icon(
            _osIcon(peer.os),
            color: peer.online ? DeskwardTheme.accent : null,
          ),
        ),
        title: Text(peer.name),
        subtitle: Text('${peer.ipv4} · ${peer.os}'),
        trailing: peer.online
            ? FilledButton(
                style: FilledButton.styleFrom(backgroundColor: DeskwardTheme.accent),
                onPressed: onConnect,
                child: const Text('Bağlan'),
              )
            : const Text('Çevrimdışı'),
      ),
    );
  }

  IconData _osIcon(String os) {
    final lower = os.toLowerCase();
    if (lower.contains('mac')) return Icons.laptop_mac;
    if (lower.contains('win')) return Icons.laptop_windows;
    if (lower.contains('ios') || lower.contains('iphone')) {
      return Icons.phone_iphone;
    }
    return Icons.computer;
  }
}
