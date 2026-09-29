use reqwest::Client;
use std::sync::LazyLock;
use std::time::Duration;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:112.0) Gecko/20100101 Firefox/132.0";

pub(crate) static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent(UA)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .gzip(true)
        .tcp_keepalive(Some(Duration::from_secs(10)))
        .http2_keep_alive_interval(Some(Duration::from_secs(10)))
        .build()
        .expect("Failed to build HTTP client")
});

pub(crate) static NO_REDIRECT_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent(UA)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Failed to build no-redirect HTTP client")
});

#[cfg(test)]
mod tests {
    use super::{CLIENT, NO_REDIRECT_CLIENT};
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn assert_stalled_response_times_out(client: &reqwest::Client, send_headers: bool) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buffer = [0; 4096];
            let received = socket.read(&mut buffer).await.unwrap();
            assert!(received > 0, "client sent no request");
            if send_headers {
                socket
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n")
                    .await
                    .unwrap();
            }
            std::future::pending::<()>().await;
        });
        let result = tokio::time::timeout(Duration::from_secs(32), async {
            client.get(url).send().await?.text().await
        })
        .await;
        server.abort();
        assert!(server.await.unwrap_err().is_cancelled());
        let error = result
            .expect("client did not enforce its deadline")
            .unwrap_err();
        assert!(error.is_timeout(), "unexpected error: {error}");
    }

    #[tokio::test]
    async fn normal_client_times_out_while_reading_body() {
        assert_stalled_response_times_out(&CLIENT, true).await;
    }

    #[tokio::test]
    async fn no_redirect_client_times_out_waiting_for_headers() {
        assert_stalled_response_times_out(&NO_REDIRECT_CLIENT, false).await;
    }
}
