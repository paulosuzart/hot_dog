use crate::components::{button::*, card::*};
use crate::models::Kid;
use dioxus::prelude::*;

#[component]
pub fn KidCard(
    kid: Kid,
    on_increment: EventHandler<u32>,
    on_decrement: EventHandler<u32>,
) -> Element {
    let kid_id = kid.id;

    rsx! {
        Card {
            CardHeader {
                CardTitle { "{kid.name}" }
                CardDescription {
                    p { "Count: {kid.count}" }
                }
                CardAction {
                    Button {
                        variant: ButtonVariant::Destructive,
                        onclick: move |_| on_increment.call(kid_id),
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "16",
                            height: "16",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
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
                    Button {
                        style: "background-color: #026031; color: #fff;",
                        onclick: move |_| on_decrement.call(kid_id),
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "16",
                            height: "16",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
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
                }
            }
            CardContent {
                p { "Latest note: {kid.latest_note}" }
            }
            CardFooter {
                Button { variant: ButtonVariant::Primary, "Stats" }
            }
        }
    }
}
