#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dioxus::prelude::*;
use meteorite_core::Client;
use meteorite_core::{auth, base_path, init};
use native_dialog::MessageLevel;

mod components;
mod utils;
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
    Error { kind: ErrorKind, message: String },
    Login,
    Main,
}

#[derive(PartialEq, Clone, Copy)]
enum ErrorKind {
    NoAccountActive,
    Other,
}

const ICON: Asset = asset!("/assets/icon/icon.png");

const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[tokio::main]
async fn main() {
    // all errors are handled within the setup function
    if let Err(e) = init::setup() {
        match e {
            init::SetupError::Keyring(s) => {
                utils::show_dialog_window("Keyring Error", s, MessageLevel::Error);
            }
            init::SetupError::Folder(s) => {
                utils::show_dialog_window("Folder Error", s, MessageLevel::Error);
            }
        }
        return;
    }
    let _keyring_guard = meteorite_core::KeyringGuard;

    // TODO: set icon
    let mut builder = LaunchBuilder::new();

    #[cfg(all(not(target_os = "android"), not(target_os = "ios")))]
    {
        use dioxus::desktop::{Config, WindowBuilder};
        builder = builder.with_cfg(
            Config::default()
                .with_data_directory(base_path())
                .with_menu(None)
                .with_window(WindowBuilder::new().with_title("meteorite")),
        );
    }

    builder.launch(App);
}

#[component]
fn App() -> Element {
    let current_state = use_signal(|| UiState::Loading);
    // TODO: switch to use_context_provider
    let client = use_signal(|| None::<Client>);

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
                UiState::Error { kind, message } => rsx! {
                    error::ErrorScreen {
                        state: current_state,
                        kind: *kind,
                        message: message.clone(),
                    }
                },
                UiState::Login => rsx! {
                    login::LoginScreen {
                        state: current_state,
                        client,
                    }
                },
                UiState::Main => rsx! {
                    main::MainScreen {}
                },
            }
        }
    }
}
