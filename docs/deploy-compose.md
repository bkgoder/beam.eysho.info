# Docker Compose Deployment

Diese Datei beschreibt den einfachen Compose-Betrieb für `beam.eysho.info`.

## DNS

Vorausgesetzt sind:

- `beam.eysho.info` zeigt auf `217.160.144.62`
- `*.beam.eysho.info` zeigt auf `217.160.144.62`

## Services

Der Compose-Stack startet zwei Services:

- `beam-server` betreibt Control-Port und Admin-API
- `beam-router-ssh` mapped einen öffentlichen TCP-Port auf eine Tunnel-ID

Standard-Ports:

- Control: `8080`
- Admin API: `8081`
- SSH-Router: `2222`

## Vorbereitung

`.env.example` nach `.env` kopieren und bei Bedarf anpassen.

## Start

`docker compose up -d --build`

## Status

`docker compose ps`

## Logs

`docker compose logs -f beam-server`

`docker compose logs -f beam-router-ssh`

## Admin Healthcheck

`curl http://127.0.0.1:8081/health`

## Beispielablauf

1. Server starten.
2. Client registriert lokalen SSH-Port als Tunnel `22-me_up-22`.
3. Router lauscht auf Port `2222`.
4. Externe Verbindung auf Port `2222` wird intern als `CONNECT 22-me_up-22` an den Beam-Server weitergegeben.

## Hinweis zu Subdomains und SSH

Wildcard-DNS ist gesetzt, aber rohes SSH sendet beim Verbindungsaufbau keinen Hostnamen. Für SSH ist deshalb aktuell Port-Mapping sauberer als Subdomain-Routing.

Subdomain-/Host-Routing bleibt sinnvoll für HTTP, HTTPS, WebSocket oder später SNI-basierte Frontends.
