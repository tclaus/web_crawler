extern crate postgres;

// TODO: read connection from environment (for Server use)
// Create config file?
// Create Seed file to create databse and tables

use postgres::{Client, NoTls};
use chrono::{Utc};

fn connection() -> Client {
    return Client::connect("postgres://postgres@localhost/webcrawler_dev", NoTls).unwrap();
}
/*
    Read URLS to visit from database
*/
pub fn read_urls_to_scan() -> Vec<String> {
    // How to hanle millions of urls? A: Scan on batches
    // A: Scan only for Urls with last_scan_date less than a treshold
    // TODO: make Doc

    let mut conn = connection();
    let mut url_list: Vec<String> = Vec::new();

    for row in conn
        .query("SELECT url FROM url_list order by visited_at desc", &[])
        .expect("Error reading URL list")
    {
        let url: String = row.get("url");
        url_list.push(url.clone());

        println!("Url: {}", &url);
    }

    url_list
}

/*
 Update visited date and count for URL
*/
pub fn update_url(url_to_update : &str) {
    println!("Update URL in Database: {:?}", url_to_update );
    let mut conn = connection();
    let is_updated = conn.execute(
        "UPDATE url_list set visited_at=$1, visited_count = visited_count + 1 where url=$2",
        &[
        &Utc::now().naive_utc(),
        &url_to_update],
    );
    if is_updated.is_err() {
        println!(" Update failed {:?}", is_updated );
    }
}

/*
Add a unique new external link to list of urls
*/
pub fn add_new_url(origin_url_to_add: &str) {
    println!("Add new url to database: {} ", origin_url_to_add);

    let mut conn = connection();
    // Database has an unique index - so dont care about repetitions
    let is_added = conn.execute(
        "INSERT INTO url_list (url, created_at) VALUES ($1, localtimestamp)",
        &[&origin_url_to_add],
    );

    if is_added.is_err() {
        println!(" Not adding URL to database. Possible duplicate");
    }
}
