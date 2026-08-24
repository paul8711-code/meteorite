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

use super::{CLIENT, LoginStage, auth, components};
use dioxus::prelude::*;

#[component]
pub fn LoginScreen() -> Element {
    let mut current_stage = use_signal(LoginStage::default);
    let mut show_validation_errors = use_signal(|| false);

    let homeserver = use_signal(String::new);
    let username = use_signal(String::new);
    let password = use_signal(String::new);

    let mut error = use_signal(|| Option::<String>::None);
    let mut sso_link = use_signal(|| Option::<String>::None);
    let mut is_authenticating = use_signal(|| false);
    let mut current_task = use_signal(|| Option::<dioxus_core::Task>::None);

    let window_height = match current_stage() {
        LoginStage::Homeserver => "h-[275px]",
        LoginStage::Credentials => "h-[465px]",
    };

    let mut cancel_active_task = move || {
        if let Some(task) = current_task.take() {
            task.cancel();
        }
    };

    let mut check_homeserver = move || {
        cancel_active_task();

        error.set(None);

        is_authenticating.set(true);

        let hs = homeserver.read().clone();

        let task = spawn(async move {
            let handle = tokio::spawn(auth::get_login_types(hs));

            match handle.await {
                Ok(Ok(choices)) => {
                    todo!("handle login choices");
                }
                Ok(Err(e)) => {
                    error.set(Some(e.to_string()));
                }
                Err(_) => {
                    // *should* not happen
                    error.set(Some("Validation process aborted".into()));
                }
            }

            is_authenticating.set(false);
        });

        current_task.set(Some(task));
    };

    let cancel_login = move |_| {
        cancel_active_task();
        is_authenticating.set(false);
        sso_link.set(None);
    };

    let start_sso_login = move |_| {
        if is_authenticating() {
            return;
        }

        cancel_active_task();

        error.set(None);
        is_authenticating.set(true);

        let hs = homeserver.read().clone();

        let task = spawn(async move {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

            let mut handle = tokio::spawn(auth::login_sso(hs, tx));

            loop {
                tokio::select! {
                    Some(link) = rx.recv() => {
                        sso_link.set(Some(link));
                    }
                    res = &mut handle => {
                        match res {
                            Ok(Ok(c)) => {
                                *CLIENT.write() = Some(c);
                            }
                            Ok(Err(e)) => {
                                error.set(Some(e.to_string()));
                            }
                            Err(_) => {
                                // *should* not happen
                                error.set(Some("Authentication process aborted".into()));
                            }
                        }
                        break;
                    }
                }
            }

            is_authenticating.set(false);
        });
        current_task.set(Some(task));
    };

    let start_username_login = move |_| {
        if is_authenticating() {
            return;
        }

        cancel_active_task();

        if username.read().is_empty() || password.read().is_empty() {
            show_validation_errors.set(true);
            return;
        }

        show_validation_errors.set(false);
        is_authenticating.set(true);
        error.set(None);

        let hs = homeserver.read().clone();
        let user = username.read().clone();
        let pass = password.read().clone();

        let task = spawn(async move {
            let handle = tokio::spawn(auth::login_username(hs, user, pass));

            match handle.await {
                Ok(Ok(c)) => {
                    *CLIENT.write() = Some(c);
                }
                Ok(Err(e)) => {
                    error.set(Some(e.to_string()));
                }
                Err(_) => {
                    // *should* not happen
                    error.set(Some("Authentication process aborted".into()));
                }
            }

            is_authenticating.set(false);
        });

        current_task.set(Some(task));
    };

    rsx! {
        components::Bg {
            div {
                class: "relative, min-h-screen flex flex-col items-center justify-center p-4",
                if let Some(err_msg) = error() {
                    div {
                        class: "absolute top-12 z-50 w-full max-w-md p-4 rounded-lg bg-red-400 border-2 border-red-600 text-neutral-800 text-center shadow-lg transition-all duration-300 animate-fade-in",
                        "{err_msg}"
                    }
                }

                if is_authenticating() {
                    div {
                        class: "absolute inset-0 z-40 bg-black/40 flex flex-col items-center justify-center gap-4 backdrop-blur-sm",
                        components::Spinner {
                            size: "h-[50px] w-[50px]",
                        }
                        if let Some(link) = sso_link() {
                            div {
                                class: "p-3 bg-neutral-900 border border-neutral-700 rounded-lg text-white text-sm shadow-xl animate-fade-in",
                                "{link}"
                            }
                        }
                    }
                }

                div {
                    class: "w-full max-w-85 bg-neutral-800 border border-neutral-700 rounded-xl p-4 shadow-2xl transition-all duration-300 ease-in-out overflow-hidden {window_height}",
                    div {
                        class: "flex flex-col items-center mb-4",
                        components::Icon {
                            size: "h-16 w-16",
                        }
                    }

                    div {
                        class: "grid grid-cols-1 grid-rows-1",

                        div {
                            class: format_args!(
                                "flex flex-col gap-3 col-start-1 row-start-1 transition-all duration-200 ease-in-out {}",
                                if current_stage() == LoginStage::Homeserver {
                                    "opacity-100 delay-200 visible"
                                } else {
                                    "opacity-0 delay-0 invisible pointer-events-none"
                                }
                            ),

                            TextField {
                                label: "Homeserver",
                                prefix: "https://",
                                value: homeserver,
                                disabled: is_authenticating(),
                                show_error: show_validation_errors() && homeserver.read().is_empty(),
                            }

                            hr {
                                class: "border-neutral-700 my-2",
                            }

                            button {
                                class: "w-full py-2 bg-blue-600 hover:bg-blue-500 rounded-lg text-white font-medium transition-colors cursor-pointer",
                                onclick: move |_| {
                                    error.set(None);
                                    if homeserver.read().is_empty() {
                                        show_validation_errors.set(true);
                                    } else {
                                        show_validation_errors.set(false);
                                        check_homeserver();
                                        if error().is_none() && !is_authenticating() {
                                            current_stage.set(LoginStage::Credentials);
                                        }
                                    }
                                },
                                "Check"
                            }
                        }

                        div {
                            class: format_args!(
                                "flex flex-col gap-3 col-start-1 row-start-1 transition-all duration-200 ease-in-out {}",
                                if current_stage() == LoginStage::Credentials {
                                    "opacity-100 delay-200 visible"
                                } else {
                                    "opacity-0 delay-0 invisible pointer-events-none"
                                }
                            ),

                            TextField {
                                label: "Username",
                                prefix: "",
                                value: username,
                                disabled: is_authenticating(),
                                show_error: show_validation_errors() && username.read().is_empty(),
                            }
                            TextField {
                                label: "Password",
                                prefix: "",
                                value: password,
                                is_password: true,
                                disabled: is_authenticating(),
                                show_error: show_validation_errors() && password.read().is_empty(),
                            }

                            hr {
                                class: "border-neutral-700 my-1",
                            }

                            button {
                                class: "w-full py-2 bg-blue-600 hover:bg-blue-500 disabled:opacity-50 rounded-lg text-white font-medium transition-colors cursor-pointer",
                                disabled: is_authenticating(),
                                onclick: start_username_login,
                                "Login"
                            }
                            div {
                                class: "text-center text-xs text-neutral-400 font-semibold",
                                "or"
                            }
                            if !is_authenticating() {
                                button {
                                    class: "w-full py-2 bg-neutral-700 hover:bg-neutral-600 rounded-lg text-white text-sm transition-colors cursor-pointer",
                                    onclick: start_sso_login,
                                    "Login with Homeserver"
                                }
                            } else {
                                button {
                                    class: "z-60 w-full py-2 bg-red-600 hover:bg-red-500 rounded-lg text-white text-sm transition-colors cursor-pointer",
                                    onclick: cancel_login,
                                    "Cancel"
                                }
                            }
                            button {
                                class: "w-full py-2 bg-transparent hover:bg-neutral-700/50 rounded-lg text-neutral-400 text-sm transition-colors cursor-pointer",
                                disabled: is_authenticating(),
                                onclick: move |_| {
                                    error.set(None);
                                    show_validation_errors.set(false);
                                    current_stage.set(LoginStage::Homeserver);
                                },
                                "Back"
                            }
                        }
                    }
                }
                components::Footer {}
            }
        }
    }
}

#[component]
fn TextField(
    label: &'static str,
    #[props(default = "")] prefix: &'static str,
    mut value: Signal<String>,
    #[props(default = false)] is_password: bool,
    #[props(default = false)] disabled: bool,
    #[props(default = false)] show_error: bool,
) -> Element {
    let input_type = if is_password { "password" } else { "text" };

    rsx! {
        div {
            class: "flex flex-col gap-1 w-full",

            label {
                class: "text-xs font-semibold text-neutral-300 px-1",
                "{label}"
            }

            div {
                class: "flex items-center w-full bg-neutral-900 border border-neutral-700 focus-within:border-blue-500 rounded-md transition-all disabled:opacity-50",

                span {
                    class: "pointer-events-none select-none pl-3 text-neutral-500 text-sm",
                    "{prefix}"
                }

                input {
                    r#type: "{input_type}",
                    class: "w-full pl-1 pr-3 py-1.5 bg-transparent text-white text-sm outline-none",
                    value: "{value}",
                    disabled,
                    oninput: move |e| value.set(e.value()),
                }
            }

            if show_error {
                span {
                    class: "text-xs text-red-400 px-1 animate-fade-in",
                    "This field is required"
                }
            } else {
                div { class: "h-[15px]" }
            }
        }
    }
}
