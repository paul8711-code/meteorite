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

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use meteorite_core::Client;
use meteorite_core::{auth, base_path, init};

mod components;
mod views;

use views::{error, loading, login, main};

#[derive(PartialEq, Clone, Copy, Default)]
enum LoginStage {
    #[default]
    Homeserver,
    Credentials,
}

// TODO: no states but instead functions for each state
#[derive(PartialEq, Clone)]
enum UiState {
    Loading,
    Error { message: String },
    Login,
    Main,
}

const ICON: Asset = asset!("/assets/icon/icon.png");

const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

static CLIENT: GlobalSignal<Option<Client>> = Signal::global(|| None::<Client>);

#[tokio::main]
async fn main() {
    let keyring_error = init::setup().err();

    // TODO: set icon
    LaunchBuilder::new()
        .with_context(keyring_error)
        .with_cfg(desktop! {
            Config::default()
                .with_data_directory(base_path())
                .with_menu(None)
                .with_window(WindowBuilder::new().with_title("meteorite"))
        })
        .launch(App);
}

#[component]
fn App() -> Element {
    let mut current_state = use_signal(|| UiState::Loading);
    let keyring_error = use_context::<Option<String>>();
    if let Some(e) = keyring_error {
        current_state.set(UiState::Error { message: e })
    }
    let _keyring_guard = use_signal(|| meteorite_core::KeyringGuard);

    rsx! {
        // TODO: adjust title based on what the user is doing, e.g. (3) meteorite - Matrix HQ
        document::Title {
            "meteorite"
        }
        document::Link {
            rel: "stylesheet",
            href: MAIN_CSS,
        }
        document::Link {
            rel: "stylesheet",
            href: TAILWIND_CSS,
        }

        div {
            class: "app-container font-sans text-base antialiased bg-neutral-900 text-white min-h-screen p-4",

            match &*current_state.read() {
                UiState::Loading => rsx! {
                    loading::LoadingScreen {
                        state: current_state,
                    }
                },
                UiState::Error { message } => rsx! {
                    error::ErrorScreen {
                        state: current_state,
                        message: message.clone(),
                    }
                },
                UiState::Login => rsx! {
                    login::LoginScreen {
                        state: current_state,
                    }
                },
                UiState::Main => rsx! {
                    main::MainScreen {}
                },
            }
        }
    }
}
