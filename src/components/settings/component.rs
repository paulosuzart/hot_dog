use crate::backend::kids::{
    add_kid, delete_kid, get_granularity, list_kids, rename_kid, update_granularity,
};
use crate::components::button::*;
use crate::components::popover::*;
use crate::Route;
use dioxus::prelude::*;
use dioxus_primitives::toast::{consume_toast, ToastOptions};
use std::time::Duration;

const GRANULARITY_OPTIONS: &[(&str, &str)] = &[
    ("DAILY", "Daily"),
    ("WEEKLY", "Weekly"),
    ("MONTHLY", "Monthly"),
    ("YEARLY", "Yearly"),
];

/// Returns a color based on the kid's name for the avatar circle.
fn kid_color(name: &str) -> &'static str {
    let hash: u32 = name
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    let colors = [
        "#6366f1", // indigo
        "#8b5cf6", // violet
        "#ec4899", // pink
        "#f43f5e", // rose
        "#f97316", // orange
        "#eab308", // yellow
        "#22c55e", // green
        "#14b8a6", // teal
        "#06b6d4", // cyan
        "#3b82f6", // blue
    ];
    colors[(hash as usize) % colors.len()]
}

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
        // Inject hover CSS for kid rows (since Tailwind group-hover is not compiled)
        style { "
            .kid-row .kid-actions {{ opacity: 0; transition: opacity 0.15s ease; }}
            .kid-row:hover .kid-actions {{ opacity: 1; }}
            .kid-row:hover {{ background-color: #f9fafb; }}
            .kid-name:hover {{ color: #2563eb; }}
            .action-btn {{ transition: all 0.15s ease; }}
            .action-btn:hover {{ background-color: #fef3c7; }}
            .action-btn.delete:hover {{ background-color: #fee2e2; color: #ef4444; }}
            .action-btn.reset:hover {{ background-color: #ffedd5; color: #f97316; }}
        " }

        div { style: "max-width: 520px; margin: 0 auto;",

            // ── Header ──
            div { class: "mb-8 flex items-center gap-4",
                Link {
                    to: Route::MainView,
                    style: "display: flex; align-items: center; justify-content: center; width: 2rem; height: 2rem; border-radius: 50%; color: #9ca3af; transition: all 0.15s;",
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
                h1 { class: "text-2xl font-semibold text-gray-900",
                    "Settings"
                }
            }

            div { class: "space-y-6",

                // ── Kids Section ──
                div { style: "border-radius: 0.75rem; border: 1px solid #e5e7eb; background: white; box-shadow: 0 1px 2px rgba(0,0,0,0.05); overflow: hidden;",

                    div { style: "padding: 1.25rem 1.25rem 0.75rem;",
                        h2 { class: "text-lg font-semibold text-gray-900", "Kids" }
                        p { class: "text-sm text-gray-400", style: "margin-top: 2px;", "Manage your kids and their counters." }
                    }

                    match &*kids_resource.read() {
                        Some(Ok(kids)) => rsx! {
                            if kids.is_empty() {
                                div { style: "padding: 2rem 1.25rem; text-align: center;",
                                    p { class: "text-sm text-gray-400", "No kids added yet. Add your first one below!" }
                                }
                            } else {
                                div {
                                    for kid in kids.iter() {
                                        {
                                            let kid_id = kid.id;
                                            let kid_name = kid.name.clone();
                                            let is_editing = editing_kid_id() == Some(kid_id);
                                            let initial = kid.name.chars().next().unwrap_or('?').to_uppercase().to_string();
                                            let color = kid_color(&kid.name);

                                            rsx! {
                                                div {
                                                    class: "kid-row",
                                                    style: "display: flex; align-items: center; gap: 0.75rem; padding: 0.625rem 1.25rem; border-top: 1px solid #f3f4f6; transition: background-color 0.15s;",

                                                    // Avatar circle
                                                    div {
                                                        style: "flex-shrink: 0; width: 2rem; height: 2rem; border-radius: 50%; display: flex; align-items: center; justify-content: center; color: white; font-size: 0.75rem; font-weight: 700; background-color: {color};",
                                                        "{initial}"
                                                    }

                                                    if is_editing {
                                                        // Edit mode
                                                        input {
                                                            style: "flex: 1; border-radius: 0.5rem; border: 1px solid #93c5fd; background-color: #eff6ff; padding: 0.375rem 0.75rem; font-size: 0.875rem; outline: none;",
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
                                                                                let toast = consume_toast();
                                                                                toast.error(
                                                                                    "Failed to rename kid".to_string(),
                                                                                    ToastOptions::new()
                                                                                        .description(format!("{e}"))
                                                                                        .duration(Duration::from_secs(5)),
                                                                                );
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
                                                            style: "font-size: 0.75rem; color: #9ca3af; padding: 0.25rem 0.5rem; border-radius: 0.25rem; border: none; cursor: pointer; background: transparent;",
                                                            onclick: move |_| editing_kid_id.set(None),
                                                            "Cancel"
                                                        }
                                                    } else {
                                                        // Kid name (click to edit)
                                                        span {
                                                            class: "kid-name",
                                                            style: "flex: 1; font-size: 0.875rem; font-weight: 500; color: #374151; cursor: pointer; transition: color 0.15s;",
                                                            onclick: {
                                                                let kid_name = kid_name.clone();
                                                                move |_| {
                                                                    editing_kid_id.set(Some(kid_id));
                                                                    edit_name.set(kid_name.clone());
                                                                }
                                                            },
                                                            "{kid.name}"
                                                        }

                                                        // Per-kid action buttons (show on hover)
                                                        div {
                                                            class: "kid-actions",
                                                            style: "display: flex; align-items: center; gap: 0.25rem;",

                                                            // Reset counter button
                                                            button {
                                                                class: "action-btn reset",
                                                                style: "display: flex; align-items: center; justify-content: center; padding: 0.375rem; border-radius: 0.375rem; border: none; cursor: pointer; color: #9ca3af; background: transparent;",
                                                                title: "Reset counter",
                                                                onclick: move |_| {
                                                                    // TODO: wire to backend reset_kid_count(kid_id)
                                                                    let toast = consume_toast();
                                                                    toast.warning(
                                                                        "Not available yet".to_string(),
                                                                        ToastOptions::new()
                                                                            .description("Reset counter is not wired to the backend yet.".to_string())
                                                                            .duration(Duration::from_secs(3)),
                                                                    );
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
                                                                        d: "M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182",
                                                                    }
                                                                }
                                                            }

                                                            // Delete button
                                                            button {
                                                                class: "action-btn delete",
                                                                style: "display: flex; align-items: center; justify-content: center; padding: 0.375rem; border-radius: 0.375rem; border: none; cursor: pointer; color: #9ca3af; background: transparent;",
                                                                title: "Remove kid",
                                                                onclick: move |_| {
                                                                    spawn(async move {
                                                                        if let Err(e) = delete_kid(kid_id).await {
                                                                            let toast = consume_toast();
                                                                            toast.error(
                                                                                "Failed to delete kid".to_string(),
                                                                                ToastOptions::new()
                                                                                    .description(format!("{e}"))
                                                                                    .duration(Duration::from_secs(5)),
                                                                            );
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
                                                                        d: "m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0",
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

                            // Add new kid input
                            if kids.len() < 10 {
                                div { style: "padding: 0.75rem 1.25rem; background-color: #f9fafb; border-top: 1px solid #f3f4f6;",
                                    div { class: "flex gap-2",
                                        input {
                                            style: "flex: 1; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; padding: 0.5rem 0.75rem; font-size: 0.875rem; outline: none;",
                                            r#type: "text",
                                            placeholder: "Add a kid...",
                                            value: "{new_kid_name}",
                                            oninput: move |e: Event<FormData>| new_kid_name.set(e.value()),
                                            onkeydown: move |e: Event<KeyboardData>| {
                                                if e.key() == Key::Enter {
                                                    let name = new_kid_name().trim().to_string();
                                                    if !name.is_empty() {
                                                        new_kid_name.set(String::new());
                                                        spawn(async move {
                                                            if let Err(e) = add_kid(name).await {
                                                                let toast = consume_toast();
                                                                toast.error(
                                                                    "Failed to add kid".to_string(),
                                                                    ToastOptions::new()
                                                                        .description(format!("{e}"))
                                                                        .duration(Duration::from_secs(5)),
                                                                );
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
                                                            let toast = consume_toast();
                                                            toast.error(
                                                                "Failed to add kid".to_string(),
                                                                ToastOptions::new()
                                                                    .description(format!("{e}"))
                                                                    .duration(Duration::from_secs(5)),
                                                            );
                                                        }
                                                        kids_resource.restart();
                                                    });
                                                }
                                            },
                                            "Add"
                                        }
                                    }
                                }
                            } else {
                                div { style: "padding: 0.75rem 1.25rem; background-color: #fffbeb; border-top: 1px solid #fef3c7;",
                                    p { class: "text-xs", style: "color: #d97706;", "Maximum of 10 kids reached." }
                                }
                            }
                        },
                        Some(Err(_)) => rsx! {
                            div { style: "padding: 1.5rem 1.25rem; text-align: center;",
                                p { class: "text-sm", style: "color: #ef4444;", "Failed to load kids." }
                            }
                        },
                        None => rsx! {
                            div { style: "padding: 1.5rem 1.25rem; text-align: center;",
                                p { class: "text-sm text-gray-400", "Loading..." }
                            }
                        },
                    }
                }

                // ── Aggregation Section ──
                div { style: "border-radius: 0.75rem; border: 1px solid #e5e7eb; background: white; box-shadow: 0 1px 2px rgba(0,0,0,0.05); position: relative;",
                    div { style: "padding: 1.25rem;",
                        h2 { class: "text-lg font-semibold text-gray-900", "Aggregation" }
                        p { class: "text-sm text-gray-400", style: "margin-top: 2px; margin-bottom: 1rem;",
                            "How are notes counted over time?"
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
                                div { style: "display: flex; flex-direction: column;",
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
                                                                    let toast = consume_toast();
                                                                    toast.error(
                                                                        "Failed to update aggregation".to_string(),
                                                                        ToastOptions::new()
                                                                            .description(format!("{e}"))
                                                                            .duration(Duration::from_secs(5)),
                                                                    );
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
                }

                // ── History Section ──
                div { style: "border-radius: 0.75rem; border: 1px solid #e5e7eb; background: white; box-shadow: 0 1px 2px rgba(0,0,0,0.05);",
                    div { style: "padding: 1.25rem; display: flex; align-items: center; justify-content: space-between;",
                        div {
                            h2 { class: "text-lg font-semibold text-gray-900", "History" }
                            p { class: "text-sm text-gray-400", style: "margin-top: 2px;", "View past notes and activity." }
                        }
                        Button { variant: ButtonVariant::Secondary, "View History" }
                    }
                }
            }
        }
    }
}
