use once_cell::sync::Lazy;
use reqwest::header::HeaderMap;
use reqwest::{header, Client};
use std::time::Duration;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:112.0) Gecko/20100101 Firefox/112.0";

pub(crate) static CLIENT: Lazy<Client> = Lazy::new(|| {
    let mut headers: HeaderMap = HeaderMap::new();
    headers.insert(header::USER_AGENT, header::HeaderValue::from_static(UA));
    Client::builder()
        .default_headers(headers)
        .gzip(true)
        .tcp_keepalive(Some(Duration::from_secs(10)))
        .http2_keep_alive_interval(Some(Duration::from_secs(10)))
        .build()
        .unwrap()
});

pub(crate) fn no_redirect_client() -> reqwest::Result<Client> {
    Client::builder()
        .user_agent(UA)
        .redirect(reqwest::redirect::Policy::none())
        .build()
}
