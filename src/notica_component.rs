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
                div { class: "p-6",
                    div { class: "overflow-x-auto",
                        p { "No kids found." }
                        Button { "Add Kid" }
                    }
                }
            },
            KidsResponseWrapper::Loaded(data) => {
                let aggregation = &data.count_metadata.aggregation;
                let agg_label = aggregation.label();
                let agg_unit = aggregation.unit();

                rsx! {
                    div { class: "space-y-4",
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
                    footer { class: "mt-8 rounded-lg border border-gray-200 bg-gray-50 px-5 py-4",
                        div { class: "flex items-center justify-between",
                            div { class: "flex items-center gap-4",
                                div {
                                    p { class: "text-xs font-medium uppercase tracking-wide text-gray-400",
                                        "Aggregation"
                                    }
                                    p { class: "text-sm font-semibold text-gray-700",
                                        "{agg_label}"
                                    }
                                }
                                div { class: "h-8 w-px bg-gray-200" }
                                div {
                                    p { class: "text-xs font-medium uppercase tracking-wide text-gray-400",
                                        "Current unit"
                                    }
                                    p { class: "text-sm font-semibold text-gray-700",
                                        "{agg_unit}"
                                    }
                                }
                            }
                            Link {
                                to: Route::SettingsView,
                                class: "flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium text-gray-600 transition-colors hover:bg-gray-200 hover:text-gray-900",
                                svg {
                                    xmlns: "http://www.w3.org/2000/svg",
                                    fill: "none",
                                    view_box: "0 0 24 24",
                                    stroke_width: "1.5",
                                    stroke: "currentColor",
                                    class: "h-5 w-5",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7.723 7.723 0 0 1 0 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.248a1.125 1.125 0 0 1-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.47 6.47 0 0 1-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 0 1-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 0 1-1.369-.49l-1.297-2.247a1.125 1.125 0 0 1 .26-1.431l1.004-.827c.292-.24.437-.613.43-.991a6.932 6.932 0 0 1 0-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 0 1-.26-1.43l1.297-2.247a1.125 1.125 0 0 1 1.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28Z"
                                    }
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"
                                    }
                                }
                                "Settings"
                            }
                        }
                    }
                }
            }
            KidsResponseWrapper::Loading => rsx! {
                div { class: "p-6",
                    div { class: "overflow-x-auto",
                        p { "Loading..." }
                    }
                }
            },
        }
    }
}
