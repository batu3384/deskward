import 'dart:io';

import 'package:flutter/material.dart';

import '../state/setup_controller.dart';
import '../theme/deskward_theme.dart';

class SettingsScreen extends StatelessWidget {
  const SettingsScreen({super.key, required this.setup});

  final SetupController setup;

  Iterable<CodecOption> get _codecOptions {
    if (Platform.isMacOS) return CodecOption.values;
    return CodecOption.values.where((o) => o != CodecOption.h264Hw);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Ayarlar')),
      body: ListenableBuilder(
        listenable: setup,
        builder: (context, _) => ListView(
          padding: const EdgeInsets.all(24),
          children: [
            Text('Video codec', style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 8),
            Text(
              'Host cihazda kayıt kalitesi. H.264 HW yalnızca macOS VideoToolbox.',
              style: Theme.of(context).textTheme.bodySmall,
            ),
            const SizedBox(height: 12),
            ..._codecOptions.map(
              (opt) => RadioListTile<CodecOption>(
                contentPadding: EdgeInsets.zero,
                title: Text(opt.label),
                subtitle: Text(opt.hint),
                value: opt,
                groupValue: setup.codec,
                activeColor: DeskwardTheme.accent,
                onChanged: (v) {
                  if (v != null) setup.setCodec(v);
                },
              ),
            ),
            const SizedBox(height: 24),
            Text(
              'Host agent `~/.deskward/setup.json` dosyasını okur. '
              'Değişiklikler host yeniden başlatılınca uygulanır.',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ),
      ),
    );
  }
}
