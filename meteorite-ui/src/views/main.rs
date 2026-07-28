use dioxus::prelude::*;

#[component]
pub fn MainScreen() -> Element {
    rsx! {
        div {
            class: "flex h-screen w-screen font-sans m-0 p-0",
            div {
                class: "w-[75px] min-w-[75px] flex flex-col items-center border-r border-neutral-600",

                div {
                    class: "h-[5px]",
                }

                button {
                    onclick: move |_| println!("home"),
                    class: "w-[50px] h-[50px] rounded-[20px] m-1 p-0 bg-none overflow-hidden border border-neutral-600",

                    img {
                        src: asset!("/assets/home.png"),
                        width: "50",
                        height: "50",
                        class: "block",
                    }
                }

                hr {
                    class: "w-[80%] mx-[10px] my-0 border-t border-neutral-600",
                }
            }

            div {
                class: "grow shrink basis-0 p-4",
                h1 {
                    "meteorite"
                }
                button {
                    class: "border border-neutral-600 rounded-xl p-1",
                    onclick: move |_| println!("clicked button"),
                    "test"
                }
            }

            div {
                class: "w-[350px] min-w-[350px] p-4 border-l border-neutral-600",
                button {
                    class: "border border-neutral-600 rounded-xl p-1",
                    onclick: move |_| println!("settings"),
                    "settings"
                }
            }
        }
    }
}
