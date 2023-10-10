use std::collections::HashMap;

use once_cell::sync::Lazy;

use crate::rules;
use crate::FnSignature;

pub static FNRULES: Lazy<HashMap<&'static str, FnSignature>> = Lazy::new(|| {
    let mapper: [(&str, FnSignature); 8] = [
        ("DevManView", rules::parse_dev_man_view),
        ("EmEditor", rules::parse_emeditor),
        ("FS Capture", rules::parse_faststone),
        ("FS Viewer", rules::parse_faststone),
        ("VMware", rules::parse_vmware),
        ("WinRAR", rules::parse_winrar),
        ("PDF-XChange", rules::parse_pdf_xchange),
        ("VSCode", rules::parse_vscode),
    ];
    HashMap::from(mapper)
});

pub static CSSRULES: Lazy<HashMap<&'static str, &str>> = Lazy::new(|| {
    let mapper: [(&str, &str); 13] = [
        ("SecureCRT", "#download-tabs>h4"),
        ("Registry Workshop", "p"),
        ("Firefox", ".c-release-version"),
        (
            "Navicat[Mac]",
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
