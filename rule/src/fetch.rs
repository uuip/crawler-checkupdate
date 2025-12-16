use crate::client::{CLIENT, NO_REDIRECT_CLIENT};
use crate::parser::appcast::parse_appcast;
use crate::parser::fn_index::FNRULES;
use crate::parser::html::parse_css;
use anyhow::{Context, Error};
use models::ver;
use regex::Regex;
use serde_json_path::JsonPath;
use std::env;
use std::sync::LazyLock;

static TOKEN: LazyLock<String> = LazyLock::new(|| env::var("GITHUB_TOKEN").unwrap_or_default());
static VER_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[.\d]*\d+").unwrap());

fn preview(s: &str, max_len: usize) -> &str {
    let end = s
        .char_indices()
        .nth(max_len)
        .map_or(s.len(), |(idx, _)| idx);
    &s[..end]
}

pub async fn parse_app(app: &ver::Model) -> Result<String, Error> {
    let request = match (app.check_type.as_str(), app.name.as_str()) {
        ("json", _) if app.url.starts_with("https://api.github.com") => CLIENT
            .get(&app.url)
            .header("Authorization", format!("token {}", *TOKEN)),
        ("headers", "Fences") => CLIENT.head(&app.url),
        ("headers", _) => NO_REDIRECT_CLIENT.get(&app.url),
        _ => CLIENT.get(&app.url),
    };
    let resp = request.send().await.context("请求失败")?;

    match app.check_type.as_str() {
        "headers" => parse_headers(&resp, &app.name),
        "json" => parse_json_response(resp, app).await,
        "css" => parse_css_response(resp, app).await,
        "xml" => parse_xml_response(resp).await,
        _ => parse_custom_function(resp, &app.name).await,
    }
}

fn parse_headers(resp: &reqwest::Response, app_name: &str) -> Result<String, Error> {
    match app_name {
        "Fences" => {
            let length = resp
                .headers()
                .get("Content-Length")
                .context("缺少 Content-Length 响应头")?
                .to_str()?;
            Ok(length.to_string())
        }
        _ => {
            let location = resp
                .headers()
                .get("location")
                .context("缺少 location 响应头")?
                .to_str()?;
            location
                .split('_')
                .next_back()
                .and_then(num_version)
                .with_context(|| format!("解析 location 失败: {}", preview(location, 10)))
        }
    }
}

async fn parse_json_response(resp: reqwest::Response, app: &ver::Model) -> Result<String, Error> {
    let value: serde_json::Value = resp.json().await?;
    let jsonpath = app
        .version_rule
        .as_deref()
        .with_context(|| format!("缺少 jsonpath 规则: {}", app.name))?;
    let path = JsonPath::parse(jsonpath)?;
    let version_str = path
        .query(&value)
        .exactly_one()?
        .as_str()
        .context("json 节点不是字符串")?;

    num_version(version_str)
        .with_context(|| format!("json 中未找到版本号: {}", preview(version_str, 10)))
}

async fn parse_css_response(resp: reqwest::Response, app: &ver::Model) -> Result<String, Error> {
    let text = resp.text().await?;
    let css = app
        .version_rule
        .as_deref()
        .with_context(|| format!("缺少 css 规则: {}", app.name))?;

    parse_css(&text, css)
        .and_then(num_version)
        .with_context(|| format!("css 解析失败: {}", preview(&text, 10)))
}

async fn parse_xml_response(resp: reqwest::Response) -> Result<String, Error> {
    let text = resp.text().await?;
    parse_appcast(&text)
        .and_then(num_version)
        .with_context(|| format!("xml 解析失败: {}", preview(&text, 10)))
}

async fn parse_custom_function(resp: reqwest::Response, app_name: &str) -> Result<String, Error> {
    let text = resp.text().await?;
    FNRULES
        .get(app_name)
        .and_then(|f| f(&text))
        .and_then(num_version)
        .with_context(|| format!("自定义函数解析失败: {}", preview(&text, 10)))
}

pub fn num_version<T: AsRef<str>>(ver_info: T) -> Option<String> {
    VER_RE
        .find(ver_info.as_ref())
        .map(|m| m.as_str().to_string())
}
