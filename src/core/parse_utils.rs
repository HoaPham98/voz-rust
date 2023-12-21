use std::error::Error;

use super::models::ForumItem;

pub fn parse_catagories() {

}

pub fn parse_forum() -> Result<ForumItem, Box<dyn Error>> {

    Err("Failed".into())
}

#[cfg(test)]
mod tests {
    use std::{path::Path, fs};

    use super::*;
    use lol_html::{element, HtmlRewriter, Settings};

    #[test]
    fn test_forum_item() {
        let path = Path::new("resources/tests/forum_item.html");
        let content = fs::read_to_string(path).unwrap();
        
    }
}