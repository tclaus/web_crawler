extern crate hyper;
extern crate hyper_native_tls;
extern crate url;

use std::fmt;
use std::io::Read;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use self::hyper::status::StatusCode;
use self::hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use url::{ParseError, Url};

use super::link_checker;
use super::parse;
use crate::metadata;

const TIMEOUT: u64 = 3;

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
            UrlState::InvalidLink(ref url) => format!("x {} (invalid link)", url).fmt(f),
        }
    }
}

fn build_url(domain: &str, path: &str) -> Result<Url, ParseError> {
    let base_url = Url::parse(&domain)?;
    base_url.join(path)
}

pub fn url_status(domain: &str, path: &str) -> UrlState {
    println!("Fetch URL Domain, path: {},{}", domain, path);

    // Ignore known invalid links
    if !is_valid_path(domain, path) {
        return UrlState::InvalidLink(path.to_string());
    }

    match build_url(domain, path) {
        Ok(url) => {
            let (tx, rx) = channel();
            let req_tx = tx.clone();
            let u = url.clone();

            thread::spawn(move || {
                let ssl = NativeTlsClient::new().unwrap();
                let connector = HttpsConnector::new(ssl);
                let client = Client::with_connector(connector);

                let url_string = url.to_string();

                println!(" Fetching {}", &url_string);
                let resp = client.get(&url_string).send();

                let _ = req_tx.send(match resp {
                    Ok(r) => {
                        if let StatusCode::Ok = r.status {
                            println!(" Response: OK");
                            // TODO: Parse here!
                            UrlState::Accessible(url)
                        } else {
                            println!(" Response: Bad Status ({})", r.status);
                            UrlState::BadStatus(url, r.status)
                        }
                    }
                    Err(_) => UrlState::ConnectionFailed(url),
                });
            });

            // Timeout watcher
            thread::spawn(move || {
                thread::sleep(Duration::from_secs(TIMEOUT));
                let _ = tx.send(UrlState::TimedOut(u));
            });

            rx.recv().unwrap()
        }
        Err(_) => {
            println!("ERROR");
            UrlState::Malformed(path.to_owned())
        }
    }
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

pub fn fetch_url(url: &Url) -> String {
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    //TODO: Here add caller header
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

pub fn fetch_all_urls(url: &Url) -> Vec<String> {
    let html_src = fetch_url(url);
    let dom = parse::parse_html(&html_src);
    parse::get_urls(dom.document)
}
