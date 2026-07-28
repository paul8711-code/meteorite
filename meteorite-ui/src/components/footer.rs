use dioxus::prelude::*;

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer {
            class: "fixed bottom-4 left-4 right-4 p-4 bg-black/25 border-2 border-white/20 rounded-full backdrop-blur-md flex justify-between items-center z-50",
            span {


                "{env!(\"CARGO_PKG_VERSION\")}"
            }
        }
    }
}
