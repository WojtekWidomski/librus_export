use dialoguer::{Input, Password};

use crate::api::SynergiaClient;

fn login() -> SynergiaClient {
    println!("Log in to your LIBRUS Synergia account.");
    println!("Remember to use Synergia account and not LIBRUS mobile app account.");

    loop {
        let username: String = Input::new()
            .with_prompt("Username")
            .interact_text()
            .unwrap();

        let password = Password::new().with_prompt("Password").interact().unwrap();

        let client_result = SynergiaClient::login(username.as_str(), password.as_str());

        match client_result {
            Ok(client) => {return client;}
            Err(e) => {
                println!("Login failed:\n{}\nPlease try again.", e);
                println!("Use ^C to exit.")
            }
        }
    }
}

pub fn run_cli() {
    login();
}
