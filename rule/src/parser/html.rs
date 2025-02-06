use regex::Regex;
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

pub(crate) fn parse_pdf_xchange(resp: &str) -> Option<String> {
    let html = Html::parse_document(resp);
    let selector = Selector::parse("div.version").unwrap();
    let element = html.select(&selector).next()?.text().nth(2)?.trim();
    Some(element.to_owned())
}
