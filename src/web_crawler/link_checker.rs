
extern crate url;

use url::{Url};

/*
Compares Domain and url. Returns true if same
*/
pub fn url_has_same_origin_path(domain: &str, path: &str) -> bool {
    // current: Ful URL
    // link: is origon the same? 
    let url_result : Url = Url::parse(&path).unwrap();
    let origin_path = url_result.origin();

    let origin_path_string = origin_path.ascii_serialization();
    println!(" Comparing Current path base  {} with link-base: {}", domain, origin_path_string);

    domain.eq_ignore_ascii_case(&origin_path_string)
}