extern crate html5ever;
extern crate url;

use std::env;
use std::io::stdout;
use std::io::Write;
use url::Url;

use fetch::UrlState;

mod crawler;
mod fetch;
mod parse;

fn main() {
    let args: Vec<_> = env::args().collect();
    println!("Starting crawler...");
    if args.len() > 1 {
        let start_url_string = &args[1];
        let start_url = Url::parse(start_url_string).unwrap();
        println!(" Start parsing {}", start_url);
        let origin = start_url
                .origin();
        println!("Origin URL {}", origin.ascii_serialization());

        let mut success_count = 0;
        let mut fail_count = 0;
        for url_state in crawler::crawl(&origin.ascii_serialization(), &start_url) {
            match url_state {
                UrlState::Accessible(_) => {
                    success_count += 1;
                }
                status => {
                    fail_count += 1;
                    println!("{}", status);
                }
            }
            println!("Succeeded: {}, Failed: {}\r", success_count, fail_count);
            stdout().flush().unwrap();
        }
    }
}
