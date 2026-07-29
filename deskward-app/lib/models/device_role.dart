enum DeviceRole {
  host,
  controller,
  both;

  String get storageValue => name;

  static DeviceRole? fromStorage(String? value) {
    switch (value) {
      case 'host':
        return DeviceRole.host;
      case 'controller':
        return DeviceRole.controller;
      case 'both':
        return DeviceRole.both;
      default:
        return null;
    }
  }

  bool get allowsHost => this == DeviceRole.host || this == DeviceRole.both;
  bool get allowsController =>
      this == DeviceRole.controller || this == DeviceRole.both;

  String get label {
    switch (this) {
      case DeviceRole.host:
        return 'Uzaktan erişilecek';
      case DeviceRole.controller:
        return 'Başka cihaza bağlanacağım';
      case DeviceRole.both:
        return 'İkisi de';
    }
  }

  String get subtitle {
    switch (this) {
      case DeviceRole.host:
        return 'Bu Mac/PC başkaları tarafından kontrol edilebilir';
      case DeviceRole.controller:
        return 'Uzak cihazlara bağlanırsın';
      case DeviceRole.both:
        return 'Hem host hem controller';
    }
  }
}
