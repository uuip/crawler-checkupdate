use std::collections::HashMap;

use crate::FnSignature;
use crate::parser::html;
use std::sync::LazyLock;

pub static FNRULES: LazyLock<HashMap<&'static str, FnSignature>> = LazyLock::new(|| {
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
