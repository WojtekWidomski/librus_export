use std::collections::HashSet;

use anyhow::{Context, Ok, Result};
use base64::prelude::*;
use serde::Serialize;
use serde_json::Value;

use super::SynergiaClient;

// There are 3 message types in Librus and they use different API.
#[derive(Debug, Clone, Copy)]
pub enum MessageType {
    Inbox,
    Sent,
    Trash,
}

impl MessageType {
    pub fn get_path(&self, archive: bool) -> String {
        let path_end = match self {
            MessageType::Inbox => "inbox",
            MessageType::Sent => "outbox",
            MessageType::Trash => "trash-bin",
        };

        if archive {
            return format!("archive/{}", path_end);
        }

        path_end.to_string()
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Serialize)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
}

/// Small group of users is stored as vector of users
/// Large group is index in `receivers_groups` in `SynergiaClient`
#[derive(Debug, Serialize)]
pub enum UserGroup {
    Small(HashSet<User>),
    Large(usize),
}

#[derive(Debug, Serialize)]
pub struct Message {
    pub sender: User,
    pub topic: String,
    pub content: String,
    pub send_date: String,
    pub receivers: UserGroup,
}

fn parse_receivers(receivers_value: &Value) -> Result<HashSet<User>> {
    const ERROR_MESSAGE: &str = "Receivers parsing error";

    let rec_vec_res: Result<HashSet<User>> = receivers_value
        .as_array()
        .context(ERROR_MESSAGE)?
        .iter()
        .map(|user| {
            Ok(User {
                first_name: user["firstName"].as_str().unwrap_or("").to_string(),
                last_name: user["lastName"].as_str().unwrap_or("").to_string(),
            })
        })
        .collect();

    let rec_vec = rec_vec_res?;

    Ok(rec_vec)
}

pub struct MessageHandle<'a> {
    in_archive: bool,
    message_type: MessageType,
    id: i64,
    synergia_client: &'a SynergiaClient,
}

impl<'a> MessageHandle<'a> {
    pub fn new(
        in_archive: bool,
        message_type: MessageType,
        id: i64,
        client: &'a SynergiaClient,
    ) -> Self {
        MessageHandle {
            in_archive,
            message_type,
            id,
            synergia_client: client,
        }
    }

    /// Remove content xml/html tagi. If they do not exist return content
    fn remove_content_prefix_and_suffix(&self, content: String) -> String {
        let without_pref = match content.strip_prefix("<Message><Content><![CDATA[") {
            Some(text) => text.to_string(),
            None => {return content},
        };
        let without_suf = match without_pref.strip_suffix("]]></Content><Actions><Actions/></Actions></Message>") {
            Some(text) => text.to_string(),
            None => {return content},
        };
        return without_suf;
    }

    pub fn get_message(&self) -> Result<Message> {
        let folder_path = self.message_type.get_path(self.in_archive);

        let msg = self
            .synergia_client
            .client
            .get(format!(
                "https://wiadomosci.librus.pl/api/{}/messages/{}",
                folder_path, self.id
            ))
            .send()?
            .text()?;

        let msg_deserialized: Value = serde_json::from_str(&msg)?;

        let content_field = match self.message_type {
            MessageType::Inbox => "content",
            MessageType::Sent => "Message", // why Message not content?
            MessageType::Trash => "content",
        };

        let content = msg_deserialized["data"][content_field]
            .as_str()
            .context("Failed to derserialize message")?
            .to_string();

        // Message content in Librus is encoded using base64
        let content = String::from_utf8(BASE64_STANDARD.decode(content)?)?;

        let content = self.remove_content_prefix_and_suffix(content);

        let topic = msg_deserialized["data"]["topic"]
            .as_str()
            .context("Failed to deserialize message")?
            .to_string();

        let send_date = msg_deserialized["data"]["sendDate"]
            .as_str()
            .context("Failed to get send date")?
            .to_string();

        let sender_first_name = msg_deserialized["data"]["senderFirstName"]
            .as_str()
            .context("Failed to get sender name")?
            .to_string();
        let sender_last_name = msg_deserialized["data"]["senderLastName"]
            .as_str()
            .context("Failed to get sender name")?
            .to_string();

        let receivers_set = parse_receivers(&msg_deserialized["data"]["receivers"])?;

        let receivers = if receivers_set.len() >= self.synergia_client.min_big_group {
            UserGroup::Large(self.synergia_client.get_group(receivers_set))
        } else {
            UserGroup::Small(receivers_set)
        };

        Ok(Message {
            topic,
            content,
            send_date,
            sender: User {
                first_name: sender_first_name,
                last_name: sender_last_name,
            },
            receivers,
        })
    }
}
