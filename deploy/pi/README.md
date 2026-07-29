# Deskward on Raspberry Pi

Runs `deskward-id` + `deskward-relay` via Docker.

## Ports

| Port | Service |
|------|---------|
| 29115 TCP | deskward-id |
| 29116 UDP | deskward-id punch (reserved) |
| 29117 TCP | deskward-relay |

## Quick start

```bash
cd deploy/pi
cp .env.example .env
# set RELAY_HOST to Pi hostname or WAN IP
docker compose up -d --build
```

Client → Settings → Network: ID server, relay, and pinned public key from `data/server.pub` (generated on first run in Faz 1).
