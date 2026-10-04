use super::*;

#[test]
fn errors_reach_the_ui_as_key_and_message() {
    let json = serde_json::to_value(errors::NOT_FOUND.with("Note was not found")).unwrap();
    assert_eq!(
        json,
        serde_json::json!({ "key": "NOT_FOUND", "message": "Note was not found" })
    );
    let json = serde_json::to_value(errors::NO_VAULT.err()).unwrap();
    assert_eq!(json["message"], "No vault is open");
}

#[test]
fn file_errors_map_to_registry_entries() {
    let denied = std::io::Error::from(ErrorKind::PermissionDenied);
    assert!(AppError::from(denied).is(errors::NO_PERMISSION));
    let missing = std::io::Error::from(ErrorKind::NotFound);
    assert!(AppError::from(missing).is(errors::IO));
}
