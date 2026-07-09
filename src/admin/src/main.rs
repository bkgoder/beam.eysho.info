use dioxus::prelude::*;
use dioxus::launch;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct TunnelInfo {
    pub tunnel_id: String,
    pub local_port: u16,
    pub remote_port: u16,
    pub url: String,
    pub status: String,
    pub created_at: String,
}

#[component]
pub fn App() -> Element {
    let mut tunnels = use_signal(|| HashMap::<String, TunnelInfo>::new());
    let mut stats = use_signal(|| TunnelStats {
        total_tunnels: 0,
        active_connections: 0,
        uptime: "0s".to_string(),
    });
    let mut server_url = use_signal(|| "beam.eysho.info".to_string());
    let mut local_port = use_signal(|| 0u16);
    let mut remote_port = use_signal(|| 0u16);

    rsx! {
        div {
            class: "min-h-screen bg-gray-100",
            div {
                class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                div { class: "py-6",
                    h1 { class: "text-3xl font-bold text-gray-900", "Beam Tunnel Admin" }
                    p { class: "text-sm text-gray-600 mt-1", "Manage your tunnel service" }
                }
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-8",
                    StatCard { title: "Total Tunnels", value: stats().total_tunnels.to_string(), icon: "🚇" }
                    StatCard { title: "Active Connections", value: stats().active_connections.to_string(), icon: "🔗" }
                    StatCard { title: "Uptime", value: stats().uptime.clone(), icon: "⏱️" }
                }
                div { class: "bg-white shadow rounded-lg p-6 mb-8",
                    h2 { class: "text-xl font-semibold text-gray-900 mb-4", "Create New Tunnel" }
                    div { class: "space-y-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "Server URL" }
                            input {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md",
                                r#type: "text",
                                placeholder: "beam.eysho.info",
                                value: "{server_url()}",
                                oninput: move |evt| server_url.set(evt.value().clone()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "Local Port" }
                            input {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md",
                                r#type: "number",
                                placeholder: "e.g., 22",
                                value: "{local_port()}",
                                oninput: move |evt| { if let Ok(p) = evt.value().parse::<u16>() { local_port.set(p); } },
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "Remote Port" }
                            input {
                                class: "w-full px-3 py-2 border border-gray-300 rounded-md",
                                r#type: "number",
                                placeholder: "e.g., 22",
                                value: "{remote_port()}",
                                oninput: move |evt| { if let Ok(p) = evt.value().parse::<u16>() { remote_port.set(p); } },
                            }
                        }
                        button {
                            class: "w-full py-2 px-4 bg-indigo-600 text-white rounded-md hover:bg-indigo-700",
                            onclick: move |_| {
                                if *local_port.read() > 0 && *remote_port.read() > 0 {
                                    let tunnel_id = format!("{}-me_up-{}", local_port(), remote_port());
                                    let url = format!("{}-me.up-{}.{}", local_port(), remote_port(), server_url());
                                    tunnels.write().insert(tunnel_id, TunnelInfo {
                                        tunnel_id: tunnel_id.clone(),
                                        local_port: *local_port.read(),
                                        remote_port: *remote_port.read(),
                                        url: url.clone(),
                                        status: "active".to_string(),
                                        created_at: chrono::Local::now().to_rfc3339(),
                                    });
                                    stats.write().total_tunnels = tunnels.read().len();
                                }
                            },
                            "Create Tunnel"
                        }
                    }
                }
                div { class: "bg-white shadow rounded-lg p-6",
                    h2 { class: "text-xl font-semibold text-gray-900 mb-4", "Active Tunnels" }
                    match tunnels().is_empty() {
                        true => rsx! { p { class: "text-gray-500 text-center py-8", "No tunnels yet. Create one above!" } },
                        false => rsx! {
                            table { class: "min-w-full divide-y",
                                thead { class: "bg-gray-50",
                                    tr {
                                        th { class: "px-6 py-3 text-left text-xs", "Tunnel ID" }
                                        th { class: "px-6 py-3 text-left text-xs", "URL" }
                                        th { class: "px-6 py-3 text-left text-xs", "Status" }
                                        th { class: "px-6 py-3 text-right text-xs", "Actions" }
                                    }
                                }
                                tbody {
                                    for (tunnel_id, tunnel) in tunnels().iter() {
                                        rsx! {
                                            tr {
                                                td { class: "px-6 py-4", tunnel_id.clone() }
                                                td { class: "px-6 py-4 text-blue-600", a { href: format!("http://{}", tunnel.url), target: "_blank", tunnel.url.clone() } }
                                                td { 
                                                    span { 
                                                        class: if tunnel.status == "active" { "px-2 py-1 text-xs bg-green-100 text-green-800 rounded" } else { "px-2 py-1 text-xs bg-red-100 text-red-800 rounded" },
                                                        tunnel.status.clone()
                                                    }
                                                }
                                                td { class: "px-6 py-4 text-right",
                                                    button {
                                                        class: "text-red-600",
                                                        onclick: move |_| { tunnels.write().remove(tunnel_id); stats.write().total_tunnels = tunnels.read().len(); },
                                                        "Delete"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn StatCard(title: String, value: String, icon: String) -> Element {
    rsx! {
        div { class: "bg-white shadow rounded-lg p-6",
            div { class: "flex items-center",
                div { class: "text-3xl mr-4", {icon} }
                div {
                    p { class: "text-sm text-gray-500", {title} }
                    p { class: "text-2xl font-bold", {value} }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TunnelStats {
    pub total_tunnels: usize,
    pub active_connections: usize,
    pub uptime: String,
}

fn main() {
    env_logger::init();
    launch(App);
}
