extern crate robots_txt;

use robots_txt::Robots;
use robots_txt::matcher::SimpleMatcher;

pub fn is_allowed_by_robots(robot_value: &str, url_path: &str) -> bool {
    let robots = Robots::from_str_lossy(robot_value);
    let matcher = SimpleMatcher::new(&robots.choose_section("*").rules);
    matcher.check_path(url_path)
}
