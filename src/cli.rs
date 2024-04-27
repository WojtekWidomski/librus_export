use std::{fs, path::Path};

use anyhow::{Ok, Result, anyhow};
use dialoguer::{Input, Password};

use crate::api::{
    messages::{Message, MessageType},
    SynergiaClient,
};

fn login() -> SynergiaClient {
    println!("Some teachers often send messages to very large group of users");
    println!("(e.g. all students in the school). This program can save such");
    println!("groups in separate file and save only group id in file with");
    println!("messages. This will result in smaller, and easier to browse files.");
    println!("Enter number of users in group to be considered as large.");
    println!("Use 0 to disable this feature.");

    let min_big_groups: usize = Input::new()
        .with_prompt("Min users in big group")
        .default(10)
        .show_default(true)
        .interact()
        .unwrap();

    println!("Log in to your LIBRUS Synergia account.");
    println!("Remember to use Synergia account and not LIBRUS mobile app account.");
    println!("No characters will be displayed in terminal while entering password.");

    loop {
        let username: String = Input::new()
            .with_prompt("Username")
            .interact_text()
            .unwrap();

        let password = Password::new().with_prompt("Password").interact().unwrap();

        let client_result =
            SynergiaClient::login(username.as_str(), password.as_str(), min_big_groups);

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

    let messages_json = serde_json::to_string_pretty(&messages)?;

    if Path::new(filename).exists() {
        return Err(anyhow!("File {} already exists", filename));
    }

    fs::write(filename, messages_json)?;

    Ok(())
}

pub fn run_cli() {
    let client = login();

    download_messages_to_file(&client, false, MessageType::Sent, "messages.json").unwrap();
}
