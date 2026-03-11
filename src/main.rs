extern crate rpassword;

use matrix_sdk::{
    Client,
    config::SyncSettings,
    RoomState,
    ruma::events::room::{
    message::RoomMessageEventContent
    },
    ruma::{UserId, RoomId, RoomOrAliasId, RoomAliasId, events::room::message::SyncRoomMessageEvent},
};

use std::io;
use rpassword::read_password;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("input user name");

    let mut user_inp = String::new();

    io::stdin()
        .read_line(&mut user_inp)
        .expect("Failed to read line");

    user_inp = user_inp.trim().to_string();

    let user = UserId::parse(&user_inp)?; 
    let client = Client::builder().server_name(user.server_name()).build().await?;

    println!("input password");

    let password_inp = read_password().unwrap();

    // First we need to log in.
    client.matrix_auth().login_username(user, &password_inp).send().await?;

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
        }
    }

    Ok(())
}
