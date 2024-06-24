use std::env;
use std::time::Duration;

use anyhow::{anyhow, Error};
use once_cell::sync::Lazy;
use reqwest::header::HeaderMap;
use reqwest::{header, Client, Response};

use models::ver;

use crate::rule_index::{CSSRULES, FNRULES};
use crate::rules::parse_css;

static TOKEN: Lazy<String> = Lazy::new(|| env::var("GITHUB_TOKEN").unwrap_or_default());
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:112.0) Gecko/20100101 Firefox/112.0";
static CLIENT: Lazy<Client> = Lazy::new(|| {
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

pub async fn parse_app(app: &ver::Model) -> Result<String, Error> {
    if app.name == *"Fences" {
        let resp: Response = CLIENT.head(&app.url).send().await?;
        let head: &str = resp.headers()["Content-Length"].to_str()?;
        Ok(head.to_owned())
    } else if app.name == *"EmEditor" {
        let resp: Response = Client::builder()
            .user_agent(UA)
            .redirect(reqwest::redirect::Policy::none())
            .build()?
            .get(&app.url)
            .send()
            .await?;
        let arg: &str = resp.headers()["location"].to_str().unwrap();
        find_version(app, arg).ok_or(anyhow!("解析版本错误"))
    } else if app.json == 1 {
        let resp: Response = {
            if app.url.starts_with("https://api.github.com") {
                CLIENT
                    .get(&app.url)
                    .header("Authorization", format!("token {}", *TOKEN))
                    .send()
                    .await?
            } else {
                CLIENT.get(&app.url).send().await?
            }
        };
        let j: serde_json::Value = resp.json::<serde_json::Value>().await?;
        let v: String = match app.name.as_str() {
            "PyCharm" => j["PCP"][0]["version"].to_string(),
            "CLion" => j["CL"][0]["version"].to_string(),
            "GoLand" => j["GO"][0]["version"].to_string(),
            "RustRover" => j["RR"][0]["version"].to_string(),
            "Clash" => j["name"].to_string(),
            _ => j["tag_name"].to_string(),
        };
        Ok(v)
    } else {
        let resp: Response = CLIENT.get(&app.url).send().await?;
        let arg: String = resp.text().await?;
        find_version(app, &arg).ok_or(anyhow!("解析版本错误"))
    }
}

fn find_version(app: &ver::Model, resp: &str) -> Option<String> {
    let app_name = app.name.as_str();
    let func = FNRULES.get(app_name);
    if let Some(f) = func {
        f(resp)
    } else {
        CSSRULES.get(app_name).map(|css| parse_css(resp, css))?
    }
}
