use super::*;

#[test]
fn keys_sort_between_neighbours() {
    let first = between(None, None);
    let earlier = before(Some(&first));
    let later = between(Some(&first), None);
    assert!(earlier < first && first < later);
    let mid = between(Some("a"), Some("b"));
    assert!(mid.as_str() > "a" && mid.as_str() < "b");
    let mut lo = first.clone();
    for _ in 0..100 {
        let k = between(Some(&lo), Some(&later));
        assert!(k > lo && k < later);
        lo = k;
    }
}

#[test]
fn matches_the_ui_implementation() {
    // Values from src/lib/util/order-key.ts for the same inputs.
    assert_eq!(between(None, None), "V");
    assert_eq!(between(Some("U"), None), "k");
    assert_eq!(between(None, Some("U")), "F");
    assert_eq!(between(Some("a"), Some("b")), "aV");
}
