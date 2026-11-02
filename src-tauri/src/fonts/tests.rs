use super::*;

#[test]
fn names_are_sorted_without_case_once_each_and_hidden_ones_left_out() {
    let names = ["lato", "Menlo", ".SF NS", "Avenir", "Menlo"].map(String::from);
    assert_eq!(listed(names.into_iter()), ["Avenir", "lato", "Menlo"]);
}

