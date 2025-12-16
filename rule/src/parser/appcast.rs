use anyhow::Result;
use chrono::{DateTime, FixedOffset, NaiveDateTime, ParseError, Utc};
use roxmltree::{ExpandedName, Node};

#[derive(Debug)]
struct AppItem {
    version: String,
    short_version: String,
    channel: String,
    pub_date: DateTime<Utc>,
}

pub(crate) fn parse_appcast(text: &str) -> Option<String> {
    let doc = roxmltree::Document::parse(text).ok()?;
    let sparkle = doc
        .root_element()
        .namespaces()
        .find(|ns| ns.name() == Some("sparkle"))
        .map(|ns| ns.uri());

    let mut versions: Vec<AppItem> = doc
        .descendants()
        .filter(|e| e.has_tag_name("item"))
        .filter_map(|item| parse_item(item, sparkle).ok())
        .collect();

    versions.sort_by_key(|v| v.pub_date);
    versions
        .into_iter()
        .rfind(|v| v.channel != "beta")
        .map(|v| {
            if v.version.contains('.') {
                v.version
            } else {
                v.short_version
            }
        })
}

fn parse_item(item: Node, sparkle: Option<&str>) -> Result<AppItem> {
    let pub_date = find_text(&item, "pubDate")
        .unwrap_or_default()
        .replace("Web", "Wed");

    let version_title = find_text(&item, "title").unwrap_or_default();
    let mut version = String::new();
    let mut short_version = String::new();
    let mut channel = String::from("release");

    if let Some(ns) = sparkle {
        channel = find_sparkle_text(&item, ns, "channel").unwrap_or_else(|| "release".to_string());

        if let Some(v) = find_sparkle_text(&item, ns, "version") {
            version = v;
        }
        if let Some(sv) = find_sparkle_text(&item, ns, "shortVersionString") {
            short_version = sv;
        }

        if let Some(enclosure) = item.descendants().find(|e| e.has_tag_name("enclosure")) {
            for attr in enclosure
                .attributes()
                .filter(|a| a.namespace().unwrap_or_default() == ns)
            {
                match attr.name() {
                    "version" => version = attr.value().to_string(),
                    "shortVersionString" => short_version = attr.value().to_string(),
                    _ => {}
                }
            }
        }
    }

    if version.is_empty() {
        version = version_title;
    }

    Ok(AppItem {
        version,
        short_version,
        channel,
        pub_date: parse_dt(&pub_date).unwrap_or_else(|_| Utc::now()),
    })
}

fn find_text(item: &Node, tag: &str) -> Option<String> {
    item.descendants()
        .find(|e| e.has_tag_name(tag))
        .and_then(|e| e.text())
        .map(|t| t.trim().to_owned())
}

fn find_sparkle_text(item: &Node, ns: &str, tag: &str) -> Option<String> {
    let name = ExpandedName::from((ns, tag));
    item.descendants()
        .find(|e| e.has_tag_name(name))
        .and_then(|e| e.text())
        .map(|t| t.trim().to_owned())
}

fn parse_dt(pub_date: &str) -> Result<DateTime<Utc>, ParseError> {
    type ParseFunc = for<'a> fn(&'a str) -> Result<DateTime<FixedOffset>, ParseError>;

    let parsers: [ParseFunc; 4] = [
        DateTime::parse_from_rfc3339,
        DateTime::parse_from_rfc2822,
        |s| DateTime::parse_from_str(s, "%d, %a %b %Y %H:%M:%S %z"),
        |s| DateTime::parse_from_str(s, "%B %d, %Y %H:%M:%S %z"),
    ];

    for parser in &parsers {
        match parser(pub_date) {
            Ok(dt) => return Ok(dt.to_utc()),
            Err(_) => continue,
        }
    }

    NaiveDateTime::parse_from_str(pub_date, "%Y-%m-%d %H:%M:%S").map(|d| d.and_utc())
}
