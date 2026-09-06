use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type SharedStatus = Arc<Mutex<HashMap<&'static str, Vec<String>>>>;

pub const SUCCESS_KEY: &str = "success";
pub const FAILED_KEY: &str = "failed";

pub trait StatusPrinter {
    fn get_list(&self, key: &str) -> Vec<String>;
}

pub trait StatusRecorder: StatusPrinter {
    fn add_to_list(&self, key: &'static str, value: String);
}

impl StatusPrinter for HashMap<&str, Vec<String>> {
    fn get_list(&self, key: &str) -> Vec<String> {
        self.get(key).cloned().unwrap_or_default()
    }
}

impl StatusPrinter for SharedStatus {
    fn get_list(&self, key: &str) -> Vec<String> {
        self.lock().unwrap().get(key).cloned().unwrap_or_default()
    }
}

impl StatusRecorder for SharedStatus {
    fn add_to_list(&self, key: &'static str, value: String) {
        self.lock().unwrap().entry(key).or_default().push(value);
    }
}

#[cfg(feature = "dashmap-support")]
impl StatusPrinter for Arc<dashmap::DashMap<&'static str, Vec<String>>> {
    fn get_list(&self, key: &str) -> Vec<String> {
        self.get(key).map(|v| v.clone()).unwrap_or_default()
    }
}

#[cfg(feature = "dashmap-support")]
impl StatusRecorder for Arc<dashmap::DashMap<&'static str, Vec<String>>> {
    fn add_to_list(&self, key: &'static str, value: String) {
        self.entry(key).or_default().push(value);
    }
}

pub fn print_status<T: StatusPrinter>(status: &T) {
    let success = status.get_list(SUCCESS_KEY).join(", ");
    let failed = status.get_list(FAILED_KEY).join(", ");
    println!(
        "成功: {}\n失败: {}",
        if success.is_empty() { "无" } else { &success },
        if failed.is_empty() { "无" } else { &failed }
    );
}

pub fn init_status() -> SharedStatus {
    Arc::new(Mutex::new(HashMap::from([
        (SUCCESS_KEY, Vec::new()),
        (FAILED_KEY, Vec::new()),
    ])))
}
