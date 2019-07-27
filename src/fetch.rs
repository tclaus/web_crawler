extern crate hyper;
extern crate hyper_native_tls;
extern crate url;

use std::io::Read;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::channel;
use std::fmt;

use self::hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use self::hyper::status::StatusCode;
use url::{Url, ParseError};


use crate::parse;

const TIMEOUT: u64 = 10;

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

fn build_url(domain: &str, path: &str) ->  Result<Url, ParseError> {
    let base_url = Url::parse(&domain)?;
    println!("Build_url: {}", base_url);
    base_url.join(path)
}

pub fn url_status(domain: &str, path: &str) -> UrlState {
   println!("Fetch URL Domain, path: {},{}",domain, path);

    // Ignore known invalid links
    if path.starts_with("mailto") {
        UrlState::InvalidLink(path.to_owned());
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
                let resp = client.get(&url_string).send();
            
                let _ = req_tx.send(match resp {
                    Ok(r) => if let StatusCode::Ok = r.status {
                        UrlState::Accessible(url)
                    } else {
                        UrlState::BadStatus(url, r.status)
                    },
                    Err(_) => UrlState::ConnectionFailed(url),
                });
            });

            thread::spawn(move || {
                thread::sleep(Duration::from_secs(TIMEOUT));
                let _ = tx.send(UrlState::TimedOut(u));
            });

            rx.recv().unwrap()
        }
        Err(_) => {
            println!("ERROR");
                UrlState::Malformed(path.to_owned())
            } ,
    }
}

pub fn fetch_url(url: &Url) -> String {
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);

    let url_string = url.to_string();
    let mut res = client
        .get(&url_string)
        .send()
        .ok()
        .expect("Unknown url");
    
    let mut body = String::new();
    // TODO: Here parse body for indexing 
    match res.read_to_string(&mut body) {
        Ok(_) => body,
        Err(_) => String::new(),
    }
}

pub fn fetch_all_urls(url: &Url) -> Vec<String> {
    let html_src = fetch_url(url);
    let dom = parse::parse_html(&html_src);

    parse::get_urls(dom.document)
}