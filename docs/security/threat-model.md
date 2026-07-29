# Deskward Threat Model (Tailscale-first)

**Version:** 0.1 · **Date:** 2026-07-29

## Assets

| Asset | Sensitivity |
|-------|-------------|
| Screen / input stream | Critical |
| Unattended host password (hash only) | Critical |
| Device identity keys (Ed25519) | High |
| Tailscale node identity | High |
| Local audit log | Medium |
| Clipboard / file payloads (Faz 2) | High |

## Adversaries

- **Network attacker** on public Internet — cannot reach host without Tailscale membership.
- **Malicious tailnet peer** — limited by Deskward session password + future E2EE payload.
- **Local malware** on controller or host — OS-level; mitigated by least privilege and permission gates.
- **Credential brute force** — rate-limited unattended password attempts.

## Trust boundaries

```
[Internet] ──X──> [Host public bind]   (no bind on 0.0.0.0 by default)

[Tailscale tailnet / 100.x] ──> [Deskward host listener]
                                      │
                                      ▼
                              [Session auth + E2EE planned]
                                      │
                                      ▼
                              [Screen capture / input inject]
```

1. **Tailscale** — WireGuard mesh; device must be invited to tailnet.
2. **Deskward session** — password verification, handshake signatures, future Noise/AEAD for media.
3. **OS permissions** — Screen Recording + Accessibility required before host listens.

## Mitigations

| Threat | Mitigation |
|--------|------------|
| Silent host exposure | Checklist complete + user "Erişilebilir" arm + permission checks (`may_listen_as_host`) |
| Password leak via logs | Argon2id hash only; no plaintext in logs |
| Unauthorized tailnet peer | Tailscale ACL + Deskward session password |
| MITM on session | Ed25519 handshake; Noise E2EE planned for payload |
| Brute force password | 5 failures → lockout (Faz 1 UI) |
| Clipboard/file exfil | Default off; explicit toggle (Faz 2) |
| Supply chain | Signed releases (planned) |

## Out of scope (this phase)

- LAN mDNS shortcut without Tailscale
- Cloud-hosted Deskward ID/relay (VPS)
- iOS as host

## References

- [Tailscale onboarding spec](../superpowers/specs/2026-07-29-tailscale-onboarding-security-design.md) §6
