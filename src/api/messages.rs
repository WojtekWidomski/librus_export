use anyhow::{Context, Ok, Result};
use base64::prelude::*;
use serde_json::Value;

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

#[derive(Debug)]
pub struct User {
    id: i32,
    first_name: String,
    last_name: String,
}

#[derive(Debug)]
pub struct Message {
    pub sender_first_name: String,
    pub sender_last_name: String,
    pub topic: String,
    pub content: String,
    pub send_date: String,
    pub receivers: Option<Vec<User>>,
}

#[derive(Debug)]
pub struct MessageHandle<'a> {
    in_archive: bool,
    message_type: MessageType,
    id: i64,
    client: &'a reqwest::blocking::Client,
}

impl<'a> MessageHandle<'a> {
    pub fn new(
        in_archive: bool,
        message_type: MessageType,
        id: i64,
        client: &'a reqwest::blocking::Client,
    ) -> Self {
        MessageHandle {
            in_archive,
            message_type,
            id,
            client,
        }
    }

    pub fn get_message(&mut self) -> Result<Message> {
        let folder_path = self.message_type.get_path(self.in_archive);

        let msg = self
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

        let topic = msg_deserialized["data"]["topic"]
            .as_str()
            .context("Failed to deserialize message")?
            .to_string();

        let send_date = msg_deserialized["data"]["sendDate"]
            .as_str()
            .context("Failed to get send date")?
            .to_string();

        Ok(Message {
            sender_first_name: "".to_string(),
            sender_last_name: "".to_string(),
            topic,
            content,
            send_date,
            receivers: None,
        })
    }
}
