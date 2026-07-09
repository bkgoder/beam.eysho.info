# Beam WebUI

Die WebUI liegt absichtlich im Root-Verzeichnis unter `webui/` und nicht mehr unter `src/`.

## Bereiche

- `webui/admin` ist das Admin-Dashboard
- `webui/users` ist das User-Dashboard

## User Dashboard

User sollen dort:

- eine Lizenz anlegen beziehungsweise aktivieren
- einen API-Key erstellen
- eigene API-Keys sehen
- eigene Tunnels sehen

Zugehörige API-Flächen:

- `POST /api/users/api-keys`
- `GET /api/users/:user_id/api-keys`
- `GET /api/users/:user_id/tunnels`

## Admin Dashboard

Admin sieht und verwaltet:

- Systemzustand
- User
- Lizenzen
- API-Keys
- alle Tunnels
- Router-Mappings

Zugehörige API-Flächen:

- `GET /api/admin/overview`
- `GET /api/admin/users`
- `GET /api/admin/api-keys`
- `GET /api/admin/tunnels`
- `GET /api/admin/router-mappings`

## Start

Admin:

`cd webui/admin`

`dx serve`

User:

`cd webui/users`

`dx serve`

## Hinweis

Die Dashboards sind bewusst nicht Teil des Root-Cargo-Workspaces. Der Root-Workspace baut native Rust-Services. Die WebUIs sind Dioxus-Web-Projekte und werden separat gebaut.
