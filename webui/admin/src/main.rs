use dioxus::prelude::*;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        main { class: "dashboard admin-dashboard",
            section { class: "hero",
                div {
                    p { class: "kicker", "BKG Beam Admin" }
                    h1 { "Kontrollzentrum" }
                    p { class: "subtitle", "Admin sieht Systemzustand, User, Lizenzen, API-Keys, Tunnels und Router-Mappings. Also alles, was ein Admin sehen muss, weil irgendwer die Menschheit ja beaufsichtigen muss." }
                }
                div { class: "status-pill", "Admin API: /api/admin/overview" }
            }

            section { class: "grid grid-four",
                div { class: "card",
                    div { class: "card-label", "Server" }
                    div { class: "card-value", "beam.eysho.info" }
                }
                div { class: "card",
                    div { class: "card-label", "Control" }
                    div { class: "card-value", "8080" }
                }
                div { class: "card",
                    div { class: "card-label", "Admin API" }
                    div { class: "card-value", "8081" }
                }
                div { class: "card",
                    div { class: "card-label", "Router SSH" }
                    div { class: "card-value", "2222" }
                }
            }

            section { class: "panel",
                div { class: "panel-header",
                    h2 { class: "panel-title", "Admin Aufgaben" }
                    button { class: "refresh-button", "Refresh" }
                }
                div { class: "admin-actions",
                    div { class: "action-card", h3 { "User verwalten" } p { "Accounts, Rollen, aktive Lizenzen und gesperrte Nutzer prüfen." } }
                    div { class: "action-card", h3 { "API-Keys prüfen" } p { "Keys nach Besitzer, Status und Erstellzeit überwachen. Secrets werden nicht offen angezeigt." } }
                    div { class: "action-card", h3 { "Tunnels überwachen" } p { "Pending/Worker-Verbindungen, Tunnel-IDs und Zuordnung sehen." } }
                    div { class: "action-card", h3 { "Router mappen" } p { "Public Ports und später Host/SNI-Routing auf Tunnel-IDs legen." } }
                }
            }

            section { class: "panel",
                div { class: "panel-header",
                    h2 { class: "panel-title", "Systemübersicht" }
                    span { class: "status-pill", "Live-Endpunkte vorbereitet" }
                }
                table { class: "table",
                    thead {
                        tr { th { "Bereich" } th { "Endpoint" } th { "Zweck" } }
                    }
                    tbody {
                        tr { td { "Overview" } td { "/api/admin/overview" } td { "Gesamtbild für Dashboard" } }
                        tr { td { "User" } td { "/api/admin/users" } td { "Alle User und Lizenzen" } }
                        tr { td { "Keys" } td { "/api/admin/api-keys" } td { "Alle API-Keys maskiert" } }
                        tr { td { "Tunnels" } td { "/api/admin/tunnels" } td { "Alle aktiven Tunnels" } }
                        tr { td { "Router" } td { "/api/admin/router-mappings" } td { "Port/Host zu Tunnel-ID" } }
                    }
                }
            }
        }
    }
}
