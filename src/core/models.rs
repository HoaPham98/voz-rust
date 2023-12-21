use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum VozResponse<T: Serialize> {
    Success { data: T },
    Failed { message: String }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ForumItem {
    title: String,
    thread_number: String,
    message_number: String,
    is_unread: bool
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    title: String,
    forums: Vec<ForumItem>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Forum {
    title: String,
    sub_forums: Vec<ForumItem>,
    threads: Vec<ThreadItem>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ThreadItem {
    prefix:  Option<ThreadPrefix>,
    title: String,
    is_pinned: String,
    is_read: String,
    replies: String,
    latest: String,
    author: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ThreadPrefix {
    id: String,
    title: String,
    text_color: String,
    border_color: String,
    background_color: String
}
