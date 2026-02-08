mod backend;
mod components;
mod notica_component;

use dioxus::prelude::*;

use notica_component::NoticaApp;

fn main() {
    dioxus::launch(|| {
        rsx! {
            document::Stylesheet {
                // Urls are relative to your Cargo.toml file
                href: asset!("/assets/tailwind.css"),
            }
            Router::<Route> {}

        }
    });
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Routable)]
enum Route {
    #[route("/")]
    MainView,
}

#[component]
fn MainView() -> Element {
    rsx! {
        NoticaApp {}
    }
}
