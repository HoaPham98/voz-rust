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
    MFA { session: String, token: String },
    Success { user: String, session: String, info: User }
}

pub struct LoginInfo {
    pub url: String,
    pub token: String
}
