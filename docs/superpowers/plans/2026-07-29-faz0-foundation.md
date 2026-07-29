# Faz 0 — Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Monorepo iskeleti, protokol v0, deskward-id + deskward-relay, lokal handshake doğrulaması.

**Architecture:** Rust workspace; `deskward-core` paylaşılan protokol/crypto; id ve relay ayrı binary; integration test iki mock client handshake.

**Tech Stack:** Rust 1.75+, tokio, serde_json, ed25519-dalek, tracing.

## Global Constraints

- MIT license; RustDesk kaynak kopyalanmaz.
- Portlar: id TCP 29115, UDP 29116; relay TCP 29117.
- Protokol v0: length-prefixed JSON.
- Deskward isimlendirme; `hbbs`/`hbbr` yok.

---

### Task 1: Workspace + deskward-core protocol

**Files:**
- Create: `Cargo.toml`, `deskward-core/Cargo.toml`, `deskward-core/src/lib.rs`, `protocol.rs`, `error.rs`, `crypto.rs`

**Interfaces:**
- Produces: `deskward_core::protocol::{Message, encode_frame, decode_frame}`
- Produces: `deskward_core::crypto::{Identity, sign_handshake, verify_handshake}`

- [x] Protocol enum + frame codec
- [x] Ed25519 identity + handshake helpers
- [x] Unit test encode/decode roundtrip

### Task 2: deskward-id server

**Files:**
- Create: `deskward-id/Cargo.toml`, `deskward-id/src/main.rs`

- [x] TCP register/heartbeat/punch
- [x] In-memory peer registry

### Task 3: deskward-relay server

**Files:**
- Create: `deskward-relay/Cargo.toml`, `deskward-relay/src/main.rs`

- [x] Session allocate + byte pipe between two TCP legs

### Task 4: Integration handshake test

**Files:**
- Create: `deskward-core/tests/handshake.rs`

- [x] Two Identity handshake Hello/Ack verify

### Task 5: Deploy + docs

**Files:**
- Create: `deploy/pi/docker-compose.yml`, `deploy/pi/.env.example`, `deploy/pi/README.md`, `README.md`

- [x] Docker compose id + relay
- [x] Root README
