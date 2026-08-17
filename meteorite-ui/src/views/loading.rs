/*
    meteorite  Fast, Secure & Easy-to-use Matrix client in Rust
    Copyright (C) 2026  Paul8711

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as
    published by the Free Software Foundation, either version 3 of the
    License, or (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use super::{CLIENT, UiState, auth, components};
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
                if let Some(client) = client {
                    *CLIENT.write() = Some(client);
                    state.set(UiState::Main)
                } else {
                    state.set(UiState::Login);
                }
            }
            Err(e) => state.set(UiState::Error {
                message: e.to_string(),
            }),
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
                            size: "h-64 w-64",
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
