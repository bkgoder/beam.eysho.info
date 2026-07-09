# beam.eysho.info

`bkg-beam` ist ein kleiner Rust-basierter Reverse-Tunnel-Prototyp. Der Client registriert einen lokalen Dienst beim Beam-Server. Wenn der Server eine eingehende Tunnel-Verbindung bekommt, fordert er beim Client einen Worker an. Dieser Worker verbindet sich lokal zum Ziel-Port und bridged dann beide TCP-Streams.

## DNS

Aktueller Zielstand:

- `beam.eysho.info` zeigt per A-Record auf `217.160.144.62`
- `*.beam.eysho.info` zeigt per Wildcard-A-Record ebenfalls auf `217.160.144.62`

Damit landen Hauptdomain, Admin-Host und spätere Tunnel-Subdomains auf derselben Maschine. DNS löst damit nur auf. Das Routing übernimmt der Beam-Router oder später ein Host/SNI-Router.

## Status

Dieses Repo enthält jetzt:

- `bkg-beam` als Client
- `bkg-beam-server` als Control-/Tunnel-Server
- `bkg-beam-router` als statischen TCP-Port-Router
- `webui/admin` als Admin-Dashboard
- `webui/users` als User-Dashboard
- User-/Admin-API-Flächen für Lizenzen, API-Keys, Tunnels und Router-Mappings
- Dockerfile und Docker Compose Stack

Wichtig: Ein normaler SSH-Client sendet keine Subdomain als erste TCP-Zeile. Für rohes SSH ist deshalb aktuell statisches Port-Mapping der robuste Weg, zum Beispiel Public-Port `2222` auf Tunnel `22-me_up-22`.

## Struktur

- `src/client` enthält den Tunnel-Client
- `src/server` enthält den Beam-Control-Server plus Admin/User-API
- `src/router` enthält den statischen TCP-Router
- `webui/admin` enthält das Admin-Dashboard
- `webui/users` enthält das User-Dashboard
- `docker-compose.yml` startet Server und Router
- `docs/deploy-compose.md` beschreibt den Compose-Betrieb

## Docker Compose starten

`.env.example` nach `.env` kopieren und bei Bedarf anpassen.

`docker compose up -d --build`

Status prüfen:

`docker compose ps`

Logs prüfen:

`docker compose logs -f beam-server`

`docker compose logs -f beam-router-ssh`

Healthcheck:

`curl http://127.0.0.1:8081/health`

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
- `GET /api/users/:user_id/api-keys`
- `GET /api/users/:user_id/tunnels`

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

## WebUI starten

Admin-Dashboard:

`cd webui/admin`

`dx serve`

User-Dashboard:

`cd webui/users`

`dx serve`

Die Dashboards liegen bewusst außerhalb des Root-Workspace-CI, weil Dioxus-Web-Builds Browser/WASM-Ziele verwenden.

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
- Dashboards haben erste Oberflächen und API-Zielpunkte, Live-Fetching kommt als nächster Schnitt.
- Compose startet aktuell Server und einen statischen SSH-Router.
- Kein systemd-Service.

## Nächste sinnvolle Schritte

1. Live-Fetching in Admin/User-Dashboard gegen die neuen API-Flächen verdrahten.
2. Persistenz für User, Lizenzen und API-Keys ergänzen.
3. Auth-Token für `REGISTER`, `CONNECT`, `WORKER` und Admin/User-API einführen.
4. Host-/SNI-Router für `*.beam.eysho.info` ergänzen.
5. Heartbeats und automatische Cleanup-Logik ergänzen.
6. Integrationstest mit Echo-Server hinzufügen.
7. Release-Build und systemd-Units ergänzen.
