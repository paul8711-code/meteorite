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
pub struct KeyringGuard;

impl Drop for KeyringGuard {
    fn drop(&mut self) {
        // this ensures that even if the program exits with an error, the keyring store is unset.
        keyring_core::unset_default_store();
    }
}
