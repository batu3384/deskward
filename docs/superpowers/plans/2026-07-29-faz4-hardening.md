# Faz 4 — Üretim sertleştirme

> **Goal:** Fuzz-safe protocol, session perf panel, store release checklist.

## Tasks

### Task 1: Protocol fuzz / property tests

- [ ] `proptest` — `decode_frame` arbitrary bytes never panics

### Task 2: Session perf metrics

- [ ] `deskward-core::perf::SessionMetrics`
- [ ] FFI `deskward_session_metrics`
- [ ] Flutter session overlay (FPS / KB)

### Task 3: Store release checklist

- [ ] `docs/store-release.md` — codesign, notarization, TestFlight

### Success

`cargo test` green; session screen shows live FPS; protocol decode robust on garbage input.
