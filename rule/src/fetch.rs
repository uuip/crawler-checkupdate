use crate::client::{CLIENT, no_redirect_client};
use crate::parser::appcast::parse_appcast;
use crate::parser::fn_index::FNRULES;
use crate::parser::html::parse_css;
use anyhow::{Error, anyhow};
use models::ver;
use regex::Regex;
use serde_json_path::JsonPath;
use std::env;
use std::sync::LazyLock;

static TOKEN: LazyLock<String> = LazyLock::new(|| env::var("GITHUB_TOKEN").unwrap_or_default());
static VER_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[.\d]*\d+").unwrap());

pub async fn parse_app(app: &ver::Model) -> Result<String, Error> {
    let request = if app.check_type == "json" && app.url.starts_with("https://api.github.com") {
        CLIENT
            .get(&app.url)
            .header("Authorization", format!("token {}", *TOKEN))
    } else if app.check_type == "headers" && app.name == "Fences" {
        CLIENT.head(&app.url)
    } else if app.check_type == "headers" {
        no_redirect_client()?.get(&app.url)
    } else {
        CLIENT.get(&app.url)
    };
    let resp = request.send().await?;
    match app.check_type.as_ref() {
        "headers" => match app.name.as_ref() {
            "Fences" => {
                let length: &str = resp.headers()["Content-Length"].to_str()?;
                Ok(length.to_string())
            }
            _ => {
                let location: &str = resp.headers()["location"].to_str()?;
                location
                    .split('_')
                    .next_back()
                    .and_then(num_version)
                    .ok_or_else(|| {
                        anyhow!("通过location解析版本错误: {}", first_10_chars(location))
                    })
            }
        },
        "json" => {
            let value: serde_json::Value = resp.json::<serde_json::Value>().await?;
            let jsonpath = app
                .version_rule
                .as_deref()
                .unwrap_or_else(|| panic!("丢失 jsonpath: {}", &app.name));
            let path = JsonPath::parse(jsonpath)?;
            let node = path.query(&value).exactly_one()?.as_str();
            if let Some(version) = node {
                num_version(version)
                    .ok_or(anyhow!("未从json中找到数字: {}", first_10_chars(version)))
            } else {
                Err(anyhow!("json应答为空"))
            }
        }
        "css" => {
            let resp: String = resp.text().await?;
            let css = app
                .version_rule
                .as_deref()
                .unwrap_or_else(|| panic!("丢失css: {}", &app.name));
            parse_css(&resp, css)
                .and_then(num_version)
                .ok_or(anyhow!("通过css解析版本错误: {}", first_10_chars(&resp)))
        }
        "xml" => {
            let resp: String = resp.text().await?;
            parse_appcast(&resp)
                .and_then(num_version)
                .ok_or(anyhow!("通过xml解析版本错误: {}", first_10_chars(&resp)))
        }
        _ => {
            let resp: String = resp.text().await?;
            FNRULES
                .get(app.name.as_str())
                .and_then(|f| f(&resp))
                .and_then(num_version)
                .ok_or(anyhow!("解析 函数 版本错误: {}", first_10_chars(&resp)))
        }
    }
}

pub fn num_version<T: AsRef<str>>(ver_info: T) -> Option<String> {
    VER_RE
        .find(ver_info.as_ref())
        .map(|x| x.as_str().to_string())
}

fn first_10_chars(s: &str) -> &str {
    let end = s.char_indices().nth(10).map_or(s.len(), |(idx, _)| idx);
    &s[..end]
}
