mod backend;
mod components;
mod models;
mod notica_component;

use dioxus::prelude::*;

use components::settings::SettingsPage;
use components::toast::ToastProvider;
use notica_component::NoticaApp;

fn main() {
    dioxus::launch(|| {
        rsx! {
            document::Stylesheet {
                // Urls are relative to your Cargo.toml file
                href: asset!("/assets/tailwind.css"),
            }
            document::Stylesheet { href: asset!("/assets/dx-components-theme.css") }
            ToastProvider { Router::<Route> {} }
        }
    });
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Routable)]
pub enum Route {
    #[route("/")]
    MainView,
    #[route("/settings")]
    SettingsView,
    #[route("/about")]
    AboutView,
}

#[component]
fn MainView() -> Element {
    rsx! {
        div { style: "min-height: 100vh; background-color: #f3f4f6;",
            div { style: "max-width: 520px; margin: 0 auto; padding: 2rem 1rem;", NoticaApp {} }
        }
    }
}

#[component]
fn SettingsView() -> Element {
    rsx! {
        div { style: "min-height: 100vh; background-color: #f3f4f6;",
            div { style: "max-width: 520px; margin: 0 auto; padding: 2rem 1rem;", SettingsPage {} }
        }
    }
}

#[component]
fn AboutView() -> Element {
    rsx! {
        div { style: "min-height: 100vh; background-color: #f3f4f6;",
            div { style: "max-width: 520px; margin: 0 auto; padding: 2rem 1rem;",

                // ── Header ──
                div { class: "mb-8 flex items-center gap-4",
                    button {
                        style: "display: flex; align-items: center; justify-content: center; width: 2rem; height: 2rem; border-radius: 50%; color: #9ca3af; border: none; background: transparent; cursor: pointer; transition: all 0.15s;",
                        onclick: move |_| {
                            navigator().push(Route::MainView);
                        },
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            fill: "none",
                            view_box: "0 0 24 24",
                            stroke_width: "2",
                            stroke: "currentColor",
                            class: "h-5 w-5",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                d: "M15.75 19.5 8.25 12l7.5-7.5",
                            }
                        }
                    }
                    h1 { class: "text-2xl font-semibold text-gray-900", "About" }
                }

                // ── About card ──
                div { style: "border-radius: 0.75rem; border: 1px solid #e5e7eb; background: white; box-shadow: 0 1px 2px rgba(0,0,0,0.05);",
                    div { style: "padding: 2rem 1.5rem; display: flex; flex-direction: column; align-items: center; text-align: center;",
                        img {
                            src: asset!("/assets/hotdog.svg"),
                            alt: "Hot Dog mascot",
                            style: "width: 8rem; height: 8rem; margin-bottom: 1.25rem;",
                        }
                        h2 { style: "font-size: 1.25rem; font-weight: 700; color: #111827; margin-bottom: 0.75rem;",
                            "Hot Dog"
                        }
                        p { style: "font-size: 0.875rem; color: #6b7280; line-height: 1.7; max-width: 380px;",
                            "A fun little app for tracking your kids\u{2019} daily scores \u{2014} think gold stars, good deeds, or the occasional oopsie. Pick how you want to count (daily, weekly, monthly) and tap away. Built with love, Rust, and a dash of mustard."
                        }
                    }
                }
            }
        }
    }
}
