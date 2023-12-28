use std::fs;
use std::io;
#[cfg(unix)]
use std::os::unix;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use filetime::FileTime;
use tempfile::TempDir;

use rusync::progress::ProgressInfo;

fn assert_same_contents(a: &Path, b: &Path) {
    assert!(a.exists(), "{:?} does not exist", a);
    assert!(b.exists(), "{:?} does not exist", b);
    let status = Command::new("diff")
        .args([a, b])
        .status()
        .expect("Failed to execute process");
    assert!(status.success(), "{:?} and {:?} differ", a, b)
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    let metadata = std::fs::metadata(path)
        .unwrap_or_else(|e| panic!("Could not get metadata of {:?}: {}", path, e));
    let permissions = metadata.permissions();
    let mode = permissions.mode();
    mode & 0o111 != 0
}

#[cfg(unix)]
fn assert_executable(path: &Path) {
    assert!(
        is_executable(path),
        "{:?} does not appear to be executable",
        path
    );
}

#[cfg(unix)]
fn assert_not_executable(path: &Path) {
    assert!(!is_executable(path), "{:?} appears to be executable", path);
}

fn setup_test(tmp_path: &Path) -> (PathBuf, PathBuf) {
    let src_path = tmp_path.join("src");
    let dest_path = tmp_path.join("dest");
    let status = Command::new("cp")
        .args(["-R", "tests/data", &src_path.to_string_lossy()])
        .status()
        .expect("Failed to start cp process");
    assert!(status.success(), "could not copy test data");
    (src_path, dest_path)
}

fn make_recent(path: &Path) -> io::Result<()> {
    let metadata = fs::metadata(path)?;
    let atime = FileTime::from_last_access_time(&metadata);
    let mtime = FileTime::from_last_modification_time(&metadata);
    let mut epoch = mtime.unix_seconds();
    epoch += 1;
    let mtime = FileTime::from_unix_time(epoch, 0);
    filetime::set_file_times(path, atime, mtime)?;
    Ok(())
}

struct DummyProgressInfo {}
impl ProgressInfo for DummyProgressInfo {}

fn new_test_syncer(src: &Path, dest: &Path) -> rusync::Syncer {
    let dummy_progress_info = DummyProgressInfo {};
    let options = rusync::SyncOptions {
        preserve_permissions: true,
    };
    rusync::Syncer::new(src, dest, options, Box::new(dummy_progress_info))
}

#[test]
fn fresh_copy() -> Result<(), std::io::Error> {
    let tmp_dir = TempDir::new()?;
    let (src_path, dest_path) = setup_test(tmp_dir.path());
    let syncer = new_test_syncer(&src_path, &dest_path);
    let outcome = syncer.sync();
    assert!(outcome.is_ok());

    let src_top = src_path.join("top.txt");
    let dest_top = dest_path.join("top.txt");
    assert_same_contents(&src_top, &dest_top);

    Ok(())
}

#[test]
fn skip_up_to_date_files() -> Result<(), std::io::Error> {
    let tmp_dir = TempDir::new()?;
    let (src_path, dest_path) = setup_test(tmp_dir.path());
    let syncer = new_test_syncer(&src_path, &dest_path);

    let stats = syncer.sync().unwrap();
    assert_eq!(stats.up_to_date, 0);

    let src_top_txt = src_path.join("top.txt");
    make_recent(&src_top_txt)?;
    let syncer = new_test_syncer(&src_path, &dest_path);

    let stats = syncer.sync().unwrap();
    assert_eq!(stats.copied, 1);
    Ok(())
}

#[test]
#[cfg(unix)]
fn preserve_permissions() -> Result<(), std::io::Error> {
    let tmp_dir = TempDir::new()?;
    let (src_path, dest_path) = setup_test(tmp_dir.path());
    let syncer = new_test_syncer(&src_path, &dest_path);
    syncer.sync().unwrap();

    let dest_exe = &dest_path.join("a_dir/foo.exe");
    assert_executable(dest_exe);
    Ok(())
}

#[test]
#[cfg(unix)]
fn do_not_preserve_permissions() -> Result<(), std::io::Error> {
    let tmp_dir = TempDir::new()?;
    let (src_path, dest_path) = setup_test(tmp_dir.path());
    let options = rusync::SyncOptions {
        preserve_permissions: false,
    };
    let syncer = rusync::Syncer::new(
        &src_path,
        &dest_path,
        options,
        Box::new(DummyProgressInfo {}),
    );
    syncer.sync().unwrap();

    let dest_exe = &dest_path.join("a_dir/foo.exe");
    assert_not_executable(dest_exe);
    Ok(())
}

#[test]
fn rewrite_partially_written_files() -> Result<(), std::io::Error> {
    let tmp_dir = TempDir::new()?;
    let (src_path, dest_path) = setup_test(tmp_dir.path());
    let src_top = src_path.join("top.txt");
    let expected = fs::read_to_string(src_top)?;

    let syncer = new_test_syncer(&src_path, &dest_path);
    syncer.sync().unwrap();
    let dest_top = dest_path.join("top.txt");
    // Corrupt the dest/top.txt
    fs::write(&dest_top, "this is")?;

    let syncer = new_test_syncer(&src_path, &dest_path);
    syncer.sync().unwrap();
    let actual = fs::read_to_string(&dest_top)?;
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn dest_read_only() -> Result<(), std::io::Error> {
    let tmp_dir = TempDir::new()?;
    let (src_path, dest_path) = setup_test(tmp_dir.path());
    fs::create_dir_all(&dest_path)?;

    let dest_top = dest_path.join("top.txt");
    fs::write(&dest_top, "this is read only")?;

    let mut perms = fs::metadata(&dest_top)?.permissions();
    perms.set_readonly(true);
    fs::set_permissions(&dest_top, perms)?;

    let src_top = src_path.join("top.txt");
    make_recent(&src_top)?;

    let syncer = new_test_syncer(&src_path, &dest_path);
    let result = syncer.sync().unwrap();
    assert_eq!(result.errors, 1);
    Ok(())
}

#[test]
#[cfg(unix)]
fn broken_link_in_src() -> Result<(), std::io::Error> {
    let tmp_dir = TempDir::new()?;
    let (src_path, dest_path) = setup_test(tmp_dir.path());
    let src_broken_link = &src_path.join("broken");
    unix::fs::symlink("no-such", src_broken_link)?;

    let syncer = new_test_syncer(&src_path, &dest_path);
    let result = syncer.sync();

    let dest_broken_link = &dest_path.join("broken");
    assert!(!dest_broken_link.exists());
    assert_eq!(dest_broken_link.read_link()?.to_string_lossy(), "no-such");
    assert!(result.is_ok());
    Ok(())
}
