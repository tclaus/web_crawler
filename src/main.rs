extern crate html5ever;
extern crate url;

use std::env;
use std::io::stdout;
use std::io::Write;

mod web_crawler;
use web_crawler::{crawler};

mod metadata;
use metadata::{read_urls_to_scan};

fn main() {
    let args: Vec<_> = env::args().collect();
    println!("Starting crawler...");
    
    if args.len() > 1 {
         println!("Scan urls from commandline");
        let start_url_string = &args[1];
        println!(" Start parsing {}", start_url_string);
        crawler::crawl_start_url(&start_url_string);

    } else {
        println!("Scan urls from database");
        let urls = read_urls_to_scan();
        // TODO: Prepaire for very large Return values
        // TODO: Prepaire for contious runtime - restart aber a full loop 
        for url_string in &urls {
            // store last visited time
             crawler::crawl_start_url(&url_string);
        }
        // wait // Loop
    }

    stdout().flush().unwrap();
}
