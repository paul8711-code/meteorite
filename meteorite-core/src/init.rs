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

use crate::{ACCOUNT_PATH, APP_NAME, BASE_PATH};
use std::fs;
use std::sync::Mutex;

// all functions in this file are used to inititalize something (e.g. set default keyring store)

/// Initializes important variables and sets the default keyring store.
///
/// # Errors
/// If any of the setup steps fail, an error containing the message for the UI to display is returned.
pub fn setup() -> Result<(), String> {
    match setup_keyring() {
        Ok(()) => {}
        Err(e) => {
            return Err(format!(
                "The application failed to set up the keyring store.\n\nDetails: {e}"
            ));
        }
    }
    setup_folders();

    Ok(())
}

// sets default keyring store depending on os you are on
fn setup_keyring() -> Result<(), keyring_core::Error> {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    keyring_core::set_default_store(zbus_secret_service_keyring_store::Store::new()?);
    #[cfg(target_os = "windows")]
    keyring_core::set_default_store(windows_native_keyring_store::Store::new().unwrap());
    #[cfg(target_os = "macos")]
    // TODO: transition to protected in the future?
    // protected requires code-signed application (which we should do anyways, possibly just use self-signed one to avoid paying)
    keyring_core::set_default_store(apple_native_keyring_store::keychain::Store::new().unwrap());

    // TODO: add keyring store for android
    Ok(())
}

// sets some path variables and creates necessary folders
fn setup_folders() {
    let base_path = sysdirs::data_local_dir().unwrap().join(APP_NAME);
    let account_path = base_path.join("accounts");

    // create dirs on first run
    if !base_path.exists() {
        // should only panic if perms are set wrong
        fs::create_dir_all(&base_path).unwrap_or_else(|e| {
            panic!(
                "failed to create application data directory {:?}: {e}",
                base_path
            )
        });
    }
    if !account_path.exists() {
        fs::create_dir_all(&account_path).unwrap_or_else(|e| {
            panic!(
                "failed to create application data directory {:?}: {e}",
                account_path
            )
        });
    }

    // set() can only return an error when it has already been set, which in this case cannot
    // happen.
    BASE_PATH.set(Mutex::new(base_path)).unwrap();
    ACCOUNT_PATH.set(Mutex::new(account_path)).unwrap();
}
