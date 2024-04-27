use std::{fs, path::Path};

use anyhow::{anyhow, Ok, Result};
use dialoguer::{Input, MultiSelect, Password};

use crate::api::{
    messages::{Message, MessageType},
    SynergiaClient,
};

struct MessageFolder {
    displayed_name: String,
    in_archive: bool,
    message_type: MessageType,
    filename: String,
}

impl ToString for MessageFolder {
    fn to_string(&self) -> String {
        self.displayed_name.clone()
    }
}

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

fn download_messages_to_file(client: &SynergiaClient, folder: &MessageFolder) -> Result<()> {
    let handles = client.get_messages(folder.in_archive, folder.message_type)?;

    let messages: Result<Vec<Message>> = handles.iter().map(|h| Ok(h.get_message()?)).collect();
    let messages = messages?;

    let messages_json = serde_json::to_string_pretty(&messages)?;

    if Path::new(&folder.filename).exists() {
        return Err(anyhow!("File {} already exists", &folder.filename));
    }

    fs::write(&folder.filename, messages_json)?;

    Ok(())
}

pub fn run_cli() {
    let client = login();

    let folders = vec![
        MessageFolder {
            displayed_name: String::from("Inbox"),
            in_archive: false,
            message_type: MessageType::Inbox,
            filename: String::from("messages_inbox.json"),
        },
        MessageFolder {
            displayed_name: String::from("Sent"),
            in_archive: false,
            message_type: MessageType::Sent,
            filename: String::from("messages_sent.json"),
        },
        MessageFolder {
            displayed_name: String::from("Trash"),
            in_archive: false,
            message_type: MessageType::Trash,
            filename: String::from("messages_trash.json"),
        },
        MessageFolder {
            displayed_name: String::from("Archive/Inbox"),
            in_archive: true,
            message_type: MessageType::Inbox,
            filename: String::from("messages_archive_inbox.json"),
        },
        MessageFolder {
            displayed_name: String::from("Archive/Sent"),
            in_archive: true,
            message_type: MessageType::Sent,
            filename: String::from("messages_archive_sent.json"),
        },
        MessageFolder {
            displayed_name: String::from("Archive/Trash"),
            in_archive: true,
            message_type: MessageType::Trash,
            filename: String::from("messages_archive_trash.json"),
        },
    ];

    let selected_folders = MultiSelect::new()
        .with_prompt(
            "Select folders to download\nUse arrows and space to select. Enter to confirm.",
        )
        .items(&folders)
        .defaults(&vec![true; folders.len()])
        .interact()
        .unwrap();

    for folder_idx in selected_folders {
        let folder = &folders[folder_idx];
        download_messages_to_file(&client, folder).unwrap();
    }
}
