use anyhow::{Context, Ok, Result};
use base64::prelude::*;
use serde_json::Value;

// There are 3 message types in Librus and they use different API.
#[derive(Debug)]
pub enum MessageType {
    Inbox,
    Sent,
    Trash,
}

impl MessageType {
    fn get_path(&self, archive: bool) -> String {
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
pub struct Message<'a> {
    pub id: i64,
    pub sender_first_name: String,
    pub sender_last_name: String,
    pub topic: String,
    pub content: Option<String>,
    pub send_date: String,
    pub receivers: Option<Vec<User>>,
    pub client: &'a reqwest::blocking::Client,
    pub message_type: MessageType,
}

impl<'a> Message<'a> {
    pub fn get_content(&mut self) -> Result<String> {
        if let Some(saved_content) = &self.content {
            return Ok(saved_content.to_string());
        }

        let msg = self
            .client
            .get(format!(
                "https://wiadomosci.librus.pl/api/trash-bin/messages/{}",
                self.id
            ))
            .send()?
            .text()?;

        let msg_deserialized: Value = serde_json::from_str(&msg)?;

        let content_field = match self.message_type {
            MessageType::Inbox => "Message",
            MessageType::Sent => "Message",
            MessageType::Trash => "content",
        };

        let content = msg_deserialized["data"][content_field]
            .as_str()
            .context("Failed to derserialize message")?
            .to_string();

        let content = String::from_utf8(BASE64_STANDARD.decode(content)?)?;

        self.content = Some(content.clone());

        Ok(content)
    }
}
