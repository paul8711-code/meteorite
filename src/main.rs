extern crate rpassword;

use matrix_sdk::{
    Client, config::SyncSettings,
    ruma::{UserId, events::room::message::SyncRoomMessageEvent},
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

    let mut password_inp = read_password().unwrap();

    // First we need to log in.
    client.matrix_auth().login_username(user, &password_inp).send().await?;

    client.add_event_handler(|ev: SyncRoomMessageEvent| async move {
        println!("Received a message {:?}", ev);
    });

    // Syncing is important to synchronize the client state with the server.
    // This method will never return unless there is an error.
    client.sync(SyncSettings::default()).await?;

    Ok(())
}
