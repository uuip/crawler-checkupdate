use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use semver::Version;

pub(crate) fn parse_css(resp: &str, css: &str) -> Option<String> {
    let html = Html::parse_document(resp);
    let selector = Selector::parse(css).ok()?;
    let element = html.select(&selector).next()?.text().next()?.trim();
    Some(element.to_owned())
}

pub(crate) fn parse_navicat_mac(resp: &str) -> Option<String> {
    parse_navicat(resp, "M", "macOS")
}

pub(crate) fn parse_navicat_windows(resp: &str) -> Option<String> {
    parse_navicat(resp, "W", "Windows")
}

fn parse_navicat(resp: &str, platform: &str, system: &str) -> Option<String> {
    let start = Regex::new(&format!(r#"<table[^>]*platform="{platform}"[^>]*>"#))
        .ok()?
        .find(resp)?;
    let section = resp[start.end()..].split_once("</table>")?.0;
    Regex::new(&format!(r"Navicat Premium \({system}\) version ([\d.]+)"))
        .ok()?
        .captures(section)?
        .get(1)
        .map(|m| m.as_str().to_owned())
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
    html.select(&selector)
        .filter_map(|x| Version::parse(x.text().next().unwrap_or("0.0.0")).ok())
        .max()
        .map(|version| version.to_string())
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
