use matrix_sdk::{
    Room, RoomState,
    config::SyncSettings,
    ruma::{RoomAliasId, RoomId, UserId, events::room::message::RoomMessageEventContent},
};
use std::fs;
use std::io;

mod auth;

const APP_NAME: &str = "meteorite_client";
const KEYRING_SESSION: &str = "meteorite_session_json";
const KEYRING_DB_PASS: &str = "meteorite_db_password";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    keyring_core::set_default_store(zbus_secret_service_keyring_store::Store::new()?);
    #[cfg(target_os = "windows")]
    keyring_core::set_default_store(windows_native_keyring_store::Store::new()?);
    #[cfg(target_os = "macos")]
    keyring_core::set_default_store(apple_native_keyring_store::Store::new()?);
    // create the storage folder (for sql stuff)
    let mut storage_path =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    storage_path.push(".meteorite");

    if !storage_path.exists() {
        fs::create_dir_all(&storage_path)?;
    }

    let storage_str = storage_path.to_str().expect("Path invalid");

    let client = auth::login(APP_NAME, KEYRING_DB_PASS, KEYRING_SESSION, storage_str).await?;

    client.sync_once(SyncSettings::default()).await?;

    // client.add_event_handler(|ev: SyncRoomMessageEvent| async move {
    //     println!("Received a message {:?}", ev);
    // });

    // Syncing is important to synchronize the client state with the server.
    // This method will never return unless there is an error.
    // client.sync(SyncSettings::default()).await?;

    // this is test code for sending messages to rooms (dms dont work yet)
    println!("enter room id/alias");

    let mut room_inp = String::new();

    io::stdin()
        .read_line(&mut room_inp)
        .expect("Failed to read line");

    room_inp = room_inp.trim().to_string();

    // here room id gets defined, checking if the inputted thing is a user, a room alias or a room
    // id
    let room_id: Option<Room> = if let Ok(user_id) = UserId::parse(&room_inp) {
        // if the input is a dm (starting with "@")
        client.get_dm_room(&user_id)
    } else if let Ok(room_id) = RoomId::parse(&room_inp) {
        // if the input is just a normal room id (starting with "!")
        client.get_room(&room_id)
    } else if let Ok(alias) = RoomAliasId::parse(&room_inp) {
        if let Ok(response) = client.resolve_room_alias(&alias).await {
            // if the input is an alias (starting with "#")
            client.get_room(&response.room_id)
        } else {
            None
        }
    } else {
        None
    };

    // output all rooms (the client knows of), do no uncomment please
    // println!("{:?}", client.rooms());
    if let Some(room) = room_id {
        if let RoomState::Joined = room.state() {
            // only send if you are in the room
            // set the content to send
            println!("message to send");
            let mut message = String::new();
            io::stdin()
                .read_line(&mut message)
                .expect("Failed to read line");

            let content = RoomMessageEventContent::text_plain(message.trim().to_string());

            println!("sending");
            // (hopefully) send message
            room.send(content).await?;
        } else {
            println!("You are not in this room");
        }
    } else {
        println!("maybe you arent in the room or the server is too slow rn");
    }

    keyring_core::unset_default_store();
    Ok(())
}
