extern crate hyper;
extern crate hyper_native_tls;
extern crate url;

use std::fmt;
use std::io::Read;

use self::hyper::status::StatusCode;
use self::hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use url::{ParseError, Url};

use crate::metadata;
use crate::web_crawler::link_checker;
use crate::web_crawler::parse;

#[derive(Debug, Clone)]
pub enum UrlState {
    Accessible(Url),
    BadStatus(Url, StatusCode),
    ConnectionFailed(Url),
    TimedOut(Url),
    Malformed(String),
    InvalidLink(String),
}

impl fmt::Display for UrlState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UrlState::Accessible(ref url) => format!("OK {}", url).fmt(f),
            UrlState::BadStatus(ref url, ref status) => format!("x {} ({})", url, status).fmt(f),
            UrlState::ConnectionFailed(ref url) => format!("x {} (connection failed)", url).fmt(f),
            UrlState::TimedOut(ref url) => format!("x {} (timed out)", url).fmt(f),
            UrlState::Malformed(ref url) => format!("x {} (malformed)", url).fmt(f),
            UrlState::InvalidLink(ref url) => {
                format!("x {} (Invalid or external Link)", url).fmt(f)
            }
        }
    }
}

/// Buils a URL from a domain and a path part
pub fn build_url(domain: &str, path: &str) -> Result<Url, ParseError> {
    let base_url = Url::parse(&domain)?;
    base_url.join(path)
}

pub fn url_status(domain: &str, path: &str) -> bool {
    return is_valid_path(domain, path);
}

/// Check if url is valid to crawl.
/// Dont follow external links here. Add them to a list of data to crawl.
fn is_valid_path(domain: &str, path: &str) -> bool {
    if path.starts_with("https://") || path.starts_with("http://") {
        let is_same = link_checker::url_has_same_origin_path(domain, path);
        if is_same {
            // Follow link - it's on the same domain
            println!(" Follow internal link");
            return true;
        } else {
            println!(" External link");
            let url_result: Url = Url::parse(&path).unwrap();
            let origin_path = url_result.origin().unicode_serialization();
            if origin_path.starts_with("http://localhost")
                || origin_path.starts_with("https://localhost")
            {
                return false;
            }
            // Tore to database
            metadata::add_new_url(&origin_path);
            return false;
        }
    }

    if path.starts_with("tel:")
        || path.starts_with("ftp:")
        || path.starts_with("mailto:")
        || path.starts_with('#')
    {
        println!(" Ignoring reference other than http: {}", path);
        return false;
    }

    if path.starts_with('/') {
        return true;
    }

    false
}

/// Fetch and parse target
pub fn parse_and_fetch_all_urls(url: &Url) -> Vec<String> {
    let html_src = fetch_url(url);
    let document = parse::parse_html(&html_src);
    parse::parse_for_index(&document, &url.to_string());
    parse::get_urls(&document)
}

pub fn fetch_url(url: &Url) -> String {
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);

    let url_string = url.to_string();
    let res = client.get(&url_string).send();

    match res {
        Ok(response) => {
            let mut body = String::new();
            let mut response = response;
            match response.read_to_string(&mut body) {
                Ok(_) => body,
                Err(_) => String::new(),
            }
        }
        Err(_) => String::new(),
    }
}
