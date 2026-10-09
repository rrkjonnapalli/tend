use super::*;

#[test]
fn a_token_is_saved_read_replaced_and_cleared_per_remote() {
    let url = "https://example.com/a.git";
    let other = "https://example.com/b.git";
    assert_eq!(load(url).unwrap(), None);
    save(url, "one").unwrap();
    save(url, "two").unwrap();
    assert_eq!(load(url).unwrap().as_deref(), Some("two"));
    assert_eq!(load(other).unwrap(), None, "each remote has its own token");
    clear(url).unwrap();
    clear(url).unwrap();
    assert_eq!(load(url).unwrap(), None);
}
