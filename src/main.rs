extern crate html5ever;
extern crate url;

use std::env;
use std::io::stdout;
use std::io::Write;
use std::{thread, time};

mod web_crawler;
use web_crawler::crawler;

mod metadata;
use metadata::read_urls_to_scan;

fn main() {
    let args: Vec<_> = env::args().collect();
    println!("Starting crawler...");

    if args.len() > 1 {
        println!("Scan urls from commandline");
        let start_url_string = &args[1];
        println!(" Start parsing {}", &start_url_string);
        crawler::crawl_start_url(&start_url_string);
    } else {
        println!("Scan urls from database");
        loop {
            let urls = read_urls_to_scan(); // TODO: read urls scaned foir a longer time
                                            // TODO: Prepaire for very large Return values
            for url_string in &urls {
                crawler::crawl_start_url(&url_string);
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
