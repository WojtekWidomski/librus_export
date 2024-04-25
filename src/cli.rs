use anyhow::{Ok, Result};
use dialoguer::{Input, Password};

use crate::api::{
    messages::{Message, MessageType},
    SynergiaClient,
};

fn login() -> SynergiaClient {
    println!("Log in to your LIBRUS Synergia account.");
    println!("Remember to use Synergia account and not LIBRUS mobile app account.");
    println!("No characters will be displayed in terminal while entering password.");

    loop {
        let username: String = Input::new()
            .with_prompt("Username")
            .interact_text()
            .unwrap();

        let password = Password::new().with_prompt("Password").interact().unwrap();

        let client_result = SynergiaClient::login(username.as_str(), password.as_str(), 10);

        match client_result {
            std::result::Result::Ok(client) => {
                return client;
            }
            Err(e) => {
                println!("Login failed:\n{}\nPlease try again.", e);
                println!("Use ^C to exit.")
            }
        }
    }
}

fn download_messages_to_file(
    client: &SynergiaClient,
    in_archive: bool,
    msg_type: MessageType,
    filename: &str,
) -> Result<()> {
    let handles = client.get_messages(in_archive, msg_type)?;

    let messages: Result<Vec<Message>> = handles.iter().map(|h| Ok(h.get_message()?)).collect();
    let messages = messages?;

    // TODO: Save to file

    Ok(())
}

pub fn run_cli() {
    let client = login();

    download_messages_to_file(&client, false, MessageType::Sent, "messages.json").unwrap();
}
