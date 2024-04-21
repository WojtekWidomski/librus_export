use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::Value;

const AUTH_URL1: &str =
    "https://api.librus.pl/OAuth/Authorization?client_id=46&response_type=code&scope=mydata";
const AUTH_URL2: &str = "https://api.librus.pl/OAuth/Authorization?client_id=46";
const GRANT_URL: &str = "https://api.librus.pl/OAuth/Authorization/Grant?client_id=46";
const MSG_URL: &str = "https://synergia.librus.pl/wiadomosci3";

// There are only few, predefined E-mail folder in Librus
pub enum MessagesFolder {
    Inbox,
    Sent,
    Trash,
    ArchiveInbox,
    ArchiveSent,
    ArchiveTrash,
}

#[derive(Serialize)]
struct LoginData {
    action: String,
    login: String,
    pass: String,
}

struct User<'a> {
    id: i32,
    first_name: &'a str,
    last_name: &'a str,
}

struct Message<'a> {
    id: i32,
    sender_first_name: &'a str,
    sender_last_name: &'a str,
    topic: &'a str,
    content: Option<&'a str>,
    send_date: &'a str,
    receivers: Option<Vec<User<'a>>>,
}

pub struct SynergiaClient {
    client: reqwest::blocking::Client,
}

impl SynergiaClient {
    pub fn login(username: &str, password: &str) -> Result<Self> {
        let client = reqwest::blocking::ClientBuilder::new()
            .cookie_store(true)
            .build()?;

        client
            .get(AUTH_URL1)
            .send()
            .context("Failed to connect to Librus server.")?;

        let login_data = LoginData {
            action: String::from("login"),
            login: String::from(username),
            pass: String::from(password),
        };

        let auth_res = client.post(AUTH_URL2).form(&login_data).send()?;

        if !auth_res.status().is_success() {
            return Err(anyhow!("Error when trying to log in. Make sure your password is correct. If it is, then it is possible, that Librus changed API."));
        }

        let grant_res = client.get(GRANT_URL).send()?;
        let grant_res_text = grant_res.text()?;

        if grant_res_text.contains("error") {
            return Err(anyhow!("Error when authenticating: {}", grant_res_text));
        }

        // Authenticate Librus Synergia messages
        client.get(MSG_URL).send()?;

        Ok(SynergiaClient { client })
    }

    pub fn get_messages(&self, folder: MessagesFolder) -> Result<()> {
        let folder_path = match folder {
            MessagesFolder::Inbox => "inbox",
            MessagesFolder::Sent => "outbox",
            MessagesFolder::Trash => "trash-bin",
            MessagesFolder::ArchiveInbox => "archive/inbox",
            MessagesFolder::ArchiveSent => "archive/outbox",
            MessagesFolder::ArchiveTrash => "archive/trash-bin",
        };

        // Get only first message, because API will return all messages count at the same time
        let messages_res = self
            .client
            .get(format!(
                "https://wiadomosci.librus.pl/api/{}/messages?page=1&limit=1",
                folder_path
            ))
            .send()?;

        let first_msg: Value = serde_json::from_str(messages_res.text()?.as_str())
            .context("Message deserialization error")?;

        let msg_count = &first_msg["total"]
            .as_i64()
            .context("Failed to get message number")?;

        dbg!(msg_count);

        // Get msg_count messages.
        let messages_res = self
            .client
            .get(format!(
                "https://wiadomosci.librus.pl/api/{}/messages?page=1&limit={}",
                folder_path, msg_count
            ))
            .send()?;

        let msg_list: Value = serde_json::from_str(messages_res.text()?.as_str())
            .context("Messages deserialization error")?;

        let converted_list = msg_list["data"]
            .as_array()
            .context("Message deserialization error")?
            .iter()
            .map(|msg| {
                // TODO: Convert messages from api to structs.
            });

        Ok(())
    }
}
