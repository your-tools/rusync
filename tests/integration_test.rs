extern crate tempdir;
extern crate filetime;

extern crate rusync;

use std::io;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use filetime::FileTime;
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

fn assert_executable(path: &Path) {
    let metadata = std::fs::metadata(&path).expect(&format!("Could not get metadata of {:?}", path));
    let permissions = metadata.permissions();
    let mode = permissions.mode();
    assert!(mode & 0o111 != 0, "{:?} does not appear to be executable", path);
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

fn make_recent(path: &Path) -> io::Result<()> {
    let metadata = fs::metadata(&path)?;
    let atime = FileTime::from_last_access_time(&metadata);
    let mtime = FileTime::from_last_modification_time(&metadata);
    let mut epoch = mtime.seconds_relative_to_1970();
    epoch += 1;
    let mtime = FileTime::from_seconds_since_1970(epoch, 0);
    filetime::set_file_times(&path, atime, mtime)?;
    Ok(())
}

#[test]
fn fresh_copy() {
    let tmp_dir = TempDir::new("test-rusync").expect("failed to create temp dir");
    let (src_path, dest_path) = setup_test(tmp_dir.path().to_path_buf());
    let outcome = app::sync(&src_path, &dest_path);
    assert!(outcome.is_ok(), "app::sync failed with: {}", outcome.err().expect(""));

    let src_top = src_path.join("top.txt");
    let dest_top = dest_path.join("top.txt");
    assert_same_contents(&src_top, &dest_top);
}

#[test]
fn skip_up_to_date_files() {
    let tmp_dir = TempDir::new("test-rusync").expect("failed to create temp dir");
    let (src_path, dest_path) = setup_test(tmp_dir.path().to_path_buf());

    let stats = app::sync(&src_path, &dest_path).unwrap();
    assert_eq!(stats.up_to_date, 0);

    let src_top_txt = src_path.join("top.txt");
    make_recent(&src_top_txt).expect("could not make top.txt recent");
    let stats = app::sync(&src_path, &dest_path).unwrap();
    assert_eq!(stats.copied, 1);
}

#[test]
fn preserve_perms() {
    let tmp_dir = TempDir::new("test-rusync").expect("failed to create temp dir");
    let (src_path, dest_path) = setup_test(tmp_dir.path().to_path_buf());

    app::sync(&src_path, &dest_path).expect("sync failed");

    let dest_exe = &dest_path.join("a_dir/foo.exe");
    assert_executable(&dest_exe);
}
