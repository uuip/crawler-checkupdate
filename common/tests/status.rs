use common::{FAILED_KEY, SUCCESS_KEY, StatusPrinter, StatusRecorder, init_status};
use std::collections::HashMap;

#[test]
fn empty_and_missing_status_lists_are_empty() {
    let status = init_status();
    assert!(status.get_list(SUCCESS_KEY).is_empty());
    assert!(status.get_list(FAILED_KEY).is_empty());
    assert!(status.get_list("missing").is_empty());
}

#[test]
fn plain_status_can_be_read() {
    let status = HashMap::from([(SUCCESS_KEY, vec!["app".to_owned()])]);
    assert_eq!(status.get_list(SUCCESS_KEY), ["app"]);
    assert!(status.get_list(FAILED_KEY).is_empty());
}

#[test]
fn shared_status_keeps_concurrent_updates() {
    let status = init_status();
    std::thread::scope(|scope| {
        for i in 0..16 {
            let status = &status;
            scope.spawn(move || status.add_to_list(SUCCESS_KEY, i.to_string()));
        }
    });
    assert_eq!(status.get_list(SUCCESS_KEY).len(), 16);
    assert!(status.get_list(FAILED_KEY).is_empty());
}

#[cfg(feature = "dashmap-support")]
#[test]
fn dashmap_status_keeps_concurrent_updates() {
    let status = std::sync::Arc::new(dashmap::DashMap::new());
    assert!(status.get_list(SUCCESS_KEY).is_empty());
    std::thread::scope(|scope| {
        for i in 0..16 {
            let status = &status;
            scope.spawn(move || status.add_to_list(SUCCESS_KEY, i.to_string()));
        }
    });
    assert_eq!(status.get_list(SUCCESS_KEY).len(), 16);
    assert!(status.get_list(FAILED_KEY).is_empty());
}
