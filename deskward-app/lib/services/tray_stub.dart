/// Menu bar / system tray stub — wire via platform channel in Faz 2.1.
///
/// Planned actions:
/// - Show / hide Deskward
/// - Erişilebilir toggle
/// - Quit
class TrayStub {
  static Future<void> init() async {
    // ponytail: no-op until win32/macos tray plugin lands
  }
}
