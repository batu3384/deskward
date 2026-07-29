#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceRole {
    Host,
    Controller,
    Both,
}

pub fn role_allows_host(role: DeviceRole) -> bool {
    matches!(role, DeviceRole::Host | DeviceRole::Both)
}

pub fn role_allows_controller(role: DeviceRole) -> bool {
    matches!(role, DeviceRole::Controller | DeviceRole::Both)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ios_must_not_use_host_only_helpers() {
        assert!(!role_allows_host(DeviceRole::Controller));
        assert!(role_allows_host(DeviceRole::Host));
        assert!(role_allows_host(DeviceRole::Both));
        assert!(role_allows_controller(DeviceRole::Controller));
        assert!(role_allows_controller(DeviceRole::Both));
        assert!(!role_allows_controller(DeviceRole::Host));
    }
}
