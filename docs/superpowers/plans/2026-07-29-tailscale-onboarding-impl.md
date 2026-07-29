# Tailscale Onboarding + Security — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Role picker, gated setup checklist, Tailscale device discovery, and host listen-only-when-ready — no VPS.

**Architecture:** `deskward-core` owns role/checklist/password/tailscale traits + pure logic; platform adapters stub Tailscale LocalAPI and macOS permissions; Flutter `deskward-app` shows role → checklist → locked Home until all required items green.

**Tech Stack:** Rust 1.75+, tokio, argon2, Flutter Material 3, design tokens from `design-system/deskward/`.

**Spec:** `docs/superpowers/specs/2026-07-29-tailscale-onboarding-security-design.md`

## Global Constraints

- VPS / Deskward Cloud yok; uzak erişim Tailscale-first.
- iOS host değil; host kartı disabled.
- Host: Screen Recording + Accessibility yoksa dinleme yok.
- Şifre: Argon2id hash only; plaintext log yok.
- Bind sadece Tailscale `100.x` (LAN shortcut bu planda yok).
- UI: `design-system/deskward/pages/onboarding.md` + MASTER.
- MIT; RustDesk kaynak kopyalanmaz.

## File map

| Path | Responsibility |
|------|----------------|
| `deskward-core/src/role.rs` | DeviceRole enum + persistence helpers |
| `deskward-core/src/checklist.rs` | ChecklistItem, evaluate required/complete |
| `deskward-core/src/auth/password.rs` | Argon2id set/verify |
| `deskward-core/src/tailscale/mod.rs` | TailscaleStatus, Peer, TailscaleClient trait |
| `deskward-core/src/tailscale/mock.rs` | In-memory mock for tests |
| `deskward-core/src/host_gate.rs` | `may_listen_as_host(...)` |
| `deskward-app/lib/models/...` | Dart mirrors / JSON from FFI later; for now pure Dart state mirroring core types |
| `deskward-app/lib/screens/role_picker_screen.dart` | Role UI |
| `deskward-app/lib/screens/checklist_screen.dart` | Checklist UI |
| `deskward-app/lib/screens/home_screen.dart` | Gated home + device list |
| `deskward-app/lib/state/setup_controller.dart` | Local prefs + poll Tailscale stub |
| `docs/security/threat-model.md` | Short threat model |

---

### Task 1: DeviceRole + prefs shape

**Files:**
- Create: `deskward-core/src/role.rs`
- Modify: `deskward-core/src/lib.rs`
- Test: `deskward-core/src/role.rs` (unit tests inline)

**Interfaces:**
- Produces: `DeviceRole::{Host, Controller, Both}`, `role_allows_host(role) -> bool`, `role_allows_controller(role) -> bool`

- [ ] **Step 1: Write failing tests**

```rust
#[test]
fn ios_must_not_use_host_only_helpers() {
    assert!(!role_allows_host(DeviceRole::Controller));
    assert!(role_allows_host(DeviceRole::Host));
    assert!(role_allows_host(DeviceRole::Both));
    assert!(role_allows_controller(DeviceRole::Controller));
    assert!(role_allows_controller(DeviceRole::Both));
    assert!(!role_allows_controller(DeviceRole::Host));
}
```

- [ ] **Step 2: Run test — expect FAIL**

Run: `cargo test -p deskward-core role -- --nocapture`

- [ ] **Step 3: Implement `role.rs` and export from `lib.rs`**

```rust
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
```

- [ ] **Step 4: Run tests — PASS**

- [ ] **Step 5: Commit**

```bash
git add deskward-core/src/role.rs deskward-core/src/lib.rs
git commit -m "feat(core): add DeviceRole host/controller helpers"
```

---

### Task 2: Checklist engine

**Files:**
- Create: `deskward-core/src/checklist.rs`
- Modify: `deskward-core/src/lib.rs`
- Test: inline in `checklist.rs`

**Interfaces:**
- Consumes: `DeviceRole`, `role_allows_*`
- Produces: `CheckId`, `CheckStatus`, `CheckItem`, `SetupSnapshot`, `required_checks(role, platform) -> Vec<CheckId>`, `is_setup_complete(snapshot) -> bool`

- [ ] **Step 1: Failing test — host incomplete blocks**

```rust
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
```

- [ ] **Step 2: Run — FAIL**

Run: `cargo test -p deskward-core checklist`

- [ ] **Step 3: Implement checklist types**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform { MacOs, Windows, Ios }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckId {
    TailscaleInstalled,
    TailscaleRunning,
    TailscaleSelfVisible,
    ScreenRecording,
    Accessibility,
    LaunchAtLogin,       // optional — not required for is_setup_complete
    UnattendedPassword,
    HostListeningArmed,
    PeerVisible,         // controller
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus { Pending, ActionNeeded, Done }

pub struct SetupSnapshot { /* role, platform, map CheckId -> Status */ }

pub fn required_checks(role: DeviceRole, platform: Platform) -> Vec<CheckId> { /* per spec H1-H5,H7,H8 / C1-C3 */ }

pub fn is_setup_complete(snap: &SetupSnapshot) -> bool {
    required_checks(snap.role, snap.platform)
        .into_iter()
        .all(|id| snap.get(id) == CheckStatus::Done)
}
```

Required Mac host: TailscaleInstalled, TailscaleRunning, TailscaleSelfVisible, ScreenRecording, Accessibility, UnattendedPassword, HostListeningArmed.  
Required controller: TailscaleInstalled, TailscaleRunning, PeerVisible.  
`Both` = union.  
`LaunchAtLogin` optional (never blocks Home).

- [ ] **Step 4: Tests PASS**

- [ ] **Step 5: Commit**

```bash
git commit -am "feat(core): setup checklist engine with required gates"
```

---

### Task 3: Password Argon2id

**Files:**
- Create: `deskward-core/src/auth/mod.rs`, `deskward-core/src/auth/password.rs`
- Modify: `deskward-core/Cargo.toml` (add `argon2`, `password-hash`)
- Test: `password.rs` tests

**Interfaces:**
- Produces: `PasswordHash(String)`, `hash_password(plain: &str) -> Result<PasswordHash>`, `verify_password(plain, hash) -> Result<bool>`, `password_meets_policy(plain) -> bool` (len >= 12)

- [ ] **Step 1: Failing tests**

```rust
#[test]
fn rejects_short_password() {
    assert!(!password_meets_policy("short"));
}

#[test]
fn hash_verify_roundtrip() {
    let h = hash_password("correct horse battery staple").unwrap();
    assert!(verify_password("correct horse battery staple", &h).unwrap());
    assert!(!verify_password("wrong password!!", &h).unwrap());
}
```

- [ ] **Step 2: FAIL then implement with argon2 defaults (OsRng salt)**

- [ ] **Step 3: PASS + commit**

```bash
git commit -am "feat(core): Argon2id unattended password hash/verify"
```

---

### Task 4: Tailscale trait + mock

**Files:**
- Create: `deskward-core/src/tailscale/mod.rs`, `mock.rs`
- Modify: `lib.rs`

**Interfaces:**
- Produces:

```rust
pub struct TailscalePeer {
    pub name: String,       // MagicDNS / HostName
    pub ipv4: String,       // 100.x.x.x
    pub online: bool,
    pub os: String,
}

pub struct TailscaleStatus {
    pub installed: bool,
    pub running: bool,
    pub self_name: Option<String>,
    pub self_ipv4: Option<String>,
    pub peers: Vec<TailscalePeer>,
}

#[async_trait::async_trait]
pub trait TailscaleClient: Send + Sync {
    async fn status(&self) -> crate::Result<TailscaleStatus>;
}
```

- Mock: `MockTailscale` with setters for tests.
- Test: mock returns peers → checklist PeerVisible Done when `peers.iter().any(|p| p.online)`.

- [ ] Steps: fail test → implement → pass → commit `feat(core): TailscaleClient trait and mock`

---

### Task 5: Host listen gate

**Files:**
- Create: `deskward-core/src/host_gate.rs`
- Test: inline

**Interfaces:**
- Consumes: `SetupSnapshot`, `is_setup_complete`, `role_allows_host`
- Produces: `may_listen_as_host(snap: &SetupSnapshot, user_armed: bool) -> bool`

```rust
pub fn may_listen_as_host(snap: &SetupSnapshot, user_armed: bool) -> bool {
    role_allows_host(snap.role)
        && is_setup_complete(snap)
        && user_armed
        && snap.get(CheckId::ScreenRecording) == CheckStatus::Done
        && snap.get(CheckId::Accessibility) == CheckStatus::Done
}
```

- [ ] Test: complete snapshot but `user_armed=false` → false; armed true → true; missing screen → false even if other Done wrongly set.
- [ ] Commit: `feat(core): host listen gate requires checklist and arm`

---

### Task 6: Threat model doc

**Files:**
- Create: `docs/security/threat-model.md`

- [ ] Write short sections: assets, adversaries, trust boundaries (Tailscale, Deskward session), mitigations matching spec §6.
- [ ] Commit: `docs: add Deskward threat model (Tailscale-first)`

---

### Task 7: Flutter role picker

**Files:**
- Create: `deskward-app/lib/models/device_role.dart`
- Create: `deskward-app/lib/screens/role_picker_screen.dart`
- Modify: `deskward-app/lib/main.dart` — start at RolePicker if no role in prefs
- Create: `deskward-app/lib/state/setup_controller.dart` (shared_preferences)

**UI:** Spec §3 + onboarding.md — three full-width rows; on iOS disable Host with subtitle.

- [ ] Persist role string `host|controller|both`.
- [ ] After pick → navigate ChecklistScreen.
- [ ] Manual verify: cold start shows picker.
- [ ] Commit: `feat(app): role picker with iOS host disabled`

Note: Flutter SDK may be missing locally — still land Dart files; verify with `dart analyze` if available else document.

---

### Task 8: Flutter checklist screen

**Files:**
- Create: `deskward-app/lib/models/check_item.dart`
- Create: `deskward-app/lib/screens/checklist_screen.dart`
- Modify: `setup_controller.dart` — poll every 2s: Tailscale mock/status + permission stubs

**UI:** Progress `3/7`, rows with status + CTA. Home route blocked until `isSetupComplete`.

- [ ] Host path shows screen/accessibility rows; controller path does not.
- [ ] CTA buttons: open URL `https://tailscale.com/download` / deep-link placeholders for macOS Settings.
- [ ] Commit: `feat(app): gated setup checklist UI`

---

### Task 9: Flutter Home gated + device list

**Files:**
- Modify: `deskward-app/lib/screens/home_screen.dart`
- Create: `deskward-app/lib/widgets/device_tile.dart`

- [ ] If `!isSetupComplete` → redirect Checklist.
- [ ] List peers from SetupController (mock peers until FFI).
- [ ] Status chip: Tailscale running / offline.
- [ ] Host: “Erişilebilir” switch → `user_armed` (calls same semantics as `may_listen_as_host` when FFI lands).
- [ ] Commit: `feat(app): home device list gated on setup complete`

---

### Task 10: Wire host-macos agent to gate (stub)

**Files:**
- Modify: `deskward-host-macos/src/main.rs`, add `gate.rs` reading env/flags for now
- Or: print and refuse listen if `DESKWARD_FORCE_LISTEN` not set and gate false

- [ ] Default: agent exits/refuses bind unless checklist JSON file marks complete + armed (simple file `~/.deskward/setup.json` for Phase 1 bridge).
- [ ] Test: unit test gate still in core; agent integration manual.
- [ ] Commit: `feat(host-macos): refuse listen unless setup gate open`

---

### Task 11: Spec status + README pointer

**Files:**
- Modify: `docs/superpowers/specs/2026-07-29-tailscale-onboarding-security-design.md` Status → Approved
- Modify: `README.md` — link onboarding spec + threat model

- [ ] Commit: `docs: mark onboarding spec approved; link security docs`

---

## Spec coverage checklist

| Spec § | Task |
|--------|------|
| Role picker | 1, 7 |
| Checklist rules | 2, 8 |
| Tailscale LocalAPI | 4, 8–9 (mock → real later) |
| Host permissions gate | 5, 10 |
| Password | 3, 8 (UI set password) |
| Ultra security / threat model | 3, 5, 6 |
| Home list | 9 |
| iOS no host | 1, 7 |
| VPS yok | Global |

**Deferred (not this plan):** real ScreenCaptureKit, Noise E2EE payload, Win host, LAN mDNS, FFI bridge core↔Flutter (mock status until Task series 2).

---

## Self-review notes

- No TBD placeholders in task steps.
- Types consistent: `DeviceRole`, `CheckId`, `SetupSnapshot`, `TailscaleClient`, `may_listen_as_host`.
- Real LocalAPI OS wiring = follow-up plan after this lands with mocks.
