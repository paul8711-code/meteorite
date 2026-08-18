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

pub mod auth;
pub mod init;
pub mod utils;

pub use matrix_sdk::Client;

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

pub const APP_NAME: &str = "com.paul8711.meteorite";
static BASE_PATH: OnceLock<Mutex<PathBuf>> = OnceLock::new();
static ACCOUNT_PATH: OnceLock<Mutex<PathBuf>> = OnceLock::new();

/// Guard used to safely unset the default keyring store on exit
#[derive(Clone)]
pub struct KeyringGuard;

impl Drop for KeyringGuard {
    fn drop(&mut self) {
        // this ensures that even if the program exits with an error, the keyring store is unset.
        keyring_core::unset_default_store();
    }
}

#[must_use]
pub fn base_path() -> PathBuf {
    utils::unwrap_lock(&BASE_PATH)
}
