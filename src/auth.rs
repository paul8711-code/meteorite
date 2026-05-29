use keyring_core::Entry;
use matrix_sdk::{
    Client, authentication::matrix::MatrixSession, ruma::UserId, store::RoomLoadSettings,
};
use random_string::generate;
use rpassword::read_password;
use std::io;

pub async fn login(
    app_name: &str,
    keyring_db_pass: &str,
    keyring_session: &str,
    storage_str: &str,
) -> anyhow::Result<Client> {
    // this is for the db (i think sql but not sure, i wrote this code way too long ago)
    let charset = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let db_entry = Entry::new(app_name, keyring_db_pass)?;
    let db_pass = match db_entry.get_password() {
        Ok(p) => p,
        Err(e) => {
            dbg!("keyring error (db pass): {}", e);
            let new_p = generate(32, charset); // generate random 32 digit password
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
        println!("enter homeserver (without https://)");
        let mut homeserver = String::new();
        io::stdin()
            .read_line(&mut homeserver)
            .expect("Failed to read line");
        homeserver
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
