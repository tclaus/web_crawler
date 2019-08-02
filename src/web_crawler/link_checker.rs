extern crate url;

use url::Url;

/*
Compares Domain and url. Returns true if same
*/
pub fn url_has_same_origin_path(domain: &str, target: &str) -> bool {
    // current: Ful URL
    // link: is origon the same?
    let target_url: Url = Url::parse(&target).unwrap();
    let target_origin = target_url.host_str().unwrap().replace("www.", "");

    let domain: Url = Url::parse(&domain).unwrap();
    let domain_origin = domain.host_str().unwrap().replace("www.", "");

    println!(
        " Comparing Current path base {} with link-base: {}",
        domain_origin, target_origin
    );

    domain_origin.eq_ignore_ascii_case(&target_origin)
}
