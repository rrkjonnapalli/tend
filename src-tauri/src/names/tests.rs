use super::*;
use crate::error::errors;

#[test]
fn name_rule() {
    for good in [
        "work",
        "Q4 Plan – draft",
        "ideas (old)",
        "Café notes",
        "a_b",
        "2024",
    ] {
        assert!(is_valid(good), "{good:?} should be valid");
    }
    let cases = [
        ("", errors::NAME_EMPTY),
        (" padded", errors::NAME_SPACES),
        ("padded ", errors::NAME_SPACES),
        ("a/b", errors::NAME_CHARS),
        ("a\\b", errors::NAME_CHARS),
        ("a:b", errors::NAME_CHARS),
        (".hidden", errors::NAME_CHARS),
        ("tab\there", errors::NAME_CHARS),
    ];
    for (bad, def) in cases {
        assert!(check(bad).unwrap_err().is(def), "{bad:?}");
    }
    assert!(check(&"x".repeat(201)).unwrap_err().is(errors::NAME_LONG));
}

#[test]
fn paths() {
    assert!(check_path("").is_ok());
    assert!(check_path("Work/Tauri app").is_ok());
    assert!(check_path("work//x").is_err());
    assert!(check_path("work/.git").is_err());
    assert_eq!(join("", "a"), "a");
    assert_eq!(join("a", "b"), "a/b");
    assert_eq!(parent("a/b/c"), "a/b");
    assert_eq!(parent("a"), "");
    assert_eq!(base("a/b/c"), "c");
    assert!(is_under("a/b", "a"));
    assert!(!is_under("ab", "a"));
    assert!(is_under("anything", ""));
}
