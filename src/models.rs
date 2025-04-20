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

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub avatar: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag="type", rename_all = "camelCase")]
pub enum LoginResult {
    MFA { url: String },
    Success { user: String, session: String, tfa_trust: Option<String>, info: User }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    pub prefix:  Option<ThreadPrefix>,
    pub title: String,
    pub current_page: String,
    pub total_page: String,
    pub can_reply: bool,
    pub posts: Vec<Post>,
    pub posts_html: String,
    pub reactions: Vec<Reaction>
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
    pub html_content: String,
    pub warning_message: Option<String>,
    pub position: i64,
    pub can_edit: bool,
    pub can_delete: bool,
    pub can_reply: bool,
    pub can_multiple_quote: bool,
    pub can_react: bool,
    pub is_reacted_to: bool,
    pub visitor_reaction_id: Option<i64>,
    pub reactions: Option<ReactionSummary>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReactionSummary {
    pub icons: Vec<String>,
    pub message: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Reaction {
    pub id: i64,
    pub icon: String,
    pub title: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ContentType {
    Html { content: String },
    Image { src: String },
    QuoteBlock { author_id: Option<String>, author_name: Option<String>, post_id: Option<String>, content: Box<Vec<ContentType>> },
    CodeBlock { language: String, content: String },
    UrlBlock { thumbnail: Option<String>, title: String, content: String, host: String, url: String },
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
