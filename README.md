# beam.eysho.info

`bkg-beam` ist ein kleiner Rust-basierter Reverse-Tunnel-Prototyp. Der Client registriert einen lokalen Dienst beim Beam-Server. Wenn der Server eine eingehende Tunnel-Verbindung bekommt, fordert er beim Client einen Worker an. Dieser Worker verbindet sich lokal zum Ziel-Port und bridged dann beide TCP-Streams.

## Status

Dieses Repo enthält jetzt eine funktionale Protokoll-Grundlage, aber noch keinen vollständigen öffentlichen Hostname-Router für rohe TCP-Protokolle wie SSH.

Wichtig: Ein normaler SSH-Client sendet keine Subdomain als erste TCP-Zeile. Für `22-me.up-22.beam.eysho.info` braucht es daher zusätzlich einen vorgeschalteten Router/Proxy, der Host/Port/SNI/HTTP-Host auswertet und daraus intern `CONNECT <tunnel-id>` an `bkg-beam-server` schreibt. Ohne diesen Router kann der Server nicht magisch wissen, welcher Tunnel gemeint ist. Leider hält TCP nichts von Gedankenlesen.

## Struktur

```text
.
├── Cargo.toml
├── src/
│   ├── client/
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   └── server/
│       ├── Cargo.toml
│       └── src/main.rs
└── README.md
```

## Bauen

```bash
cargo build
```

Oder einzeln:

```bash
cargo build -p bkg-beam
cargo build -p bkg-beam-server
```

## Server starten

```bash
RUST_LOG=info cargo run -p bkg-beam-server -- --port 8080
```

Der Server lauscht standardmäßig auf `0.0.0.0:8080`.

## Client starten

Beispiel: lokalen SSH-Port `127.0.0.1:22` als Tunnel `22-me_up-22` registrieren:

```bash
RUST_LOG=info cargo run -p bkg-beam -- 22:me up:22 --server beam.eysho.info --server-port 8080
```

Lokalen Host ändern:

```bash
RUST_LOG=info cargo run -p bkg-beam -- 3000:me up:3000 --local-host 127.0.0.1 --server beam.eysho.info --server-port 8080
```

## Protokoll

### 1. Client registriert Tunnel

```text
REGISTER 22-me_up-22
```

Server antwortet:

```text
OK
```

### 2. Öffentliche Seite meldet neue Verbindung

Ein vorgeschalteter Router oder Test-Client sendet:

```text
CONNECT 22-me_up-22
```

Der Server legt diese Verbindung in die Pending-Queue und sendet auf der Control-Verbindung an den registrierten Client:

```text
CONNECT
```

### 3. Client öffnet Worker-Verbindung

Der Client verbindet sich erneut zum Server und sendet:

```text
WORKER 22-me_up-22
```

Server antwortet:

```text
OK
```

Danach bridged der Server:

```text
pending public connection <-> worker connection <-> local service
```

Der Client bridged parallel:

```text
worker connection <-> 127.0.0.1:<local_port>
```

## Bekannte Grenzen

- Noch kein DNS-/SNI-/HTTP-Host-Router enthalten.
- Rohes SSH über Subdomain braucht einen separaten TCP-Router oder ein anderes Mapping-Modell.
- Authentifizierung fehlt noch.
- TLS fehlt noch.
- Keine Heartbeats/Keepalive-Logik.
- Keine Limits gegen Tunnel-Spam.
- Kein systemd-Service, kein Deployment-Packaging.

## Nächste sinnvolle Schritte

1. Auth-Token für `REGISTER`, `CONNECT` und `WORKER` einführen.
2. Front-Router bauen, der `*.beam.eysho.info` auf Tunnel-IDs mappt.
3. Heartbeats und automatische Cleanup-Logik ergänzen.
4. Integrationstest mit Echo-Server hinzufügen.
5. Release-Build und systemd-Units ergänzen.
