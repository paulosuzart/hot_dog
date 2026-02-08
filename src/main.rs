mod backend;
mod components;
mod models;
mod notica_component;

use dioxus::prelude::*;

use components::settings::SettingsPage;
use notica_component::NoticaApp;

fn main() {
    dioxus::launch(|| {
        rsx! {
            document::Stylesheet {
                // Urls are relative to your Cargo.toml file
                href: asset!("/assets/tailwind.css"),
            }
            document::Stylesheet {
                href: asset!("/assets/dx-components-theme.css"),
            }
            Router::<Route> {}

        }
    });
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Routable)]
pub enum Route {
    #[route("/")]
    MainView,
    #[route("/settings")]
    SettingsView,
}

#[component]
fn MainView() -> Element {
    rsx! {
        div { class: "mx-auto max-w-3xl px-4 py-8",
            NoticaApp {}
        }
    }
}

#[component]
fn SettingsView() -> Element {
    rsx! {
        div { class: "min-h-screen bg-gray-100",
            div { class: "mx-auto max-w-3xl px-4 py-8",
                SettingsPage {}
            }
        }
    }
}
