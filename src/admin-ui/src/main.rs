use dioxus::prelude::*;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        main { class: "dashboard",
            section { class: "hero",
                div {
                    p { class: "kicker", "BKG Beam" }
                    h1 { "Admin UI" }
                    p { class: "subtitle", "Tunnel-Status, Router-Mapping und Server-Gesundheit an einem Ort. Weil Debugging per Bauchgefühl zwar romantisch ist, aber trotzdem Unsinn." }
                }
                div { class: "status-pill", "API: /health und /api/tunnels" }
            }
            section { class: "grid",
                div { class: "card",
                    div { class: "card-label", "Server" }
                    div { class: "card-value", "beam.eysho.info" }
                }
                div { class: "card",
                    div { class: "card-label", "Control Port" }
                    div { class: "card-value", "8080" }
                }
                div { class: "card",
                    div { class: "card-label", "Admin API" }
                    div { class: "card-value", "8081" }
                }
            }
            section { class: "panel",
                div { class: "panel-header",
                    h2 { class: "panel-title", "Tunnel" }
                    button { class: "refresh-button", "Refresh" }
                }
                table { class: "table",
                    thead {
                        tr {
                            th { "Tunnel ID" }
                            th { "Pending" }
                            th { "Worker" }
                        }
                    }
                    tbody {
                        tr {
                            td { "22-me_up-22" }
                            td { "0" }
                            td { "0" }
                        }
                    }
                }
                div { class: "empty", "Live-Fetch wird im nächsten Schnitt an /api/tunnels gehängt." }
            }
        }
    }
}
