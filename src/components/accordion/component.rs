use dioxus::prelude::*;
use dioxus_primitives::accordion;

#[component]
pub fn Accordion(
    id: Option<String>,
    #[props(default)]
    allow_multiple_open: ReadSignal<bool>,
    #[props(default)]
    disabled: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(true)))]
    collapsible: ReadSignal<bool>,
    #[props(default)]
    horizontal: ReadSignal<bool>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    children: Element,
    #[props(default)]
    style: String,
) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("./style.css") }
        accordion::Accordion {
            class: "accordion",
            id: id,
            allow_multiple_open: allow_multiple_open,
            disabled: disabled,
            collapsible: collapsible,
            horizontal: horizontal,
            attributes: attributes,
            {children}
        }
    }
}

#[component]
pub fn AccordionItem(
    #[props(default)]
    disabled: ReadSignal<bool>,
    #[props(default)]
    default_open: bool,
    #[props(default)]
    on_change: Callback<bool, ()>,
    #[props(default)]
    on_trigger_click: Callback,
    index: usize,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
        accordion::AccordionItem {
            class: "accordion-item",
            disabled: disabled,
            default_open: default_open,
            on_change: on_change,
            on_trigger_click: on_trigger_click,
            index: index,
            attributes: attributes,
            {children}
        }
    }
}

#[component]
pub fn AccordionTrigger(
    id: Option<String>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
        accordion::AccordionTrigger {
            class: "accordion-trigger",
            id: id,
            attributes: attributes,
            {children}
            svg {
                class: "accordion-expand-icon",
                style: "margin-left: auto; margin-right: 0.5rem;",
                view_box: "0 0 24 24",
                xmlns: "http://www.w3.org/2000/svg",
                polyline { points: "6 9 12 15 18 9" }
            }
        }
    }
}

#[component]
pub fn AccordionContent(
    id: ReadSignal<Option<String>>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
        accordion::AccordionContent {
            class: "accordion-content",
            style: "--collapsible-content-width: 140px",
            id: id,
            attributes: attributes,
            {children}
        }
    }
}
