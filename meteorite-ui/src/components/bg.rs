use dioxus::prelude::*;

#[component]
pub fn Bg(children: Element) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 bg-gradient-to-b from-[#141414] to-[#00003c] z-0 overflow-hidden",
            div {
                class: "relative z-10 w-full h-full",
                {children}
            }
        }
    }
}
