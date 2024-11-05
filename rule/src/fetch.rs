use anyhow::{anyhow, Error};
use models::ver;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Response;
use std::env;

use crate::client::{no_redirect_client, CLIENT};
use crate::parser::html::parse_css;
use crate::parser::rule_index::{CSSRULES, FNRULES};

static TOKEN: Lazy<String> = Lazy::new(|| env::var("GITHUB_TOKEN").unwrap_or_default());
static VER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[.\d]*\d+").unwrap());

pub async fn parse_app(app: &ver::Model) -> Result<String, Error> {
    match app {
        _app if app.name == "Fences" => {
            let resp: Response = CLIENT.head(&app.url).send().await?;
            let head: &str = resp.headers()["Content-Length"].to_str()?;
            Ok(head.to_owned())
        }
        _app if app.name == "EmEditor" => {
            let client = no_redirect_client()?;
            let resp: Response = client.get(&app.url).send().await?;
            let arg: &str = resp.headers()["location"].to_str()?;
            find_version(app, arg).ok_or(anyhow!("解析版本错误"))
        }
        _ if app.url.starts_with("https://api.github.com") => {
            let resp: Response = CLIENT
                .get(&app.url)
                .header("Authorization", format!("token {}", *TOKEN))
                .send()
                .await?;
            let j: serde_json::Value = resp.json::<serde_json::Value>().await?;
            num_version(j["tag_name"].to_string()).ok_or(anyhow!("解析版本错误"))
        }
        _ if app.url.starts_with("https://formulae.brew.sh/") => {
            let resp: Response = CLIENT.get(&app.url).send().await?;
            let j: serde_json::Value = resp.json::<serde_json::Value>().await?;
            num_version(j["version"].to_string()).ok_or(anyhow!("解析版本错误"))
        }
        _ if app.url.starts_with("https://data.services.jetbrains.com/") => {
            let resp: Response = CLIENT.get(&app.url).send().await?;
            let j: serde_json::Value = resp.json::<serde_json::Value>().await?;
            let v: String = match app.name.as_str() {
                "PyCharm" => j["PCP"][0]["version"].to_string(),
                "CLion" => j["CL"][0]["version"].to_string(),
                "GoLand" => j["GO"][0]["version"].to_string(),
                "RustRover" => j["RR"][0]["version"].to_string(),
                _ => panic!("not support product {}", app.name),
            };
            num_version(v).ok_or(anyhow!("解析版本错误"))
        }
        _ => {
            let resp: Response = CLIENT.get(&app.url).send().await?;
            let arg: String = resp.text().await?;
            find_version(app, &arg).ok_or(anyhow!("解析版本错误"))
        }
    }
}

fn find_version(app: &ver::Model, resp: &str) -> Option<String> {
    let app_name = app.name.as_str();
    FNRULES
        .get(app_name)
        .and_then(|f| f(resp))
        .or_else(|| CSSRULES.get(app_name).and_then(|css| parse_css(resp, css)))
        .and_then(num_version)
}

pub fn num_version(ver_info: String) -> Option<String> {
    VER_RE
        .find(ver_info.as_str())
        .map(|x| x.as_str().to_string())
}
