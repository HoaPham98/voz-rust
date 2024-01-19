use reqwest::Url;
use select::{predicate::*, node::Node};
use crate::models::*;
use super::parse_utils::{parse_post_contents, TrimmedString};

enum Type {
    Quote, Code, Image, Spoiler, Embedded, Url, Table
}

impl Type {
    fn all() -> Vec<Self> {
        vec![Self::Quote, Self::Code, Self::Image, Self::Spoiler, Self::Embedded, Self::Url, Self::Table]
    }
    fn get_class(&self) -> String {
        let result = match self {
            Self::Quote => "bbCodeBlock--quote",
            Self::Code => "bbCodeBlock--code",
            Self::Image => "bbImageWrapper",
            Self::Spoiler => "bbCodeSpoiler",
            Self::Embedded => "bbMediaJustifier",
            Self::Url => "bbCodeBlock--unfurl",
            Self::Table => "table"
        };
        return result.to_string()
    }
}

fn get_content_type(class: &str) -> Option<Type> {
    Type::all().into_iter().find(|i| class.contains(i.get_class().as_str()))
}

pub fn parse_content(node: Node) -> Result<Option<ContentType>, Box<dyn std::error::Error>> {
    let class = node.attr("class").unwrap_or("");
    let mut _type = get_content_type(class);
    let mut x = node;
    if _type.is_none() {
        // Check if type is none but has child is image then add it as image
        _type = node.find(Class("bbImageWrapper")).next().map(|_| Type::Image);
        if _type.is_some() {
            x = node.find(Class("bbImageWrapper")).next().unwrap();
        }
    }
    if _type.is_none() {
        Ok(None)
    } else {
        match _type.unwrap() {
            Type::Quote => {
                let author_id = x.attr("data-attributes")
                            .unwrap_or("").split(" ").last()
                            .map(|i| i.to_string())
                            .filter(|i| !i.is_empty());
                let author_name = x.attr("data-quote").map(|i| i.to_string()).filter(|i| !i.is_empty());
                let post_id =  x.attr("data-source")
                            .unwrap_or("").split(" ").last()
                            .map(|i| i.to_string())
                            .filter(|i| !i.is_empty());
                let result = x.find(Class("bbCodeBlock-expandContent")).next()
                    .and_then(|i| parse_post_contents(i).ok())
                    .map(|n| ContentType::QuoteBlock { author_id, author_name, post_id, content: Box::new(n) });
                Ok(result)
            },
            Type::Code => {
                let lang = x.find(Attr("data-lang", ())).next().and_then(|i| i.attr("data-lang")).map(|s| s.to_string()).unwrap_or_default();
                let result = x.find(Name("code")).next().map(|i| ContentType::CodeBlock { language: lang, content: i.text().trimmed() });
                Ok(result)
            },
            Type::Image => {
                let result = x.attr("data-src").map(|s| ContentType::Image { src: parse_proxy_image(s) });
                Ok(result)
            },
            Type::Spoiler => {
                let title = x.find(Class("bbCodeSpoiler-button-title")).next().map(|i| i.text().trimmed()).unwrap_or_default();
                let content = x.find(Class("bbCodeBlock-content")).next().and_then(|i| parse_post_contents(i).ok());
                let result = content.map(|s| ContentType::Spoiler { title: title, content: Box::new(s) });
                Ok(result)
            },
            Type::Url => {
                let url = x.attr("data-url").map(|s| s.to_string()).ok_or("Not found url")?;
                let host = x.attr("data-host").map(|s| s.to_string()).unwrap_or_default();
                let thumbnail = x.find(Class("contentRow-figure").descendant(Name("img"))).next().and_then(|n| n.attr("src")).map(|s| parse_proxy_image(s));
                let title = x.find(Class("contentRow-header")).next().map(|n| n.text().trimmed()).unwrap_or(url.clone());
                let content = x.find(Class("contentRow-snippet")).next().map(|n| n.text().trimmed()).unwrap_or_default();
                let result = Some(ContentType::UrlBlock { thumbnail, title, content, host, url });
                Ok(result)
            },
            Type::Embedded => {
                let site = x.attr("data-media-site-id").unwrap_or_default().to_string();
                let title = x.attr("data-media-key").unwrap_or_default().to_string();
                let link = x.find(Attr("data-href", ())).next().and_then(|n| n.attr("data-href")).map(|s| s.to_string());
                let result = link.map(|s| ContentType::Embeded { site, title, link: s });
                Ok(result)
            },
            Type::Table => {
                // x.find(Name("table")).next().map(|n| n.html().trimmed()).map(|s| ContentType::Table { content: s })
                Ok(None)
            }
        }
    }
}

fn parse_proxy_image(uri: &str) -> String {
    let mut url_str = uri.to_string();
    if uri.starts_with("/proxy") {
        url_str = format!("https://voz.vn{uri}");
    }
    match Url::parse(&url_str) {
        Ok(url) => {
            let queries = url.query_pairs();
            for (k,v) in queries {
                if k == "image" {
                    return v.to_string();
                }
            }
            return uri.to_string();
        },
        Err(_) => uri.to_string()
    }
}

pub fn parse_reactions(node: Node) -> Option<ReactionSummary> {
    let icons = node.find(Class("reactionSummary").descendant(Name("li").descendant(Name("img")))).filter_map(|n| n.attr("src")).map(|s| s.to_string()).collect::<Vec<String>>();
    if icons.is_empty() {
        return None;
    }
    let message = node.find(Class("reactionsBar-link")).next();
    if message.is_none() {
        return None;
    }
    Some(ReactionSummary { icons, message: message.unwrap().text() })
}
