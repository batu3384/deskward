import 'package:flutter/material.dart';

import 'screens/checklist_screen.dart';
import 'screens/home_screen.dart';
import 'screens/role_picker_screen.dart';
import 'state/setup_controller.dart';
import 'theme/deskward_theme.dart';

void main() {
  runApp(const DeskwardApp());
}

class DeskwardApp extends StatefulWidget {
  const DeskwardApp({super.key});

  @override
  State<DeskwardApp> createState() => _DeskwardAppState();
}

class _DeskwardAppState extends State<DeskwardApp> {
  final _setup = SetupController();
  bool _ready = false;

  @override
  void initState() {
    super.initState();
    _setup.init().then((_) {
      if (mounted) setState(() => _ready = true);
    });
  }

  @override
  void dispose() {
    _setup.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Deskward',
      debugShowCheckedModeBanner: false,
      theme: DeskwardTheme.light,
      darkTheme: DeskwardTheme.dark,
      themeMode: ThemeMode.system,
      home: !_ready
          ? const Scaffold(body: Center(child: CircularProgressIndicator()))
          : _initialScreen(),
    );
  }

  Widget _initialScreen() {
    if (_setup.role == null) {
      return RolePickerScreen(setup: _setup);
    }
    if (!_setup.isSetupComplete) {
      return ChecklistScreen(setup: _setup);
    }
    return HomeScreen(setup: _setup);
  }
}
