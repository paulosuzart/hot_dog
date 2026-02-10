use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn AboutPage() -> Element {
    rsx! {
        // ── Header ──
        div { class: "mb-8 flex items-center gap-4",
            Link {
                to: Route::MainView,
                style: "display: flex; align-items: center; justify-content: center; width: 2rem; height: 2rem; border-radius: 50%; color: #9ca3af; border: none; background: transparent; cursor: pointer; transition: all 0.15s;",
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
                    "A fun little app for tracking your kids' daily scores — think gold stars, good deeds, or the occasional oopsie. Pick how you want to count (daily, weekly, monthly) and tap away. Built with love, Rust, and a dash of mustard."
                }
            }
        }
    }
}
