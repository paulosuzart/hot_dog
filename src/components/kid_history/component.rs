use crate::components::accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn KidHistoryPage(kid_id: u32) -> Element {
    let mut current_page = use_signal(|| 1usize);
    let total_pages = use_signal(|| 5usize);
    let items_per_page = 10usize;

    rsx! {
        style { "
            .note-row .delete-btn {{ opacity: 0; transition: opacity 0.15s ease; }}
            .note-row:hover .delete-btn {{ opacity: 1; }}
            .note-row:hover {{ background-color: #f9fafb; }}
            .note-input {{ transition: all 0.15s ease; }}
            .note-input:focus {{ background-color: #eff6ff; border-color: #3b82f6; outline: none; }}
            .accordion-item {{ border: none; background: transparent; }}
            .accordion-trigger {{ padding: 0; background: transparent; border: none; width: 100%; display: flex; align-items: center; justify-content: space-between; }}
            .accordion-expand-icon {{ transition: transform 0.2s ease; }}
            [data-expanded] > .accordion-trigger > .accordion-expand-icon {{ transform: rotate(180deg); }}
            .accordion-content {{ border-top: 1px solid #f3f4f6; width: 100% !important; }}
            .accordion {{ width: 100% !important; }}
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

                div { style: "display: flex; align-items: center; gap: 0.75rem;",
                    div {
                        style: "flex-shrink: 0; width: 2.5rem; height: 2.5rem; border-radius: 50%; display: flex; align-items: center; justify-content: center; color: white; font-size: 0.875rem; font-weight: 700; background-color: #6366f1;",
                        "J"
                    }
                    h1 { class: "text-2xl font-semibold text-gray-900", "Junior" }
                }
            }

            // ── Pagination Toolbar ──
            div { style: "margin-bottom: 1rem; display: flex; align-items: center; justify-content: center; gap: 1rem; padding: 0.75rem 1rem; border-radius: 0.75rem; border: 1px solid #e5e7eb; background: white; box-shadow: 0 1px 2px rgba(0,0,0,0.05);",
                button {
                    style: if current_page() == 1 {
                        "padding: 0.375rem 0.75rem; font-size: 0.875rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; cursor: not-allowed; color: #d1d5db; transition: all 0.15s;"
                    } else {
                        "padding: 0.375rem 0.75rem; font-size: 0.875rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; cursor: pointer; color: #9ca3af; transition: all 0.15s;"
                    },
                    disabled: current_page() == 1,
                    onclick: move |_| {
                        if current_page() > 1 {
                            current_page -= 1;
                        }
                    },
                    "Previous"
                }
                span { style: "font-size: 0.875rem; font-weight: 500; color: #374151;",
                    "{current_page()} / {total_pages()}"
                }
                button {
                    style: if current_page() == total_pages() {
                        "padding: 0.375rem 0.75rem; font-size: 0.875rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; cursor: not-allowed; color: #d1d5db; transition: all 0.15s;"
                    } else {
                        "padding: 0.375rem 0.75rem; font-size: 0.875rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; cursor: pointer; color: #374151; transition: all 0.15s;"
                    },
                    disabled: current_page() == total_pages(),
                    onclick: move |_| {
                        if current_page() < total_pages() {
                            current_page += 1;
                        }
                    },
                    "Next"
                }
            }

            // ── Cycle List (Accordion) ──
            div { style: "border-radius: 0.75rem; border: 1px solid #e5e7eb; background: white; box-shadow: 0 1px 2px rgba(0,0,0,0.05); overflow: hidden; width: 100%;",
                Accordion {
                    style: "width: 100%;",
                    allow_multiple_open: true,

                    for i in ((current_page() - 1) * items_per_page)..std::cmp::min(current_page() * items_per_page, 60) {
                        AccordionItem { index: i,
                            AccordionTrigger {
                                div { style: "display: flex; align-items: center; gap: 1rem; flex: 1; min-width: 0; padding: 1rem 1rem;",
                                    // Cycle label
                                    div {
                                        style: "display: flex; flex-direction: column; gap: 0.25rem; min-width: 120px;",
                                        span { style: "font-size: 0.875rem; font-weight: 600; color: #111827;",
                                            "February 2026"
                                        }
                                        span { style: "font-size: 0.75rem; color: #9ca3af;",
                                            "12 notes"
                                        }
                                    }

                                    // Total count badge
                                    span { style: "flex-shrink: 0; font-size: 1.5rem; font-weight: 700; padding: 0.5rem 1rem; border-radius: 0.5rem; background-color: #dcfce7; color: #16a34a;",
                                        "+24"
                                    }
                                }
                            }

                            AccordionContent {
                                div { style: "background-color: #f9fafb; width: 100%;",

                                    for j in 0..3usize {
                                        div {
                                            class: "note-row",
                                            style: "display: flex; align-items: center; gap: 0.75rem; padding: 0.75rem 1rem; border-top: 1px solid #f3f4f6; transition: background-color 0.15s;",

                                            // Date
                                            span { style: "flex-shrink: 0; font-size: 0.75rem; color: #9ca3af; width: 140px;",
                                                "Feb 10, 2026 14:30"
                                            }

                                            // Quantity badge
                                            span { style: "flex-shrink: 0; font-size: 0.75rem; font-weight: 600; padding: 0.25rem 0.5rem; border-radius: 0.375rem; background-color: #dcfce7; color: #16a34a;",
                                                "+1"
                                            }

                                            // Note text (editable input)
                                            input {
                                                class: "note-input",
                                                style: "flex: 1; font-size: 0.875rem; color: #374151; border: 1px solid transparent; border-radius: 0.375rem; padding: 0.375rem 0.75rem; background: white; min-width: 0;",
                                                r#type: "text",
                                                value: "Good behavior today",
                                            }

                                            // Delete button
                                            button {
                                                class: "delete-btn",
                                                style: "flex-shrink: 0; display: flex; align-items: center; justify-content: center; width: 1.5rem; height: 1.5rem; border-radius: 0.375rem; border: none; cursor: pointer; color: #ef4444; background: transparent; transition: all 0.15s;",
                                                title: "Delete note",
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
                            }
                        }
                    }
                }
            }
        }
    }
}


