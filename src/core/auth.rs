use crate::{ACCOUNT_PATH, APP_NAME, core::utils};
use age::secrecy::SecretString;
use keyring_core::Entry;
use matrix_sdk::{
    Client, SessionMeta, SessionTokens,
    authentication::matrix::MatrixSession,
    ruma::{
        OwnedDeviceId, OwnedUserId,
        time::{Duration, SystemTime},
    },
    store::RoomLoadSettings,
};
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use std::fs;

// This file contains all functions for authenticating a user. That includes loading necessary
// config files, getting passwords from keyring and using matrix sdks functions to send requests to
// the users homeserver.

// SESSION STORING
// store sessions in encrypted files (using age crate) and store passphrase to tht in keyring
// user_ids are stored in unencrypted with information if its active or not

// this struct only exists for toml
#[derive(Default, Deserialize, Serialize, Clone)]
struct AccountList {
    accounts: Vec<AccountData>,
}

// stored in unencrypted file, lets the client decide which data to load
#[derive(Deserialize, Serialize, Clone)]
struct AccountData {
    // id used for files instead of user_id
    id: String,
    user_id: OwnedUserId,
    active: bool,
}

// stored in encrypted file, passphrase stored in keyring
// only loaded when required
#[derive(Deserialize, Serialize)]
struct EncryptedAccountData {
    access_token: String,
    refresh_token: Option<String>,
    expiration: Option<SystemTime>,
    device_id: OwnedDeviceId,
}

impl EncryptedAccountData {
    fn new(
        access_token: String,
        refresh_token: Option<String>,
        expires_in: Option<Duration>,
        device_id: OwnedDeviceId,
    ) -> Self {
        let expiration = if let Some(expires_in) = expires_in {
            // overflow impossible (for now)
            SystemTime::now().checked_add(expires_in)
        } else {
            None
        };

        Self {
            access_token,
            refresh_token,
            expiration,
            device_id,
        }
    }
}

// custom error type to let the ui know what to display
#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error("No account is active")]
    NoAccountActive,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// TODO: split functions into helpers

// logs in active account by restoring persisted matrix session
pub async fn login() -> Result<Client, LoginError> {
    let account_path = utils::unwrap_lock(&ACCOUNT_PATH);
    let users_path = account_path.join("users.toml");

    if !users_path.exists() {
        return Err(LoginError::NoAccountActive);
    }
    // first read unencrypted file with all users
    let toml_account_data = fs::read_to_string(users_path).map_err(|e| anyhow::anyhow!(e))?;
    let accounts: AccountList =
        toml::from_str(&toml_account_data).map_err(|e| anyhow::anyhow!(e))?;

    // filter the accounts for only active accounts
    let mut active_accounts = accounts.accounts.iter().filter(|a| a.active);
    // if multiple, no account is active
    let account_data = match (active_accounts.next(), active_accounts.next()) {
        (Some(account), None) => Some(account),
        _ => None,
    }
    .ok_or(LoginError::NoAccountActive)?;

    // define the paths once
    let encrypted_path = account_path.join(format!("{}.enc", account_data.id));
    let sqlite_path = account_path.join(&account_data.id);

    // retrieve encryption passphrase from keyring (used for file encryption and db encryption)
    let encryption_passphrase_entry =
        Entry::new(APP_NAME, &account_data.id).map_err(|e| anyhow::anyhow!(e))?;
    let encryption_passphrase = encryption_passphrase_entry
        .get_password()
        .map_err(|e| anyhow::anyhow!(e))?;

    // load encrypted file contents
    let data = fs::read(&encrypted_path).map_err(|e| anyhow::anyhow!(e))?;

    // get identity from passphrase
    let identity = age::scrypt::Identity::new(SecretString::from(encryption_passphrase.as_str()));

    // actually decrypt the file contents
    let decrypted_bytes = age::decrypt(&identity, &data).map_err(|e| anyhow::anyhow!(e))?;

    // deserialize decrypted toml into stored account data
    let decrypted_account_data: EncryptedAccountData =
        toml::from_slice(&decrypted_bytes).map_err(|e| anyhow::anyhow!(e))?;

    // construct the client
    let client = Client::builder()
        .server_name_or_homeserver_url(account_data.user_id.server_name())
        .sqlite_store(
            sqlite_path,
            Some(&encryption_passphrase), // same as for encrypted files
        )
        .build()
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // TODO: check if access token expired
    // if yes:
    // -> refresh using refresh token
    // -> save new token and expiration date

    // restore session from the unified account struct
    client
        .matrix_auth()
        .restore_session(
            matrix_session_from_account(account_data, &decrypted_account_data),
            RoomLoadSettings::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(client)
}

// authenticates a user via sso and saves the account config locally
pub async fn login_sso(homeserver: &str) -> Result<Client, LoginError> {
    // initialize rng for later usage
    let (id, encryption_passphrase) = {
        let mut rng = rand::rng();
        // generate id for usage on sqlite store and other files
        let id = Alphanumeric.sample_string(&mut rng, 32);
        // generate passphrase for sqlite store and encrypted files
        let encryption_passphrase = Alphanumeric.sample_string(&mut rng, 32);
        (id, encryption_passphrase)
    };

    // define the paths once
    let account_path = utils::unwrap_lock(&ACCOUNT_PATH);

    let users_path = account_path.join("users.toml");
    let encrypted_path = account_path.join(format!("{id}.enc"));
    let sqlite_path = account_path.join(&id);

    fs::create_dir_all(&account_path).map_err(|e| anyhow::anyhow!(e))?;

    // construct the client
    let client = Client::builder()
        .server_name_or_homeserver_url(homeserver)
        .sqlite_store(&sqlite_path, Some(&encryption_passphrase))
        .build()
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // start sso login
    let response = client
        .matrix_auth()
        .login_sso(|sso_url| async move {
            // TODO: let ui know about the url
            if webbrowser::open(&sso_url).is_ok() {
                println!("Go to the opened website to authenticate");
            } else {
                println!("Navigate to {sso_url} in a browser of choice");
            }
            Ok(())
        })
        .initial_device_display_name("meteorite Client")
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    // construct new encrypted account data from response
    let account_data = EncryptedAccountData::new(
        response.access_token,
        response.refresh_token,
        response.expires_in,
        response.device_id,
    );

    // account_data struct -> toml
    let serialized = toml::to_string(&account_data).map_err(|e| anyhow::anyhow!(e))?;

    // get recipient from passphrase
    let recipient = age::scrypt::Recipient::new(SecretString::from(encryption_passphrase.as_str()));

    // encrypt account data
    let encrypted_bytes =
        age::encrypt(&recipient, serialized.as_bytes()).map_err(|e| anyhow::anyhow!(e))?;

    let encryption_passphrase_entry = Entry::new(APP_NAME, &id).map_err(|e| anyhow::anyhow!(e))?;

    // read unencrypted file with all users
    let mut accounts: AccountList = if users_path.exists() {
        let toml_account_data = fs::read_to_string(&users_path).map_err(|e| anyhow::anyhow!(e))?;
        let mut accounts: AccountList =
            toml::from_str(&toml_account_data).map_err(|e| anyhow::anyhow!(e))?;

        // set all accounts active to false (for the new account to be active)
        accounts.accounts.iter_mut().for_each(|a| a.active = false);
        accounts
    } else {
        AccountList::default()
    };

    // add new account to vector
    accounts.accounts.push(AccountData {
        id,
        user_id: response.user_id,
        active: true,
    });

    let toml_account_data = toml::to_string(&accounts).map_err(|e| anyhow::anyhow!(e))?;

    // TODO: handle orphaned accounts

    // save encryption passphrase to keyring (used for file encryption and db encryption)
    encryption_passphrase_entry
        .set_password(&encryption_passphrase)
        .map_err(|e| anyhow::anyhow!(e))?;

    // write bytes to encrypted file
    fs::write(&encrypted_path, &encrypted_bytes).map_err(|e| anyhow::anyhow!(e))?;

    // write unecnrypted file
    fs::write(&users_path, toml_account_data).map_err(|e| anyhow::anyhow!(e))?;

    Ok(client)
}

fn matrix_session_from_account(
    data: &AccountData,
    encrypted: &EncryptedAccountData,
) -> MatrixSession {
    MatrixSession {
        meta: SessionMeta {
            user_id: data.user_id.clone(),
            device_id: encrypted.device_id.clone(),
        },
        tokens: SessionTokens {
            access_token: encrypted.access_token.clone(),
            refresh_token: encrypted.refresh_token.clone(),
        },
    }
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
