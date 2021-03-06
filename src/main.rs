extern crate base64;
extern crate html5ever;
extern crate url;

extern crate elastic;
#[macro_use]
extern crate elastic_derive;

use std::env;
use std::io::stdout;
use std::io::Write;
use std::{thread, time};

mod web_crawler;
use web_crawler::crawler;

mod metadata;
use metadata::read_urls_to_scan;
use metadata::update_url;

mod es;

fn main() {
    env_logger::init();
    let args: Vec<_> = env::args().collect();
    println!("Starting crawler...");

    if args.len() > 1 {
        println!("Scan urls from commandline");
        let url_string = &args[1];
        println!(" Start parsing {}", &url_string);
        crawler::crawl_start_url(&url_string);
        update_url(&url_string);
    } else {
        println!("Scan urls from database");
        loop {
            let urls = read_urls_to_scan(); // TODO: read urls scaned for a longer time
                                            // TODO: Prepaire for very large Return values
            for url_string in &urls {
                crawler::crawl_start_url(&url_string);
                update_url(&url_string);
                // store last visited time for this link
            }
            // wait // Loop
            let duration = time::Duration::from_secs(60 * 60 * 2); // 2h warten
            println!("Waiting for next loop...");
            thread::sleep(duration);
            println!("Starting next loop");
        }
    }

    stdout().flush().unwrap();
}
