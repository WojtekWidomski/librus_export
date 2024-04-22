mod authentication;

use anyhow::{anyhow, Context, Ok, Result};
use base64::prelude::*;
use serde::Serialize;
use serde_json::Value;

use self::authentication::authenticate;

#[derive(Debug)]
struct User {
    id: i32,
    first_name: String,
    last_name: String,
}

// There are 3 message types in Librus and they use different API.
#[derive(Debug)]
enum MessageType {
    Inbox,
    Sent,
    Trash,
}

#[derive(Debug)]
pub struct Message<'a> {
    id: i64,
    sender_first_name: String,
    sender_last_name: String,
    topic: String,
    content: Option<String>,
    send_date: String,
    receivers: Option<Vec<User>>,
    client: &'a reqwest::blocking::Client,
    message_type: MessageType,
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

pub struct SynergiaClient {
    client: reqwest::blocking::Client,
}

impl SynergiaClient {
    pub fn login(username: &str, password: &str) -> Result<Self> {
        let client = reqwest::blocking::ClientBuilder::new()
            .cookie_store(true)
            .build()?;

        authenticate(&client, username, password)?;
        
        Ok(SynergiaClient { client })
    }

    fn get_messages(&self, folder_path: &str) -> Result<Vec<Value>> {
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

        let msg_vec = msg_list["data"]
            .as_array()
            .context("Message deserialization error")?;

        Ok(msg_vec.to_vec())
    }

    pub fn get_messages_inbox(&self, archive: bool) -> Result<()> {
        let folder_path = match archive {
            true => "archive/inbox",
            false => "inbox",
        };

        let msg_vec = self.get_messages(folder_path)?;

        dbg!(msg_vec);

        Ok(())
    }

    pub fn get_messages_sent(&self, archive: bool) -> Result<Vec<Message>> {
        let folder_path = match archive {
            true => "archive/outbox",
            false => "outbox",
        };
        let msg_vec = self.get_messages(folder_path)?;

        let converted_messages: Vec<Message> = msg_vec
            .iter()
            .map(|msg| Message {
                id: msg["messageId"].as_str().unwrap().parse().unwrap(),
                sender_first_name: String::from("user"),
                sender_last_name: String::from("user"),
                topic: msg["topic"].as_str().unwrap().to_string(),
                content: None,
                send_date: msg["sendDate"].as_str().unwrap().to_string(),
                receivers: None,
                client: &self.client,
                message_type: MessageType::Sent,
            })
            .collect();

        Ok(converted_messages)
    }

    pub fn get_messages_trash(&self, archive: bool) -> Result<Vec<Message>> {
        let folder_path = match archive {
            true => "archive/trash-bin",
            false => "trash-bin",
        };

        let msg_vec = self.get_messages(folder_path)?;

        let converted_messages: Vec<Message> = msg_vec.iter().map(|msg| Message {
            id: msg["messageId"].as_str().unwrap().parse().unwrap(),
            sender_first_name: String::from("user"),
            sender_last_name: String::from("user"),
            topic: msg["topic"].as_str().unwrap().to_string(),
            content: None,
            send_date: msg["sendDate"].as_str().unwrap().to_string(),
            receivers: None,
            client: &self.client,
            message_type: MessageType::Trash,
        }).collect();

        Ok(converted_messages)
    }
}
