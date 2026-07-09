use dioxus::prelude::*;
use dioxus::launch;

#[component]
pub fn App() -> Element {
    let mut server_url = use_signal(|| "beam.eysho.info".to_string());
    let mut local_port = use_signal(|| 0u16);
    let mut remote_port = use_signal(|| 0u16);
    let mut last_url = use_signal(|| "".to_string());

    rsx! {
        div {
            class: "min-h-screen bg-gray-100 p-8",
            h1 {
                class: "text-3xl font-bold mb-6",
                "Beam Tunnel Admin",
            }
            div {
                class: "bg-white shadow rounded-lg p-6",
                h2 {
                    class: "text-xl font-semibold mb-4",
                    "Create New Tunnel",
                }
                div {
                    class: "space-y-4",
                    div {
                        label {
                            class: "block text-sm font-medium mb-1",
                            "Server URL",
                        }
                        input {
                            class: "w-full px-3 py-2 border rounded-md",
                            r#type: "text",
                            placeholder: "beam.eysho.info",
                            value: "{server_url()}",
                            oninput: move |evt| server_url.set(evt.value().clone()),
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium mb-1",
                            "Local Port",
                        }
                        input {
                            class: "w-full px-3 py-2 border rounded-md",
                            r#type: "number",
                            placeholder: "22",
                            value: "{local_port()}",
                            oninput: move |evt| {
                                if let Ok(p) = evt.value().parse::<u16>() {
                                    local_port.set(p);
                                }
                            },
                        }
                    }
                    div {
                        label {
                            class: "block text-sm font-medium mb-1",
                            "Remote Port",
                        }
                        input {
                            class: "w-full px-3 py-2 border rounded-md",
                            r#type: "number",
                            placeholder: "22",
                            value: "{remote_port()}",
                            oninput: move |evt| {
                                if let Ok(p) = evt.value().parse::<u16>() {
                                    remote_port.set(p);
                                }
                            },
                        }
                    }
                    button {
                        class: "w-full py-2 px-4 bg-indigo-600 text-white rounded-md",
                        onclick: move |_| {
                            if *local_port.read() > 0 && *remote_port.read() > 0 {
                                let url = format!(
                                    "{}-me.up-{}.{}",
                                    local_port(),
                                    remote_port(),
                                    server_url()
                                );
                                last_url.set(url);
                            }
                        },
                        "Create Tunnel",
                    }
                }
            }
            match last_url().as_str() {
                "" => rsx! {},
                url => rsx! {
                    div {
                        class: "bg-white shadow rounded-lg p-6 mt-8",
                        h2 {
                            class: "text-xl font-semibold mb-4",
                            "Tunnel Created",
                        }
                        div {
                            class: "p-4 bg-green-50 rounded-lg",
                            p {
                                class: "text-sm text-gray-600 mb-2",
                                "Your tunnel is ready:",
                            }
                            p {
                                class: "text-green-600 font-medium",
                                {url},
                            }
                        }
                    }
                },
            }
        }
    }
}

fn main() {
    env_logger::init();
    launch(App);
}
