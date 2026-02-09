use crate::backend::kids::{add_kid, delete_kid, get_granularity, list_kids, rename_kid, update_granularity};
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

    let mut kids_resource = use_resource(list_kids);
    let mut new_kid_name = use_signal(|| String::new());
    let mut editing_kid_id: Signal<Option<u32>> = use_signal(|| None);
    let mut edit_name = use_signal(|| String::new());

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

                    match &*kids_resource.read() {
                        Some(Ok(kids)) => rsx! {
                            if kids.is_empty() {
                                p { class: "text-sm text-gray-400 italic mb-3", "No kids added yet." }
                            }
                            ul { class: "space-y-2 mb-4",
                                for kid in kids.iter() {
                                    {
                                        let kid_id = kid.id;
                                        let kid_name = kid.name.clone();
                                        let is_editing = editing_kid_id() == Some(kid_id);

                                        rsx! {
                                            li { class: "flex items-center justify-between rounded-md border border-gray-100 px-3 py-2 hover:bg-gray-50 group",
                                                if is_editing {
                                                    input {
                                                        class: "flex-1 rounded border border-blue-300 px-2 py-1 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500",
                                                        r#type: "text",
                                                        autofocus: true,
                                                        value: "{edit_name}",
                                                        oninput: move |e: Event<FormData>| edit_name.set(e.value()),
                                                        onkeydown: move |e: Event<KeyboardData>| {
                                                            if e.key() == Key::Enter {
                                                                let new_name = edit_name().trim().to_string();
                                                                if !new_name.is_empty() {
                                                                    editing_kid_id.set(None);
                                                                    spawn(async move {
                                                                        if let Err(e) = rename_kid(kid_id, new_name).await {
                                                                            eprintln!("Failed to rename kid: {e}");
                                                                        }
                                                                        kids_resource.restart();
                                                                    });
                                                                }
                                                            } else if e.key() == Key::Escape {
                                                                editing_kid_id.set(None);
                                                            }
                                                        },
                                                    }
                                                    button {
                                                        class: "ml-2 text-xs text-gray-400 hover:text-gray-600",
                                                        onclick: move |_| editing_kid_id.set(None),
                                                        "Cancel"
                                                    }
                                                } else {
                                                    span {
                                                        class: "flex-1 text-sm text-gray-700 cursor-pointer hover:text-blue-600",
                                                        onclick: {
                                                            let kid_name = kid_name.clone();
                                                            move |_| {
                                                                editing_kid_id.set(Some(kid_id));
                                                                edit_name.set(kid_name.clone());
                                                            }
                                                        },
                                                        "{kid.name}"
                                                    }
                                                    button {
                                                        class: "ml-2 text-gray-300 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-opacity",
                                                        onclick: move |_| {
                                                            spawn(async move {
                                                                if let Err(e) = delete_kid(kid_id).await {
                                                                    eprintln!("Failed to delete kid: {e}");
                                                                }
                                                                kids_resource.restart();
                                                            });
                                                        },
                                                        svg {
                                                            xmlns: "http://www.w3.org/2000/svg",
                                                            fill: "none",
                                                            view_box: "0 0 24 24",
                                                            stroke_width: "2",
                                                            stroke: "currentColor",
                                                            class: "h-4 w-4",
                                                            path {
                                                                stroke_linecap: "round",
                                                                stroke_linejoin: "round",
                                                                d: "M6 18L18 6M6 6l12 12",
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            if kids.len() < 10 {
                                div { class: "flex gap-2",
                                    input {
                                        class: "flex-1 rounded-md border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500",
                                        r#type: "text",
                                        placeholder: "New kid's name...",
                                        value: "{new_kid_name}",
                                        oninput: move |e: Event<FormData>| new_kid_name.set(e.value()),
                                        onkeydown: move |e: Event<KeyboardData>| {
                                            if e.key() == Key::Enter {
                                                let name = new_kid_name().trim().to_string();
                                                if !name.is_empty() {
                                                    new_kid_name.set(String::new());
                                                    spawn(async move {
                                                        if let Err(e) = add_kid(name).await {
                                                            eprintln!("Failed to add kid: {e}");
                                                        }
                                                        kids_resource.restart();
                                                    });
                                                }
                                            }
                                        },
                                    }
                                    Button {
                                        variant: ButtonVariant::Primary,
                                        onclick: move |_| {
                                            let name = new_kid_name().trim().to_string();
                                            if !name.is_empty() {
                                                new_kid_name.set(String::new());
                                                spawn(async move {
                                                    if let Err(e) = add_kid(name).await {
                                                        eprintln!("Failed to add kid: {e}");
                                                    }
                                                    kids_resource.restart();
                                                });
                                            }
                                        },
                                        "Add"
                                    }
                                }
                            } else {
                                p { class: "text-xs text-amber-600", "Maximum of 10 kids reached." }
                            }
                        },
                        Some(Err(_)) => rsx! {
                            p { class: "text-sm text-red-500", "Failed to load kids." }
                        },
                        None => rsx! {
                            p { class: "text-sm text-gray-400", "Loading..." }
                        },
                    }

                    div { class: "mt-4 pt-4 border-t border-gray-100",
                        Button { variant: ButtonVariant::Destructive, "Reset Counter" }
                    }
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
