use crate::{ACCOUNT_PATH, APP_NAME, core::utils};
use age::secrecy::SecretString;
use keyring_core::Entry;
use matrix_sdk::{
    Client, SessionMeta, SessionTokens,
    authentication::matrix::MatrixSession,
    ruma::{OwnedDeviceId, OwnedUserId},
    store::RoomLoadSettings,
};
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

// This file contains all functions for authenticating a user. That includes loading necessary
// config files, getting passwords from keyring and using matrix sdks functions to send requests to
// the users homeserver.

// SESSION STORING
// store sessions in encrypted files (using age crate) and store passphrase to tht in keyring
// user_ids are stored in unencrypted with information if its active or not

// stores both the encrypted data and the unencrypted data
pub struct Account {
    data: AccountData,
    encrypted_data: EncryptedAccountData,
}

// since almost all values in the account struct are the same as in matrix session struct this is
// pretty simple
impl From<&Account> for MatrixSession {
    fn from(account: &Account) -> Self {
        Self {
            meta: SessionMeta {
                user_id: account.data.user_id.clone(),
                device_id: account.encrypted_data.device_id.clone(),
            },
            tokens: SessionTokens {
                access_token: account.encrypted_data.access_token.clone(),
                refresh_token: account.encrypted_data.refresh_token.clone(),
            },
        }
    }
}

// this struct only exists for toml
#[derive(Deserialize, Serialize, Clone)]
struct AccountList {
    accounts: Vec<AccountData>,
}

// stored in unencrypted file, lets the client decide which data to load
#[derive(Deserialize, Serialize, Clone)]
struct AccountData {
    user_id: OwnedUserId,
    active: bool,
}

// stored in encrypted file, passphrase stored in keyring
// only loaded when required
#[derive(Deserialize, Serialize)]
struct EncryptedAccountData {
    access_token: String,
    refresh_token: Option<String>,
    device_id: OwnedDeviceId,
}

// custom error type to let the ui know what to display
#[derive(thiserror::Error, Debug, PartialEq, Clone)]
pub enum LoginError {
    #[error("No account is active")]
    NoAccountActive,
    #[error("IO Error: {0}")]
    IoError(String),
    #[error("Keyring Error: {0}")]
    KeyringError(String),
    #[error("Encryption Error: {0}")]
    EncryptionError(String),
    #[error("Matrix Error: {0}")]
    MatrixError(String),
}

// the login function tries to login the active user (if config exists), if it fails error is returned (so
// the gui can handle logging in itself)
pub async fn login() -> Result<Client, LoginError> {
    // first read unencrypted file with all users
    let users_path = PathBuf::from(utils::unwrap_lock(&ACCOUNT_PATH)).join("users.toml");
    if !users_path.exists() {
        return Err(LoginError::NoAccountActive);
    }
    let toml_account_data =
        fs::read_to_string(users_path).map_err(|e| LoginError::IoError(e.to_string()))?;
    let accounts: AccountList = toml::from_str(&toml_account_data)
        .map_err(|e| LoginError::IoError(e.message().to_string()))?;

    // filter the accounts for only active accounts
    let mut active_accounts = accounts.accounts.iter().filter(|a| a.active);
    // if multiple, no account is active
    let account_data = match (active_accounts.next(), active_accounts.next()) {
        (Some(account), None) => Some(account),
        _ => None,
    }
    .ok_or(LoginError::NoAccountActive)?;

    // retrieve encryption passphrase from keyring (used for file encryption and db encryption)
    let encryption_passphrase = Entry::new(APP_NAME, account_data.user_id.as_ref())
        .map_err(|e| LoginError::KeyringError(e.to_string()))?;
    let encryption_passphrase = encryption_passphrase
        .get_password()
        .map_err(|e| LoginError::KeyringError(e.to_string()))?;

    // load encrypted file contents
    let mut f = fs::File::open(
        PathBuf::from(utils::unwrap_lock(&ACCOUNT_PATH)).join(format!(
            "{}.enc",
            account_data.user_id.as_str().replace(['@', ':'], "")
        )),
    )
    .map_err(|e| LoginError::IoError(e.to_string()))?;
    let mut data = vec![];
    f.read_to_end(&mut data)
        .map_err(|e| LoginError::IoError(e.to_string()))?;

    // get identity from passphrase
    let identity = age::scrypt::Identity::new(SecretString::from(encryption_passphrase.clone()));

    // actually decrypt the file contents
    let decrypted_bytes =
        age::decrypt(&identity, &data).map_err(|e| LoginError::EncryptionError(e.to_string()))?;

    // saved in toml, using toml crates from_slice function to read into struct directly from bytes
    let encrypted_data: EncryptedAccountData =
        toml::from_slice(&decrypted_bytes).map_err(|e| LoginError::IoError(e.to_string()))?;

    // construct the client
    let client = Client::builder()
        .server_name_or_homeserver_url(account_data.user_id.server_name())
        .sqlite_store(
            PathBuf::from(utils::unwrap_lock(&ACCOUNT_PATH))
                .join(account_data.user_id.as_str().replace(['@', ':'], "")),
            Some(&encryption_passphrase), // same as for encrypted files
        )
        .build()
        .await
        .map_err(|e| LoginError::MatrixError(e.to_string()))?;

    // merge account data and encrypted account data in our unified struct
    let account = Account {
        data: account_data.clone(),
        encrypted_data,
    };

    // restore session from the unified account struct
    client
        .matrix_auth()
        .restore_session(MatrixSession::from(&account), RoomLoadSettings::default())
        .await
        .map_err(|e| LoginError::MatrixError(e.to_string()))?;
    Ok(client)
}
/*
pub async fn login(
    app_name: &str,
    keyring_db_pass: &str,
    keyring_session: &str,
    storage_str: &str,
) -> anyhow::Result<Client> {
    // this is for the db
    let db_entry = Entry::new(app_name, keyring_db_pass)?;
    let db_pass = match db_entry.get_password() {
        Ok(p) => p,
        Err(e) => {
            dbg!("keyring error (db pass): {}", e); // TODO: change this to display error in gui
            let new_p = Alphanumeric.sample_string(&mut rand::rng(), 32); // generate random 32 digit password
            db_entry.set_password(&new_p)?;
            new_p.to_string()
        }
    };

    let session_entry = Entry::new(app_name, keyring_session)?;

    let homeserver_url = if let Ok(session_json) = session_entry.get_password() {
        let session: MatrixSession = serde_json::from_str(&session_json)?;
        let user = &session.meta.user_id;
        user.server_name().to_string()
    } else {
        dbg!("no session found, please login");
        /*
        println!("enter homeserver (without https://)");
        let mut homeserver = String::new();
        io::stdin()
            .read_line(&mut homeserver)
            .expect("Failed to read line");
        homeserver
        */
        String::from("matrix.org")
    };

    // this is now the real client

    let client = Client::builder()
        .server_name_or_homeserver_url(&homeserver_url)
        .sqlite_store(storage_str, Some(&db_pass))
        .build()
        .await?;

    // nobody change this, i almost got depressions here
    if let Ok(session_json) = session_entry.get_password() {
        // parse session and restore
        let session: MatrixSession = serde_json::from_str(&session_json)?;
        // restore session with access token
        client
            .matrix_auth()
            .restore_session(session, RoomLoadSettings::default())
            .await?;
        dbg!("Session was in keyring"); // yay it worked
    } else {
        // session not in keyring, one time login

        println!("(1) login with username");
        println!("(2) login with sso");

        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");

        let choice_num: i32 = choice.trim().parse().expect("Not a valid number");

        if choice_num == 1 {
            // input things are self explanatory
            println!("input user name (without homeserver)");

            let mut user_inp = String::new();

            io::stdin()
                .read_line(&mut user_inp)
                .expect("Failed to read line");

            user_inp = user_inp.trim().to_string();

            let user_final = format!("@{}:{}", user_inp, &homeserver_url);

            let user = UserId::parse(&user_final)?;

            println!("input password");
            let password_inp = read_password().unwrap();

            // why did i make this span across multiple lines? nobody knows
            // but anyways, this is login
            let _response = client
                .matrix_auth()
                .login_username(&user, &password_inp)
                .initial_device_display_name("meteorite Client")
                .send()
                .await?;
        } else if choice_num == 2 {
            let _response = client
                .matrix_auth()
                .login_sso(|sso_url| async move {
                    if webbrowser::open(&sso_url).is_ok() {
                        println!("Go to the opened website to authenticate");
                    }
                    Ok(())
                })
                .initial_device_display_name("meteorite Client")
                .await
                .unwrap();
        } else {
            println!("invalid input");
        }

        // put session in keyring
        if let Some(auth_session) = client.session()
            && let matrix_sdk::AuthSession::Matrix(session) = auth_session
        {
            let json = serde_json::to_string(&session)?;
            session_entry.set_password(&json)?;
            dbg!("success! login is now in keyring");
        }
    }

    Ok(client)
}
*/
