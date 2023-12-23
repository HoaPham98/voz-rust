mod core;
pub mod models;
use std::{fmt::Debug, collections::HashMap};
use core::parse_utils::{parse_catagories, parse_forum, parse_login_form, parse_current_user};
use select::{document::Document, predicate::Class};
use serde::Serialize;
use core::session::Session;
use models::*;
pub trait VozResponseMapping<T: Serialize> {
    fn voz_response(self) -> VozResponse<T>;
}

impl<T: Serialize + Debug> VozResponseMapping<T> for Result<T, Box<dyn std::error::Error>> {
    fn voz_response(self) -> VozResponse<T> {
        if self.is_ok() {
            VozResponse::Success { data: self.unwrap() }
        } else {
            VozResponse::Failed { message: self.unwrap_err().to_string() }
        }
    }
}

pub struct VozCore {
    client: Session
}

impl VozCore {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Session::new(base_url)
        }
    }
}

impl VozCore {
    pub fn set_user(&self, user: String, session: String) {
        self.client.set_cookie("xf_user".to_string(), user);
        self.client.set_cookie("xf_session".to_string(), session);
    }
    
    pub async fn get_categories(&self) -> Result<Vec<Category>, Box<dyn std::error::Error>> {
        let content = self.client.get("/").send().await?.text().await?;
        let document = Document::from_read(content.as_bytes()).ok().ok_or("Invalid request")?;
        let results = document.find(Class("block--category")).filter_map(|x| parse_catagories(x).ok()).collect::<Vec<Category>>();
        Ok(results)
    }

    pub async fn get_forum(&self, id: String, forum_type: String) -> Result<Forum, Box<dyn std::error::Error>> {
        let content = self.client.get(format!("/{forum_type}/{id}")).send().await?.text().await?;
        let document = Document::from_read(content.as_bytes()).ok().ok_or("Invalid request")?;
        let node = document.find(Class("p-body")).next().ok_or("p-body does not exist")?;
        let result = parse_forum(node)?;
        Ok(result)
    }

    pub async fn login(&self, username: String, password: String) -> Result<LoginResult, Box<dyn std::error::Error>> {
        let content = self.client.get("/login/login").send().await?.text().await?;
        let document = Document::from_read(content.as_bytes()).ok().ok_or("Invalid request")?;
        let node = document.find(Class("p-body")).next().ok_or("p-body does not exist")?;
        let login_info = parse_login_form(node)?;
        let form = HashMap::from([
            ("_xfToken", login_info.token),
            ("login", username),
            ("password", password),
            ("remember", "1".to_string()),
        ]);
        let content = self.client.post(format!("/{0}", login_info.url)).form(&form).send().await?.text().await?;
        let document = Document::from_read(content.as_bytes()).ok().ok_or("Invalid request")?;
        let cookies = self.client.get_cookies();
        if cookies.contains_key("xf_session") {
            if cookies.contains_key("xf_user") {
                let node = document.find(Class("p-nav")).next().ok_or("p-nav does not exist")?;
                let user_info = parse_current_user(node)?;
                Ok(LoginResult::Success { user: cookies.get("xf_user").unwrap().to_string(), session: cookies.get("xf_session").unwrap().to_string(), info: user_info})
            } else {
                let node = document.find(Class("p-body")).next().ok_or("p-body does not exist")?;
                let login_info = parse_login_form(node)?;
                Ok(LoginResult::MFA { session: cookies.get("xf_session").unwrap().to_string(), token: login_info.token })
            }
        } else {
            Err("Incorrect login information. Please try again".into())
        }
        
    }

    pub async fn get_current_user(&self) -> Result<User, Box<dyn std::error::Error>> {
        let content = self.client.get("/").send().await?.text().await?;
        let document = Document::from_read(content.as_bytes()).ok().ok_or("Invalid request")?;
        let node = document.find(Class("p-nav")).next().ok_or("p-nav does not exist")?;
        let user_info = parse_current_user(node)?;
        Ok(user_info)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_categories() {
        let core = VozCore::new("voz.vn".to_string());
        let result = core.get_categories().await.voz_response();
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_forum() {
        let core = VozCore::new("voz.vn".to_string());
        let result = core.get_forum("17".to_string(), "f".to_string()).await.voz_response();
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_login() {
        let core = VozCore::new("voz.vn".to_string());
        let result = core.login("xxxxx".to_string(), "xxxxx".to_string()).await.voz_response();
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_current_user() {
        let core = VozCore::new("voz.vn".to_string());
        core.set_user("xxxxx".to_string(), "xxxxx".to_string());
        let result = core.get_current_user().await.voz_response();
        println!("{:?}", result);
    }
}
