use super::*;
use crate::error::errors;

#[test]
fn name_rule() {
    assert!(is_valid("work_2024") && is_valid("a_b_c") && is_valid("2024"));
    for bad in [
        "", "Work", "my notes", "a-b", "a.b", "é", "_a", "a_", "a__b", "_",
    ] {
        assert!(!is_valid(bad), "{bad:?} should be invalid");
    }
}

#[test]
fn slugs() {
    assert_eq!(slug("Meeting").as_deref(), Some("meeting"));
    assert_eq!(slug("Café notes (Q4)!").as_deref(), Some("cafe_notes_q4"));
    assert_eq!(slug("  Résumé -- 2024 ").as_deref(), Some("resume_2024"));
    assert_eq!(slug(".hidden").as_deref(), Some("hidden"));
    assert_eq!(slug("日本"), None);
    assert_eq!(slug("🔥"), None);
    assert_eq!(slug(&"é".repeat(300)).unwrap().len(), 200);
    for text in ["a — b • c", "Plan_1", "x__y"] {
        let made = slug(text).unwrap();
        assert!(is_valid(&made), "{made}");
    }
}

#[test]
fn typed_folder_names() {
    assert_eq!(accept("Client Work").unwrap(), "client_work");
    assert!(accept("  ").unwrap_err().is(errors::NAME_EMPTY));
    assert!(accept("!!").unwrap_err().is(errors::NAME_CHARS));
    assert_eq!(accept_path("/Work / Q4 Plans/").unwrap(), "work/q4_plans");
    assert_eq!(accept_path("").unwrap(), "");
    assert!(accept_path("work//x").is_err());
}

#[test]
fn clashes_get_a_suffix() {
    let taken = ["plan", "plan_1"];
    let is_taken = |n: &str| Ok(taken.contains(&n));
    assert_eq!(free("plan", is_taken).unwrap(), "plan_2");
    assert_eq!(free("other", is_taken).unwrap(), "other");
}

#[test]
fn paths() {
    assert_eq!(join("", "a"), "a");
    assert_eq!(join("a", "b"), "a/b");
    assert_eq!(parent("a/b/c"), "a/b");
    assert_eq!(parent("a"), "");
    assert_eq!(base("a/b/c"), "c");
    assert!(is_under("a/b", "a"));
    assert!(!is_under("ab", "a"));
    assert!(is_under("anything", ""));
}
