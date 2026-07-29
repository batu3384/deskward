import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../models/check_item.dart';
import '../state/setup_controller.dart';
import '../theme/deskward_theme.dart';
import 'home_screen.dart';

class ChecklistScreen extends StatefulWidget {
  const ChecklistScreen({super.key, required this.setup});

  final SetupController setup;

  @override
  State<ChecklistScreen> createState() => _ChecklistScreenState();
}

class _ChecklistScreenState extends State<ChecklistScreen> {
  @override
  void initState() {
    super.initState();
    widget.setup.addListener(_onSetupChanged);
  }

  @override
  void dispose() {
    widget.setup.removeListener(_onSetupChanged);
    super.dispose();
  }

  void _onSetupChanged() {
    if (widget.setup.isSetupComplete && mounted) {
      Navigator.of(context).pushReplacement(
        MaterialPageRoute(builder: (_) => HomeScreen(setup: widget.setup)),
      );
    }
    setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    final items = widget.setup.checklistItems;
    final done = widget.setup.completedCount;
    final total = widget.setup.requiredCount;

    return Scaffold(
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 32),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Text(
                'Kurulum',
                style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                      fontWeight: FontWeight.w700,
                    ),
              ),
              const SizedBox(height: 8),
              Text(
                '$done / $total tamamlandı',
                style: Theme.of(context).textTheme.bodyLarge,
              ),
              const SizedBox(height: 24),
              Expanded(
                child: ListView.separated(
                  itemCount: items.length,
                  separatorBuilder: (_, __) => const SizedBox(height: 12),
                  itemBuilder: (context, i) => _ChecklistRow(
                    item: items[i],
                    onCta: () => _handleCta(context, items[i]),
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Future<void> _handleCta(BuildContext context, CheckItem item) async {
    if (item.id == CheckId.unattendedPassword) {
      await _showPasswordDialog(context);
      return;
    }
    final url = item.ctaUrl;
    if (url != null) {
      await Clipboard.setData(ClipboardData(text: url));
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Bağlantı kopyalandı: $url')),
        );
      }
    }
  }

  Future<void> _showPasswordDialog(BuildContext context) async {
    final controller = TextEditingController();
    final ok = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Kalıcı erişim şifresi'),
        content: TextField(
          controller: controller,
          obscureText: true,
          decoration: const InputDecoration(
            labelText: 'Şifre (en az 12 karakter)',
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
            child: const Text('Kaydet'),
          ),
        ],
      ),
    );
    if (ok == true) {
      final saved = await widget.setup.setPassword(controller.text);
      if (context.mounted && !saved) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Şifre en az 12 karakter olmalı')),
        );
      }
    }
    controller.dispose();
  }
}

class _ChecklistRow extends StatelessWidget {
  const _ChecklistRow({required this.item, required this.onCta});

  final CheckItem item;
  final VoidCallback onCta;

  @override
  Widget build(BuildContext context) {
    final icon = switch (item.status) {
      CheckStatus.done => Icon(Icons.check_circle, color: DeskwardTheme.accent),
      CheckStatus.actionNeeded =>
        Icon(Icons.error_outline, color: Theme.of(context).colorScheme.error),
      CheckStatus.pending => const Icon(Icons.radio_button_unchecked),
    };

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                icon,
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        item.title + (item.optional ? ' (opsiyonel)' : ''),
                        style: Theme.of(context).textTheme.titleSmall?.copyWith(
                              fontWeight: FontWeight.w600,
                            ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        item.reason,
                        style: Theme.of(context).textTheme.bodySmall,
                      ),
                    ],
                  ),
                ),
              ],
            ),
            if (item.ctaLabel != null && item.status != CheckStatus.done) ...[
              const SizedBox(height: 12),
              SizedBox(
                width: double.infinity,
                child: FilledButton(
                  style: FilledButton.styleFrom(
                    backgroundColor: DeskwardTheme.accent,
                    minimumSize: const Size.fromHeight(52),
                  ),
                  onPressed: onCta,
                  child: Text(item.ctaLabel!),
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
