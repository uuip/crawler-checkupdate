use std::collections::HashMap;

use crate::FnSignature;
use crate::parser::html;
use std::sync::LazyLock;

pub static FNRULES: LazyLock<HashMap<&'static str, FnSignature>> = LazyLock::new(|| {
    let mapper: [(&str, FnSignature); 8] = [
        ("DevManView", html::parse_dev_man_view),
        ("FS Capture", html::parse_faststone),
        ("FS Viewer", html::parse_faststone),
        ("VMware", html::parse_vmware),
        ("WinRAR", html::parse_winrar),
        ("PDF-XChange", html::parse_pdf_xchange),
        ("Navicat [Mac]", html::parse_navicat_mac),
        ("Navicat", html::parse_navicat_windows),
    ];
    HashMap::from(mapper)
});

#[cfg(test)]
mod tests {
    use super::FNRULES;

    #[test]
    fn parses_navicat_release_notes_by_platform() {
        let html = r#"
            <table class="release-notes-table" platform="W"><tbody><b>Navicat Premium (Windows) version 19.0.0</b></tbody></table>
            <table class="release-notes-table" platform="M"><tbody>
                Sep 23 2026 <b>Navicat Premium (macOS) version 18.0.2</b>
                Sep 16 2026 <b>Navicat Premium (macOS) version 18.0.1</b>
            </tbody></table>
        "#;
        assert_eq!(
            FNRULES.get("Navicat [Mac]").and_then(|f| f(html)),
            Some("18.0.2".into())
        );
        assert_eq!(
            FNRULES.get("Navicat").and_then(|f| f(html)),
            Some("19.0.0".into())
        );
    }
}
