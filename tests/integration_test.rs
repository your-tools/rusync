extern crate tempdir;

extern crate rusync;

use std::path::PathBuf;
use std::process::Command;
use tempdir::TempDir;

use rusync::app;

fn assert_same_contents(a: &PathBuf, b: &PathBuf) {
    assert!(a.exists(), "{:?} does not exist", a);
    assert!(b.exists(), "{:?} does not exist", b);
    let status = Command::new("diff")
             .args(&[a, b])
             .status()
             .expect("Failed to execute process");
    assert!(status.success(), "{:?} and {:?} differ", a, b)
}

fn setup_test(tmp_path: PathBuf) -> (PathBuf, PathBuf) {
    let src_path = tmp_path.join("src");
    let dest_path = tmp_path.join("dest");
    let status = Command::new("cp")
             .args(&["-r", "tests/data", &src_path.to_string_lossy()])
             .status()
             .expect("Failed to execute process");
    assert!(status.success());
    (src_path, dest_path)
}

#[test]
fn test_zero() {
    let tmp_dir = TempDir::new("test-rusync").expect("failed to create temp dir");
    let (src_path, dest_path) = setup_test(tmp_dir.path().to_path_buf());
    let src_top = src_path.join("top.txt");
    let dest_top = dest_path.join("top.txt");
    let outcome = app::sync(src_path, dest_path);
    assert!(outcome.is_ok(), "app::sync failed with: {}", outcome.err().expect(""));

    assert_same_contents(&src_top, &dest_top);
}
