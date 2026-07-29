import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import '../models/device_role.dart';
import '../state/setup_controller.dart';
import '../theme/deskward_theme.dart';
import 'checklist_screen.dart';

class RolePickerScreen extends StatelessWidget {
  const RolePickerScreen({super.key, required this.setup});

  final SetupController setup;

  bool get _isIos {
    if (kIsWeb) return false;
    return defaultTargetPlatform == TargetPlatform.iOS;
  }

  @override
  Widget build(BuildContext context) {
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
                      color: Theme.of(context).brightness == Brightness.dark
                          ? DeskwardTheme.inkDark
                          : DeskwardTheme.primary,
                    ),
              ),
              const SizedBox(height: 8),
              Text(
                'Bu cihaz ne yapacak?',
                style: Theme.of(context).textTheme.titleLarge,
              ),
              const SizedBox(height: 32),
              _RoleRow(
                role: DeviceRole.host,
                disabled: _isIos,
                disabledReason: 'iOS uzaktan kontrol edilemez',
                onTap: () => _pick(context, DeviceRole.host),
              ),
              const SizedBox(height: 12),
              _RoleRow(
                role: DeviceRole.controller,
                onTap: () => _pick(context, DeviceRole.controller),
              ),
              const SizedBox(height: 12),
              _RoleRow(
                role: DeviceRole.both,
                disabled: _isIos,
                disabledReason: 'iOS host olamaz',
                onTap: () => _pick(context, DeviceRole.both),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Future<void> _pick(BuildContext context, DeviceRole role) async {
    await setup.setRole(role);
    if (!context.mounted) return;
    Navigator.of(context).pushReplacement(
      MaterialPageRoute(builder: (_) => ChecklistScreen(setup: setup)),
    );
  }
}

class _RoleRow extends StatelessWidget {
  const _RoleRow({
    required this.role,
    required this.onTap,
    this.disabled = false,
    this.disabledReason,
  });

  final DeviceRole role;
  final VoidCallback onTap;
  final bool disabled;
  final String? disabledReason;

  @override
  Widget build(BuildContext context) {
    return Opacity(
      opacity: disabled ? 0.45 : 1,
      child: Material(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(12),
        child: InkWell(
          onTap: disabled ? null : onTap,
          borderRadius: BorderRadius.circular(12),
          child: Padding(
            padding: const EdgeInsets.all(20),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  role.label,
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.w600,
                      ),
                ),
                const SizedBox(height: 4),
                Text(
                  disabled && disabledReason != null
                      ? disabledReason!
                      : role.subtitle,
                  style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                        color: Theme.of(context)
                            .colorScheme
                            .onSurface
                            .withValues(alpha: 0.7),
                      ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
