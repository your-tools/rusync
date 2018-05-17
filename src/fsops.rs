use std;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::io::prelude::*;
use std::os::unix;

use colored::Colorize;
use filetime::FileTime;

use entry::Entry;

const BUFFER_SIZE: usize = 100 * 1024;

#[derive(PartialEq, Debug)]
pub enum SyncOutcome {
    UpToDate,
    FileCopied,
    SymlinkUpdated,
    SymlinkCreated,
}

pub fn to_io_error(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, message)
}

fn is_more_recent_than(src: &Entry, dest: &Entry) -> io::Result<bool> {
    if !dest.exists() {
        return Ok(true);
    }

    let src_meta = &src.metadata();
    let dest_meta = &dest.metadata();

    let src_meta = &src_meta.expect("src_meta was None");
    let dest_meta = &dest_meta.expect("dest_meta was None");

    let src_mtime = FileTime::from_last_modification_time(&src_meta);
    let dest_mtime = FileTime::from_last_modification_time(&dest_meta);

    let src_precise = src_mtime.seconds() * 1000 * 1000 * 1000 + u64::from(src_mtime.nanoseconds());
    let dest_precise =
        dest_mtime.seconds() * 1000 * 1000 * 1000 + u64::from(dest_mtime.nanoseconds());

    Ok(src_precise > dest_precise)
}

pub fn copy_permissions(src: &Entry, dest: &Entry) -> io::Result<()> {
    let src_meta = &src.metadata();
    let src_meta = &src_meta.expect("src_meta was None");
    let is_link = src.is_link().expect("is_link should not be none");
    if is_link {
        return Ok(());
    }
    let permissions = src_meta.permissions();
    let dest_file = File::open(dest.path())?;
    dest_file.set_permissions(permissions)?;
    Ok(())
}

fn copy_link(src: &Entry, dest: &Entry) -> io::Result<(SyncOutcome)> {
    let src_target = std::fs::read_link(src.path())?;
    let is_link = dest.is_link();
    let outcome;
    match is_link {
        Some(true) => {
            let dest_target = std::fs::read_link(dest.path())?;
            if dest_target != src_target {
                println!("{} removing {}", "<-".red(), src.description().bold());
                fs::remove_file(dest.path())?;
                outcome = SyncOutcome::SymlinkUpdated;
            } else {
                return Ok(SyncOutcome::UpToDate);
            }
        }
        Some(false) => {
            // Never safe to delete
            return Err(to_io_error(format!(
                "Refusing to replace existing path {:?} by symlink",
                dest.path()
            )));
        }
        None => {
            // OK, dest does not exist
            outcome = SyncOutcome::SymlinkCreated;
        }
    }
    println!(
        "{} creating {} -> {}",
        "->".blue(),
        src.description().bold(),
        src_target.to_string_lossy()
    );
    unix::fs::symlink(src_target, &dest.path())?;
    Ok(outcome)
}

pub fn copy_entry(src: &Entry, dest: &Entry) -> io::Result<SyncOutcome> {
    let src_path = src.path();
    let src_file = File::open(src_path)?;
    let src_meta = src.metadata().expect("src_meta should not be None");
    let src_size = src_meta.len();
    let mut done = 0;
    let mut buf_reader = BufReader::new(src_file);
    let dest_path = dest.path();
    let dest_file = File::create(dest_path)?;
    let mut buf_writer = BufWriter::new(dest_file);
    let mut buffer = vec![0; BUFFER_SIZE];
    println!("{} copying {}", "->".blue(), src.description().bold());
    loop {
        let num_read = buf_reader.read(&mut buffer)?;
        if num_read == 0 {
            break;
        }
        done += num_read;
        let percent = ((done * 100) as u64) / src_size;
        print!("{number:>width$}%\r", number = percent, width = 3);
        let _ = io::stdout().flush();
        buf_writer.write_all(&buffer[0..num_read])?;
    }
    Ok(SyncOutcome::FileCopied)
}

fn is_truncated(src: &Entry, dest: &Entry) -> bool {
    let src_meta = src.metadata().expect("src_meta should not be None");
    let src_size = src_meta.len();

    let dest_meta = dest.metadata();
    if dest_meta.is_none() {
        return true;
    }
    let dest_size = dest_meta.unwrap().len();
    if dest_size < src_size {
        return true;
    }
    false
}

pub fn sync_entries(src: &Entry, dest: &Entry) -> io::Result<(SyncOutcome)> {
    src.is_link().expect("src.is_link should not be None");
    if src.is_link().unwrap() {
        return copy_link(&src, &dest);
    }
    let truncated = is_truncated(&src, &dest);
    let more_recent = is_more_recent_than(&src, &dest)?;
    // TODO: check if files really are different ?
    if more_recent || truncated {
        return copy_entry(&src, &dest);
    }
    Ok(SyncOutcome::UpToDate)
}

#[cfg(test)]
mod tests {

    extern crate tempdir;
    use self::tempdir::TempDir;

    use std;
    use std::error::Error;
    use std::fs::File;
    use std::io;
    use std::io::prelude::*;
    use std::os::unix;
    use std::path::Path;
    use std::path::PathBuf;

    use super::Entry;
    use super::SyncOutcome;
    use super::copy_link;

    fn create_file(path: &Path) {
        let mut out = File::create(path).expect(&format!("could not open {:?} for writing", path));
        out.write_all(b"").expect("could not write old test");
    }

    fn create_link(src: &str, dest: &Path) {
        unix::fs::symlink(&src, &dest).expect(&format!("could not link {:?} -> {:?}", src, dest));
    }

    fn assert_links_to(tmp_path: &Path, src: &str, dest: &str) {
        let src_path = tmp_path.join(src);
        let link = std::fs::read_link(src_path).expect(&format!("could not read link {:?}", src));
        assert_eq!(link.to_string_lossy(), dest);
    }

    fn setup_copy_test(tmp_path: &Path) -> PathBuf {
        let src = &tmp_path.join("src");
        create_file(&src);
        let src_link = &tmp_path.join("src_link");
        create_link("src", &src_link);
        src_link.to_path_buf()
    }

    fn sync_src_link(tmp_path: &Path, src_link: &Path, dest: &str) -> io::Result<(SyncOutcome)> {
        let src_entry = Entry::new("src", &src_link);
        let dest_path = &tmp_path.join(&dest);
        let dest_entry = Entry::new(&dest, dest_path);
        copy_link(&src_entry, &dest_entry)
    }

    #[test]
    fn copy_link_dest_does_not_exist() {
        let tmp_dir = TempDir::new("test-rusync-fsops").expect("failed to create temp dir");
        let tmp_path = tmp_dir.path();
        let src_link = setup_copy_test(tmp_path);

        let outcome = sync_src_link(&tmp_path, &src_link, "new");
        assert_eq!(outcome.expect(""), SyncOutcome::SymlinkCreated);
        assert_links_to(&tmp_path, "new", "src");
    }

    #[test]
    fn copy_link_dest_is_a_broken_link() {
        let tmp_dir = TempDir::new("test-rusync-fsops").expect("failed to create temp dir");
        let tmp_path = tmp_dir.path();
        let src_link = setup_copy_test(tmp_path);

        let broken_link = &tmp_path.join("broken");
        create_link("no-such-file", &broken_link);
        let outcome = sync_src_link(&tmp_path, &src_link, "broken");
        assert_eq!(outcome.expect(""), SyncOutcome::SymlinkUpdated);
        assert_links_to(&tmp_path, "broken", "src");
    }

    #[test]
    fn copy_link_dest_doest_not_point_to_correct_location() {
        let tmp_dir = TempDir::new("test-rusync-fsops").expect("failed to create temp dir");
        let tmp_path = tmp_dir.path();
        let src_link = setup_copy_test(tmp_path);

        let old_dest = &tmp_path.join("old");
        create_file(&old_dest);
        let existing_link = tmp_path.join("existing_link");
        create_link("old", &existing_link);
        let outcome = sync_src_link(&tmp_path, &src_link, "existing_link");
        assert_eq!(outcome.expect(""), SyncOutcome::SymlinkUpdated);
        assert_links_to(&tmp_path, "existing_link", "src");
    }

    #[test]
    fn copy_link_dest_is_a_regular_file() {
        let tmp_dir = TempDir::new("test-rusync-fsops").expect("failed to create temp dir");
        let tmp_path = tmp_dir.path();
        let src_link = setup_copy_test(tmp_path);

        let existing_file = tmp_path.join("existing");
        create_file(&existing_file);
        let outcome = sync_src_link(&tmp_path, &src_link, "existing");
        assert!(outcome.is_err());
        let err = outcome.err().unwrap();
        let desc = err.description();
        assert!(desc.contains("existing"));
    }

}
