extern crate tempdir;

extern crate rusync;

use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use std::time;
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
fn test_fresh_copy() {
    let tmp_dir = TempDir::new("test-rusync").expect("failed to create temp dir");
    let (src_path, dest_path) = setup_test(tmp_dir.path().to_path_buf());
    let outcome = app::sync(&src_path, &dest_path);
    assert!(outcome.is_ok(), "app::sync failed with: {}", outcome.err().expect(""));

    let src_top = src_path.join("top.txt");
    let dest_top = dest_path.join("top.txt");
    assert_same_contents(&src_top, &dest_top);
}

#[test]
fn test_skip_up_to_date_files() {
    let tmp_dir = TempDir::new("test-rusync").expect("failed to create temp dir");
    let (src_path, dest_path) = setup_test(tmp_dir.path().to_path_buf());

    let stats = app::sync(&src_path, &dest_path).unwrap();
    assert_eq!(stats.up_to_date, 0);

    let src_top = src_path.join("top.txt");
    let mut file = File::create(src_top).expect("Could not open {:?} for writing");
    file.write_all(b"new top\n").expect("");
    let time_to_sleep = time::Duration::from_secs(1);
    std::thread::sleep(time_to_sleep);
    let stats = app::sync(&src_path, &dest_path).unwrap();
    assert_eq!(stats.copied, 1);
}
