use std::{fmt::Debug, collections::HashMap};
use parse_utils::{parse_catagories, parse_forum, parse_login_form, parse_current_user, parse_thread_detail};
use select::{document::Document, predicate::Class};
use serde::Serialize;
use session::Session;
use models::*;

use super::{models, session, parse_utils};
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
    pub fn set_user(&self, user: String, session: String, tfa: Option<String>) {
        self.client.set_cookie("xf_user".to_string(), user);
        self.client.set_cookie("xf_session".to_string(), session);
        if tfa.is_some() {
            self.client.set_cookie("xf_tfa_trust".to_string(), tfa.unwrap());
        }
    }
    
    pub async fn get_categories(&self) -> Result<Vec<Category>, Box<dyn std::error::Error>> {
        let content = self.client.get("/").send().await?.text().await?;
        let document = Document::from_read(content.as_bytes()).ok().ok_or("Invalid request")?;
        let results = document.find(Class("block--category")).filter_map(|x| parse_catagories(x).ok()).collect::<Vec<Category>>();
        Ok(results)
    }

    pub async fn get_forum(&self, id: String, forum_type: String, page: i64) -> Result<Forum, Box<dyn std::error::Error>> {
        let content = self.client.get(format!("/{forum_type}/{id}/page-{page}")).send().await?.text().await?;
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
        let content = self.client.post(login_info.url).form(&form).send().await?.text().await?;
        let document = Document::from_read(content.as_bytes()).ok().ok_or("Invalid request")?;
        let cookies = self.client.get_cookies();
        if cookies.contains_key("xf_session") {
            if cookies.contains_key("xf_user") {
                let node = document.find(Class("p-nav")).next().ok_or("p-nav does not exist")?;
                let user_info = parse_current_user(node)?;
                Ok(LoginResult::Success { user: cookies.get("xf_user").unwrap().to_string(), session: cookies.get("xf_session").unwrap().to_string(), tfa_trust: None, info: user_info})
            } else {
                let node = document.find(Class("p-body")).next().ok_or("p-body does not exist")?;
                let login_info = parse_login_form(node)?;
                Ok(LoginResult::MFA { url: login_info.url })
            }
        } else {
            Err("Incorrect login information. Please try again".into())
        }
        
    }

    pub async fn mfa(&self, url: String, code: String, provider: String) -> Result<LoginResult, Box<dyn std::error::Error>> {
        let form: HashMap<&str, _> = HashMap::from([
            ("_xfToken", self.client.get_csrf().unwrap_or_default()),
            ("trust", 1.to_string()),
            ("confirm", 1.to_string()),
            ("remember", 1.to_string()),
            ("code", code),
            ("provider", provider)
        ]);
        let content = self.client.post(url).form(&form).send().await?.text().await?;
        let cookies = self.client.get_cookies();
        if cookies.contains_key("xf_user") {
            let document = Document::from_read(content.as_bytes()).ok().ok_or("Invalid request")?;
            let node = document.find(Class("p-nav")).next().ok_or("p-nav does not exist")?;
            let user_info = parse_current_user(node)?;
            Ok(LoginResult::Success { user: cookies.get("xf_user").unwrap().to_string(), session: cookies.get("xf_session").unwrap().to_string(), tfa_trust: cookies.get("xf_tfa_trust").map(|a| a.to_string()), info: user_info})
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

    pub async fn get_thread(&self, id: String, page: Option<i64>) -> Result<Thread, Box<dyn std::error::Error>> {
        let uri = match page {
            Some(p) => format!("page-{p}"),
            None => "unread".to_string()
        };
        let content = self.client.get(format!("/t/{id}/{uri}")).send().await?.text().await?;
        let document = Document::from_read(content.as_bytes()).ok().ok_or("Invalid request")?;
        let node = document.nth(0).ok_or("p-body does not exist")?;
        let result = parse_thread_detail(node)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;

    #[tokio::test]
    async fn test_categories() {
        let core = VozCore::new("voz.vn".to_string());
        let result = core.get_categories().await.voz_response();
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_forum() {
        let core = VozCore::new("voz.vn".to_string());
        let result = core.get_forum("17".to_string(), "f".to_string(), 1).await.voz_response();
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
        core.set_user("xxxxx".to_string(), "xxxxx".to_string(), None);
        let result = core.get_current_user().await.voz_response();
        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_new_thread() -> Result<(), Box<dyn std::error::Error>> {
        let core = VozCore::new("voz.vn".to_string());
        let result = core.get_thread("899758".to_string(), Some(1)).await?;
        let json_str = serde_json::to_string_pretty(&result).unwrap();
        let mut file = std::fs::File::create("output.json").ok().ok_or("Error")?;
        file.write_all(json_str.as_bytes())?;
        // for post in result.posts {
        //     println!("{:?}", post.contents);
        // }
        Ok(())
    }
}
