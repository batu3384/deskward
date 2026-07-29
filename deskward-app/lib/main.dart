import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

import 'theme/deskward_theme.dart';
import 'screens/home_screen.dart';

void main() {
  runApp(const DeskwardApp());
}

class DeskwardApp extends StatelessWidget {
  const DeskwardApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Deskward',
      debugShowCheckedModeBanner: false,
      theme: DeskwardTheme.light,
      darkTheme: DeskwardTheme.dark,
      themeMode: ThemeMode.system,
      home: const HomeScreen(),
    );
  }
}
