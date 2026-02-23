use crate::backend::kids::{get_history_details, get_paged_history};
use crate::models::KidHistory;
use crate::Route;
use dioxus::prelude::*;
use dioxus_primitives::toast::{consume_toast, ToastOptions};

#[component]
fn NoteRow(quantity: i32, created_at: String) -> Element {
    let qty_label = if quantity > 0 {
        format!("+{}", quantity)
    } else {
        format!("{}", quantity)
    };
    let qty_bg = if quantity >= 0 { "#dcfce7" } else { "#fee2e2" };
    let qty_color = if quantity >= 0 { "#16a34a" } else { "#dc2626" };
    let qty_style = format!("flex-shrink: 0; font-size: 0.75rem; font-weight: 600; padding: 0.25rem 0.5rem; border-radius: 0.375rem; background-color: {qty_bg}; color: {qty_color};");

    rsx! {
        div {
            class: "note-row",
            style: "display: flex; align-items: center; gap: 0.75rem; padding: 0.75rem 1rem; border-top: 1px solid #f3f4f6; transition: background-color 0.15s;",

            span { style: "flex-shrink: 0; font-size: 0.75rem; color: #9ca3af; width: 140px;",
                {created_at}
            }

            span { style: "{qty_style}", {qty_label} }

            button {
                class: "delete-btn",
                style: "flex-shrink: 0; display: flex; align-items: center; justify-content: center; width: 1.5rem; height: 1.5rem; border-radius: 0.375rem; border: none; cursor: pointer; color: #ef4444; background: transparent; transition: all 0.15s;",
                title: "Delete note",
                onclick: move |_| {},
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
                        d: "M6 18 18 6M6 6l12 12",
                    }
                }
            }
        }
    }
}

#[component]
fn HistoryDetails(kid_id: u32, period: String, expected_count: usize) -> Element {
    let details = use_resource(move || get_history_details(kid_id, period.clone(), expected_count));

    let toast = consume_toast();

    use_effect(move || {
        if let Some(Ok(response)) = details.read().as_ref() {
            if response.needs_reload {
                toast.warning(
                    "Data might have been changed, please reload app".to_string(),
                    ToastOptions::new(),
                );
            }
        } else if let Some(Err(e)) = details.read().as_ref() {
            toast.error(
                format!("Failed to load history details: {}", e),
                ToastOptions::new(),
            );
        }
    });

    let details_read = details.read();
    match &*details_read {
        None => rsx! {
            div { style: "text-align: center; padding: 2rem; color: #9ca3af;", "Loading details..." }
        },
        Some(Ok(response)) => {
            if response.notes.is_empty() {
                rsx! {
                    div { style: "text-align: center; padding: 2rem; color: #9ca3af;",
                        "No notes available"
                    }
                }
            } else {
                rsx! {
                    if response.max_notes_reached {
                        div {
                            style: "display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem 1rem; background-color: #fef3c7; color: #92400e; font-size: 0.75rem; border-bottom: 1px solid #fde68a;",
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke_width: "2",
                                stroke: "currentColor",
                                style: "width: 1rem; height: 1rem; flex-shrink: 0;",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    d: "M12 9v3.75m9-.75a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9 3.75h.008v.008H12v-.008Z",
                                }
                            }
                            span { "Showing the 50 most recent notes only." }
                        }
                    }
                    for note in response.notes.iter() {
                        NoteRow {
                            quantity: note.quantity,
                            created_at: note.created_at.format("%b %d, %Y %H:%M").to_string(),
                        }
                    }
                }
            }
        }
        Some(Err(_)) => rsx! {
            div { style: "text-align: center; padding: 2rem; color: #d1d5db;", "Failed to load details" }
        },
    }
}

// Manual accordion — no dioxus-primitives Accordion used here so that we can
// reliably collapse all items whenever the page prop changes (Dioxus `key`
// only works inside `for` loops, not on arbitrary expression blocks, so the
// primitives Accordion context would not reset on re-render).
#[component]
fn HistoryList(items: Vec<KidHistory>, page: usize) -> Element {
    // One bool per item; all start collapsed.
    let mut open = use_signal(|| vec![false; items.len()]);

    // Detect page changes and collapse every item.
    // last_page mirrors the `page` prop so we can compare across renders.
    let mut last_page = use_signal(|| page);
    if last_page() != page {
        last_page.set(page);
        open.set(vec![false; items.len()]);
    }

    rsx! {
        div { style: "border-radius: 0.75rem; border: 1px solid #e5e7eb; background: white; box-shadow: 0 1px 2px rgba(0,0,0,0.05); overflow: hidden; width: 100%;",
            for (i, h) in items.iter().enumerate() {
                div {
                    key: "{i}",
                    style: if i > 0 { "border-top: 1px solid #e5e7eb;" } else { "" },

                    // ── Trigger ────────────────────────────────────────────
                    button {
                        style: "width: 100%; display: flex; align-items: center; justify-content: space-between; background: transparent; border: none; cursor: pointer; padding: 0; text-align: left;",
                        onclick: move |_| {
                            open.with_mut(|v| {
                                if let Some(val) = v.get_mut(i) {
                                    *val = !*val;
                                }
                            });
                        },

                        div { style: "display: flex; align-items: center; gap: 1rem; flex: 1; min-width: 0; padding: 1rem 1rem;",
                            div { style: "display: flex; flex-direction: column; gap: 0.25rem; min-width: 120px;",
                                span { style: "font-size: 0.875rem; font-weight: 600; color: #111827;",
                                    "{h.period}"
                                }
                                span { style: "font-size: 0.75rem; color: #9ca3af;",
                                    "{h.neg_count + h.post_count} notes"
                                }
                            }
                            {
                                let bg = if h.total >= 0 { "#dcfce7" } else { "#fee2e2" };
                                let color = if h.total >= 0 { "#16a34a" } else { "#dc2626" };
                                rsx! {
                                    span {
                                        style: "flex-shrink: 0; font-size: 1.5rem; font-weight: 700; padding: 0.5rem 1rem; border-radius: 0.5rem; background-color: {bg}; color: {color};",
                                        "{h.total}"
                                    }
                                }
                            }
                        }

                        // Chevron rotates when item is expanded
                        {
                            let rotation = if open.read().get(i).copied().unwrap_or(false) { "rotate(180deg)" } else { "rotate(0deg)" };
                            rsx! {
                                svg {
                                    xmlns: "http://www.w3.org/2000/svg",
                                    fill: "none",
                                    view_box: "0 0 24 24",
                                    stroke_width: "2",
                                    stroke: "currentColor",
                                    style: "width: 1.25rem; height: 1.25rem; margin-right: 1rem; flex-shrink: 0; transition: transform 0.2s ease; transform: {rotation};",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "m19.5 8.25-7.5 7.5-7.5-7.5",
                                    }
                                }
                            }
                        }
                    }

                    // ── Content (conditionally rendered) ───────────────────
                    if open.read().get(i).copied().unwrap_or(false) {
                        div { style: "border-top: 1px solid #f3f4f6; width: 100%;",
                            HistoryDetails {
                                kid_id: h.id,
                                period: h.period.clone(),
                                expected_count: (h.neg_count + h.post_count) as usize,
                            }
                        }
                    }
                }
            }
        }
    }
}

const PAGE_SIZE: u8 = 1;

#[component]
pub fn KidHistoryPage(kid_id: u32) -> Element {
    let mut current_page = use_signal(|| 1usize);
    let mut total_pages = use_signal(|| 1usize);
    let mut total_count = use_signal(|| 0u32);
    let mut has_next = use_signal(|| false);
    // generation is bumped on every page navigation so HistoryList detects
    // the change and collapses its accordion items.
    let mut generation = use_signal(|| 0usize);

    // cursor_stack[i] = cursor required to fetch page (i+1).
    //   stack[0] = None          → fetch page 1 (no cursor needed)
    //   stack[1] = Some("c1")   → fetch page 2
    //   stack[2] = Some("c2")   → fetch page 3
    //   …
    let mut cursor_stack = use_signal(|| vec![None::<String>]);
    // cursor currently in use (drives the resource query)
    let mut cursor = use_signal(|| None::<String>);

    let history = use_resource(move || get_paged_history(kid_id, cursor(), PAGE_SIZE));

    let toast = consume_toast();

    use_effect(move || {
        if let Some(Ok(response)) = history.read().as_ref() {
            total_pages.set(response.total_pages as usize);
            total_count.set(response.total_count);
            has_next.set(response.cursor.is_some());

            // Push the next-page cursor into the stack at the correct index.
            //
            // We use `current_page.peek()` (non-reactive read) so that this
            // effect is NOT re-triggered by current_page changes — only by a
            // new `history` response arriving.  Without this, the effect would
            // fire with the updated current_page but the *old* response, pushing
            // the wrong cursor at the wrong index and breaking subsequent pages.
            if let Some(ref next_cursor) = response.cursor {
                let cp = *current_page.peek(); // non-reactive read
                cursor_stack.with_mut(|stack| {
                    // stack[cp] is the slot for the cursor that fetches page cp+1.
                    // Only push if we haven't stored it yet.
                    if stack.len() <= cp {
                        stack.push(Some(next_cursor.clone()));
                    }
                });
            }

            if response.history.is_empty() {
                toast.warning(
                    format!(
                        "No history data available for page {}",
                        *current_page.peek()
                    ),
                    ToastOptions::new(),
                );
            }
        } else if let Some(Err(e)) = history.read().as_ref() {
            toast.error(
                format!("Failed to load history: {}", e),
                ToastOptions::new(),
            );
        }
    });

    rsx! {
        style {
            "
            .note-row .delete-btn {{ opacity: 0; transition: opacity 0.15s ease; }}
            .note-row:hover .delete-btn {{ opacity: 1; }}
            .note-row:hover {{ background-color: #f9fafb; }}
            .note-input {{ transition: all 0.15s ease; }}
            .note-input:focus {{ background-color: #eff6ff; border-color: #3b82f6; outline: none; }}
            "
        }

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

                {
                    let history_read = history.read();
                    match history_read.as_ref().and_then(|r| r.as_ref().ok()) {
                        Some(response) => {
                            let initial = response.name.chars().next().unwrap_or('?').to_uppercase().to_string();
                            let name = response.name.clone();
                            rsx! {
                                div { style: "display: flex; align-items: center; gap: 0.75rem;",
                                    div { style: "flex-shrink: 0; width: 2.5rem; height: 2.5rem; border-radius: 50%; display: flex; align-items: center; justify-content: center; color: white; font-size: 0.875rem; font-weight: 700; background-color: #6366f1;",
                                        "{initial}"
                                    }
                                    h1 { class: "text-2xl font-semibold text-gray-900", "{name}" }
                                }
                            }
                        }
                        None => rsx! {
                            h1 { class: "text-2xl font-semibold text-gray-400", "Loading kid history..." }
                        },
                    }
                }
            }

            // ── Pagination Toolbar ──
            if total_count() > 0 {
                div { style: "margin-bottom: 1rem; display: flex; align-items: center; justify-content: space-between; gap: 1rem; padding: 0.75rem 1rem; border-radius: 0.75rem; border: 1px solid #e5e7eb; background: white; box-shadow: 0 1px 2px rgba(0,0,0,0.05);",

                    // Previous
                    button {
                        style: if current_page() == 1 {
                            "padding: 0.375rem 0.75rem; font-size: 0.875rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; cursor: not-allowed; color: #d1d5db; transition: all 0.15s;"
                        } else {
                            "padding: 0.375rem 0.75rem; font-size: 0.875rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; cursor: pointer; color: #9ca3af; transition: all 0.15s;"
                        },
                        disabled: current_page() == 1,
                        onclick: move |_| {
                            if current_page() > 1 {
                                let prev_page = current_page() - 1;
                                // cursor for page prev_page is at stack index prev_page - 1
                                let prev_cursor = cursor_stack
                                    .read()
                                    .get(prev_page - 1)
                                    .cloned()
                                    .flatten();
                                cursor.set(prev_cursor);
                                current_page.set(prev_page);
                                generation.set(generation() + 1);
                            }
                        },
                        "Previous"
                    }

                    span { style: "font-size: 0.875rem; font-weight: 500; color: #374151;",
                        "Page {current_page()} of {total_pages()} ({total_count()} total)"
                    }

                    // Next
                    button {
                        style: if !has_next() {
                            "padding: 0.375rem 0.75rem; font-size: 0.875rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; cursor: not-allowed; color: #d1d5db; transition: all 0.15s;"
                        } else {
                            "padding: 0.375rem 0.75rem; font-size: 0.875rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; cursor: pointer; color: #374151; transition: all 0.15s;"
                        },
                        disabled: !has_next(),
                        onclick: move |_| {
                            // cursor for page current_page+1 is at stack index current_page()
                            // (stack is 0-based: stack[N] fetches page N+1)
                            let next_cursor = cursor_stack
                                .read()
                                .get(current_page())
                                .cloned()
                                .flatten();
                            cursor.set(next_cursor);
                            current_page.set(current_page() + 1);
                            generation.set(generation() + 1);
                        },
                        "Next"
                    }
                }
            }

            // ── Cycle List ──
            {
                let history_read = history.read();
                match &*history_read {
                    None => rsx! {
                        div { style: "text-align: center; padding: 2rem; color: #9ca3af;", "Loading..." }
                    },
                    Some(Ok(response)) => {
                        if response.history.is_empty() {
                            rsx! {
                                div { style: "text-align: center; padding: 3rem; color: #9ca3af;",
                                    h3 { style: "font-size: 1.25rem; font-weight: 600; color: #374151; margin-bottom: 0.5rem;",
                                        "No History Found"
                                    }
                                    p { style: "color: #6b7280;", "No history data available for this kid." }
                                }
                            }
                        } else {
                            let items = response.history.clone();
                            let gen = generation();
                            rsx! {
                                HistoryList { items, page: gen }
                            }
                        }
                    }
                    Some(Err(_)) => rsx! {
                        div { style: "text-align: center; padding: 2rem; color: #d1d5db;", "No data available" }
                    },
                }
            }
        }
    }
}
