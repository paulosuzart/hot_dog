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
        div { style: "min-height: 100vh; background-color: #f3f4f6;",
            div { style: "max-width: 520px; margin: 0 auto; padding: 2rem 1rem;",
                NoticaApp {}
            }
        }
    }
}

#[component]
fn SettingsView() -> Element {
    rsx! {
        div { style: "min-height: 100vh; background-color: #f3f4f6;",
            div { style: "max-width: 520px; margin: 0 auto; padding: 2rem 1rem;",
                SettingsPage {}
            }
        }
    }
}
