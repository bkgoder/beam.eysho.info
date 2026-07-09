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
- eine erste Dioxus-Web-Admin-UI unter `src/admin-ui`
- eine Admin-HTTP-API am Server
- Dockerfile und Docker Compose Stack

Wichtig: Ein normaler SSH-Client sendet keine Subdomain als erste TCP-Zeile. Für rohes SSH ist deshalb aktuell statisches Port-Mapping der robuste Weg, zum Beispiel Public-Port `2222` auf Tunnel `22-me_up-22`.

## Struktur

- `src/client` enthält den Tunnel-Client
- `src/server` enthält den Beam-Control-Server plus Admin-API
- `src/router` enthält den statischen TCP-Router
- `src/admin-ui` enthält das Dioxus-Web-Admin-Gerüst
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

Die Admin-API lauscht standardmäßig auf `0.0.0.0:8081`.

Admin-Endpunkte:

- `/health`
- `/api/tunnels`

## Client starten

Beispiel: lokalen SSH-Port `127.0.0.1:22` als Tunnel `22-me_up-22` registrieren:

`RUST_LOG=info cargo run -p bkg-beam -- 22:me up:22 --server beam.eysho.info --server-port 8080`

Lokalen Host ändern:

`RUST_LOG=info cargo run -p bkg-beam -- 3000:me up:3000 --local-host 127.0.0.1 --server beam.eysho.info --server-port 8080`

## Router starten

Beispiel: öffentlicher Port `2222` wird auf Tunnel `22-me_up-22` geroutet:

`RUST_LOG=info cargo run -p bkg-beam-router -- --listen 0.0.0.0:2222 --server 127.0.0.1:8080 --tunnel-id 22-me_up-22`

Danach kann ein Client den öffentlichen Port ansprechen, während `bkg-beam-router` intern `CONNECT 22-me_up-22` an den Beam-Server sendet.

## Admin UI starten

Die Admin-UI liegt bewusst außerhalb des Root-Workspace-CI, weil Dioxus-Web-Builds Browser/WASM-Ziele verwenden.

Start aus dem UI-Verzeichnis:

`cd src/admin-ui`

`dx serve`

Die erste UI ist ein Dashboard-Gerüst. Der Server liefert bereits `/health` und `/api/tunnels`. Live-Fetching wird im nächsten Schnitt verdrahtet.

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

- Authentifizierung fehlt noch.
- TLS fehlt noch.
- Keine Heartbeats/Keepalive-Logik.
- Keine Limits gegen Tunnel-Spam.
- Host-/SNI-Routing für Subdomains ist noch nicht implementiert.
- Admin-UI hat aktuell noch statische Beispielwerte.
- Compose startet aktuell Server und einen statischen SSH-Router.
- Kein systemd-Service.

## Nächste sinnvolle Schritte

1. Live-Fetching in der Dioxus-UI gegen `/health` und `/api/tunnels` verdrahten.
2. Auth-Token für `REGISTER`, `CONNECT`, `WORKER` und Admin-API einführen.
3. Host-/SNI-Router für `*.beam.eysho.info` ergänzen.
4. Heartbeats und automatische Cleanup-Logik ergänzen.
5. Integrationstest mit Echo-Server hinzufügen.
6. Release-Build und systemd-Units ergänzen.
