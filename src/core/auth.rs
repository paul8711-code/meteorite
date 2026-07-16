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
use std::collections::HashMap;
use std::{fs, path::PathBuf};
use tokio::sync::mpsc;

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

// guard to clean up failed account creations
struct AccountCreationGuard {
    backup_path: PathBuf,
    backup_created: bool,

    users_path: PathBuf,

    sqlite_path: PathBuf,
    encrypted_path: PathBuf,

    users_tmp_path: PathBuf,
    encrypted_tmp_path: PathBuf,
    backup_tmp_path: PathBuf,

    keyring_entry: Entry,
    keyring_created: bool,

    committed: bool,
}

impl AccountCreationGuard {
    fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for AccountCreationGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.encrypted_tmp_path);
        let _ = fs::remove_file(&self.users_tmp_path);
        let _ = fs::remove_file(&self.backup_tmp_path);

        if self.committed {
            return;
        }

        let _ = fs::remove_file(&self.encrypted_path);
        let _ = fs::remove_dir_all(&self.sqlite_path);

        if self.backup_created {
            let _ = fs::rename(&self.backup_path, &self.users_path);
        }

        if self.keyring_created {
            let _ = self.keyring_entry.delete_credential();
        }
    }
}

#[derive(Debug)]
enum AccountResource {
    Folder(String, PathBuf),
    File(String, PathBuf),
    KeyringEntry(String),
    User(String),
}

const USER: u8 = 1 << 0;
const FILE: u8 = 1 << 1;
const FOLDER: u8 = 1 << 2;
const KEYRING: u8 = 1 << 3;

const ALL: u8 = USER | FILE | FOLDER | KEYRING;

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
    // first remove possible leftovers
    remove_orphaned_accounts();

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
pub async fn login_sso(
    homeserver: &str,
    tx: mpsc::UnboundedSender<String>,
) -> Result<Client, LoginError> {
    // first remove possible leftovers
    remove_orphaned_accounts();

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

    let backup_tmp_path = account_path.join("users.toml.backup.tmp");
    let backup_path = account_path.join("users.toml.backup");
    let users_tmp_path = account_path.join("users.toml.tmp");
    let users_path = account_path.join("users.toml");
    let encrypted_tmp_path = account_path.join(format!("{id}.enc.tmp"));
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
            if webbrowser::open(&sso_url).is_ok() {
                tx.send("Go to the opened website to authenticate".to_string())
                    .ok();
            } else {
                tx.send(format!("Navigate to {sso_url} in a browser of choice"))
                    .ok();
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

    // create guard as soon as possible
    let mut guard = AccountCreationGuard {
        backup_path: backup_path.clone(),
        backup_tmp_path: backup_tmp_path.clone(),
        backup_created: false,

        users_path: users_path.clone(),

        sqlite_path: sqlite_path.clone(),
        encrypted_path: encrypted_path.clone(),

        users_tmp_path: users_tmp_path.clone(),
        encrypted_tmp_path: encrypted_tmp_path.clone(),

        keyring_entry: encryption_passphrase_entry,
        keyring_created: false,

        committed: false,
    };

    // read unencrypted file with all users
    let mut accounts: AccountList = if users_path.exists() {
        let toml_account_data = fs::read_to_string(&users_path).map_err(|e| anyhow::anyhow!(e))?;
        toml::from_str(&toml_account_data).map_err(|e| anyhow::anyhow!(e))?
    } else {
        AccountList::default()
    };

    // create backup of accounts
    let toml_account_data = toml::to_string(&accounts).map_err(|e| anyhow::anyhow!(e))?;
    fs::write(&backup_tmp_path, &toml_account_data).map_err(|e| anyhow::anyhow!(e))?;
    fs::rename(&backup_tmp_path, &backup_path).map_err(|e| anyhow::anyhow!(e))?;

    guard.backup_created = true;

    // set all accounts active to false (for the new account to be active)
    accounts.accounts.iter_mut().for_each(|a| a.active = false);

    // add new account to vector
    accounts.accounts.push(AccountData {
        id,
        user_id: response.user_id,
        active: true,
    });

    let toml_account_data = toml::to_string(&accounts).map_err(|e| anyhow::anyhow!(e))?;

    // write bytes to encrypted file
    fs::write(&encrypted_tmp_path, &encrypted_bytes).map_err(|e| anyhow::anyhow!(e))?;

    fs::rename(&encrypted_tmp_path, &encrypted_path).map_err(|e| anyhow::anyhow!(e))?;

    // write unecnrypted file
    fs::write(&users_tmp_path, toml_account_data).map_err(|e| anyhow::anyhow!(e))?;

    fs::rename(&users_tmp_path, &users_path).map_err(|e| anyhow::anyhow!(e))?;

    // save encryption passphrase to keyring (used for file encryption and db encryption)
    guard
        .keyring_entry
        .set_password(&encryption_passphrase)
        .map_err(|e| anyhow::anyhow!(e))?;

    guard.keyring_created = true;

    let _ = fs::remove_file(&backup_path);

    guard.commit();

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

fn remove_orphaned_accounts() {
    let account_path = utils::unwrap_lock(&ACCOUNT_PATH);
    let users_path = account_path.join("users.toml");

    if !users_path.exists() {
        return;
    }

    // - collect vector of:
    //  - sqlite folders
    //  - encrypted files
    //  - accounts
    //  - keyring entries
    let mut resources = Vec::new();

    // get the user list
    let Ok(toml_account_data) = fs::read_to_string(&users_path) else {
        return;
    };

    let mut accounts: AccountList = match toml::from_str(&toml_account_data) {
        Ok(data) => data,
        Err(_) => return,
    };

    resources.extend(
        accounts
            .accounts
            .iter()
            .map(|account| AccountResource::User(account.id.clone())),
    );

    for entry in match fs::read_dir(&account_path) {
        Ok(entries) => entries,
        Err(_) => return,
    }
    .flatten()
    {
        // collect everything into resources vec
        let path = entry.path();

        // filter out only the folders
        if path.is_dir()
            && let Some(name) = entry.file_name().to_str().map(str::to_owned)
        {
            resources.push(AccountResource::Folder(name, path.clone()));
        }

        // filter out only the files except the users.toml file (so it doesnt get deleted)
        if path.is_file()
            && path != users_path
            && let Some(name) = entry.file_name().to_str().map(str::to_owned)
        {
            resources.push(AccountResource::File(name, path));
        }
    }

    match Entry::search(&[("service", APP_NAME)].into()) {
        Ok(entries) => resources.extend(
            entries
                .iter()
                .filter_map(|entry| Some(AccountResource::KeyringEntry(entry.get_specifiers()?.1))),
        ),
        Err(_) => return,
    }

    // all values in resource vector now

    // hashmap of all seen (if all are seen: 1111, each bit represents one variant seen)
    let mut seen: HashMap<&str, u8> = HashMap::new();

    for resource in &resources {
        let (name, bit) = match resource {
            AccountResource::User(s) => (s.as_str(), USER),
            AccountResource::File(s, _) => (s.as_str(), FILE),
            AccountResource::Folder(s, _) => (s.as_str(), FOLDER),
            AccountResource::KeyringEntry(s) => (s.as_str(), KEYRING),
        };

        *seen.entry(name).or_default() |= bit;
    }

    for resource in &resources {
        let name = match resource {
            AccountResource::User(s)
            | AccountResource::File(s, _)
            | AccountResource::Folder(s, _)
            | AccountResource::KeyringEntry(s) => s.as_str(),
        };

        if let Some(mask) = seen.get(name)
            && *mask != ALL
        {
            match resource {
                AccountResource::File(_, path) => {
                    // delete file
                    fs::remove_file(path).ok();
                }

                AccountResource::Folder(_, path) => {
                    // delete folder
                    fs::remove_dir_all(path).ok();
                }

                AccountResource::KeyringEntry(name) => {
                    // delete keyring entry
                    let Ok(entry) = Entry::new(APP_NAME, name) else {
                        return;
                    };
                    entry.delete_credential().ok();
                }

                AccountResource::User(s) => {
                    // remove user from users.toml
                    accounts.accounts.retain(|account| &account.id != s);
                }
            }
        }
    }

    let Ok(toml_account_data) = toml::to_string(&accounts) else {
        return;
    };

    fs::write(&users_path, toml_account_data).ok();
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
