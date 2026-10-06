use super::*;
use crate::error::errors;

#[test]
fn name_rule() {
    assert!(is_valid("work_2024"));
    assert!(is_valid("a_b_c") && is_valid("2024"));
    for bad in [
        "", "Work", "my notes", "a-b", "a.b", "é", "_a", "a_", "a__b", "_",
    ] {
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

#[test]
fn slugs_from_titles() {
    assert_eq!(slug("Window config!").as_deref(), Some("window_config"));
    assert_eq!(slug(" Q3 -- plan ").as_deref(), Some("q3_plan"));
    assert_eq!(slug("Café notes").as_deref(), Some("caf_notes"));
    assert_eq!(slug("___").as_deref(), None);
    assert_eq!(slug("").as_deref(), None);
    let made = slug("a  -  B").unwrap();
    assert!(is_valid(&made), "{made}");
}
