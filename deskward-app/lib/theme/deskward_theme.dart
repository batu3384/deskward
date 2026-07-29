import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

/// Tokens from design-system/deskward/MASTER.md
class DeskwardTheme {
  static const primary = Color(0xFF1E3A5F);
  static const accent = Color(0xFF059669);
  static const background = Color(0xFFF8FAFC);
  static const foreground = Color(0xFF0F172A);
  static const surfaceDark = Color(0xFF0F1011);
  static const inkDark = Color(0xFFF7F8F8);

  static ThemeData get light => ThemeData(
        useMaterial3: true,
        colorScheme: ColorScheme.fromSeed(
          seedColor: primary,
          primary: primary,
          secondary: accent,
          surface: background,
          onSurface: foreground,
        ),
        textTheme: GoogleFonts.interTextTheme(),
        scaffoldBackgroundColor: background,
      );

  static ThemeData get dark => ThemeData(
        useMaterial3: true,
        brightness: Brightness.dark,
        colorScheme: ColorScheme.fromSeed(
          seedColor: primary,
          brightness: Brightness.dark,
          primary: const Color(0xFF5E6AD2),
          surface: surfaceDark,
          onSurface: inkDark,
        ),
        textTheme: GoogleFonts.interTextTheme(ThemeData.dark().textTheme),
        scaffoldBackgroundColor: surfaceDark,
      );
}
