use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::role::{role_allows_controller, role_allows_host, DeviceRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    MacOs,
    Windows,
    Ios,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckId {
    TailscaleInstalled,
    TailscaleRunning,
    TailscaleSelfVisible,
    ScreenRecording,
    Accessibility,
    LaunchAtLogin,
    UnattendedPassword,
    HostListeningArmed,
    PeerVisible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pending,
    ActionNeeded,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupSnapshot {
    pub role: DeviceRole,
    pub platform: Platform,
    checks: HashMap<CheckId, CheckStatus>,
}

impl SetupSnapshot {
    pub fn empty(role: DeviceRole, platform: Platform) -> Self {
        Self {
            role,
            platform,
            checks: HashMap::new(),
        }
    }

    pub fn set(&mut self, id: CheckId, status: CheckStatus) {
        self.checks.insert(id, status);
    }

    pub fn get(&self, id: CheckId) -> CheckStatus {
        self.checks.get(&id).copied().unwrap_or(CheckStatus::Pending)
    }
}

fn host_checks(platform: Platform) -> Vec<CheckId> {
    match platform {
        Platform::MacOs | Platform::Windows => vec![
            CheckId::TailscaleInstalled,
            CheckId::TailscaleRunning,
            CheckId::TailscaleSelfVisible,
            CheckId::ScreenRecording,
            CheckId::Accessibility,
            CheckId::UnattendedPassword,
            CheckId::HostListeningArmed,
        ],
        Platform::Ios => vec![],
    }
}

fn controller_checks(_platform: Platform) -> Vec<CheckId> {
    vec![
        CheckId::TailscaleInstalled,
        CheckId::TailscaleRunning,
        CheckId::PeerVisible,
    ]
}

pub fn required_checks(role: DeviceRole, platform: Platform) -> Vec<CheckId> {
    let mut ids = Vec::new();
    if role_allows_host(role) {
        ids.extend(host_checks(platform));
    }
    if role_allows_controller(role) {
        for id in controller_checks(platform) {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    ids
}

pub fn is_setup_complete(snap: &SetupSnapshot) -> bool {
    required_checks(snap.role, snap.platform)
        .into_iter()
        .all(|id| snap.get(id) == CheckStatus::Done)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_incomplete_without_screen_perm() {
        let mut snap = SetupSnapshot::empty(DeviceRole::Host, Platform::MacOs);
        snap.set(CheckId::TailscaleRunning, CheckStatus::Done);
        snap.set(CheckId::ScreenRecording, CheckStatus::ActionNeeded);
        snap.set(CheckId::Accessibility, CheckStatus::Done);
        snap.set(CheckId::UnattendedPassword, CheckStatus::Done);
        assert!(!is_setup_complete(&snap));
    }

    #[test]
    fn host_complete_when_all_required_done() {
        let mut snap = SetupSnapshot::empty(DeviceRole::Host, Platform::MacOs);
        for id in required_checks(DeviceRole::Host, Platform::MacOs) {
            snap.set(id, CheckStatus::Done);
        }
        assert!(is_setup_complete(&snap));
    }

    #[test]
    fn ios_controller_has_no_host_checks() {
        let ids = required_checks(DeviceRole::Controller, Platform::Ios);
        assert!(!ids.contains(&CheckId::ScreenRecording));
    }

    #[test]
    fn launch_at_login_never_blocks() {
        let mut snap = SetupSnapshot::empty(DeviceRole::Host, Platform::MacOs);
        for id in required_checks(DeviceRole::Host, Platform::MacOs) {
            snap.set(id, CheckStatus::Done);
        }
        snap.set(CheckId::LaunchAtLogin, CheckStatus::Pending);
        assert!(is_setup_complete(&snap));
    }
}
