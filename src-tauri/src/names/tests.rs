use super::*;
use crate::error::errors;

#[test]
fn name_rule() {
    for good in [
        "work",
        "Q4 Plan - draft",
        "notes, old",
        "a_b",
        "2024",
        "v1.2",
    ] {
        assert!(is_valid(good), "{good:?} should be valid");
    }
    for bad in ["Café", "a/b", "a:b", "ideas (old)", "tab\there"] {
        assert!(check(bad).unwrap_err().is(errors::NAME_CHARS), "{bad:?}");
    }
    assert!(check("").unwrap_err().is(errors::NAME_EMPTY));
    assert!(check(&"x".repeat(201)).unwrap_err().is(errors::NAME_LONG));
}

#[test]
fn typed_names_are_trimmed() {
    assert_eq!(accept("  .Plan. ").unwrap(), "Plan");
    assert_eq!(accept(".hidden").unwrap(), "hidden");
    assert!(accept(" . ").unwrap_err().is(errors::NAME_EMPTY));
    assert!(
        !is_valid(".hidden") && !is_valid("Plan "),
        "a scan takes names as they are"
    );
}

#[test]
fn clashes_get_a_suffix() {
    let taken = ["plan", "Plan_1"];
    let is_taken = |n: &str| Ok(taken.iter().any(|t| same(t, n)));
    assert_eq!(free("Plan", is_taken).unwrap(), "Plan_2");
    assert_eq!(free("Other", is_taken).unwrap(), "Other");
}

#[test]
fn paths() {
    assert_eq!(accept_path("").unwrap(), "");
    assert_eq!(accept_path("/Work / Tauri app/").unwrap(), "Work/Tauri app");
    assert!(accept_path("work//x").is_err());
    assert!(accept_path("work/Café").is_err());
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
fn names_from_disk_are_normalized() {
    assert_eq!(
        normalize("Café notes (old)!").as_deref(),
        Some("Cafe notes old")
    );
    assert_eq!(normalize("  Résumé  2024 ").as_deref(), Some("Resume 2024"));
    assert_eq!(normalize(".hidden").as_deref(), Some("hidden"));
    assert_eq!(normalize("日本語").as_deref(), None);
    assert_eq!(normalize("🔥").as_deref(), None);
    assert_eq!(normalize(&"é".repeat(300)).unwrap().len(), 200);
    let made = normalize("a — b • c").unwrap();
    assert!(is_valid(&made), "{made}");
}
