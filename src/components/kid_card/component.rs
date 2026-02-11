use crate::components::button::*;
use crate::models::Kid;
use crate::Route;
use dioxus::prelude::*;

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
pub fn KidCard(
    kid: Kid,
    on_increment: EventHandler<u32>,
    on_decrement: EventHandler<u32>,
) -> Element {
    let kid_id = kid.id;
    let initial = kid
        .name
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();
    let color = kid_color(&kid.name);
    let latest_note = kid
        .latest_note
        .map(|dt| dt.format("%b %d, %Y %H:%M").to_string())
        .unwrap_or_else(|| "No notes for this cycle".to_string());

    rsx! {
        div { style: "border: 1px solid #e5e7eb; border-radius: 0.75rem; background: #fff; box-shadow: 0 1px 2px rgba(0,0,0,0.05); overflow: hidden;",

            // ── Header row: avatar + name + count + buttons ──
            div { style: "display: flex; align-items: center; gap: 0.875rem; padding: 1rem 1.25rem;",

                // Avatar + Name (linked to history)
                Link {
                    to: Route::KidHistory { id: kid.id },
                    style: "flex: 1; min-width: 0; display: flex; align-items: center; gap: 0.875rem; text-decoration: none;",
                    div { style: "flex-shrink: 0; width: 2.5rem; height: 2.5rem; border-radius: 50%; display: flex; align-items: center; justify-content: center; color: white; font-size: 0.875rem; font-weight: 700; background-color: {color}; transition: transform 0.15s ease;",
                        "{initial}"
                    }
                    div { style: "flex: 1; min-width: 0;",
                        p { style: "font-size: 1rem; font-weight: 600; color: #111827; line-height: 1.3;",
                            "{kid.name}"
                        }
                        p { style: "font-size: 0.8125rem; color: #9ca3af; margin-top: 2px;",
                            "Count: "
                            span { style: "font-weight: 600; color: #374151; font-size: 0.875rem;",
                                "{kid.count}"
                            }
                        }
                    }
                }

                // Action buttons
                div { style: "display: flex; align-items: center; gap: 0.5rem; flex-shrink: 0;",
                    // Decrement (red minus)
                    Button {
                        style: "background-color: #fee2e2; color: #dc2626; padding: 8px; border-radius: 0.5rem;",
                        onclick: move |_| on_decrement.call(kid_id),
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "16",
                            height: "16",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2.5",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            line {
                                x1: "5",
                                y1: "12",
                                x2: "19",
                                y2: "12",
                            }
                        }
                    }
                    // Increment (green plus)
                    Button {
                        style: "background-color: #dcfce7; color: #16a34a; padding: 8px; border-radius: 0.5rem;",
                        onclick: move |_| on_increment.call(kid_id),
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "16",
                            height: "16",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2.5",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            line {
                                x1: "12",
                                y1: "5",
                                x2: "12",
                                y2: "19",
                            }
                            line {
                                x1: "5",
                                y1: "12",
                                x2: "19",
                                y2: "12",
                            }
                        }
                    }
                }
            }

            // ── Footer: latest note ──
            div { style: "padding: 0.625rem 1.25rem; background-color: #f9fafb; border-top: 1px solid #f3f4f6; display: flex; align-items: center; justify-content: space-between;",
                p { style: "font-size: 0.75rem; color: #9ca3af;",
                    "Latest note: "
                    span { style: "color: #6b7280;", "{latest_note}" }
                }
            }
        }
    }
}
