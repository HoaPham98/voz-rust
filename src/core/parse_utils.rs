use std::error::Error;

use select::{predicate::*, node::Node};

use super::models::*;

trait TrimmedString {
    fn trimmed(&self) -> String;
}

impl TrimmedString for String {
    fn trimmed(&self) -> String {
        self.as_str().trim().to_string()
    }
}

#[allow(dead_code)]
pub fn parse_catagories(node: Node) -> Result<Category, Box<dyn Error>> {
    let title = node.find(Class("block-header").descendant(Name("a"))).next().expect("title element not found").text();
    let mut errors = vec![];
    let forums = node.find(Class("node"))
                                    .filter_map(|child| parse_forum_item(child).map_err(|e| errors.push(e)).ok())
                                    .collect::<Vec<ForumItem>>();
    if errors.is_empty() {
        Ok(Category { title: title.trim().to_string(), forums: forums })
    } else {
        Err(errors.pop().unwrap())
    }
}

#[allow(dead_code)]
pub fn parse_forum_item(node: Node) -> Result<ForumItem, Box<dyn Error>> {
    let title = node.find(Name("h3")).next().expect("title element not found").text();
    let classes = node.attr("class").unwrap_or("");
    let mut meta = node.find(Class("pairs--rows").descendant(Name("dd")));
    let thread_number = meta.next().unwrap().text();
    let message_number = meta.next().unwrap().text();
    let is_unread = classes.contains("node--read");
    let forum_type = if classes.contains("node--forum") { "f" } else { "s" };
    let id_str = classes.split(' ').find(|x| x.starts_with("node--id")).unwrap();
    let id = id_str.split("node--id").last().unwrap();
    Ok(ForumItem {
        id: id.to_string(),
        title: title.trim().to_string(),
        thread_number: thread_number,
        message_number: message_number,
        forum_type: forum_type.to_string(),
        is_unread: is_unread
    })
}

#[allow(dead_code)]
pub fn parse_forum(node: Node) -> Result<Forum, Box<dyn Error>> {
    let title = node.find(Class("p-title-value")).next().expect("Title does not exist").text();
    let sub_forums = node.find(Class("node")).map(|x| parse_forum_item(x).unwrap()).collect::<Vec<ForumItem>>();
    let threads = node.find(Class("structItem--thread")).map(|x| parse_thread(x).unwrap()).collect::<Vec<ThreadItem>>();
    Ok(Forum { title, sub_forums, threads })
}

#[allow(dead_code)]
pub fn parse_thread(node: Node) -> Result<ThreadItem, Box<dyn Error>> {
    let prefix = parse_prefix(node).ok();
    let author = node.attr("data-author").unwrap().to_string();
    let classes = node.attr("class").unwrap();
    let id = classes
                        .split(" ").find(|x| x.starts_with("js-threadListItem-")).unwrap()
                        .split("-").last().unwrap().to_string();
    let title = node.find(Class("structItem-title").child(Attr("class", ""))).next().unwrap().text().trimmed();
    let replies = node.find(And(Class("pairs--justified"), Not(Class("structItem-minor"))).descendant(Name("dd"))).next().unwrap().text();
    // let views = node.find(And(Class("pairs--justified"), Class("structItem-minor")).descendant(Name("dd"))).next().unwrap().text();
    let latest = node.find(Class("structItem-latestDate")).next().unwrap().text().trimmed();
    let is_pinned = node.find(Class("structItem-status--sticky")).count() > 0;
    let is_read = !classes.contains("is-unread");

    Ok(ThreadItem { id, prefix, title, is_pinned, is_read, replies, latest, author })
}

#[allow(dead_code)]
pub fn parse_prefix(node: Node) -> Result<ThreadPrefix, Box<dyn Error>> {
    let prefix_node = node.find(Class("labelLink")).next().expect("prefix not found");
    let title = prefix_node.text().trimmed();
    let url = prefix_node.attr("href").expect("Prefix url not found");
    let id = url.split("prefix_id=").last().unwrap_or("0").to_string();
    let prefix_type = prefix_node.first_child().unwrap().attr("class").unwrap_or("").split("label--").last().unwrap().to_string();

    Ok(ThreadPrefix {
        id: id,
        title: title,
        prefix_type: prefix_type
    })
}

#[cfg(test)]
mod tests {
    use std::{path::Path, fs};

    use select::{document::Document, predicate::Class};

    use super::*;

    #[test]
    fn test_forum_item() {
        let path = Path::new("resources/tests/forum_item.html");
        let content = fs::read_to_string(path).expect("File not found");
        let document = Document::from_read(content.as_bytes()).expect("Invalid Html");
        let node = document.find(Class("node")).next().expect("Forum node not found");
        let forum = parse_forum_item(node).unwrap();
        let expected = ForumItem {
            id: "2".to_string(),
            title: "Thông báo".to_string(),
            thread_number: "18".to_string(),
            message_number: "47".to_string(),
            forum_type: "f".to_string(),
            is_unread: true
        };
        assert_eq!(forum, expected);
    }

    #[test]
    fn test_categories() {
        let path = Path::new("resources/tests/categories.html");
        let content = fs::read_to_string(path).expect("File not found");
        let document = Document::from_read(content.as_bytes()).expect("Invalid Html");
        
        let result = parse_catagories(document.nth(0).unwrap()).unwrap();
        
        assert_eq!(result.title, "Đại sảnh");
        assert_eq!(result.forums.len(), 5);
        assert_eq!(result.forums.into_iter().filter(|x| x.forum_type == "s").count(), 3);
    }

    #[test]
    fn test_thread() {
        let path = Path::new("resources/tests/thread_item.html");
        let content = fs::read_to_string(path).expect("File not found");
        let document = Document::from_read(content.as_bytes()).expect("Invalid Html");
        
        let result = parse_thread(document.nth(3).unwrap()).unwrap();
        let expectation = ThreadItem { id: "73313".to_string(), prefix: Some(ThreadPrefix { id: "17".to_string(), title: "kiến thức".to_string(), prefix_type: "royalBlue".to_string() }), title: "[Dịch] Hướng dẫn OC DDR4".to_string(), is_pinned: true, is_read: false, replies: "1K".to_string(), latest: "Yesterday at 11:03 AM".to_string(), author: "troll159753".to_string() };
        assert_eq!(result, expectation);
    }
}