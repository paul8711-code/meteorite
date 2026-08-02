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

use crate::{ACCOUNT_PATH, APP_NAME, BASE_PATH, utils};
use std::fs;
use std::sync::Mutex;

// all functions in this file are used to inititalize something (e.g. set default keyring store)

pub enum SetupError {
    /// An error occured when setting up the default keyring store
    Keyring(String),
    /// An error occured when creating necessary directories
    Folder(String),
}

/// Initializes important variables and sets the default keyring store.
///
/// # Errors
/// If any of the setup steps fail, an error containing the message for the UI to display is returned.
pub fn setup() -> Result<(), SetupError> {
    match setup_keyring() {
        Ok(()) => {}
        Err(e) => {
            return Err(SetupError::Keyring(format!(
                "The application failed to set up the keyring store.\n\nDetails: {e}"
            )));
        }
    }
    match setup_folders() {
        Ok(()) => {}
        Err(e) => {
            return Err(SetupError::Folder(format!(
                "The application failed to set up required folders.\n\nDetails: {e}"
            )));
        }
    }
    Ok(())
}

// sets default keyring store depending on os you are on
fn setup_keyring() -> anyhow::Result<()> {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    keyring_core::set_default_store(zbus_secret_service_keyring_store::Store::new()?);
    #[cfg(target_os = "windows")]
    keyring_core::set_default_store(windows_native_keyring_store::Store::new()?);
    #[cfg(target_os = "macos")]
    // TODO: transition to protected in the future?
    // possibly protected only on mobile and keychain on desktop.
    // protected requires code-signed application (99$/year)
    keyring_core::set_default_store(apple_native_keyring_store::keychain::Store::new()?);

    // TODO: add keyring store for android
    Ok(())
}

// sets some path variables and creates necessary folders
fn setup_folders() -> anyhow::Result<()> {
    let base_path = utils::local_data_dir(APP_NAME).ok_or(anyhow::anyhow!(
        "The application was unable to find the data path",
    ))?;
    // set() can only return an error when it has already been set, which in this case cannot
    // happen.
    BASE_PATH.set(Mutex::new(base_path)).unwrap();

    // shadow the upper variable because it is not required anymore
    let base_path = utils::unwrap_lock(&BASE_PATH);
    // create dirs on first run
    if !base_path.exists() {
        fs::create_dir_all(&base_path)?;
    }

    // account path is literally just base path with an extra folder
    let account_path = base_path.join("accounts");
    ACCOUNT_PATH.set(Mutex::new(account_path)).unwrap();

    let account_path = utils::unwrap_lock(&ACCOUNT_PATH);
    // also create dirs
    if !account_path.exists() {
        fs::create_dir_all(&account_path)?;
    }
    Ok(())
}
