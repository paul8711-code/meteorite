use super::{CLIENT, ErrorKind, UiState, auth, components};
use dioxus::prelude::*;

#[component]
pub fn LoadingScreen(mut state: Signal<UiState>) -> Element {
    let mut starts_fading = use_signal(|| false);

    use_future(move || async move {
        let login_result = auth::login().await;

        starts_fading.set(true);
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;

        match login_result {
            Ok(client) => {
                *CLIENT.write() = Some(client);
                state.set(UiState::Main)
            }
            Err(e) => {
                let kind = match e {
                    auth::LoginError::NoAccountActive => ErrorKind::NoAccountActive,
                    auth::LoginError::Other(_) => ErrorKind::Other,
                };

                state.set(UiState::Error {
                    kind,
                    message: e.to_string(),
                });
            }
        }
    });

    let opacity = if starts_fading() {
        "opacity-0"
    } else {
        "opacity-100"
    };

    rsx! {
        components::Bg {
            div {
                class: "{opacity} transition-opacity duration-250 ease-in-out",
                div {
                    class: "flex flex-col items-center min-h-screen relative",
                    div {
                        class: "mt-12",
                        components::Icon {
                            size: "h-[256px] w-[256px]",
                        }
                    }

                    div {
                        class: "text-center mt-16",
                        components::Spinner {
                            size: "h-32 w-32",
                        }
                    }
                }
                components::Footer {}
            }
        }
    }
}
