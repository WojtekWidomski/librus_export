use anyhow::{anyhow, Context, Result};
use serde::Serialize;

const AUTH_URL1: &str =
    "https://api.librus.pl/OAuth/Authorization?client_id=46&response_type=code&scope=mydata";
const AUTH_URL2: &str = "https://api.librus.pl/OAuth/Authorization?client_id=46";
const GRANT_URL: &str = "https://api.librus.pl/OAuth/Authorization/Grant?client_id=46";
const MSG_URL: &str = "https://synergia.librus.pl/wiadomosci3";

#[derive(Serialize)]
struct LoginData {
    action: String,
    login: String,
    pass: String,
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
}
