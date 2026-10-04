use super::*;
use crate::error::errors;

#[test]
fn name_rule() {
    assert!(is_valid("work_2024"));
    for bad in ["", "Work", "my notes", "a-b", "a.b", "é"] {
        assert!(!is_valid(bad), "{bad:?} should be invalid");
    }
    assert!(check("").unwrap_err().is(errors::NAME_EMPTY));
    assert!(check("A").unwrap_err().is(errors::NAME_CHARS));
}

#[test]
fn paths() {
    assert!(check_path("").is_ok());
    assert!(check_path("work/tauri").is_ok());
    assert!(check_path("work/Tauri").is_err());
    assert!(check_path("work//x").is_err());
    assert_eq!(join("", "a"), "a");
    assert_eq!(join("a", "b"), "a/b");
    assert_eq!(parent("a/b/c"), "a/b");
    assert_eq!(parent("a"), "");
    assert_eq!(base("a/b/c"), "c");
    assert!(is_under("a/b", "a"));
    assert!(!is_under("ab", "a"));
    assert!(is_under("anything", ""));
}
