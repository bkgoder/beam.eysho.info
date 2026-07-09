use dioxus::prelude::*;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        main { class: "dashboard user-dashboard",
            section { class: "hero",
                div {
                    p { class: "kicker", "BKG Beam User" }
                    h1 { "Tunnel Dashboard" }
                    p { class: "subtitle", "User erstellen ihre Lizenz und ihren API-Key, sehen eigene Tunnels und behalten ihre Zugänge im Blick. Keine Admin-Schaltflächen, kein Root-Spielplatz, keine Menschen mit Schraubenzieher im Reaktorraum." }
                }
                div { class: "status-pill", "User API: /api/users" }
            }

            section { class: "grid",
                div { class: "card",
                    div { class: "card-label", "User" }
                    div { class: "card-value", "demo-user" }
                }
                div { class: "card",
                    div { class: "card-label", "Lizenz" }
                    div { class: "card-value", "free" }
                }
                div { class: "card",
                    div { class: "card-label", "Default Tunnel" }
                    div { class: "card-value", "22" }
                }
            }

            section { class: "panel split-panel",
                div {
                    h2 { class: "panel-title", "Lizenz / API-Key erstellen" }
                    p { class: "muted", "Erstellt serverseitig einen User-Key. Das Secret wird nur einmal zurückgegeben, danach nur noch maskiert angezeigt." }
                }
                form { class: "key-form",
                    label { "Label" input { value: "ssh-main" } }
                    label { "Plan" input { value: "free" } }
                    button { class: "primary-button", r#type: "button", "API-Key erstellen" }
                }
            }

            section { class: "panel",
                div { class: "panel-header",
                    h2 { class: "panel-title", "Meine Tunnels" }
                    span { class: "status-pill", "/api/users/demo-user/tunnels" }
                }
                table { class: "table",
                    thead {
                        tr { th { "Tunnel ID" } th { "Pending" } th { "Worker" } th { "Status" } }
                    }
                    tbody {
                        tr { td { "22-me_up-22" } td { "0" } td { "0" } td { "bereit" } }
                    }
                }
            }

            section { class: "panel",
                div { class: "panel-header",
                    h2 { class: "panel-title", "Meine API-Keys" }
                    span { class: "status-pill", "/api/users/demo-user/api-keys" }
                }
                table { class: "table",
                    thead {
                        tr { th { "Key" } th { "Label" } th { "Status" } th { "Erstellt" } }
                    }
                    tbody {
                        tr { td { "beam_demou…" } td { "ssh-main" } td { "active" } td { "nach Erstellung" } }
                    }
                }
            }
        }
    }
}
