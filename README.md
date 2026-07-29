# Deskward

Self-hosted remote desktop platform. Rust core + Flutter UI.

## Workspace

| Path | Role |
|------|------|
| `deskward-core` | Protocol, crypto, session, media/input traits |
| `deskward-id` | ID / rendezvous server (TCP 29115) |
| `deskward-relay` | Relay fallback (TCP 29117) |
| `deskward-app` | Flutter client (Win / macOS / iOS) |
| `deskward-host-macos` | macOS host agent (Faz 1) |
| `deskward-host-windows` | Windows host agent (Faz 2) |
| `deskward-console` | Admin API stub (Faz 3) |
| `deploy/pi` | Docker Compose for Pi |

## Quick start

```bash
# Build
cargo test
cargo build --release

# Run servers (two terminals)
./target/release/deskward-id
./target/release/deskward-relay
```

## Docs

- [Platform design](docs/superpowers/specs/2026-07-29-deskward-platform-design.md)
- [Faz 0 plan](docs/superpowers/plans/2026-07-29-faz0-foundation.md)
- [Faz 1 plan](docs/superpowers/plans/2026-07-29-faz1-personal.md)
- [Design system](design-system/deskward/MASTER.md)

## License

MIT
