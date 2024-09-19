use std::collections::HashMap;

use crate::parser::{appcast, html};
use crate::FnSignature;
use once_cell::sync::Lazy;

pub static FNRULES: Lazy<HashMap<&'static str, FnSignature>> = Lazy::new(|| {
    let mapper: [(&str, FnSignature); 14] = [
        ("DevManView", html::parse_dev_man_view),
        ("EmEditor", html::parse_emeditor),
        ("FS Capture", html::parse_faststone),
        ("FS Viewer", html::parse_faststone),
        ("VMware", html::parse_vmware),
        ("WinRAR", html::parse_winrar),
        ("PDF-XChange", html::parse_pdf_xchange),
        ("VSCode", html::parse_vscode),
        ("Postico 2", appcast::parse_appcast),
        ("Input Source Pro", appcast::parse_appcast),
        ("SwiftBar", appcast::parse_appcast),
        ("LinearMouse", appcast::parse_appcast),
        ("Docker", appcast::parse_appcast),
        ("AltTab", appcast::parse_appcast),
    ];
    HashMap::from(mapper)
});

pub static CSSRULES: Lazy<HashMap<&'static str, &str>> = Lazy::new(|| {
    let mapper: [(&str, &str); 13] = [
        ("SecureCRT", "#download-tabs>h4"),
        ("Registry Workshop", "p"),
        ("Firefox", ".c-release-version"),
        (
            "Navicat [Mac]",
            r#".release-notes-table[platform="M"] td>.note-title"#,
        ),
        (
            "Navicat",
            r#".release-notes-table[platform="W"] td>.note-title"#,
        ),
        ("Everything", "h2"),
        ("Python", ".download-widget a"),
        ("Contexts [Mac]", ".section--history__item__header>h1"),
        ("WGestures 2", "a#download:nth-of-type(1)"),
        ("WGestures 2 [Mac]", "a#download:nth-of-type(2)"),
        ("Git", ".version"),
        ("AIDA64", "td.version"),
        ("Beyond Compare", "div#content h2"),
    ];
    HashMap::from(mapper)
});
