mod authentication;
pub mod messages;

use std::{cell::RefCell, collections::HashSet};

use anyhow::{Context, Ok, Result};
use serde_json::Value;

use self::{
    authentication::authenticate,
    messages::{MessageHandle, MessageType, User},
};

pub struct SynergiaClient {
    client: reqwest::blocking::Client,
    receivers_groups: RefCell<Vec<HashSet<User>>>,
    min_big_group: usize,
}

impl SynergiaClient {
    /// Create new `SynergiaClient` struct logged in as user with username and password.
    pub fn login(username: &str, password: &str, min_big_group: usize) -> Result<Self> {
        let client = reqwest::blocking::ClientBuilder::new()
            .cookie_store(true)
            .build()?;

        authenticate(&client, username, password)?;

        Ok(SynergiaClient {
            client,
            min_big_group,
            receivers_groups: RefCell::new(Vec::new()),
        })
    }

    fn get_group(&self, users: HashSet<User>) -> usize {
        let mut receivers_groups = self.receivers_groups.borrow_mut();

        for (idx, group) in receivers_groups.iter().enumerate() {
            if *group == users {
                return idx;
            }
        }

        receivers_groups.push(users);

        receivers_groups.len() - 1
    }

    fn get_message_count(&self, folder_path: &String) -> Result<i64> {
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

        Ok(*msg_count)
    }

    pub fn get_messages(
        &self,
        in_archive: bool,
        message_type: MessageType,
    ) -> Result<Vec<MessageHandle>> {
        let folder_path = message_type.get_path(in_archive);
        let msg_count = self.get_message_count(&folder_path)?;

        // Get msg_count messages.
        let messages_res = self
            .client
            .get(format!(
                "https://wiadomosci.librus.pl/api/{}/messages?page=1&limit={}",
                folder_path, msg_count
            ))
            .send()?;

        // Deserialize messages
        let res_deserialized: Value = serde_json::from_str(messages_res.text()?.as_str())
            .context("Messages deserialization error")?;

        let handles_result: Result<Vec<MessageHandle>> = res_deserialized["data"]
            .as_array()
            .context("Message deserialization error")?
            .iter()
            .map(|msg| {
                Ok(MessageHandle::new(
                    in_archive,
                    message_type,
                    msg["messageId"]
                        .as_str()
                        .context("Message id parsing error")?
                        .parse()?,
                    self,
                ))
            })
            .collect();

        let handles = handles_result?;

        Ok(handles)
    }
}
