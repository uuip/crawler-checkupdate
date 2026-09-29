use regex::Regex;
use scraper::{ElementRef, Html, Selector};

pub(crate) fn parse_css(resp: &str, css: &str) -> Option<String> {
    let html = Html::parse_document(resp);
    let selector = Selector::parse(css).ok()?;
    let text = html.select(&selector).next()?.text().collect::<String>();
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
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

    html.select(&selector).find_map(|x| {
        let text = x.text().collect::<String>();
        re.find(&text).map(|m| m.as_str().to_owned())
    })
}

pub(crate) fn parse_winrar(resp: &str) -> Option<String> {
    let html = Html::parse_document(resp);
    let selector = Selector::parse("b").ok()?;
    let re = Regex::new("^WinRAR.*elease").ok()?;

    html.select(&selector)
        .find_map(|x| re.find(x.text().next().unwrap_or_default()))
        .map(|m| m.as_str().to_owned())
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

#[cfg(test)]
mod tests {
    use super::{parse_css, parse_faststone};

    #[test]
    fn css_reads_text_across_nested_elements() {
        assert_eq!(
            parse_css("<h2>Version <b>5.2.6</b></h2>", "h2"),
            Some("Version 5.2.6".into())
        );
        assert_eq!(parse_css("<h2> 5.2.6 </h2>", "h2"), Some("5.2.6".into()));
        assert_eq!(parse_css("<h2></h2>", "h2"), None);
        assert_eq!(parse_css("<h2>5.2.6</h2>", ".missing"), None);
        assert_eq!(parse_css("<h2>5.2.6</h2>", "["), None);
    }

    #[test]
    fn faststone_reads_nested_release_heading() {
        for (html, expected) in [
            (
                "<b><font>&nbsp;&nbsp; <font>Version\n      11.3</font> </font></b>",
                "11.3",
            ),
            (
                "<b><font>\n      &nbsp;&nbsp; <font>Version 8.5</font> </font></b>",
                "8.5",
            ),
            ("<b>Version 7.9</b>", "7.9"),
        ] {
            assert_eq!(
                parse_faststone(html)
                    .and_then(crate::fetch::num_version)
                    .as_deref(),
                Some(expected)
            );
        }
        assert_eq!(parse_faststone("<b>No release</b>"), None);
    }
}
