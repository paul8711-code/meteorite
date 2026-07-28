use super::ICON;
use dioxus::prelude::*;

#[component]
pub fn Icon(size: String) -> Element {
    rsx! {
        img {
            class: "rounded-2xl object-cover {size}",
            src: ICON,
            alt: "meteorite",
        }
    }
}
