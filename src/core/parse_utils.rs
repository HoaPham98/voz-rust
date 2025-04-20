use std::{error::Error, collections::HashMap};

use select::{predicate::*, node::Node};

use crate::models::*;

use super::post_parse_utils::{parse_content, parse_reactions, parse_list_reactions};

pub trait TrimmedString {
    fn trimmed(&self) -> String;
}

impl TrimmedString for String {
    fn trimmed(&self) -> String {
        self.as_str().trim().to_string()
    }
}

pub fn parse_catagories(node: Node) -> Result<Category, Box<dyn Error>> {
    let title = node.find(Class("block-header").descendant(Name("a"))).next().ok_or("title element not found")?.text();
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

pub fn parse_forum_item(node: Node) -> Result<ForumItem, Box<dyn Error>> {
    let title = node.find(Name("h3")).next().ok_or("title element not found")?.text();
    let classes = node.attr("class").unwrap_or("");
    let mut meta = node.find(Class("pairs--rows").descendant(Name("dd")));
    let thread_number = meta.next().ok_or("Cannot find thread number node")?.text();
    let message_number = meta.next().ok_or("Cannot find message number node")?.text();
    let is_read = classes.contains("node--read");
    let forum_type = if classes.contains("node--forum") { "f" } else { "s" };
    let id_str = classes.split(' ').find(|x| x.starts_with("node--id")).ok_or("Cannot find forum id class")?;
    let id = id_str.split("node--id").last().ok_or("Cannot find forum id")?;
    Ok(ForumItem {
        id: id.to_string(),
        title: title.trim().to_string(),
        thread_number: thread_number,
        message_number: message_number,
        forum_type: forum_type.to_string(),
        is_read: is_read
    })
}

pub fn parse_forum(node: Node) -> Result<Forum, Box<dyn Error>> {
    let title = node.find(Class("p-title-value")).next().ok_or("Title does not exist")?.text();
    let sub_forums = node.find(Class("node")).filter_map(|x| parse_forum_item(x).ok()).collect::<Vec<ForumItem>>();
    let threads = node.find(Class("structItem--thread")).filter_map(|x| parse_thread(x).ok()).collect::<Vec<ThreadItem>>();
    Ok(Forum { title, sub_forums, threads })
}

pub fn parse_thread(node: Node) -> Result<ThreadItem, Box<dyn Error>> {
    let prefix = parse_prefix(node);
    let author = node.attr("data-author").unwrap_or_default().to_string();
    let classes = node.attr("class").unwrap_or_default();
    let id = classes
                        .split(" ").find(|x| x.starts_with("js-threadListItem-")).ok_or("Cannot find thread id class")?
                        .split("-").last().ok_or("Cannot find thread id")?.to_string();
    let title = node.find(Class("structItem-title").child(Attr("class", ""))).next().ok_or("Cannot find thread title data")?.text().trimmed();
    let replies = node.find(And(Class("pairs--justified"), Not(Class("structItem-minor"))).descendant(Name("dd"))).next().ok_or("Cannot find thread replies data")?.text();
    // let views = node.find(And(Class("pairs--justified"), Class("structItem-minor")).descendant(Name("dd"))).next().unwrap().text();
    let latest = node.find(Class("structItem-latestDate")).next().ok_or("Cannot find thread latest reply data")?.text().trimmed();
    let is_pinned = node.find(Class("structItem-status--sticky")).count() > 0;
    let is_read = !(classes.contains("is-unread"));

    Ok(ThreadItem { id, prefix, title, is_pinned, is_read, replies, latest, author })
}

pub fn parse_prefix(node: Node) -> Option<ThreadPrefix> {
    match node.find(Class("labelLink")).next() {
        Some(prefix_node) => {
            let title = prefix_node.text().trimmed();
            let url = prefix_node.attr("href")?;
            let id = url.split("prefix_id=").last().unwrap_or("0").to_string();
            let prefix_type = prefix_node.first_child().unwrap().attr("class").unwrap_or("").split("label--").last().unwrap().to_string();

            Some(ThreadPrefix {
                id: id,
                title: title,
                prefix_type: prefix_type
            })
        },
        None => None
    }
    
}

pub fn parse_login_form(node: Node) -> Result<LoginInfo, Box<dyn Error>> {
    let form = node.find(Name("form")).next().ok_or("Login form should be existed")?;
    let action = form.attr("action").ok_or("Form action not found")?.to_string();
    let token = form.find(And(Name("input"), Attr("name", "_xfToken"))).next().ok_or("XF Token is not found")?.attr("value").unwrap_or("").to_string();

    Ok(LoginInfo { url: action, token: token })
}

pub fn parse_current_user(node: Node) -> Result<User, Box<dyn Error>> {
    let div = node.find(Class("p-account").descendant(Class("avatar--xxs"))).next().ok_or("Not login yet")?;
    let id = div.attr("data-user-id").unwrap_or("").to_string();
    let info_node = div.find(Name("img")).next().ok_or("Not found avatar node")?;
    let name = info_node.attr("alt").unwrap_or("").to_string();
    let avatar = info_node.attr("src").unwrap_or("").to_string();

    Ok(User { id, name, avatar })
}

pub fn parse_thread_detail(node: Node) -> Result<Thread, Box<dyn Error>> {
    let title = node.find(Class("p-title-value")).next().ok_or("Not found thread title")?.text();
    let total_page_node = node.find(Class("pageNav-main")).next();
    let mut current_page = "1".to_string();
    let mut total_page = "1".to_string();
    let can_reply = node.find(And(Name("form"), Class("js-quickReply"))).next().is_some();
    match total_page_node {
        Some(p_node) => {
            let children = p_node.find(Class("pageNav-page"));
            children.for_each(|x| {
                if x.attr("class").unwrap_or("").contains("pageNav-page--current") {
                    current_page = x.text().trimmed();
                }
                total_page = x.text().trimmed();
            })
        },
        None => {}
    }
    let content = node.find(And(Name("article"), Class("js-post"))).map(|x| parse_post(x).unwrap()).collect::<Vec<Post>>();
    let reactions = node.find(Attr("id", "xfReactTooltipTemplate")).next().and_then(|n| parse_list_reactions(n.text()).ok()).unwrap_or_default();
    let posts_html = node.find(And(Name("article"), Class("js-post"))).map(|x| x.html()).collect::<Vec<String>>().join("").replace("\n", "");
    Ok(Thread { title, current_page, total_page, can_reply, posts: content, posts_html, prefix: None, reactions })
}

pub fn parse_post(node: Node) -> Result<Post, Box<dyn Error>> {
    let mut post_info = node.find(Class("u-anchorTarget")).next().ok_or("Not found post info node")?.attr("id").unwrap_or_default().split("-").into_iter();
    let post_type = post_info.next().map(|s| s.to_string()).ok_or("Not found post id")?;
    let post_id = post_info.next().map(|s| s.to_string()).ok_or("Not found post id")?;
    let user_node = node.find(Class("message-user")).next().ok_or("Not found user node for post")?;
    let author_id = user_node.find(Class("avatar--m")).next().ok_or("Not found author id node")?.attr("data-user-id").ok_or("Not found author id")?.to_string();
    let author_name = node.attr("data-author").ok_or("Not found author name attr")?.to_string().trimmed();
    let avatar_node = user_node.find(Class("avatar--m")).next();
    let author_avatar = parse_avatar_image(avatar_node, author_name.clone());
    let created = node.find(Class("message-attribution-main").descendant(Name("time"))).next().ok_or("Not found created node")?.text().trimmed();
    let last_edited: Option<String> = node.find(Class("message-lastEdit").descendant(Name("time"))).next().map(|n| n.text().trimmed());
    let reactions: Option<ReactionSummary> = node.find(And(Class("reactionsBar"), Class("is-active"))).next().and_then(|n| parse_reactions(n));
    let content_node = node.find(Class("message-body").descendant(Class("bbWrapper"))).next().ok_or("Not found content node")?;
    let html_content: String = content_node.html();
    let can_edit = node.find(Class("actionBar-action--edit")).count() > 0;
    let can_delete = node.find(Class("actionBar-action--delete")).count() > 0;
    let can_react = node.find(Class("actionBar-action--reaction")).count() > 0;
    let can_reply = node.find(Class("actionBar-action--reply")).count() > 0;
    let can_multiple_quote = node.find(Class("actionBar-action--reply")).count() > 0;
    let is_reacted_to = node.find(Class("has-reaction")).count() > 0;
    let visitor_reaction_id = node.find(Class("has-reaction")).next().and_then(|n| n.attr("data-reaction-id")).and_then(|s| s.parse::<i64>().ok());
    let position = node.find(Class("message-attribution-opposite--list").descendant(Name("li"))).last().and_then(|n| n.text().replace("#", "").parse::<i64>().ok()).unwrap_or(0);
    Ok(Post { post_id, post_type, author_id, author_name, author_avatar, created, last_edited, reactions, html_content, warning_message: None, position, can_edit, can_delete, can_react, is_reacted_to, visitor_reaction_id, can_reply, can_multiple_quote })
}

pub fn parse_post_contents(node: Node) -> Result<Vec<ContentType>, Box<dyn Error>> {
    let mut content_string: String = "".to_string();
    let mut results: Vec<ContentType> = vec![];
    for x in node.children() {
        let content_type = parse_content(x)?;
        if content_type.is_some() {
            if !content_string.trimmed().is_empty() && content_string.trimmed().ne("<br>") {
                results.push(ContentType::Html { content: content_string.clone() });
                content_string.clear();
            }
            results.push(content_type.unwrap())
        } else if x.name().is_some_and(|s| s.eq("script")) {

        } else {
            content_string += x.html().trimmed().as_str();
        }
    }
    if !content_string.is_empty() && content_string.ne("<br>") {
        results.push(ContentType::Html { content: content_string.clone() });
    }
    Ok(results)
}

fn parse_avatar_image(node: Option<Node>, username: String) -> String {
    let letter = username.replace(" ", "+");
    let base = format!("https://ui-avatars.com/api/?length=1&rounded=true&name={letter}");
    match node {
        Some(node) => {
            let class = node.attr("class").unwrap_or_default();
            if class.contains("avatar--default--dynamic") {
                let style = node.attr("style").unwrap_or_default().split("; ").filter_map(|p| p.split_once(": #")).collect::<HashMap<_,_>>();
                let background_color = style.get("background-color").map(|s| format!("&background={s}")).unwrap_or_default();
                let color = style.get("color").map(|s| format!("&color={s}")).unwrap_or_default();
                format!("{base}{background_color}{color}")
            } else {
                node.find(Name("img")).next().and_then(|n| n.attr("src")).map(|s| s.to_string()).unwrap_or(base)
            }
        },
        None => base
    }
}

#[cfg(test)]
mod tests {
    use std::{path::Path, fs};

    use select::{document::Document, predicate::Class};

    use super::*;

    #[test]
    fn test_post() {
        let path = Path::new("resources/tests/post.html");
        let content = fs::read_to_string(path).expect("File not found");
        let document = Document::from_read(content.as_bytes()).expect("Invalid Html");
        let result = parse_post_contents(document.find(Class("message-body").descendant(Class("bbWrapper"))).next().unwrap()).unwrap();
        // assert_eq!(result.len(), 5);
        if let ContentType::CodeBlock { .. } = result[2] {
            assert!(true);
        } else {
            assert!(false);
        }
    }

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
            is_read: true
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

    #[test]
    fn test_forum() {
        let path = Path::new("resources/tests/forum.html");
        let content = fs::read_to_string(path).expect("File not found");
        let document = Document::from_read(content.as_bytes()).expect("Invalid Html");
        
        let result = parse_forum(document.nth(3).unwrap()).unwrap();
        assert_eq!(result.sub_forums.len(), 2);
    }

    #[test]
    fn test_login_form() {
        let path = Path::new("resources/tests/login_form.html");
        let content = fs::read_to_string(path).expect("File not found");
        let document = Document::from_read(content.as_bytes()).expect("Invalid Html");
        
        let result = parse_login_form(document.nth(3).unwrap()).unwrap();
        assert_eq!(result.token, "1703298707,4eca196109282894d9e1576d23e489fd");
        assert_eq!(result.url, "/login/login");
    }

    #[test]
    fn test_user() {
        let path = Path::new("resources/tests/current_user.html");
        let content = fs::read_to_string(path).expect("File not found");
        let document = Document::from_read(content.as_bytes()).expect("Invalid Html");
        
        let result = parse_current_user(document.nth(3).unwrap()).unwrap();
        assert_eq!(result.id, "1932329");
        assert_eq!(result.avatar, "https://data.voz.vn/avatars/s/1932/1932329.jpg?1700878980");
    }

    // #[test]
    // fn test_thread_detail() {
    //     let path = Path::new("resources/tests/thread.html");
    //     let content = fs::read_to_string(path).expect("File not found");
    //     let document = Document::from_read(content.as_bytes()).expect("Invalid Html");
        
    //     let result = parse_new_thread_detail(document.nth(3).unwrap()).unwrap();
    //     assert_eq!(result.current_page, "1");
    //     assert_eq!(result.total_page, "3");
    //     assert_eq!(result.content.split("</article>").count(), 41);
    //     assert_eq!(result.can_reply, true);
    // }

    #[test]
    fn test_thread_detail() {
        let path = Path::new("resources/tests/thread.html");
        let content = fs::read_to_string(path).expect("File not found");
        let document = Document::from_read(content.as_bytes()).expect("Invalid Html");
        
        let result = parse_thread_detail(document.nth(3).unwrap()).unwrap();
        assert_eq!(result.current_page, "1");
        assert_eq!(result.total_page, "3");
        assert_eq!(result.posts.len(), 20);
        assert_eq!(result.can_reply, true);
    }
}