use crate::es::es::WebDocument;
use crate::es::*;
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};
use std::string::String;

pub fn parse_html(source: &str) -> Document {
    Document::from(source)
}

/// Parses the contents of handle with the URL
pub fn parse_for_index(document: &Document, url: &str) {
    // Idee: Verschiedene Seiten müsten unterschiedlich gescannt werden?
    //       Niccht jede Seite hat die wichtigen Dinge als H1..Hn Tag
    //
    // Page Title * Save
    // Description   *save
    // H1..Hn - Tags (no save)
    // URL.
    // Url tags
    // Keywords aus Meta Tags
    // Date ( automatisch in elastic search?
    // Hat die Seite selber ein Date? )
    let title = get_title(&document); // keywords, Visible in Search Results
    let description = get_description(&document); // keywords, visible in search results
                                                  // Keywords
                                                  // let headlines = get_head_tags(&document);

    let web_document = WebDocument {
        url: url.to_owned(),
        title: title.to_owned(),
        description: description.to_owned(),
    };

    es::add_to_index(web_document);

    // println!("Title: {}, description:{}", title, description);
    //println!("Headlines: {:?}", headlines);
    // println!("-----------------");
}

pub fn get_urls(document: &Document) -> Vec<String> {
    let mut anchor_tags = vec![];
    for node in document.find(Name("a")) {
        let a_link = node.attr("href");
        if a_link.is_some() {
            anchor_tags.push(a_link.unwrap().to_string());
        }
    }
    anchor_tags
}

fn get_title(document: &Document) -> String {
    for node in document.find(Name("title")) {
        return node.text().trim().to_string();
    }
    "".to_string() // Document has no title. Sad.
}

fn get_description(document: &Document) -> String {
    if let Some(description) =
        get_meta_tag_content(document, "name".to_string(), "description".to_string())
    {
        return description;
    }

    if let Some(description) = get_meta_tag_content(
        document,
        "property".to_string(),
        "og:description".to_string(),
    ) {
        return description;
    }
    //twitter:description
    if let Some(description) = get_meta_tag_content(
        document,
        "property".to_string(),
        "twitter:description".to_string(),
    ) {
        return description;
    }
    "".to_string() // Document has no description. Sad. Get some text from body?
}

fn get_meta_tag_content(document: &Document, name: String, value: String) -> Option<String> {
    for node in document.find(Name("meta")) {
        if let Some(node_name) = node.attr(&name) {
            if node_name.eq_ignore_ascii_case(&value) {
                if let Some(content) = node.attr("content") {
                    return Some(content.trim().to_string());
                }
            }
        }
    }
    None
}

fn get_head_tags(document: &Document) -> Vec<String> {
    let mut headlines = Vec::new();

    for node in document.find(Name("h1")) {
        headlines.push(node.text());
    }
    headlines
}
