use reqwest::header::HeaderMap;
use reqwest::{Client, header};
use std::sync::LazyLock;
use std::time::Duration;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:112.0) Gecko/20100101 Firefox/132.0";

pub(crate) static CLIENT: LazyLock<Client> = LazyLock::new(|| {
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

pub(crate) static NO_REDIRECT_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent(UA)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
});
