extern crate bloom;

use crate::web_crawler::fetch::build_url;
use super::robots::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use url::Url;

use bloom::BloomFilter;
use bloom::ASMS;

const THREADS: i32 = 20;
const MAX_URL_LENGTH: u32 = 100;
const MAX_LINK_DEEPH: u8 = 3;

use super::fetch::{parse_and_fetch_all_urls, fetch_url, url_status};

#[derive(Debug)]
pub struct UrlData {
    url: String,
    deeph: u8,
}

fn init_bloom() -> BloomFilter {
    let expected_num_items = 1000;
    let false_positive_rate = 0.01;

    BloomFilter::with_rate(false_positive_rate, expected_num_items)
}

// Start crawling this url
pub fn crawl_start_url(start_url_string: &str) {
    let start_url = Url::parse(start_url_string).unwrap();

    let origin = start_url.origin();
    println!(" Origin URL {}", origin.unicode_serialization());

    // OK: TODO: Step1: loop through read urls from database
    // OK: TODO: Step2: read from database, loop through all URLS, wait and loop again (with lastread< days / hours.. )
    // TODO: Stop3: Read chunks from database (expect data to grow significant)

    let robots_value = load_robot_value(&start_url);
    crawl(&origin.unicode_serialization(), &start_url, &robots_value);
}

fn crawl(origin: &str, start_url: &Url, robots_value: &str) {
    let start_url_data = UrlData {
        url: start_url.to_string(),
        deeph: 0,
    };
    println!(" {:?} ", start_url_data);
    println!(" New Crawl of {}", start_url);
    let mut url_hash = HashMap::new();
    url_hash.insert(start_url.to_string(), start_url_data);

    let to_visit = Arc::new(Mutex::new(url_hash));
    let filter = Arc::new(Mutex::new(init_bloom()));

    println!(" Crawling: origin:{}, Path:{}", origin, start_url);
    println!(" Generating Threads");

    let mut threads = Vec::new();

    for _ in 0..THREADS {
        let origin = origin.to_owned();
        let to_visit = to_visit.clone();
        let robots_value = robots_value.to_owned();
        let filter = filter.clone();

        let handler =  thread::spawn(move || {
            crawl_worker_thread(
                &origin,
                to_visit,
                &robots_value,
                filter,
            );
        });
        threads.push(handler);
    }
    for handler in threads {
        handler.join().expect("Thread can not joined");
    }
    println!("Finish craling");
}

fn get_first_element(hash_map: &mut HashMap<String, UrlData>) -> UrlData {

    let urldata = match hash_map.values().next() {
        Some(url_data) => UrlData {
            url: url_data.url.clone(),
            deeph: url_data.deeph,
        },
        None => UrlData {
            url: String::new(),
            deeph: 0,
        },
    };

    hash_map.remove(&urldata.url);
    urldata
}

fn crawl_worker_thread(
    origin: &str,
    to_visit: Arc<Mutex<HashMap<String, UrlData>>>,
    robots_value: &str,
    filter: Arc<Mutex<BloomFilter>>,
) {

    loop {
        let current: UrlData;
        {
            let mut to_visit_val = to_visit.lock().unwrap();
            if to_visit_val.is_empty() {
                break;
            };
            // Get a new url from stack
            current = get_first_element(&mut to_visit_val);
            println!(" Links left to check: {:?}", to_visit_val.len());
        }
            // Check for max deeph
            if current.deeph > MAX_LINK_DEEPH {
                println!("  Max deeph reached: {:?}", &current);
                continue;
            }
        {
            // Don't visit already visited urls or too deeph
            let mut filter = filter.lock().unwrap();
            if filter.contains(&current.url)  {
                println!("  Already visited: {:?}", &current);
                continue;
            } else {
                filter.insert(&current.url.clone());
            }
        }

        // Base URL is nopt too deeph and not already visited.So dive deeper
        // and crawl next level;
        if url_status(&origin, &current.url) == true {
            let url = build_url(&origin, &current.url).unwrap();
            let base_url = url.origin().unicode_serialization();
            if base_url
                .eq_ignore_ascii_case(&origin)
            {
                // Parse this URL and fetch all links
                let new_urls = parse_and_fetch_all_urls(&url);
                println!("  Found new Urls: {:?}", new_urls.len());
                let mut to_visit_val = to_visit.lock().unwrap();
                // Add all valid URLs to list of to_visit URLs
                for new_url in new_urls {
                        if !is_allowed_by_robots(&robots_value, &new_url) {
                            println!("  Not allowed by robots.txt: {}", &new_url);
                        } else if new_url.len() as u32 > MAX_URL_LENGTH {
                            println!("  Not allowed. Url too long. {}", &new_url);
                        } else {
                            println!("  add to bucket {}", to_visit_val.len());
                            // Add new URLS to list of urls to_visit
                            to_visit_val.insert(
                                new_url.clone(),
                                UrlData {
                                    url: new_url.clone(),
                                    deeph: (current.deeph + 1),
                                },
                            );
                        }
                }
                println!("  Links left to visit: {}", to_visit_val.len());
            } else {
                println!("  Found no links");
            }
        }
    }
}

fn load_robot_value(url: &Url) -> String {
    let base_url = Url::parse(&url.origin().unicode_serialization());
    match base_url {
        Ok(url) => {
            let robot_url = url.join(&"/robots.txt").unwrap();
            println!(" Loading robots.txt from: {}", robot_url);
            fetch_url(&robot_url)
        }
        Err(_) => {
            println!(" Error loading Robots.txt from {}", url);
            String::new()
        }
    }
}
