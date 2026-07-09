# Docker Compose Deployment

Diese Datei beschreibt den Compose-Betrieb für `beam.eysho.info`.

## DNS

Vorausgesetzt sind:

- `beam.eysho.info` zeigt auf `217.160.144.62`
- `*.beam.eysho.info` zeigt auf `217.160.144.62`

## Repo-Layout

Die Standard-Annahme ist, dass diese Repos nebeneinander liegen:

- `beam.eysho.info`
- `router.beam.eysho.info`
- `admin.beam.eysho.info`
- `public.beam.eysho.info`

Der Router ist kein Core-Crate mehr. Er wird als externer Sidecar aus `../router.beam.eysho.info` gebaut. Falls der Pfad anders liegt, `BEAM_ROUTER_CONTEXT` in `.env` anpassen.

## Services

Der Compose-Stack startet vier Services:

- `beam-server` betreibt Control-Port und Admin/User-API
- `beam-router-ssh` ist der Router-Sidecar aus `bkgoder/router.beam.eysho.info`
- `beam-admin-ui` startet das Admin-Dashboard aus `bkgoder/admin.beam.eysho.info`
- `beam-public-ui` startet das Public/User-Dashboard aus `bkgoder/public.beam.eysho.info`

Damit läuft Beam als Core plus drei Sidecars: Router, Admin UI und Public UI.

Standard-Ports:

- Control: `8080`
- Admin/User API: `8081`
- SSH-Router: `2222`
- Public/User UI: `3000`
- Admin UI: `3001`

## Vorbereitung

`.env.example` nach `.env` kopieren und bei Bedarf anpassen.

Wichtige Router-Variablen:

- `BEAM_ROUTER_CONTEXT=../router.beam.eysho.info`
- `BEAM_ROUTER_IMAGE=router-beam-eysho-info:local`
- `BEAM_ROUTER_SSH_PORT=2222`
- `BEAM_ROUTER_SSH_TUNNEL_ID=22-me_up-22`

## Start

`docker compose up -d --build`

## Status

`docker compose ps`

## Logs

`docker compose logs -f beam-server`

`docker compose logs -f beam-router-ssh`

`docker compose logs -f beam-admin-ui`

`docker compose logs -f beam-public-ui`

## Admin/User API Healthcheck

`curl http://127.0.0.1:8081/health`

## WebUIs

Lokal:

- Public/User UI: `http://127.0.0.1:3000`
- Admin UI: `http://127.0.0.1:3001`

Die WebUI-Sidecars installieren im aktuellen Schnitt `dioxus-cli 0.7.9` im Container und starten dann `dx serve`. Das ist bewusst als schneller Sidecar-Schnitt gedacht. Für Produktion sollte daraus später ein festes WebUI-Image gebaut werden, damit Container nicht bei jedem Start Rust und Dioxus nachziehen müssen.

## Beispielablauf

1. Server starten.
2. Client registriert lokalen SSH-Port als Tunnel `22-me_up-22`.
3. Router-Sidecar lauscht auf Port `2222`.
4. Externe Verbindung auf Port `2222` wird intern als `CONNECT 22-me_up-22` an den Beam-Server weitergegeben.
5. Public/Admin UI sprechen gegen die API auf `beam-server:8081`.

## Hinweis zu Subdomains und SSH

Wildcard-DNS ist gesetzt, aber rohes SSH sendet beim Verbindungsaufbau keinen Hostnamen. Für SSH ist deshalb aktuell Port-Mapping sauberer als Subdomain-Routing.

Subdomain-/Host-Routing bleibt sinnvoll für HTTP, HTTPS, WebSocket oder später SNI-basierte Frontends.
