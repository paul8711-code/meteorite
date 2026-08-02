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

use native_dialog::{DialogBuilder, MessageLevel};

// helper for displaying a dialog message (usually indicating that something went wrong)
pub fn show_dialog_window(title: impl ToString, text: impl ToString, level: MessageLevel) {
    DialogBuilder::message()
        .set_title(title)
        .set_text(text)
        .set_level(level)
        .alert()
        .show()
        .unwrap();
}
