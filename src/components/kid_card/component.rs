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
                        "Add"
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| on_decrement.call(kid_id),
                        "Remove"
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
