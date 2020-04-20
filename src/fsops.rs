extern crate pathdiff;

use std;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
#[cfg(unix)]
use std::os::unix;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;

use filetime::FileTime;

use crate::entry::Entry;
use crate::error::Error;
use crate::progress::ProgressMessage;

const BUFFER_SIZE: usize = 100 * 1024;

#[derive(PartialEq, Debug)]
pub enum SyncOutcome {
    UpToDate,
    FileCopied,
    SymlinkUpdated,
    SymlinkCreated,
}

pub fn get_rel_path(a: &Path, b: &Path) -> Result<PathBuf, Error> {
    let rel_path = pathdiff::diff_paths(&a, &b).ok_or_else(|| Error::new(""))?;
    Ok(rel_path)
}

fn is_more_recent_than(src: &Entry, dest: &Entry) -> bool {
    if !dest.exists() {
        return true;
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

    src_precise > dest_precise
}

#[cfg(unix)]
pub fn copy_permissions(src: &Entry, dest: &Entry) -> Result<(), Error> {
    let src_meta = &src.metadata();
    // is_link should not be none because we should have been able to
    // read its metadata way back in WalkWorker
    let is_link = src
        .is_link()
        .unwrap_or_else(|| panic!("is_link was None for {:#?}", src));
    if is_link {
        return Ok(());
    }
    // The only way for src_meta to be None is if src is a broken symlink
    // and we checked that right above:
    let src_meta = &src_meta.unwrap_or_else(|| panic!("src_meta was None for {:#?}", src));
    let permissions = src_meta.permissions();
    let dest_file = File::open(dest.path()).map_err(|e| {
        Error::new(&format!(
            "could not open {} while copying permissions: {}",
            dest.description(),
            e
        ))
    })?;
    dest_file.set_permissions(permissions).map_err(|e| {
        Error::new(&format!(
            "Could not set permissions for {}: {}",
            dest.description(),
            e
        ))
    })?;
    Ok(())
}

fn copy_link(src: &Entry, dest: &Entry) -> Result<SyncOutcome, Error> {
    let src_target = std::fs::read_link(src.path())
        .map_err(|e| Error::new(&format!("Could not read link {}: {}", src.description(), e)))?;
    let is_link = dest.is_link();
    let outcome;
    match is_link {
        Some(true) => {
            let dest_target = std::fs::read_link(dest.path()).map_err(|e| {
                Error::new(&format!(
                    "Could not read link {}: {}",
                    dest.description(),
                    e
                ))
            })?;
            if dest_target != src_target {
                fs::remove_file(dest.path()).map_err(|e| {
                    Error::new(&format!(
                        "Could not remove {} while updating link: {}",
                        dest.description(),
                        e
                    ))
                })?;
                outcome = SyncOutcome::SymlinkUpdated;
            } else {
                return Ok(SyncOutcome::UpToDate);
            }
        }
        Some(false) => {
            // Never safe to delete
            return Err(Error::new(&format!(
                "Refusing to replace existing path {} by symlink",
                dest.description()
            )));
        }
        None => {
            // OK, dest does not exist
            outcome = SyncOutcome::SymlinkCreated;
        }
    }
    #[cfg(unix)]
    {
        let symlink_result = unix::fs::symlink(&src_target, &dest.path());
        match symlink_result {
            Err(e) => Err(Error::new(&format!(
                "Could not create link from {} to {}: {}",
                dest.description(),
                src.description(),
                e
            ))),
            Ok(_) => Ok(outcome),
        }
    }

    #[cfg(windows)]
    {
        Ok(outcome)
    }
}

pub fn copy_entry(
    progress_sender: &mpsc::Sender<ProgressMessage>,
    src: &Entry,
    dest: &Entry,
) -> Result<SyncOutcome, Error> {
    let src_path = src.path();
    let mut src_file = File::open(src_path).map_err(|e| {
        Error::new(&format!(
            "Could not open {} for reading: {}",
            src.description(),
            e
        ))
    })?;
    let src_meta = src.metadata().expect("src_meta should not be None");
    let src_size = src_meta.len();
    let dest_path = dest.path();
    let mut dest_file = File::create(dest_path).map_err(|e| {
        Error::new(&format!(
            "Could not open {} for writing: {}",
            dest.description(),
            e
        ))
    })?;
    let mut buffer = vec![0; BUFFER_SIZE];
    loop {
        let num_read = src_file.read(&mut buffer).map_err(|e| {
            Error::new(&format!("Could not read from {}: {}", src.description(), e))
        })?;
        if num_read == 0 {
            break;
        }
        dest_file.write_all(&buffer[0..num_read]).map_err(|e| {
            Error::new(&format!("Could not write to {}: {}", dest.description(), e))
        })?;
        let progress = ProgressMessage::Syncing {
            description: src.description().clone(),
            size: src_size as usize,
            done: num_read,
        };
        let _ = progress_sender.send(progress);
    }
    Ok(SyncOutcome::FileCopied)
}

fn has_different_size(src: &Entry, dest: &Entry) -> bool {
    let src_meta = src.metadata().expect("src_meta should not be None");
    let dest_meta = dest.metadata();
    match dest_meta {
        None => true,
        Some(dest_meta) => (dest_meta.len() != src_meta.len()),
    }
}

pub fn sync_entries(
    progress_sender: &mpsc::Sender<ProgressMessage>,
    src: &Entry,
    dest: &Entry,
) -> Result<SyncOutcome, Error> {
    let _ = progress_sender.send(ProgressMessage::StartSync(src.description().to_string()));
    let is_link = src.is_link().expect("src.is_link should not be None");
    if is_link {
        return copy_link(&src, &dest);
    }
    let different_size = has_different_size(&src, &dest);
    let more_recent = is_more_recent_than(&src, &dest);
    // TODO: check if files really are different ?
    if more_recent || different_size {
        return copy_entry(&progress_sender, &src, &dest);
    }
    Ok(SyncOutcome::UpToDate)
}

#[cfg(test)]
mod tests {

    use super::*;
    extern crate tempdir;
    use self::tempdir::TempDir;

    use std;
    #[cfg(unix)]
    use std::os::unix;
    use std::path::Path;
    use std::path::PathBuf;

    fn create_file(path: &Path) -> Result<(), std::io::Error> {
        std::fs::write(path, "")
    }

    #[cfg(unix)]
    fn create_link(src: &str, dest: &Path) -> Result<(), std::io::Error> {
        unix::fs::symlink(&src, &dest)
    }

    #[cfg(unix)]
    fn assert_links_to(tmp_path: &Path, src: &str, dest: &str) {
        let src_path = tmp_path.join(src);
        let link = std::fs::read_link(src_path)
            .unwrap_or_else(|e| panic!("could not read link {:?}: {}", src, e));
        assert_eq!(link.to_string_lossy(), dest);
    }

    #[cfg(unix)]
    fn setup_copy_test(tmp_path: &Path) -> Result<PathBuf, std::io::Error> {
        let src = &tmp_path.join("src");
        create_file(&src)?;
        let src_link = &tmp_path.join("src_link");
        create_link("src", &src_link)?;
        Ok(src_link.to_path_buf())
    }

    #[cfg(unix)]
    fn sync_src_link(tmp_path: &Path, src_link: &Path, dest: &str) -> Result<SyncOutcome, Error> {
        let src_entry = Entry::new("src", &src_link);
        let dest_path = &tmp_path.join(&dest);
        let dest_entry = Entry::new(&dest, dest_path);
        copy_link(&src_entry, &dest_entry)
    }

    #[cfg(unix)]
    #[test]
    fn copy_link_dest_does_not_exist() -> Result<(), std::io::Error> {
        let tmp_dir = TempDir::new("test-rusync-fsops")?;
        let tmp_path = tmp_dir.path();
        let src_link = setup_copy_test(tmp_path)?;

        let outcome = sync_src_link(&tmp_path, &src_link, "new");
        assert_eq!(outcome.unwrap(), SyncOutcome::SymlinkCreated);
        assert_links_to(&tmp_path, "new", "src");
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn copy_link_dest_is_a_broken_link() -> Result<(), std::io::Error> {
        let tmp_dir = TempDir::new("test-rusync-fsops")?;
        let tmp_path = tmp_dir.path();
        let src_link = setup_copy_test(tmp_path)?;

        let broken_link = &tmp_path.join("broken");
        create_link("no-such-file", &broken_link)?;
        let outcome = sync_src_link(&tmp_path, &src_link, "broken");
        assert_eq!(outcome.unwrap(), SyncOutcome::SymlinkUpdated);
        assert_links_to(&tmp_path, "broken", "src");
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn copy_link_dest_doest_not_point_to_correct_location() -> Result<(), std::io::Error> {
        let tmp_dir = TempDir::new("test-rusync-fsops")?;
        let tmp_path = tmp_dir.path();
        let src_link = setup_copy_test(tmp_path)?;

        let old_dest = &tmp_path.join("old");
        create_file(&old_dest)?;
        let existing_link = tmp_path.join("existing_link");
        create_link("old", &existing_link)?;
        let outcome = sync_src_link(&tmp_path, &src_link, "existing_link");
        assert_eq!(outcome.unwrap(), SyncOutcome::SymlinkUpdated);
        assert_links_to(&tmp_path, "existing_link", "src");
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn copy_link_dest_is_a_regular_file() -> Result<(), std::io::Error> {
        let tmp_dir = TempDir::new("test-rusync-fsops")?;
        let tmp_path = tmp_dir.path();
        let src_link = setup_copy_test(tmp_path)?;

        let existing_file = tmp_path.join("existing");
        create_file(&existing_file)?;
        let outcome = sync_src_link(&tmp_path, &src_link, "existing");
        assert!(outcome.is_err());
        let err = outcome.err().unwrap();
        let desc = err.to_string();
        assert!(desc.contains("existing"));
        Ok(())
    }
}
