use std::collections::HashMap;

use crate::parser::html;
use crate::FnSignature;
use once_cell::sync::Lazy;

pub static FNRULES: Lazy<HashMap<&'static str, FnSignature>> = Lazy::new(|| {
    let mapper: [(&str, FnSignature); 6] = [
        ("DevManView", html::parse_dev_man_view),
        ("FS Capture", html::parse_faststone),
        ("FS Viewer", html::parse_faststone),
        ("VMware", html::parse_vmware),
        ("WinRAR", html::parse_winrar),
        ("PDF-XChange", html::parse_pdf_xchange),
    ];
    HashMap::from(mapper)
});