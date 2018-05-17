extern crate filetime;
extern crate tempdir;

extern crate rusync;

use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use filetime::FileTime;
use tempdir::TempDir;

use rusync::sync::Syncer;

fn assert_same_contents(a: &Path, b: &Path) {
    assert!(a.exists(), "{:?} does not exist", a);
    assert!(b.exists(), "{:?} does not exist", b);
    let status = Command::new("diff")
        .args(&[a, b])
        .status()
        .expect("Failed to execute process");
    assert!(status.success(), "{:?} and {:?} differ", a, b)
}

fn is_executable(path: &Path) -> bool {
    let metadata =
        std::fs::metadata(&path).expect(&format!("Could not get metadata of {:?}", path));
    let permissions = metadata.permissions();
    let mode = permissions.mode();
    mode & 0o111 != 0
}

fn assert_executable(path: &Path) {
    assert!(
        is_executable(&path),
        "{:?} does not appear to be executable",
        path
    );
}

fn assert_not_executable(path: &Path) {
    assert!(!is_executable(&path), "{:?} appears to be executable", path);
}

fn setup_test(tmp_path: &Path) -> (PathBuf, PathBuf) {
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
    let (src_path, dest_path) = setup_test(&tmp_dir.path());
    let mut syncer = Syncer::new(&src_path, &dest_path);
    let outcome = syncer.sync();
    assert!(
        outcome.is_ok(),
        "sync::sync failed with: {}",
        outcome.err().expect("")
    );

    let src_top = src_path.join("top.txt");
    let dest_top = dest_path.join("top.txt");
    assert_same_contents(&src_top, &dest_top);

    let link_dest = dest_path.join("a_dir/link_to_one");
    let target = fs::read_link(link_dest).expect("failed to read metada");
    assert_eq!(target.to_string_lossy(), "one.txt");
}

#[test]
fn skip_up_to_date_files() {
    let tmp_dir = TempDir::new("test-rusync").expect("failed to create temp dir");
    let (src_path, dest_path) = setup_test(&tmp_dir.path());
    let mut syncer = Syncer::new(&src_path, &dest_path);
    syncer.sync().expect("");

    let stats = syncer.stats();
    assert_eq!(stats.up_to_date, 0);

    let src_top_txt = src_path.join("top.txt");
    make_recent(&src_top_txt).expect("could not make top.txt recent");
    let mut syncer = Syncer::new(&src_path, &dest_path);
    syncer.sync().expect("");

    let stats = syncer.stats();
    assert_eq!(stats.copied, 1);
}

#[test]
fn preserve_permissions() {
    let tmp_dir = TempDir::new("test-rusync").expect("failed to create temp dir");
    let (src_path, dest_path) = setup_test(&tmp_dir.path());
    let mut syncer = Syncer::new(&src_path, &dest_path);
    syncer.sync().expect("");

    let dest_exe = &dest_path.join("a_dir/foo.exe");
    assert_executable(&dest_exe);
}

#[test]
fn do_not_preserve_permissions() {
    let tmp_dir = TempDir::new("test-rusync").expect("failed to create temp dir");
    let (src_path, dest_path) = setup_test(&tmp_dir.path());
    let mut syncer = Syncer::new(&src_path, &dest_path);
    syncer.preserve_permissions(false);
    syncer.sync().expect("");

    let dest_exe = &dest_path.join("a_dir/foo.exe");
    assert_not_executable(&dest_exe);
}

#[test]
fn rewrite_partially_written_files() {
    let tmp_dir = TempDir::new("test-rusync").expect("failed to create temp dir");
    let (src_path, dest_path) = setup_test(&tmp_dir.path());
    let src_top = src_path.join("top.txt");
    let expected = fs::read_to_string(&src_top).expect("");

    let mut syncer = Syncer::new(&src_path, &dest_path);
    syncer.sync().expect("");
    let dest_top = dest_path.join("top.txt");
    // Corrupt the dest/top.txt
    fs::write(&dest_top, "this is").expect("");
    syncer.sync().expect("");

    let actual = fs::read_to_string(&dest_top).expect("");
    assert_eq!(actual, expected);
}
