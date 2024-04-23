mod authentication;
pub mod messages;

use anyhow::{Context, Ok, Result};
use serde_json::Value;

use self::{
    authentication::authenticate,
    messages::{MessageHandle, MessageType},
};

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

    pub fn get_messages(
        &self,
        archive: bool,
        message_type: MessageType,
    ) -> Result<Vec<MessageHandle>> {
        let folder_path = message_type.get_path(archive);

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

        let msg_vec_result: Result<Vec<MessageHandle>> = msg_list["data"]
            .as_array()
            .context("Message deserialization error")?
            .iter()
            .map(|msg| {
                Ok(MessageHandle::new(
                    archive,
                    message_type,
                    msg["messageId"]
                        .as_str()
                        .context("Message id parsing error")?
                        .parse()?,
                    &self.client,
                ))
            })
            .collect();

        let msg_vec = msg_vec_result?;

        Ok(msg_vec)
    }
}
