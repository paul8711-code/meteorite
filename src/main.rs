use matrix_sdk::{
    config::SyncSettings,
    RoomState,
    ruma::{
        RoomId,
        RoomOrAliasId,
        RoomAliasId,
        events::room::message::{
            RoomMessageEventContent,
        },
    },
};
use std::io;
use std::fs;

mod auth;

const APP_NAME: &str = "meteorite_client";
const KEYRING_SESSION: &str = "meteorite_session_json";
const KEYRING_DB_PASS: &str = "meteorite_db_password";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // create the storage folder (for sql stuff)
    let mut storage_path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

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

    // here the alias is "converted" to an id
    let room_alias_id = RoomOrAliasId::parse(room_inp).expect("Invalid input");

    let room_id = if room_alias_id.is_room_alias_id() {
        let alias = RoomAliasId::parse(&room_alias_id).expect("if you see this, you broke something");
        let response = client.resolve_room_alias(&alias).await?;
        response.room_id
    } else {
        let id = RoomId::parse(&room_alias_id).expect("what did you break this time...");
        id.to_owned()
    };

    // output all rooms (the client knows of), do no uncomment please
    // println!("{:?}", client.rooms());
    if let Some(room) = client.get_room(&room_id) {
        if let RoomState::Joined = room.state() { // only send if you are in the room
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
