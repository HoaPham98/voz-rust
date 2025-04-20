use std::io::stdin;

use vozclient::core::{models, voz_core::VozCore};
use models::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let voz = VozCore::new("voz.vn".to_string());
    let mut username = String::new();
    let mut password = String::new();
    println!("Your username: ");
    stdin().read_line(&mut username).unwrap();
    println!("Your password: ");
    stdin().read_line(&mut password).unwrap();
    let result = voz.login(username, password).await?;
    match result {
        LoginResult::Success { user, session, info, tfa_trust } => {
            println!("Login successfully with user info: {:?}", info);
            println!("User {:?}, session: {:?}", user, session);
        },
        LoginResult::MFA { url } => {
            let mut code = String::new();
            let mut provider = String::new();
            println!("Your code: ");
            stdin().read_line(&mut code).unwrap();
            println!("Your provider: ");
            stdin().read_line(&mut provider).unwrap();
            let login = voz.mfa(url, code, provider).await?;
            match login {
                LoginResult::Success { user, session, info , tfa_trust} => println!("Login successfully with user info: {:?}", info),
                LoginResult::MFA { url } => println!("This should not happend :|")
            }
        }
    }
    Ok(())
}
