extern crate postgres;


// TODO: read connection from environment (for Server use) 
// Create config file? 
// Create Seed file to cfreate databse and tables

use postgres::{Connection, TlsMode};

fn connection() -> Connection {
    //  postgres://user:secret@localhost:5432/mydatabasename 
    
    //TODO: Read connection from environment
    Connection::connect("postgres://postgres@localhost/webcrawler_dev",
                               TlsMode::None).unwrap()
}

pub fn read_urls_to_scan() -> Vec<String> {
    // How to hanle millions of urls? 
    // A: Scan only for Urls with last_scan_date less than a treshold

    let conn = connection();
    let mut url_list : Vec<String> = Vec::new();

    for row in &conn.query("SELECT url FROM url_list ",&[]).expect("Error readiung URL list") {
        let url : String = row.get("url");
        url_list.push(url.clone());

        println!("Url: {}", &url);
    }

    url_list

}