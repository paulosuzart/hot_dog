use crate::backend::kids::{decrement_kid_count, get_kids, increment_kid_count};
use crate::components::{button::*, kid_card::*};
use crate::models::KidsResponseWrapper;
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn NoticaApp() -> Element {
    let mut kids = use_signal(|| KidsResponseWrapper::Loading);

    let mut rs = use_resource(move || async move {
        match get_kids().await {
            Ok(k) => {
                if k.kids.is_empty() {
                    kids.set(KidsResponseWrapper::NoKids);
                } else {
                    kids.set(KidsResponseWrapper::Loaded(k));
                }
            }
            Err(_) => (),
        }
    });

    let kids_snapshot = kids.read().clone();

    rsx! {
        match kids_snapshot {
            KidsResponseWrapper::NoKids => rsx! {
                // ── Empty state ──
                div { style: "border: 1px solid #e5e7eb; border-radius: 0.75rem; background: #fff; box-shadow: 0 1px 2px rgba(0,0,0,0.05); padding: 3rem 1.5rem; text-align: center;",
                    // Empty icon
                    div { style: "margin: 0 auto 1rem; width: 3rem; height: 3rem; border-radius: 50%; background-color: #f3f4f6; display: flex; align-items: center; justify-content: center;",
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            fill: "none",
                            view_box: "0 0 24 24",
                            stroke_width: "1.5",
                            stroke: "#9ca3af",
                            style: "width: 1.5rem; height: 1.5rem;",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                d: "M15 19.128a9.38 9.38 0 0 0 2.625.372 9.337 9.337 0 0 0 4.121-.952 4.125 4.125 0 0 0-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 0 1 8.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0 1 11.964-3.07M12 6.375a3.375 3.375 0 1 1-6.75 0 3.375 3.375 0 0 1 6.75 0Zm8.25 2.25a2.625 2.625 0 1 1-5.25 0 2.625 2.625 0 0 1 5.25 0Z",
                            }
                        }
                    }
                    p { style: "font-size: 1rem; font-weight: 600; color: #111827; margin-bottom: 0.25rem;",
                        "No kids yet"
                    }
                    p { style: "font-size: 0.875rem; color: #9ca3af; margin-bottom: 1.25rem;",
                        "Head to settings to add your first kid."
                    }
                    Link {
                        to: Route::SettingsView,
                        style: "display: inline-flex; align-items: center; gap: 0.5rem; padding: 0.5rem 1rem; border-radius: 0.5rem; background-color: #111; color: #fff; font-size: 0.875rem; font-weight: 500; text-decoration: none; transition: background-color 0.15s;",
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            fill: "none",
                            view_box: "0 0 24 24",
                            stroke_width: "2",
                            stroke: "currentColor",
                            style: "width: 1rem; height: 1rem;",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                d: "M12 4.5v15m7.5-7.5h-15",
                            }
                        }
                        "Add Kids"
                    }
                }
            },
            KidsResponseWrapper::Loaded(data) => {
                let aggregation = &data.count_metadata.aggregation;
                let agg_label = aggregation.label();
                let agg_unit = aggregation.unit_str();

                rsx! {
                    // ── Kid cards ──
                    div { style: "display: flex; flex-direction: column; gap: 0.75rem;",
                        {
                            data.kids
                                .into_iter()
                                .map(|kid| {
                                    rsx! {
                                        KidCard {
                                            kid,
                                            on_increment: move |kid_id: u32| async move {
                                                match increment_kid_count(kid_id).await {
                                                    Ok(_) => rs.restart(),
                                                    Err(e) => {
                                                        eprintln!(
                                                            "Failed to increment count for kid with ID: {}: {:?}",
                                                            kid_id,
                                                            e,
                                                        )
                                                    }
                                                }
                                            },
                                            on_decrement: move |kid_id: u32| async move {
                                                match decrement_kid_count(kid_id).await {
                                                    Ok(_) => rs.restart(),
                                                    Err(e) => {
                                                        eprintln!(
                                                            "Failed to decrement count for kid with ID: {}: {:?}",
                                                            kid_id,
                                                            e,
                                                        )
                                                    }
                                                }
                                            },
                                        }
                                    }
                                })
                        }
                    }

                    // ── Footer: aggregation info + settings link ──
                    footer { style: "margin-top: 1.25rem; border-radius: 0.75rem; border: 1px solid #e5e7eb; background: #fff; box-shadow: 0 1px 2px rgba(0,0,0,0.05); padding: 1rem 1.25rem;",
                        div { style: "display: flex; align-items: center; justify-content: space-between;",
                            div { style: "display: flex; align-items: center; gap: 1.25rem;",
                                div {
                                    p { style: "font-size: 0.625rem; font-weight: 500; text-transform: uppercase; letter-spacing: 0.05em; color: #9ca3af;",
                                        "Aggregation"
                                    }
                                    p { style: "font-size: 0.875rem; font-weight: 600; color: #374151; margin-top: 1px;",
                                        "{agg_label}"
                                    }
                                }
                                div { style: "width: 1px; height: 1.75rem; background-color: #e5e7eb;" }
                                div {
                                    p { style: "font-size: 0.625rem; font-weight: 500; text-transform: uppercase; letter-spacing: 0.05em; color: #9ca3af;",
                                        "Current unit"
                                    }
                                    p { style: "font-size: 0.875rem; font-weight: 600; color: #374151; margin-top: 1px;",
                                        "{agg_unit}"
                                    }
                                }
                            }
                            Link {
                                to: Route::SettingsView,
                                style: "display: flex; align-items: center; gap: 0.375rem; padding: 0.5rem 0.75rem; border-radius: 0.5rem; font-size: 0.8125rem; font-weight: 500; color: #6b7280; text-decoration: none; transition: all 0.15s;",
                                svg {
                                    xmlns: "http://www.w3.org/2000/svg",
                                    fill: "none",
                                    view_box: "0 0 24 24",
                                    stroke_width: "1.5",
                                    stroke: "currentColor",
                                    style: "width: 1.125rem; height: 1.125rem;",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7.723 7.723 0 0 1 0 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.248a1.125 1.125 0 0 1-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.47 6.47 0 0 1-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 0 1-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 0 1-1.369-.49l-1.297-2.247a1.125 1.125 0 0 1 .26-1.431l1.004-.827c.292-.24.437-.613.43-.991a6.932 6.932 0 0 1 0-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 0 1-.26-1.43l1.297-2.247a1.125 1.125 0 0 1 1.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28Z",
                                    }
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z",
                                    }
                                }
                                "Settings"
                            }
                        }
                    }
                }
            }
            KidsResponseWrapper::Loading => rsx! {
                // ── Loading state ──
                div { style: "border: 1px solid #e5e7eb; border-radius: 0.75rem; background: #fff; box-shadow: 0 1px 2px rgba(0,0,0,0.05); padding: 3rem 1.5rem; text-align: center;",
                    p { style: "font-size: 0.875rem; color: #9ca3af;", "Loading..." }
                }
            },
        }
    }
}
