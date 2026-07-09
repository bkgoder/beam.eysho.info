# beam.eysho.info

`bkg-beam` ist ein kleiner Rust-basierter Reverse-Tunnel-Prototyp. Der Client registriert einen lokalen Dienst beim Beam-Server. Wenn der Server eine eingehende Tunnel-Verbindung bekommt, fordert er beim Client einen Worker an. Dieser Worker verbindet sich lokal zum Ziel-Port und bridged dann beide TCP-Streams.

## DNS

Aktueller Zielstand:

- `beam.eysho.info` zeigt per A-Record auf `217.160.144.62`
- `*.beam.eysho.info` zeigt per Wildcard-A-Record ebenfalls auf `217.160.144.62`

Damit landen Hauptdomain, Admin-Host und spätere Tunnel-Subdomains auf derselben Maschine. DNS löst damit nur auf. Das Routing übernimmt der Beam-Router oder später ein Host/SNI-Router.

## Status

Dieses Repo enthält jetzt nur den Beam-Core:

- `bkg-beam` als Client
- `bkg-beam-server` als Control-/Tunnel-Server
- `bkg-beam-router` als statischen TCP-Port-Router
- User-/Admin-API-Flächen für Lizenzen, API-Keys, Tunnels und Router-Mappings
- Dockerfile und Docker Compose Stack
- zwei WebUI-Sidecars für Admin und Public/User UI

Die WebUIs liegen absichtlich in eigenen Repos:

- `bkgoder/admin.beam.eysho.info` für das Admin-Dashboard
- `bkgoder/public.beam.eysho.info` für das Public/User-Dashboard

Wichtig: Ein normaler SSH-Client sendet keine Subdomain als erste TCP-Zeile. Für rohes SSH ist deshalb aktuell statisches Port-Mapping der robuste Weg, zum Beispiel Public-Port `2222` auf Tunnel `22-me_up-22`.

## Struktur

- `src/client` enthält den Tunnel-Client
- `src/server` enthält den Beam-Control-Server plus Admin/User-API
- `src/router` enthält den statischen TCP-Router
- `docker-compose.yml` startet Server, Router und WebUI-Sidecars
- `docs/deploy-compose.md` beschreibt den Compose-Betrieb

## Docker Compose starten

`.env.example` nach `.env` kopieren und bei Bedarf anpassen.

`docker compose up -d --build`

Status prüfen:

`docker compose ps`

Logs prüfen:

`docker compose logs -f beam-server`

`docker compose logs -f beam-router-ssh`

`docker compose logs -f beam-admin-ui`

`docker compose logs -f beam-public-ui`

Healthcheck:

`curl http://127.0.0.1:8081/health`

Lokale WebUIs:

- Public/User UI: `http://127.0.0.1:3000`
- Admin UI: `http://127.0.0.1:3001`

## Bauen

Workspace bauen:

`cargo build`

Einzeln bauen:

`cargo build -p bkg-beam`

`cargo build -p bkg-beam-server`

`cargo build -p bkg-beam-router`

## Server starten

`RUST_LOG=info cargo run -p bkg-beam-server -- --port 8080 --admin-port 8081`

Der Control-Server lauscht standardmäßig auf `0.0.0.0:8080`.

Die Admin/User-API lauscht standardmäßig auf `0.0.0.0:8081`.

Basis-Endpunkte:

- `/health`
- `/api/tunnels`

User-Endpunkte:

- `POST /api/users/api-keys`
- `GET /api/users/{user_id}/api-keys`
- `GET /api/users/{user_id}/tunnels`

Admin-Endpunkte:

- `GET /api/admin/overview`
- `GET /api/admin/users`
- `GET /api/admin/api-keys`
- `GET /api/admin/tunnels`
- `GET /api/admin/router-mappings`

## Client starten

Beispiel: lokalen SSH-Port `127.0.0.1:22` als Tunnel `22-me_up-22` registrieren:

`RUST_LOG=info cargo run -p bkg-beam -- 22:me up:22 --server beam.eysho.info --server-port 8080`

Lokalen Host ändern:

`RUST_LOG=info cargo run -p bkg-beam -- 3000:me up:3000 --local-host 127.0.0.1 --server beam.eysho.info --server-port 8080`

## Router starten

Beispiel: öffentlicher Port `2222` wird auf Tunnel `22-me_up-22` geroutet:

`RUST_LOG=info cargo run -p bkg-beam-router -- --listen 0.0.0.0:2222 --server 127.0.0.1:8080 --tunnel-id 22-me_up-22`

Danach kann ein Client den öffentlichen Port ansprechen, während `bkg-beam-router` intern `CONNECT 22-me_up-22` an den Beam-Server sendet.

## WebUIs

Admin-Dashboard:

`https://github.com/bkgoder/admin.beam.eysho.info`

Public/User-Dashboard:

`https://github.com/bkgoder/public.beam.eysho.info`

## Protokoll

Client registriert Tunnel:

`REGISTER 22-me_up-22`

Server antwortet:

`OK`

Router oder Test-Client meldet neue öffentliche Verbindung:

`CONNECT 22-me_up-22`

Server legt diese Verbindung in die Pending-Queue und sendet auf der Control-Verbindung an den registrierten Client:

`CONNECT`

Client öffnet Worker-Verbindung:

`WORKER 22-me_up-22`

Danach bridged der Server die Pending-Verbindung mit dem Worker. Der Client bridged den Worker mit dem lokalen Dienst.

## Bekannte Grenzen

- Authentifizierung ist noch nicht hart verdrahtet.
- API-Key- und Lizenzdaten liegen aktuell im Speicher und sind noch nicht persistent.
- TLS fehlt noch.
- Keine Heartbeats/Keepalive-Logik.
- Keine Limits gegen Tunnel-Spam.
- Host-/SNI-Routing für Subdomains ist noch nicht implementiert.
- WebUI-Sidecars installieren im aktuellen Schnitt `dioxus-cli` beim Containerstart; feste Images sind der nächste saubere Schritt.
- Kein systemd-Service.

## Nächste sinnvolle Schritte

1. Feste WebUI-Images für Admin/Public bauen.
2. Persistenz für User, Lizenzen und API-Keys ergänzen.
3. Auth-Token für `REGISTER`, `CONNECT`, `WORKER` und Admin/User-API einführen.
4. Live-Fetching in den externen Admin/Public-WebUIs verdrahten.
5. Host-/SNI-Router für `*.beam.eysho.info` ergänzen.
6. Heartbeats und automatische Cleanup-Logik ergänzen.
7. Integrationstest mit Echo-Server hinzufügen.
8. Release-Build und systemd-Units ergänzen.
