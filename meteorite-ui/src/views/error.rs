use super::{ErrorKind, UiState, components};
use dioxus::prelude::*;

#[component]
pub fn ErrorScreen(mut state: Signal<UiState>, kind: ErrorKind, message: String) -> Element {
    use_effect(move || {
        if kind == ErrorKind::NoAccountActive {
            *state.write() = UiState::Login;
        }
    });

    match kind {
        ErrorKind::NoAccountActive => {
            rsx! {}
        }
        ErrorKind::Other => {
            rsx! {
                DisplayError {
                    message,
                }
            }
        }
    }
}

#[component]
fn DisplayError(message: String) -> Element {
    rsx! {
        components::Bg {
            div {
                div {
                    class: "fixed top-[50px] left-1/2 -translate-x-1/2 z-50 flex flex-col items-center transition-opacity duration-300 ease-in-out opacity-100 animate-fade-in",
                    div {
                        class: "p-4 rounded-[10px] bg-[#ff7878] border-[3px] border-[#ff0000] text-[#141414] font-medium shadow-lg text-center",
                        "{message}"
                    }

                    div {
                        class: "h-[50px]",
                    }

                    div {
                        class: "flex items-center justify-center",
                        components::Icon {
                            size: "w-[256px] h-[256px]",
                        }
                    }
                    // TODO: possibly add retry button
                }
                components::Footer {}
            }
        }
    }
}
