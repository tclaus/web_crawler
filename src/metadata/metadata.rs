extern crate postgres;

// TODO: read connection from environment (for Server use)
// Create config file?
// Create Seed file to create databse and tables

use postgres::{Connection, TlsMode};

fn connection() -> Connection {
    //  postgres://user:secret@localhost:5432/mydatabasename

    //TODO: Read connection from environment
    Connection::connect(
        "postgres://postgres@localhost/webcrawler_dev",
        TlsMode::None,
    )
    .unwrap()
}

pub fn read_urls_to_scan() -> Vec<String> {
    // How to hanle millions of urls?
    // A: Scan only for Urls with last_scan_date less than a treshold

    let conn = connection();
    let mut url_list: Vec<String> = Vec::new();

    for row in &conn
        .query("SELECT url FROM url_list ", &[])
        .expect("Error reading URL list")
    {
        let url: String = row.get("url");
        url_list.push(url.clone());

        println!("Url: {}", &url);
    }

    url_list
}

/*
Add a unique new external link to list of urls
*/
pub fn add_new_url(origin_url_to_add: &str) {
    println!("Add new url to database: {} ", origin_url_to_add);

    let conn = connection();
    // Database has an unique index - so dont care about repetitions
    let is_added = conn.execute(
        "INSERT INTO url_list (url, created_at) VALUES ($1, localtimestamp)",
        &[&origin_url_to_add],
    );

    if is_added.is_err() {
        println!(" Not adding URL to database. Possible duplicate");
    }
}
