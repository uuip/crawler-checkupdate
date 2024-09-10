use chrono::{DateTime, NaiveDateTime, ParseError, Utc};
use regex::Regex;
use roxmltree::ExpandedName;
use scraper::{ElementRef, Html, Selector};
use semver::Version;

/*
use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};
pub(crate) fn parse_navicat(resp: &str) -> Option<String> {
    let html = Document::from(resp);
    let element = html
        .find(
            Class("release-notes-table")
                .and(Attr("platform", "W"))
                .descendant(Name("td"))
                .descendant(Class("note-title")),
        )
        .next()?
        .text();
    Some(element)
}
*/

pub(crate) fn parse_css(resp: &str, css: &str) -> Option<String> {
    let html = Html::parse_document(resp);
    let selector = Selector::parse(css).unwrap();
    let element = html.select(&selector).next()?.text().next()?.trim();
    Some(element.to_owned())
}

pub(crate) fn parse_faststone(resp: &str) -> Option<String> {
    let html = Html::parse_document(resp);
    let selector = Selector::parse("b").unwrap();
    let re = Regex::new(r"Version\s*[.\d]+").unwrap();

    let element = html
        .select(&selector)
        .find_map(|x| re.find(x.text().next().unwrap_or_default()))?;
    Some(element.as_str().to_owned())
}

pub(crate) fn parse_winrar(resp: &str) -> Option<String> {
    let html = Html::parse_document(resp);
    let selector = Selector::parse("b").unwrap();
    let re = Regex::new("^WinRAR.*elease").unwrap();

    let element = html
        .select(&selector)
        .find_map(|x| re.find(x.text().next().unwrap_or_default()))?;
    Some(element.as_str().to_owned())
}

pub(crate) fn parse_vmware(resp: &str) -> Option<String> {
    let html = Html::parse_fragment(resp);
    let selector = Selector::parse("metadata>version").unwrap();
    let mut element: Vec<Version> = html
        .select(&selector)
        .filter_map(|x| Version::parse(x.text().next().unwrap_or("0.0.0")).ok())
        .collect();
    element.sort();
    let ver = element.last()?;
    Some(ver.to_string())
}

pub(crate) fn parse_dev_man_view(resp: &str) -> Option<String> {
    let html = Html::parse_document(resp);
    let selector = Selector::parse("h4").unwrap();
    let element = html
        .select(&selector)
        .find(|x| x.text().next().unwrap_or_default() == "Versions History")?;
    let element = element.next_siblings().nth(1)?.children().nth(1)?;
    let element = ElementRef::wrap(element)?.text().next()?;
    Some(element.to_owned())
}

pub(crate) fn parse_emeditor(resp: &str) -> Option<String> {
    Some(resp.split('_').last()?.to_owned())
}

pub(crate) fn parse_pdf_xchange(resp: &str) -> Option<String> {
    let html = Html::parse_document(resp);
    let selector = Selector::parse("div.version").unwrap();
    let element = html.select(&selector).next()?.text().nth(2)?.trim();
    Some(element.to_owned())
}

pub(crate) fn parse_vscode(resp: &str) -> Option<String> {
    let html = Html::parse_document(resp);
    let selector = Selector::parse("h1").unwrap();
    let element = html.select(&selector).next()?.text().next()?.trim();
    let re = Regex::new(r" (\d+\.\d+(\.\d+)?)").unwrap();
    let rst = re.captures(element)?.get(1)?.as_str();
    Some(rst.to_owned())
}

#[derive(Debug)]
struct AppItem {
    version: String,
    short_version: String,
    channel: String,
    pub_date: DateTime<Utc>,
}

pub(crate) fn parse_appcast(text: &str) -> Option<String> {
    let mut versions: Vec<AppItem> = vec![];

    let doc = roxmltree::Document::parse(text).expect("Failed to parse XML text");
    let sparkle = doc
        .root_element()
        .namespaces()
        .find(|ns| ns.name() == Some("sparkle"));

    let items = doc.descendants().filter(|e| e.has_tag_name("item"));
    for item in items {
        let mut pub_date = String::new();
        let mut version1 = String::new();
        let mut version2 = String::new();
        let mut version3 = String::new();
        let mut channel = String::from("release");
        let mut short_version = String::new();

        if let Some(Some(t)) = item
            .descendants()
            .find(|e| e.has_tag_name("pubDate"))
            .map(|e| e.text())
        {
            pub_date = t.trim().to_owned();
        }

        if let Some(Some(t)) = item
            .descendants()
            .find(|e| e.has_tag_name("title"))
            .map(|e| e.text())
        {
            version1 = t.trim().to_owned();
        }

        if let Some(ns) = sparkle {
            let name = ExpandedName::from((ns.uri(), "channel"));
            if let Some(Some(t)) = item
                .descendants()
                .find(|e| e.has_tag_name(name))
                .map(|e| e.text())
            {
                channel = t.trim().to_owned();
            }

            let name = ExpandedName::from((ns.uri(), "version"));
            if let Some(Some(t)) = item
                .descendants()
                .find(|e| e.has_tag_name(name))
                .map(|e| e.text())
            {
                version2 = t.trim().to_owned();
            }

            let name = ExpandedName::from((ns.uri(), "shortVersionString"));
            if let Some(Some(t)) = item
                .descendants()
                .find(|e| e.has_tag_name(name))
                .map(|e| e.text())
            {
                short_version = t.trim().to_owned();
            }

            if let Some(t) = item.descendants().find(|e| e.has_tag_name("enclosure")) {
                for a in t
                    .attributes()
                    .filter(|a| a.namespace().unwrap_or_default() == ns.uri())
                {
                    if a.name() == "version" {
                        version3 = a.value().to_owned();
                    } else if a.name() == "shortVersionString" {
                        short_version = a.value().to_owned();
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

        versions.push(AppItem {
            version,
            short_version,
            channel,
            pub_date: parse_dt(&pub_date).expect("parse date error at parse_appcast"),
        });
    }
    let mut rc = versions
        .into_iter()
        .filter(|x| x.channel != "beta")
        .collect::<Vec<_>>();
    rc.sort_by(|a, b| a.pub_date.cmp(&b.pub_date));
    rc.into_iter().last().map(|x| {
        if x.version.contains(".") {
            x.version
        } else {
            x.short_version
        }
    })
}

fn parse_dt(pub_date: &str) -> Result<DateTime<Utc>, ParseError> {
    DateTime::parse_from_rfc3339(pub_date)
        .or_else(|_| DateTime::parse_from_rfc2822(pub_date))
        .map(|d| d.to_utc())
        .or_else(|_| {
            NaiveDateTime::parse_from_str(pub_date, "%Y-%m-%d %H:%M:%S").map(|d| d.and_utc())
        })
}
