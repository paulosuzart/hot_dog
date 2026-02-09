use crate::backend::kids::{get_granularity, update_granularity};
use crate::components::button::*;
use crate::components::popover::*;
use crate::Route;
use dioxus::prelude::*;

const GRANULARITY_OPTIONS: &[(&str, &str)] = &[
    ("DAILY", "Daily"),
    ("WEEKLY", "Weekly"),
    ("MONTHLY", "Monthly"),
    ("YEARLY", "Yearly"),
];

#[component]
pub fn SettingsPage() -> Element {
    let mut granularity = use_resource(get_granularity);
    let mut popover_open = use_signal(|| false);

    let current = match &*granularity.read() {
        Some(Ok(g)) => g.clone(),
        _ => "MONTHLY".to_string(),
    };

    let current_label = GRANULARITY_OPTIONS
        .iter()
        .find(|(val, _)| *val == current.as_str())
        .map(|(_, label)| *label)
        .unwrap_or("Monthly");

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
                            d: "M15.75 19.5 8.25 12l7.5-7.5",
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
                    p { class: "text-sm text-gray-500 mb-4",
                        "Configure how notes are counted (daily, weekly, monthly, yearly)."
                    }
                    PopoverRoot {
                        open: popover_open(),
                        on_open_change: move |open: bool| popover_open.set(open),
                        PopoverTrigger {
                            "{current_label} ▾"
                        }
                        PopoverContent {
                            side: dioxus_primitives::ContentSide::Bottom,
                            align: dioxus_primitives::ContentAlign::Start,
                            div { class: "flex flex-col",
                                for (value , label) in GRANULARITY_OPTIONS.iter() {
                                    {
                                        let value = value.to_string();
                                        let is_selected = value == current;
                                        rsx! {
                                            button {
                                                style: if is_selected {
                                                    "padding: 8px 16px; text-align: left; border: none; background: #e0e7ff; color: #3730a3; font-weight: 600; cursor: default; border-radius: 0.375rem; font-size: 0.875rem;"
                                                } else {
                                                    "padding: 8px 16px; text-align: left; border: none; background: transparent; cursor: pointer; border-radius: 0.375rem; font-size: 0.875rem; color: inherit;"
                                                },
                                                disabled: is_selected,
                                                onclick: {
                                                    let value = value.clone();
                                                    move |_| {
                                                        let value = value.clone();
                                                        popover_open.set(false);
                                                        spawn(async move {
                                                            if let Err(e) = update_granularity(value).await {
                                                                eprintln!("Failed to update granularity: {e}");
                                                            }
                                                            granularity.restart();
                                                        });
                                                    }
                                                },
                                                "{label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
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
