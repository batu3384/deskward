import 'package:flutter/material.dart';

import '../theme/deskward_theme.dart';

/// Home: brand + ID + connect CTA (no dashboard clutter).
class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  final _peerController = TextEditingController();
  String _status = 'Hazır';

  @override
  void dispose() {
    _peerController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    return Scaffold(
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
                      color: isDark ? DeskwardTheme.inkDark : DeskwardTheme.primary,
                    ),
              ),
              const SizedBox(height: 8),
              Text(
                'Uzak masaüstü — self-host',
                style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                      color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.7),
                    ),
              ),
              const Spacer(),
              TextField(
                controller: _peerController,
                decoration: const InputDecoration(
                  labelText: 'Hedef ID',
                  hintText: 'ör. mac-ev',
                  border: OutlineInputBorder(),
                ),
                textInputAction: TextInputAction.done,
              ),
              const SizedBox(height: 12),
              Text(
                _status,
                textAlign: TextAlign.center,
                style: Theme.of(context).textTheme.bodySmall,
              ),
              const SizedBox(height: 24),
              FilledButton(
                style: FilledButton.styleFrom(
                  backgroundColor: DeskwardTheme.accent,
                  minimumSize: const Size.fromHeight(52),
                ),
                onPressed: () {
                  setState(() => _status = 'Bağlanıyor… (Faz 1 FFI)');
                },
                child: const Text('Bağlan'),
              ),
              const SizedBox(height: 12),
              TextButton(
                onPressed: () {},
                child: const Text('Ağ ayarları'),
              ),
              const Spacer(flex: 2),
            ],
          ),
        ),
      ),
    );
  }
}
