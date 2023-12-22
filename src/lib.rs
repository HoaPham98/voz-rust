mod core;
use std::fmt::Debug;
use core::{models::*, parse_utils::{parse_catagories, parse_forum}};
use reqwest::{Client, ClientBuilder, header::{HeaderMap, HOST, ACCEPT}};
use select::{document::Document, predicate::Class};
use serde::Serialize;

trait VozResponseMapping<T: Serialize> {
    fn map(self) -> VozResponse<T>;
}

impl<T: Serialize + Debug> VozResponseMapping<T> for Result<T, Box<dyn std::error::Error>> {
    fn map(self) -> VozResponse<T> {
        if self.is_ok() {
            VozResponse::Success { data: self.unwrap() }
        } else {
            VozResponse::Failed { message: self.unwrap_err().to_string() }
        }
    }
}

pub struct VozCore {
    base_url: String,
    client: Client
}

impl VozCore {
    pub fn new(base_url: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.append(HOST, base_url.parse().unwrap());
        headers.append(ACCEPT, "*/*".parse().unwrap());
        VozCore {
            base_url: base_url,
            client: ClientBuilder::new()
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .default_headers(headers)
                .build().unwrap()
        }
    }
}

impl VozCore {
    pub async fn get_categories(&self) -> Result<Vec<Category>, Box<dyn std::error::Error>> {
        let content = self.client.get(format!("{}", self.base_url)).send().await?.text().await?;
        let document = Document::from_read(content.as_bytes()).expect("Invalid request");
        let results = document.find(Class("block--category")).map(|x| parse_catagories(x).unwrap()).collect::<Vec<Category>>();
        Ok(results)
    }

    pub async fn get_forum(&self, id: String, forum_type: String) -> Result<Forum, Box<dyn std::error::Error>> {
        let content = self.client.get(format!("{0}/{forum_type}/{id}", self.base_url)).send().await?.text().await?;
        let document = Document::from_read(content.as_bytes()).expect("Invalid request");
        let node = document.find(Class("p-body")).next().expect("p-body does not exist");
        let result = parse_forum(node).unwrap();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_categories() {
        let core = VozCore::new("https://voz.vn".to_owned());
        let result = core.get_categories().await;
        println!("{:?}", result);
    }
}
