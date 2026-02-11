use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn KidHistoryPage(kid_id: u32) -> Element {
    let show_all_notes = use_signal(|| true);

    rsx! {
        style { "
            .note-row .delete-btn {{ opacity: 0; transition: opacity 0.15s ease; }}
            .note-row:hover .delete-btn {{ opacity: 1; }}
            .note-row:hover {{ background-color: #f9fafb; }}
            .note-input {{ transition: all 0.15s ease; }}
            .note-input:focus {{ background-color: #eff6ff; border-color: #3b82f6; outline: none; }}
            .cycle-row:hover {{ background-color: #f9fafb; }}
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

            // ── View Toggle ──
            div { style: "margin-bottom: 1rem; display: flex; align-items: center; justify-content: center; gap: 1rem; padding: 0.75rem 1rem; border-radius: 0.75rem; border: 1px solid #e5e7eb; background: white; box-shadow: 0 1px 2px rgba(0,0,0,0.05);",
                div { style: "display: flex; align-items: center; gap: 0.25rem; border-radius: 0.5rem; padding: 0.25rem; background-color: #f3f4f6;",
                    button {
                        style: if show_all_notes() {
                            "padding: 0.5rem 1rem; font-size: 0.875rem; font-weight: 500; border-radius: 0.375rem; border: none; cursor: pointer; color: #374151; background: white; box-shadow: 0 1px 2px rgba(0,0,0,0.05);"
                        } else {
                            "padding: 0.5rem 1rem; font-size: 0.875rem; font-weight: 500; border-radius: 0.375rem; border: none; cursor: pointer; color: #9ca3af; background: transparent;"
                        },
                        onclick: move |_| show_all_notes.set(true),
                        "All Notes"
                    }
                    button {
                        style: if !show_all_notes() {
                            "padding: 0.5rem 1rem; font-size: 0.875rem; font-weight: 500; border-radius: 0.375rem; border: none; cursor: pointer; color: #374151; background: white; box-shadow: 0 1px 2px rgba(0,0,0,0.05);"
                        } else {
                            "padding: 0.5rem 1rem; font-size: 0.875rem; font-weight: 500; border-radius: 0.375rem; border: none; cursor: pointer; color: #9ca3af; background: transparent;"
                        },
                        onclick: move |_| show_all_notes.set(false),
                        "By Cycle"
                    }
                }
            }

            if *show_all_notes.read() {
                // ── Pagination Toolbar (All Notes) ──
                div { style: "margin-bottom: 1rem; display: flex; align-items: center; justify-content: center; gap: 1rem; padding: 0.75rem 1rem; border-radius: 0.75rem; border: 1px solid #e5e7eb; background: white; box-shadow: 0 1px 2px rgba(0,0,0,0.05);",
                    button {
                        style: "padding: 0.375rem 0.75rem; font-size: 0.875rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; cursor: pointer; color: #9ca3af; transition: all 0.15s;",
                        "Previous"
                    }
                    span { style: "font-size: 0.875rem; font-weight: 500; color: #374151;",
                        "1 / 5"
                    }
                    button {
                        style: "padding: 0.375rem 0.75rem; font-size: 0.875rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; cursor: pointer; color: #374151; transition: all 0.15s;",
                        "Next"
                    }
                }
            } else {
                // ── Cycle Selector (By Cycle) ──
                div { style: "margin-bottom: 1rem; display: flex; align-items: center; justify-content: center; gap: 1rem; padding: 0.75rem 1rem; border-radius: 0.75rem; border: 1px solid #e5e7eb; background: white; box-shadow: 0 1px 2px rgba(0,0,0,0.05);",
                    button {
                        style: "padding: 0.375rem 0.75rem; font-size: 0.875rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; cursor: pointer; color: #9ca3af; transition: all 0.15s;",
                        "Previous"
                    }
                    span { style: "font-size: 0.875rem; font-weight: 500; color: #374151;",
                        "February 2026"
                    }
                    button {
                        style: "padding: 0.375rem 0.75rem; font-size: 0.875rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; cursor: pointer; color: #374151; transition: all 0.15s;",
                        "Next"
                    }
                }
            }

            // ── Note List ──
            div { style: "border-radius: 0.75rem; border: 1px solid #e5e7eb; background: white; box-shadow: 0 1px 2px rgba(0,0,0,0.05); overflow: hidden;",

                if *show_all_notes.read() {
                    // All Notes view
                    for i in 0..30 {
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
                                style: "flex: 1; font-size: 0.875rem; color: #374151; border: 1px solid transparent; border-radius: 0.375rem; padding: 0.375rem 0.75rem; background: transparent; min-width: 0;",
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
                } else {
                    // By Cycle view
                    for i in 0..6 {
                        div {
                            class: "cycle-row",
                            style: "display: flex; align-items: center; justify-content: space-between; padding: 1rem 1.25rem; border-top: 1px solid #f3f4f6; transition: background-color 0.15s;",

                            div { style: "display: flex; align-items: center; gap: 1rem; flex: 1; min-width: 0;",
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

                            // Chevron to expand
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke_width: "2",
                                stroke: "currentColor",
                                class: "h-5 w-5",
                                style: "color: #9ca3af; flex-shrink: 0;",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    d: "m19.5 8.25-7.5 7.5-7.5-7.5",
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
            div { style: "margin-bottom: 1rem; display: flex; align-items: center; justify-content: center; gap: 1rem; padding: 0.75rem 1rem; border-radius: 0.75rem; border: 1px solid #e5e7eb; background: white; box-shadow: 0 1px 2px rgba(0,0,0,0.05);",
                button {
                    style: "padding: 0.375rem 0.75rem; font-size: 0.875rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; cursor: pointer; color: #9ca3af; transition: all 0.15s;",
                    "Previous"
                }
                span { style: "font-size: 0.875rem; font-weight: 500; color: #374151;",
                    "1 / 5"
                }
                button {
                    style: "padding: 0.375rem 0.75rem; font-size: 0.875rem; border-radius: 0.5rem; border: 1px solid #e5e7eb; background: white; cursor: pointer; color: #374151; transition: all 0.15s;",
                    "Next"
                }
            }

            // ── Note List ──
            div { style: "border-radius: 0.75rem; border: 1px solid #e5e7eb; background: white; box-shadow: 0 1px 2px rgba(0,0,0,0.05); overflow: hidden;",

                for i in 0..30 {
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
                            style: "flex: 1; font-size: 0.875rem; color: #374151; border: 1px solid transparent; border-radius: 0.375rem; padding: 0.375rem 0.75rem; background: transparent; min-width: 0;",
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
