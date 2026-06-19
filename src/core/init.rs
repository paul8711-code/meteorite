use crate::{ACCOUNT_PATH, APP_NAME, BASE_PATH};
use native_dialog::MessageLevel;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::core::utils;

// all functions in this file are used to inititalize something (e.g. set default keyring store)

// this function calls all other init functions and handles errors, if any
// interesting return type, i know but it lets main know that there was an error and drop _guard safely
pub async fn setup() -> Result<(), ()> {
    match setup_keyring().await {
        Ok(_) => {}
        Err(e) => {
            utils::show_dialog_window(
                "Keyring Error",
                format!(
                    "The application failed to set up the keyring store.\n\nDetails: {}",
                    e
                ),
                MessageLevel::Error,
            );
            return Err(());
        }
    }
    match setup_folders().await {
        Ok(_) => {}
        Err(e) => {
            utils::show_dialog_window(
                "Folder Error",
                format!(
                    "The application failed to set up required folders.\n\nDetails: {}",
                    e
                ),
                MessageLevel::Error,
            );
            return Err(());
        }
    }
    Ok(())
}

// sets default keyring store depending on os you are on
async fn setup_keyring() -> anyhow::Result<()> {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    keyring_core::set_default_store(zbus_secret_service_keyring_store::Store::new()?);
    #[cfg(target_os = "windows")]
    keyring_core::set_default_store(windows_native_keyring_store::Store::new()?);
    #[cfg(target_os = "macos")]
    keyring_core::set_default_store(apple_native_keyring_store::Store::new()?);
    Ok(())
}

// sets some path variables and creates necessary folders
async fn setup_folders() -> anyhow::Result<()> {
    let base_path = dirs::data_local_dir()
        .ok_or(anyhow::anyhow!(
            "The application was unable to find the data path",
        ))?
        .join(APP_NAME)
        .into_os_string() // stupid overcomplicated conversion
        .into_string()
        .map_err(|_| anyhow::anyhow!("Path contains invalid UTF-8"))?;
    // set() can only return an error when it has already been set, which in this case cannot
    // happen.
    BASE_PATH.set(Mutex::new(base_path)).unwrap();

    // shadow the upper variable because it is not required anymore
    let base_path = PathBuf::from(utils::unwrap_lock(&BASE_PATH));
    // create dirs on first run
    if !base_path.exists() {
        fs::create_dir_all(&base_path)?;
    }

    // account path is literally just base path with an extra folder
    let account_path = base_path.join("accounts");
    ACCOUNT_PATH
        .set(Mutex::new(
            account_path
                .into_os_string() // stupid conversion again
                .into_string()
                .map_err(|_| anyhow::anyhow!("Path contains invalid UTF-8"))?,
        ))
        .unwrap();

    let account_path = PathBuf::from(utils::unwrap_lock(&ACCOUNT_PATH));
    // also create dirs
    if !account_path.exists() {
        fs::create_dir_all(&account_path)?;
    }
    Ok(())
}
