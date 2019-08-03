extern crate bloom;

use super::robots::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::{thread, time};
use url::Url;

use bloom::BloomFilter;
use bloom::ASMS;
use hyper::status::StatusCode;

const THREADS: i32 = 20;
const MAX_URL_LENGTH: u32 = 100;
const MAX_LINK_DEEPH: u8 = 3;

use super::fetch::{fetch_all_urls, fetch_url, url_status, UrlState};

#[derive(Debug)]
pub struct UrlData {
    url: String,
    deeph: u8,
}

pub struct Crawler {
    to_visit: Arc<Mutex<HashMap<String, UrlData>>>,
    active_count: Arc<Mutex<i32>>,
    url_states: Receiver<UrlState>,
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

    let mut success_count = 0;
    let mut fail_count = 0;
    let robots_value = load_robot_value(&start_url);

    for url_state in crawl(&origin.unicode_serialization(), &start_url, &robots_value) {
        match url_state {
            //TODO: Here store successful reads
            UrlState::Accessible(_) => {
                success_count += 1;
            }
            status => {
                fail_count += 1;
                println!("{}", status);
            }
        }
        println!("Succeeded: {}, Failed: {}\r", success_count, fail_count);
    }
}

fn crawl(origin: &str, start_url: &Url, robots_value: &str) -> Crawler {
    let start_url_data = UrlData {
        url: start_url.to_string(),
        deeph: 0,
    };
    println!(" {:?} ", start_url_data);
    println!(" New Crawl of {}", start_url);
    let mut url_hash = HashMap::new();
    url_hash.insert(start_url.to_string(), start_url_data);

    let to_visit = Arc::new(Mutex::new(url_hash));
    let active_count = Arc::new(Mutex::new(0));
    let visited = Arc::new(Mutex::new(HashSet::new()));
    let filter = Arc::new(Mutex::new(init_bloom()));

    let (tx, rx) = channel();

    let crawler = Crawler {
        to_visit: to_visit.clone(),
        active_count: active_count.clone(),
        url_states: rx,
    };

    println!(" Crawling: origin:{}, Path:{}", origin, start_url);
    println!(" Generating Threads");
    for _ in 0..THREADS {
        let origin = origin.to_owned();
        let to_visit = to_visit.clone();
        let visited = visited.clone();
        let active_count = active_count.clone();
        let tx = tx.clone();
        let robots_value = robots_value.to_owned();
        let filter = filter.clone();

        thread::spawn(move || {
            crawl_worker_thread(
                &origin,
                to_visit,
                visited,
                active_count,
                tx,
                &robots_value,
                filter,
            );
        });
    }

    crawler
}

fn get_first_element(hash_map: &mut HashMap<String, UrlData>) -> UrlData {
    // TODO: Use the match to return a value

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
    visited: Arc<Mutex<HashSet<String>>>,
    active_count: Arc<Mutex<i32>>,
    url_states: Sender<UrlState>,
    robots_value: &str,
    filter: Arc<Mutex<BloomFilter>>,
) {
    let mut too_many_requests_list: HashSet<String> = HashSet::new();

    loop {
        let current: UrlData;
        {
            let mut to_visit_val = to_visit.lock().unwrap();
            let mut active_count_val = active_count.lock().unwrap();
            if to_visit_val.is_empty() {
                if *active_count_val > 0 {
                    continue;
                } else {
                    break;
                }
            };
            // Get a random url
            current = get_first_element(&mut to_visit_val);
            println!(" Links left to check: {:?}", to_visit_val.len());
            *active_count_val += 1;
            assert!(*active_count_val <= THREADS);
        }

        {
            // Dont visit already visited urls
            let mut visited_val = visited.lock().unwrap();
            if visited_val.contains(&current.url) {
                println!("  Already visited {:?}", &current);
                let mut active_count_val = active_count.lock().unwrap();
                *active_count_val -= 1;
                continue;
            } else {
                visited_val.insert(current.url.to_owned());
            }
        }

        let state = url_status(&origin, &current.url);
        if let UrlState::Accessible(ref url) = state.clone() {
            println!("  Accessible. Check for dive deeper");

            if too_many_requests_list.contains(&url.domain().unwrap().to_string()) {
                println!("  URL {} had too many requsts. Continue to next.", &url);
            } else if url
                .origin()
                .unicode_serialization()
                .eq_ignore_ascii_case(&origin)
            {
                let new_urls = fetch_all_urls(&url);
                println!("  Found new Urls: {:?}", new_urls.len());
                let mut to_visit_val = to_visit.lock().unwrap();
                let mut filter = filter.lock().unwrap();

                for new_url in new_urls {
                    if !filter.contains(&new_url) {
                        if !is_allowed_by_robots(&robots_value, &new_url) {
                            println!("  Not allowed by robots.txt: {}", &new_url);
                        } else if current.deeph > MAX_LINK_DEEPH {
                            println!("  Not allowed. Too deeph: {}", &new_url);
                        } else if new_url.len() as u32 > MAX_URL_LENGTH {
                            println!("  Not allowed. Url too long. {}", &new_url);
                        } else {
                            // Add new URLS to list of urls to visit
                            println!("  Add new Url to hashMap: {}", &new_url);
                            to_visit_val.insert(
                                new_url.clone(),
                                UrlData {
                                    url: new_url.clone(),
                                    deeph: (current.deeph + 1),
                                },
                            );
                        }

                        filter.insert(&new_url);
                    }
                }
                println!("  Links left to visit: {}", to_visit_val.len());
            } else {
                println!("  Found no links");
            }
        } else {
            // If state == (429 Too Many Requests) then ignore full domain for now
            if let UrlState::BadStatus(ref url, StatusCode::TooManyRequests) = state.clone() {
                println!("  ! Too Many requests. Add to ignore list");
                too_many_requests_list.insert(url.domain().unwrap().to_string());
            }
        }

        {
            let mut active_count_val = active_count.lock().unwrap();
            *active_count_val -= 1;
            assert!(*active_count_val >= 0);
        }

        url_states.send(state).unwrap();
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

impl Iterator for Crawler {
    type Item = UrlState;

    fn next(&mut self) -> Option<UrlState> {
        loop {
            thread::sleep(time::Duration::from_secs(1));
            match self.url_states.recv() {
                Ok(state) => return Some(state),
                Err(_) => {
                    let to_visit_val = self.to_visit.lock().unwrap();
                    let active_count_val = self.active_count.lock().unwrap();

                    if to_visit_val.is_empty() && *active_count_val == 0 {
                        return None;
                    } else {
                        continue;
                    }
                }
            }
        }
    }
}
