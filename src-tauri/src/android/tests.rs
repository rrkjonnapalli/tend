use super::*;

#[test]
fn the_first_directory_with_certificates_is_used() {
    let tmp = tempfile::tempdir().unwrap();
    let empty = tmp.path().join("apex");
    let system = tmp.path().join("system");
    fs::create_dir_all(&empty).unwrap();
    fs::create_dir_all(&system).unwrap();
    fs::write(system.join("a.0"), "A").unwrap();
    fs::write(system.join("b.0"), "B").unwrap();
    let files = cert_files(&[empty.as_path(), system.as_path()]).unwrap();
    assert_eq!(files.len(), 2);
}

#[test]
fn no_certificates_anywhere_is_none() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(cert_files(&[tmp.path().join("none").as_path()]).is_none());
}
