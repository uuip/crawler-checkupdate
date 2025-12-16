use reqwest::Client;
use std::sync::LazyLock;
use std::time::Duration;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:112.0) Gecko/20100101 Firefox/132.0";

pub(crate) static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent(UA)
        .gzip(true)
        .tcp_keepalive(Some(Duration::from_secs(10)))
        .http2_keep_alive_interval(Some(Duration::from_secs(10)))
        .build()
        .expect("Failed to build HTTP client")
});

pub(crate) static NO_REDIRECT_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent(UA)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Failed to build no-redirect HTTP client")
});
