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

use super::{UiState, components};
use dioxus::prelude::*;

#[component]
pub fn ErrorScreen(mut state: Signal<UiState>, message: String) -> Element {
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
