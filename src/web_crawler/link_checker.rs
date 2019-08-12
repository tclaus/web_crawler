extern crate url;

use url::Url;

/*
Compares Domain and url. Returns true if same
*/
pub fn url_has_same_origin_path(domain: &str, target: &str) -> bool {
    // current: Ful URL
    // link: is origon the same?
    let target_url_result = Url::parse(&target);
    let target_url = match target_url_result {
        Ok(target) => target,
        Err(_) => return false,
    };

    let target_origin = target_url.host_str().unwrap().replace("www.", "");

    let domain_result = Url::parse(&domain);
    let domain_url = match domain_result {
        Ok(domain_) => domain_,
        Err(_) => return false,
    };

    let domain_origin = domain_url.host_str().unwrap().replace("www.", "");
    domain_origin.eq_ignore_ascii_case(&target_origin)
}
