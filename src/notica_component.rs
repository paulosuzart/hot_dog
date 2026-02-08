use crate::{backend::kids::get_kids, components::card::*, components::button::*};
use chrono::DateTime;
use dioxus::{fullstack::serde::Serialize, prelude::*};
use serde::Deserialize;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CountAggregation {
    Monthly(u8),
    Weekly(u8),
    Daily(u8),
    Yearly(u8),
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CountMetadata {
    pub aggregation: CountAggregation,
}

pub enum KidsResponseWrapper {
    Loading,
    Loaded(GetKidsResponse),
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GetKidsResponse {
    pub kids: Vec<Kid>,
    pub count_metadata: CountMetadata,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Kid {
    pub name: String,
    pub id: u32,
    pub count: u8,
    pub latest_note: DateTime<chrono::Utc>,
}

#[component]
pub fn NoticaApp() -> Element {
    let mut kids = use_signal(|| KidsResponseWrapper::Loading);

    let _ = use_resource(move || async move {
        match get_kids().await {
            Ok(k) => {
                kids.set(KidsResponseWrapper::Loaded(k));
            }
            Err(_) => (),
        }
    });

    rsx! {
        match &*kids.read() {
            KidsResponseWrapper::Loaded(data) => rsx! {
                div { class: "p-6",
                    div { class: "overflow-x-auto",
                        {data.kids.iter().map(|kid| rsx! {
                            Card {
                                CardHeader {
                                    CardTitle { "{kid.name}" }
                                    CardDescription { "Card description goes here ." }
                                    CardAction {
                                        Button { variant: ButtonVariant::Destructive, "Action" }
                                        Button { variant: ButtonVariant::Secondary, "Action" }
                                    }
                                }
                                CardContent {
                                    p { "Main content of the card." }
                                }
                                CardFooter {
                                    button { "Submit" }
                                }
                            }
                        })}
                    }
                }
            },
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
