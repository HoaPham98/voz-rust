use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum VozResponse<T: Serialize> {
    Success { data: T },
    Failed { message: String }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ForumItem {
    pub id: String,
    pub title: String,
    pub thread_number: String,
    pub message_number: String,
    pub forum_type: String,
    pub is_read: bool
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub title: String,
    pub forums: Vec<ForumItem>
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Forum {
    pub title: String,
    pub sub_forums: Vec<ForumItem>,
    pub threads: Vec<ThreadItem>
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ThreadItem {
    pub id: String,
    pub prefix:  Option<ThreadPrefix>,
    pub title: String,
    pub is_pinned: bool,
    pub is_read: bool,
    pub replies: String,
    pub latest: String,
    pub author: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ThreadPrefix {
    pub id: String,
    pub title: String,
    pub prefix_type: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    pub title: String,
    pub current_page: String,
    pub total_page: String,
    pub content: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub avatar: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag="type", rename_all = "camelCase")]
pub enum LoginResult {
    MFA { token: String, url: String },
    Success { user: String, session: String, info: User }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewThread {
    pub title: String,
    pub current_page: String,
    pub total_page: String,
    pub posts: Vec<Post>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub post_id: String,
    pub post_type: String,
    pub author_id: String,
    pub author_name: String,
    pub author_avatar: String,
    pub created: String,
    pub last_edited: Option<String>,
    pub reactions: Option<String>,
    pub html_content: String,
    pub contents: Vec<ContentType>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ContentType {
    HTML { content: String },
    QuoteBlock { author_id: Option<String>, author_name: Option<String>, post_id: Option<String>, content: Box<Vec<ContentType>> },
    CodeBlock { language: String, content: String },
    Spoiler { title: String, content: Box<Vec<ContentType>> },
    Embeded { site: String, title: String, link: String },
    Table { content: String }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoginInfo {
    pub url: String,
    pub token: String
}
