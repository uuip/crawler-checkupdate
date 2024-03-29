use once_cell::sync::Lazy;
use regex::Regex;

pub use fetch::parse_app;

mod fetch;
mod rule_index;
mod rules;

type FnSignature = fn(&str) -> Option<String>;

static VER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[.\d]*\d+").unwrap());

pub fn num_version(ver_info: String) -> Option<String> {
    VER_RE
        .find(ver_info.as_str())
        .map(|x| x.as_str().to_string())
}
