extern crate rpassword;

use matrix_sdk::{
    Client,
    config::SyncSettings,
    RoomState,
    store::RoomLoadSettings,
    authentication::matrix::{
        MatrixSession
    },
    ruma::events::room::{
    message::RoomMessageEventContent
    },
    ruma::{UserId, RoomId, RoomOrAliasId, RoomAliasId, events::room::message::SyncRoomMessageEvent},
};

use std::io;
use rpassword::read_password;
use keyring::Entry;
use std::fs;
use random_string::generate;
use std::dbg;

const APP_NAME: &str = "meteorite_client";
const KEYRING_SESSION: &str = "meteorite_session_json";
const KEYRING_DB_PASS: &str = "meteorite_db_password";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut storage_path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    storage_path.push(".meteorite");

    if !storage_path.exists() {
        fs::create_dir_all(&storage_path)?;
    }

    let storage_str = storage_path.to_str().expect("Path invalid");

    let charset = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let db_entry = Entry::new(APP_NAME, KEYRING_DB_PASS)?;
    let db_pass = match db_entry.get_password() {
        Ok(p) => p,
        Err(e) => {
            dbg!("keyring error (db pass): {}", e);
            let new_p = generate(32, charset); // generate random 32 digit password
            db_entry.set_password(&new_p)?;
            new_p.to_string()
        }
    };

    let mut client = Client::builder().server_name_or_homeserver_url("matrix.org").build().await?;

    let session_entry = Entry::new(APP_NAME, KEYRING_SESSION)?;

    if let Ok(session_json) = session_entry.get_password() {
        // parse session and restore
        let session: MatrixSession = serde_json::from_str(&session_json)?;
        let user = &session.meta.user_id;
        client = Client::builder().server_name_or_homeserver_url(user.server_name()).sqlite_store(storage_str, Some(&db_pass)).build().await?;
        client.matrix_auth().restore_session(session, RoomLoadSettings::default()).await?;
        dbg!("Session was in keyring");
    } else {
        // session not in keyring, one time login
        dbg!("no session found, please login");

        println!("input user name");

        let mut user_inp = String::new();

        io::stdin()
            .read_line(&mut user_inp)
            .expect("Failed to read line");

        user_inp = user_inp.trim().to_string();

        let user = UserId::parse(&user_inp)?;

        println!("input password");
        let password_inp = read_password().unwrap();

        client = Client::builder().server_name_or_homeserver_url(user.server_name()).sqlite_store(storage_str, Some(&db_pass)).build().await?;

        let response = client
            .matrix_auth()
            .login_username(&user, &password_inp)
            .initial_device_display_name("meteorite Client")
            .send()
            .await?;

        // put session in keyring
        if let Some(auth_session) = client.session() {
            if let matrix_sdk::AuthSession::Matrix(session) = auth_session {
                let json = serde_json::to_string(&session)?;
                session_entry.set_password(&json)?;
                dbg!("success! login is now in keyring");
            }
        }
    }

    // client.add_event_handler(|ev: SyncRoomMessageEvent| async move {
    //     println!("Received a message {:?}", ev);
    // });

    // Syncing is important to synchronize the client state with the server.
    // This method will never return unless there is an error.
    // client.sync(SyncSettings::default()).await?;
    
    println!("enter room id/alias");

    let mut room_inp = String::new();

    io::stdin()
        .read_line(&mut room_inp)
        .expect("Failed to read line");

    room_inp = room_inp.trim().to_string();

    let room_alias_id = RoomOrAliasId::parse(room_inp).expect("Invalid input");

    let room_id = if room_alias_id.is_room_alias_id() {
        let alias = RoomAliasId::parse(&room_alias_id).expect("if you see this, you broke something");
        let response = client.resolve_room_alias(&alias).await?;
        response.room_id
    } else {
        let id = RoomId::parse(&room_alias_id).expect("what did you break this time...");
        id.to_owned()
    };

    client.sync_once(SyncSettings::default()).await?;

    // output all rooms (the client knows of), do no uncomment please
    // println!("{:?}", client.rooms());
   /* 
    let joined_rooms = client.joined_rooms();
    println!("joined: {:?}", joined_rooms.iter().map(|r| r.room_id()).collect::<Vec<_>>());
    */
    if let Some(room) = client.get_room(&room_id) {
        if let RoomState::Joined = room.state() {
            // set the content to send
            println!("message to send");
            let mut message = String::new();
            io::stdin()
                .read_line(&mut message)
                .expect("Failed to read line");

            let content = RoomMessageEventContent::text_plain(&message.trim().to_string());

            println!("sending");
            // (hopefully) send message
            room.send(content).await.unwrap();
        } else {
            println!("You are not in this room");
        }
    } else {
        println!("maybe you arent in the room or the server is too slow rn");
    }

    Ok(())
}
