use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use semver::Version;

pub(crate) fn parse_css(resp: &str, css: &str) -> Option<String> {
    let html = Html::parse_document(resp);
    let selector = Selector::parse(css).ok()?;
    let element = html.select(&selector).next()?.text().next()?.trim();
    Some(element.to_owned())
}

pub(crate) fn parse_faststone(resp: &str) -> Option<String> {
    let html = Html::parse_document(resp);
    let selector = Selector::parse("b").ok()?;
    let re = Regex::new(r"Version\s*[.\d]+").ok()?;

    html.select(&selector)
        .find_map(|x| re.find(x.text().next().unwrap_or_default()))
        .map(|m| m.as_str().to_owned())
}

pub(crate) fn parse_winrar(resp: &str) -> Option<String> {
    let html = Html::parse_document(resp);
    let selector = Selector::parse("b").ok()?;
    let re = Regex::new("^WinRAR.*elease").ok()?;

    html.select(&selector)
        .find_map(|x| re.find(x.text().next().unwrap_or_default()))
        .map(|m| m.as_str().to_owned())
}

pub(crate) fn parse_vmware(resp: &str) -> Option<String> {
    let html = Html::parse_fragment(resp);
    let selector = Selector::parse("metadata>version").ok()?;
    let mut versions: Vec<Version> = html
        .select(&selector)
        .filter_map(|x| Version::parse(x.text().next().unwrap_or("0.0.0")).ok())
        .collect();
    versions.sort();
    versions.last().map(ToString::to_string)
}

pub(crate) fn parse_dev_man_view(resp: &str) -> Option<String> {
    let html = Html::parse_document(resp);
    let selector = Selector::parse("h4").ok()?;
    let heading = html
        .select(&selector)
        .find(|x| x.text().next().unwrap_or_default() == "Versions History")?;
    let node = heading.next_siblings().nth(1)?.children().nth(1)?;
    let text = ElementRef::wrap(node)?.text().next()?;
    Some(text.to_owned())
}

pub(crate) fn parse_pdf_xchange(resp: &str) -> Option<String> {
    let html = Html::parse_document(resp);
    let selector = Selector::parse("div.version").ok()?;
    let version_text = html.select(&selector).next()?.text().nth(2)?.trim();
    Some(version_text.to_owned())
}
