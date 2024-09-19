pub use fetch::parse_app;

mod client;
mod fetch;
mod parser;

type FnSignature = fn(&str) -> Option<String>;
