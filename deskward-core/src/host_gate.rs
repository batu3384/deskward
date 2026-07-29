use crate::checklist::{is_setup_complete, CheckId, CheckStatus, SetupSnapshot};
use crate::role::role_allows_host;

pub fn may_listen_as_host(snap: &SetupSnapshot, user_armed: bool) -> bool {
    role_allows_host(snap.role)
        && is_setup_complete(snap)
        && user_armed
        && snap.get(CheckId::ScreenRecording) == CheckStatus::Done
        && snap.get(CheckId::Accessibility) == CheckStatus::Done
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checklist::{required_checks, Platform};
    use crate::role::DeviceRole;

    fn complete_host_snap() -> SetupSnapshot {
        let mut snap = SetupSnapshot::empty(DeviceRole::Host, Platform::MacOs);
        for id in required_checks(DeviceRole::Host, Platform::MacOs) {
            snap.set(id, CheckStatus::Done);
        }
        snap
    }

    #[test]
    fn refuses_when_not_armed() {
        let snap = complete_host_snap();
        assert!(!may_listen_as_host(&snap, false));
    }

    #[test]
    fn allows_when_armed_and_complete() {
        let snap = complete_host_snap();
        assert!(may_listen_as_host(&snap, true));
    }

    #[test]
    fn refuses_without_screen_recording() {
        let mut snap = complete_host_snap();
        snap.set(CheckId::ScreenRecording, CheckStatus::ActionNeeded);
        assert!(!may_listen_as_host(&snap, true));
    }
}
