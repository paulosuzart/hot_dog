use crate::components::button::*;
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn SettingsPage() -> Element {
    rsx! {
        div {
            div { class: "mb-6 flex items-center gap-3",
                Link {
                    to: Route::MainView,
                    class: "flex items-center gap-1 text-sm text-gray-500 hover:text-gray-700 transition-colors",
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke_width: "1.5",
                        stroke: "currentColor",
                        class: "h-4 w-4",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M15.75 19.5 8.25 12l7.5-7.5"
                        }
                    }
                    "Back"
                }
            }
            h1 { class: "text-2xl font-semibold text-gray-900 mb-8",
                "Settings"
            }
            div { class: "space-y-6",
                div { class: "rounded-lg border border-gray-200 bg-white p-5",
                    h2 { class: "text-lg font-medium text-gray-800 mb-1", "Kids" }
                    p { class: "text-sm text-gray-500 mb-4", "Add or remove kids from your list." }
                    Button { variant: ButtonVariant::Outline, "Manage Kids" }
                }
                div { class: "rounded-lg border border-gray-200 bg-white p-5",
                    h2 { class: "text-lg font-medium text-gray-800 mb-1", "Aggregation" }
                    p { class: "text-sm text-gray-500 mb-4", "Configure how notes are counted (daily, weekly, monthly, yearly)." }
                    Button { variant: ButtonVariant::Outline, "Change Granularity" }
                }
                div { class: "rounded-lg border border-gray-200 bg-white p-5",
                    h2 { class: "text-lg font-medium text-gray-800 mb-1", "Counts" }
                    p { class: "text-sm text-gray-500 mb-4", "Reset all counts back to zero." }
                    Button { variant: ButtonVariant::Destructive, "Reset Counts" }
                }
                div { class: "rounded-lg border border-gray-200 bg-white p-5",
                    h2 { class: "text-lg font-medium text-gray-800 mb-1", "History" }
                    p { class: "text-sm text-gray-500 mb-4", "View past notes and activity." }
                    Button { variant: ButtonVariant::Secondary, "View History" }
                }
            }
        }
    }
}
