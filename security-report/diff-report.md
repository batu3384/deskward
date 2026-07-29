# Security Diff Report

**Branch:** main (uncommitted workspace)  
**Base:** 43a7f0c  
**Date:** 2026-07-29  
**Files Changed:** ~90  
**Files Scanned:** 45 (production Rust/Flutter)

## Summary

| Category | New | Existing | Total |
|----------|-----|----------|-------|
| Critical | 3 | 0 | 3 |
| High | 1 | 0 | 1 |
| Medium | 2 | 0 | 2 |
| Low | 1 | 0 | 1 |

## Verdict

**FAIL → REMEDIATED** — Critical findings fixed in this pass before commit.

## New Findings (Introduced by This Change)

### DIFF-001: Unbounded frame allocation (DoS)
- **Severity:** Critical
- **Classification:** NEW → **FIXED**
- **File:** `deskward-core/src/io_framed.rs`, `protocol.rs`
- **Remediation:** `MAX_FRAME_BYTES` (16 MiB) enforced on read/decode.

### DIFF-002: Session password sent before Noise encryption
- **Severity:** Critical
- **Classification:** NEW → **FIXED**
- **File:** `deskward-core/src/connect.rs`, host `session.rs`
- **Remediation:** Flow reordered: Ed25519 handshake → Noise XX → `SessionAuth` inside AEAD.

### DIFF-003: Admin APIs without authentication
- **Severity:** Critical
- **Classification:** NEW → **FIXED**
- **File:** `deskward-id/src/admin.rs`, `deskward-console/src/main.rs`
- **Remediation:** `DESKWARD_ADMIN_TOKEN` + Bearer middleware; default bind `127.0.0.1`.

### DIFF-004: `extract_parameter_sets` aborts on empty NAL
- **Severity:** Medium
- **Classification:** NEW → **FIXED**
- **File:** `deskward-core/src/features/h264_nal.rs`

### DIFF-005: Console forwards to ID admin without credentials
- **Severity:** High
- **Classification:** NEW → **FIXED**
- **File:** `deskward-console/src/main.rs` — forwards Bearer token to ID admin.

## Dependency Changes

| Package | Change | Risk |
|---------|--------|------|
| videotoolbox, apple-cf | Added (macOS/iOS target) | Low — Apple official bindings |
| openh264, image | Added | Low — source build, no network |
| proptest | dev-dep | None |

## Changed Files Not Scanned

- `docs/**`, `design-system/**` — documentation only
- `Cargo.lock` — lockfile (dependency audit via manifest)

## PR Comment

**PASS ✓** (after fixes)

No remaining Critical/High issues in changed production code. Set `DESKWARD_ADMIN_TOKEN` when exposing admin/console beyond loopback.
