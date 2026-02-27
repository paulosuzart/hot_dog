use crate::backend::kids::list_kids;
use crate::backend::notes_history_query::NotesHistoryFilter;
use crate::models::NoteHistory;
use crate::Route;
use dioxus::prelude::*;
use dioxus_primitives::toast::{consume_toast, ToastOptions};

#[server]
pub async fn get_notes_history(
    filter: NotesHistoryFilter,
) -> Result<crate::models::NoteHistoryResponse, ServerFnError> {
    use crate::backend::notes_history_query::NotesHistoryQuery;

    let query = NotesHistoryQuery::new(filter);
    query.execute().await
}

#[component]
fn NoteRow(note: NoteHistory, show_kid: bool) -> Element {
    let qty_label = if note.quantity > 0 {
        format!("+{}", note.quantity)
    } else {
        format!("{}", note.quantity)
    };
    let qty_bg = if note.quantity >= 0 {
        "#dcfce7"
    } else {
        "#fee2e2"
    };
    let qty_color = if note.quantity >= 0 {
        "#16a34a"
    } else {
        "#dc2626"
    };

    rsx! {
        tr {
            class: "note-row",
            style: "border-bottom: 1px solid #f3f4f6; transition: background-color 0.15s;",

            if show_kid {
                td {
                    style: "padding: 0.75rem 1rem; color: #9ca3af; font-size: 0.875rem; font-family: monospace;",
                    "id: {note.kid_id}"
                }
                td {
                    style: "padding: 0.75rem 1rem; font-weight: 500; color: #374151;",
                    "{note.kid_name}"
                }
            }

            td {
                style: "padding: 0.75rem 1rem; color: #6b7280; font-size: 0.875rem;",
                "{note.created_at}"
            }

            td {
                style: "padding: 0.75rem 1rem;",
                span {
                    style: "font-size: 0.875rem; font-weight: 600; padding: 0.25rem 0.75rem; border-radius: 0.375rem; background-color: {qty_bg}; color: {qty_color};",
                    "{qty_label}"
                }
            }
        }
    }
}

const PAGE_SIZE: u8 = 20;

#[component]
pub fn NotesHistoryPage() -> Element {
    let kids_resource = use_resource(list_kids);

    // Filter state
    let mut selected_kid_id = use_signal(|| None::<u32>);
    let mut date_from = use_signal(|| String::new());
    let mut date_to = use_signal(|| String::new());

    // Sort state
    let mut sort_by = use_signal(|| "created_at".to_string());
    let mut sort_order = use_signal(|| "desc".to_string());

    // Pagination state
    let mut current_page = use_signal(|| 1usize);
    let mut total_pages = use_signal(|| 1usize);
    let mut total_count = use_signal(|| 0u32);
    let mut has_next = use_signal(|| false);
    let mut cursor = use_signal(|| None::<String>);
    let mut cursor_stack = use_signal(|| vec![None::<String>]);

    // The notes resource - reactive to all filters and cursor
    let mut notes = use_resource(move || {
        get_notes_history(NotesHistoryFilter {
            kid_id: selected_kid_id(),
            date_from: if date_from().is_empty() {
                None
            } else {
                Some(date_from())
            },
            date_to: if date_to().is_empty() {
                None
            } else {
                Some(date_to())
            },
            cursor: cursor(),
            page_size: PAGE_SIZE,
            sort_by: Some(sort_by()),
            sort_order: Some(sort_order()),
        })
    });

    let toast = consume_toast();

    use_effect(move || {
        if let Some(Ok(response)) = notes.read().as_ref() {
            total_pages.set(response.total_pages as usize);
            total_count.set(response.total_count);
            has_next.set(response.cursor.is_some());

            if let Some(ref next_cursor) = response.cursor {
                let cp = *current_page.peek();
                cursor_stack.with_mut(|stack| {
                    if stack.len() <= cp {
                        stack.push(Some(next_cursor.clone()));
                    }
                });
            }
        } else if let Some(Err(e)) = notes.read().as_ref() {
            toast.error(
                format!("Failed to load history: {}", e),
                ToastOptions::new(),
            );
        }
    });

    // Reset pagination when filters change
    let apply_filters = move |_| {
        current_page.set(1);
        cursor.set(None);
        cursor_stack.set(vec![None]);
        notes.restart();
    };

    let mut toggle_sort = move |column: String| {
        let current_column = sort_by();
        if current_column == column {
            // Toggle order
            let new_order = if sort_order() == "desc" {
                "asc"
            } else {
                "desc"
            };
            sort_order.set(new_order.to_string());
        } else {
            sort_by.set(column);
            sort_order.set("desc".to_string());
        }
        // Reset pagination
        current_page.set(1);
        cursor.set(None);
        cursor_stack.set(vec![None]);
    };

    rsx! {
        style { "
            .note-row:hover {{ background-color: #f9fafb; }}
        " }

        div { style: "max-width: 720px; margin: 0 auto;",

            // ── Header ──
            div { class: "mb-8 flex items-center gap-4",
                Link {
                    to: Route::SettingsView,
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
                h1 { class: "text-2xl font-semibold text-gray-900", "Notes History" }
            }

            // ── Filters Section ──
            div { style: "margin-bottom: 1.5rem; border-radius: 0.75rem; border: 1px solid #e5e7eb; background: white; box-shadow: 0 1px 2px rgba(0,0,0,0.05); overflow: hidden;",
                div { style: "padding: 1.25rem;",
                    h2 { class: "text-sm font-semibold text-gray-700 mb-3", "Filters" }

                    div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-bottom: 1rem;",
                        // Kid filter
                        div {
                            label { class: "block text-xs font-medium text-gray-500 mb-1", "Kid" }
                            match &*kids_resource.read() {
                                Some(Ok(kids)) => rsx! {
                                    select {
                                        style: "width: 100%; padding: 0.5rem 0.75rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; font-size: 0.875rem;",
                                        onchange: move |e: Event<FormData>| {
                                            let val = e.value();
                                            selected_kid_id.set(if val.is_empty() { None } else { val.parse().ok() });
                                        },
                                        option { value: "", "All Kids" }
                                         for kid in kids.iter() {
                                             option {
                                                 value: "{kid.id}",
                                                 selected: selected_kid_id() == Some(kid.id),
                                                 "id: {kid.id} - {kid.name}"
                                             }
                                         }
                                    }
                                },
                                _ => rsx! {
                                    select {
                                        style: "width: 100%; padding: 0.5rem 0.75rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: #f9fafb; font-size: 0.875rem;",
                                        disabled: true,
                                        option { "Loading..." }
                                    }
                                }
                            }
                        }

                        // Apply button
                        div { style: "display: flex; align-items: flex-end;",
                            button {
                                style: "width: 100%; padding: 0.5rem 1rem; border-radius: 0.5rem; border: none; background: #3b82f6; color: white; font-size: 0.875rem; font-weight: 500; cursor: pointer; transition: background-color 0.15s;",
                                onclick: apply_filters,
                                "Apply Filters"
                            }
                        }
                    }

                    div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 1rem;",
                        // Date from
                        div {
                            label { class: "block text-xs font-medium text-gray-500 mb-1", "From Date" }
                            input {
                                r#type: "date",
                                style: "width: 100%; padding: 0.5rem 0.75rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; font-size: 0.875rem;",
                                value: "{date_from}",
                                oninput: move |e: Event<FormData>| date_from.set(e.value()),
                            }
                        }

                        // Date to
                        div {
                            label { class: "block text-xs font-medium text-gray-500 mb-1", "To Date" }
                            input {
                                r#type: "date",
                                style: "width: 100%; padding: 0.5rem 0.75rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; font-size: 0.875rem;",
                                value: "{date_to}",
                                oninput: move |e: Event<FormData>| date_to.set(e.value()),
                            }
                        }
                    }
                }
            }

            // ── Table Section ──
            div { style: "border-radius: 0.75rem; border: 1px solid #e5e7eb; background: white; box-shadow: 0 1px 2px rgba(0,0,0,0.05); overflow: hidden;",

                // ── Pagination Toolbar ──
                if total_count() > 0 {
                    div { style: "display: flex; align-items: center; justify-content: space-between; gap: 1rem; padding: 0.75rem 1rem; border-bottom: 1px solid #e5e7eb; background: #f9fafb;",
                        button {
                            style: if current_page() == 1 {
                                "padding: 0.375rem 0.75rem; font-size: 0.875rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; cursor: not-allowed; color: #d1d5db;"
                            } else {
                                "padding: 0.375rem 0.75rem; font-size: 0.875rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; cursor: pointer; color: #374151;"
                            },
                            disabled: current_page() == 1,
                            onclick: move |_| {
                                if current_page() > 1 {
                                    let prev_page = current_page() - 1;
                                    let prev_cursor = cursor_stack.read().get(prev_page - 1).cloned().flatten();
                                    cursor.set(prev_cursor);
                                    current_page.set(prev_page);
                                }
                            },
                            "Previous"
                        }

                        span { style: "font-size: 0.875rem; color: #6b7280;",
                            "Page {current_page()} of {total_pages()} ({total_count()} notes)"
                        }

                        button {
                            style: if !has_next() {
                                "padding: 0.375rem 0.75rem; font-size: 0.875rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; cursor: not-allowed; color: #d1d5db;"
                            } else {
                                "padding: 0.375rem 0.75rem; font-size: 0.875rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; cursor: pointer; color: #374151;"
                            },
                            disabled: !has_next(),
                            onclick: move |_| {
                                let next_cursor = cursor_stack.read().get(current_page()).cloned().flatten();
                                cursor.set(next_cursor);
                                current_page.set(current_page() + 1);
                            },
                            "Next"
                        }
                    }
                }

                // ── Table ──
                table { style: "width: 100%; border-collapse: collapse;",
                    thead {
                        style: "background: #f9fafb;",
                        tr {
                            th {
                                style: "padding: 0.75rem 1rem; text-align: left; font-size: 0.75rem; font-weight: 600; color: #6b7280; text-transform: uppercase; letter-spacing: 0.05em;",
                                "Id"
                            }
                            th {
                                style: "padding: 0.75rem 1rem; text-align: left; font-size: 0.75rem; font-weight: 600; color: #6b7280; text-transform: uppercase; letter-spacing: 0.05em; cursor: pointer; user-select: none;",
                                onclick: move |_| toggle_sort("kid_name".to_string()),
                                span { "Kid " }
                                if sort_by() == "kid_name" {
                                    span {
                                        style: "margin-left: 0.25rem;",
                                        if sort_order() == "asc" { "↑" } else { "↓" }
                                    }
                                }
                            }
                            th {
                                style: "padding: 0.75rem 1rem; text-align: left; font-size: 0.75rem; font-weight: 600; color: #6b7280; text-transform: uppercase; letter-spacing: 0.05em; cursor: pointer; user-select: none;",
                                onclick: move |_| toggle_sort("created_at".to_string()),
                                span { "Date " }
                                if sort_by() == "created_at" {
                                    span {
                                        style: "margin-left: 0.25rem;",
                                        if sort_order() == "asc" { "↑" } else { "↓" }
                                    }
                                }
                            }
                            th {
                                style: "padding: 0.75rem 1rem; text-align: left; font-size: 0.75rem; font-weight: 600; color: #6b7280; text-transform: uppercase; letter-spacing: 0.05em;",
                                "Quantity"
                            }
                        }
                    }
                    tbody {
                        match &*notes.read() {
                            None => rsx! {
                                tr {
                                    td {
                                        colspan: 4,
                                        style: "padding: 2rem; text-align: center; color: #9ca3af;",
                                        "Loading..."
                                    }
                                }
                            },
                            Some(Ok(response)) if response.notes.is_empty() => rsx! {
                                tr {
                                    td {
                                        colspan: 4,
                                        style: "padding: 3rem; text-align: center;",
                                        div {
                                            style: "color: #9ca3af;",
                                            p { style: "font-size: 1rem; font-weight: 500; margin-bottom: 0.25rem;", "No Notes Found" }
                                            p { style: "font-size: 0.875rem;", "Try adjusting your filters." }
                                        }
                                    }
                                }
                            },
                            Some(Ok(response)) => rsx! {
                                for note in response.notes.iter() {
                                    NoteRow {
                                        note: note.clone(),
                                        show_kid: true,
                                    }
                                }
                            },
                            Some(Err(_)) => rsx! {
                                tr {
                                    td {
                                        colspan: 4,
                                        style: "padding: 2rem; text-align: center; color: #ef4444;",
                                        "Failed to load notes"
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}
