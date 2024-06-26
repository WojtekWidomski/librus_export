use std::{fs, path::Path, time::Duration};

use anyhow::{anyhow, Ok, Result};
use dialoguer::{Input, MultiSelect, Password};
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const REPO: &str = env!("CARGO_PKG_REPOSITORY");

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

fn display_welcome_message() {
    println!("LIBRUS Synergia Message Export version {}", VERSION);
    println!("Author: {}", AUTHOR);
    println!("Source code available at {} on GPL-3.0 license", REPO);
    println!(""); // Empty line
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

    println!("");

    println!("Log in to your LIBRUS Synergia account.");
    println!("Remember to use Synergia account and not LIBRUS mobile app account.");
    println!("No characters will be displayed in terminal while entering password.");

    loop {
        let username: String = Input::new()
            .with_prompt("Username")
            .interact_text()
            .unwrap();

        let password = Password::new().with_prompt("Password").interact().unwrap();

        let spinner = ProgressBar::new_spinner().with_message("Logging in...");
        spinner.enable_steady_tick(Duration::from_millis(100));

        let client_result =
            SynergiaClient::login(username.as_str(), password.as_str(), min_big_groups);

        spinner.finish_and_clear();

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
    librus_folder: &MessageFolder,
    export_folder: &String,
) -> Result<()> {
    let handles = client.get_messages(librus_folder.in_archive, librus_folder.message_type)?;

    let progress_style =
        ProgressStyle::with_template("{prefix:16}{bar:60.cyan/blue} [{pos}/{len}]").unwrap();

    let pb = ProgressBar::new(handles.len() as u64)
        .with_style(progress_style)
        .with_prefix(librus_folder.displayed_name.clone());

    let messages: Result<Vec<Message>> = handles
        .iter()
        .progress_with(pb)
        .map(|h| Ok(h.get_message()?))
        .collect();
    let messages = messages?;

    let messages_json = serde_json::to_string_pretty(&messages)?;

    let export_path = format!("{}/{}", export_folder, librus_folder.filename);

    if Path::new(&export_path).exists() {
        return Err(anyhow!("File {} already exists", &librus_folder.filename));
    }

    fs::write(&export_path, messages_json)?;

    Ok(())
}

fn download_selected(client: &SynergiaClient, export_folder: &String) -> Result<()> {
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
        .interact()?;

    for folder_idx in selected_folders {
        let folder = &folders[folder_idx];
        download_messages_to_file(&client, folder, export_folder)?;
    }

    Ok(())
}

fn download_groups_to_file(client: &SynergiaClient, filename: String) -> Result<()> {
    let groups = client.get_receivers_groups();

    let groups_json = serde_json::to_string_pretty(&groups)?;

    if Path::new(&filename).exists() {
        return Err(anyhow!("File {} already exists", &filename));
    }

    fs::write(&filename, groups_json)?;

    Ok(())
}

pub fn run_cli() -> Result<()> {
    display_welcome_message();

    let client = login();

    let account_info = client.get_account_name()?;

    println!(
        "Logged in as {} {}\n",
        account_info.first_name, account_info.last_name
    );

    let export_folder = format!(
        "export_{}_{}",
        account_info.first_name, account_info.last_name
    );

    if !Path::new(&export_folder).exists() {
        fs::create_dir(&export_folder)?;
    }

    download_selected(&client, &export_folder)?;

    download_groups_to_file(&client, format!("{}/groups.json", export_folder))?;

    println!("Succesfully exported to {}", export_folder);

    Ok(())
}
