use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, ParseError, Utc};
use roxmltree::{ExpandedName, Node};

#[derive(Debug)]
struct AppItem {
    version: String,
    short_version: String,
    channel: String,
    pub_date: DateTime<Utc>,
}

pub(crate) fn parse_appcast(text: &str) -> Option<String> {
    let doc = roxmltree::Document::parse(text).unwrap();
    let sparkle = doc
        .root_element()
        .namespaces()
        .find(|ns| ns.name() == Some("sparkle"))
        .map(|t| t.uri());

    let mut versions: Vec<AppItem> = doc
        .descendants()
        .filter(|e| e.has_tag_name("item"))
        .filter_map(|item| parse_item(item, sparkle).ok())
        .collect();

    versions.sort_by(|a, b| a.pub_date.cmp(&b.pub_date));
    versions
        .into_iter()
        .filter(|x| x.channel != "beta")
        .last()
        .map(|x| {
            if x.version.contains(".") {
                x.version
            } else {
                x.short_version
            }
        })
}

fn parse_item(item: Node, sparkle: Option<&str>) -> Result<AppItem> {
    let pub_date = find_text(&item, "pubDate").unwrap_or_default();
    let version1 = find_text(&item, "title").unwrap_or_default();
    let mut version2 = String::new();
    let mut version3 = String::new();
    let mut channel = String::from("release");
    let mut short_version = String::new();

    if let Some(ns) = sparkle {
        channel = find_sparkle_text(&item, "channel", ns).unwrap_or_else(|| "release".to_string());
        version2 = find_sparkle_text(&item, ns, "version").unwrap_or_default();
        short_version = find_sparkle_text(&item, ns, "shortVersionString").unwrap_or_default();

        if let Some(t) = item.descendants().find(|e| e.has_tag_name("enclosure")) {
            for attr in t
                .attributes()
                .filter(|a| a.namespace().unwrap_or_default() == ns)
            {
                match attr.name() {
                    "version" => version3 = attr.value().to_string(),
                    "shortVersionString" => short_version = attr.value().to_string(),
                    _ => (),
                }
            }
        }
    }
    let version = if !version3.is_empty() {
        version3
    } else if !version2.is_empty() {
        version2
    } else {
        version1
    };

    Ok(AppItem {
        version,
        short_version,
        channel,
        pub_date: parse_dt(&pub_date)?,
    })
}

fn find_text(item: &Node, tag: &str) -> Option<String> {
    item.descendants()
        .find(|e| e.has_tag_name(tag))
        .and_then(|e| e.text())
        .map(|t| t.trim().to_owned())
}

fn find_sparkle_text(item: &Node, tag: &str, ns: &str) -> Option<String> {
    let name = ExpandedName::from((ns, tag));
    item.descendants()
        .find(|e| e.has_tag_name(name))
        .and_then(|e| e.text())
        .map(|t| t.trim().to_owned())
}

fn parse_dt(pub_date: &str) -> Result<DateTime<Utc>, ParseError> {
    DateTime::parse_from_rfc3339(pub_date)
        .or_else(|_| DateTime::parse_from_rfc2822(pub_date))
        .map(|d| d.to_utc())
        .or_else(|_| {
            NaiveDateTime::parse_from_str(pub_date, "%Y-%m-%d %H:%M:%S").map(|d| d.and_utc())
        })
}