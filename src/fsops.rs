use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
#[cfg(unix)]
use std::os::unix;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;

use anyhow::{bail, Context, Error};
use filetime::FileTime;

use crate::entry::Entry;
use crate::progress::ProgressMessage;

const BUFFER_SIZE: usize = 100 * 1024;

#[derive(PartialEq, Eq, Debug)]
pub enum SyncOutcome {
    UpToDate,
    FileCopied { size: u64 },
    SymlinkUpdated,
    SymlinkCreated,
}

pub fn get_rel_path(a: &Path, b: &Path) -> PathBuf {
    pathdiff::diff_paths(a, b)
        .expect("called get_rel_path on two absolute paths '{}' and '{}', a, b")
}

fn is_more_recent_than(src: &Entry, dest: &Entry) -> bool {
    if !dest.exists() {
        return true;
    }

    let src_meta = &src.metadata();
    let dest_meta = &dest.metadata();

    let src_meta = &src_meta.expect("src_meta was None");
    let dest_meta = &dest_meta.expect("dest_meta was None");

    let src_mtime = FileTime::from_last_modification_time(src_meta);
    let dest_mtime = FileTime::from_last_modification_time(dest_meta);

    src_mtime > dest_mtime
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
    let dest_file = File::open(dest.path()).with_context(|| {
        format!(
            "Could not open '{}' while copying permissions",
            dest.description()
        )
    })?;
    dest_file
        .set_permissions(permissions)
        .with_context(|| format!("Could not set permissions for {}", dest.description()))?;
    Ok(())
}

fn copy_link(src: &Entry, dest: &Entry) -> Result<SyncOutcome, Error> {
    let src_target = std::fs::read_link(src.path())
        .with_context(|| format!("While copying source link '{}'", src.description()))?;

    let is_link = dest.is_link();
    let outcome;
    match is_link {
        Some(true) => {
            let dest_target = std::fs::read_link(dest.path())
                .with_context(|| format!("While creating target link: {}", dest.description()))?;
            if dest_target != src_target {
                fs::remove_file(dest.path()).with_context(|| {
                    format!(
                        "Could not remove {} while updating link",
                        dest.description()
                    )
                })?;
                outcome = SyncOutcome::SymlinkUpdated;
            } else {
                return Ok(SyncOutcome::UpToDate);
            }
        }
        Some(false) => {
            // Never safe to delete
            bail!(
                "Refusing to replace existing path {} by symlink",
                dest.description()
            );
        }
        None => {
            // OK, dest does not exist
            outcome = SyncOutcome::SymlinkCreated;
        }
    }
    #[cfg(unix)]
    {
        unix::fs::symlink(&src_target, dest.path()).with_context(|| {
            format!(
                "Could not create link from {} to {}",
                dest.description(),
                src.description()
            )
        })?;
        Ok(outcome)
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
    let mut src_file = File::open(src_path)
        .with_context(|| format!("Could not open '{}' for reading", src.description()))?;
    let src_meta = src.metadata().expect("src_meta should not be None");
    let src_size = src_meta.len();
    let dest_path = dest.path();
    let mut dest_file = File::create(dest_path)
        .with_context(|| format!("Could not open '{}' for writing", dest.description()))?;
    let mut buffer = vec![0; BUFFER_SIZE];
    loop {
        let num_read = src_file
            .read(&mut buffer)
            .with_context(|| format!("Could not read from '{}'", src.description()))?;
        if num_read == 0 {
            break;
        }
        dest_file
            .write_all(&buffer[0..num_read])
            .with_context(|| format!("Could not write to '{}'", dest.description()))?;
        let progress = ProgressMessage::Syncing {
            description: src.description().clone(),
            size: src_size as usize,
            done: num_read,
        };
        let _ = progress_sender.send(progress);
    }
    Ok(SyncOutcome::FileCopied { size: src_size })
}

fn has_different_size(src: &Entry, dest: &Entry) -> bool {
    let src_meta = src.metadata().expect("src_meta should not be None");
    let dest_meta = dest.metadata();
    match dest_meta {
        None => true,
        Some(dest_meta) => dest_meta.len() != src_meta.len(),
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
        return copy_link(src, dest);
    }
    let different_size = has_different_size(src, dest);
    let more_recent = is_more_recent_than(src, dest);
    // TODO: check if files really are different ?
    if more_recent || different_size {
        return copy_entry(progress_sender, src, dest);
    }
    Ok(SyncOutcome::UpToDate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;
    use tempfile::TempDir;

    #[test]
    fn create_file() -> Result<(), std::io::Error> {
        let tmp_dir = TempDir::new()?;
        let tmp_path = tmp_dir.path();
        let src = &tmp_path.join("src.txt");
        let contents = "some contents";
        std::fs::write(src, contents)?;
        let src_entry = Entry::new("src.txt", src);
        let dest = &tmp_path.join("dest.txt");
        let dest_entry = Entry::new("dest.txt", dest);

        let (progress_output, _) = channel::<ProgressMessage>();
        sync_entries(&progress_output, &src_entry, &dest_entry).unwrap();

        let actual = std::fs::read_to_string(dest)?;
        assert_eq!(actual, contents);
        Ok(())
    }

    #[test]
    fn overwrite_file() -> Result<(), std::io::Error> {
        let tmp_dir = TempDir::new()?;
        let tmp_path = tmp_dir.path();
        let src = &tmp_path.join("src.txt");
        let new_contents = "new and shiny";
        std::fs::write(src, new_contents)?;
        let src_entry = Entry::new("src.txt", src);
        let dest = &tmp_path.join("dest.txt");
        let old_contents = "old";
        let dest_entry = Entry::new("dest.txt", dest);
        std::fs::write(dest, old_contents)?;

        let (progress_output, _) = channel::<ProgressMessage>();
        sync_entries(&progress_output, &src_entry, &dest_entry).unwrap();

        let actual = std::fs::read_to_string(dest)?;
        assert_eq!(actual, new_contents);
        Ok(())
    }
}

#[cfg(unix)]
#[cfg(test)]
mod symlink_tests {
    use super::*;
    use std::os::unix;
    use std::path::Path;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_link(src: &str, dest: &Path) -> Result<(), std::io::Error> {
        unix::fs::symlink(src, dest)
    }

    fn create_file(path: &Path) -> Result<(), std::io::Error> {
        std::fs::write(path, "")
    }

    fn assert_links_to(tmp_path: &Path, src: &str, dest: &str) {
        let src_path = tmp_path.join(src);
        let link = std::fs::read_link(src_path)
            .unwrap_or_else(|e| panic!("could not read link {:?}: {}", src, e));
        assert_eq!(link.to_string_lossy(), dest);
    }

    fn setup_sync_link_test(tmp_path: &Path) -> Result<PathBuf, std::io::Error> {
        let src = &tmp_path.join("src");
        create_file(src)?;
        let src_link = &tmp_path.join("src_link");
        create_link("src", src_link)?;
        Ok(src_link.to_path_buf())
    }

    fn sync_src_link(tmp_path: &Path, src_link: &Path, dest: &str) -> Result<SyncOutcome, Error> {
        let src_entry = Entry::new("src", src_link);
        let dest_path = &tmp_path.join(dest);
        let dest_entry = Entry::new(dest, dest_path);
        copy_link(&src_entry, &dest_entry)
    }

    #[test]
    fn create_link_when_dest_does_not_exist() -> Result<(), std::io::Error> {
        let tmp_dir = TempDir::new()?;
        let tmp_path = tmp_dir.path();
        let src_link = setup_sync_link_test(tmp_path)?;

        let outcome = sync_src_link(tmp_path, &src_link, "new");
        assert_eq!(outcome.unwrap(), SyncOutcome::SymlinkCreated);
        assert_links_to(tmp_path, "new", "src");
        Ok(())
    }

    #[test]
    fn create_link_dest_is_a_broken_link() -> Result<(), std::io::Error> {
        let tmp_dir = TempDir::new()?;
        let tmp_path = tmp_dir.path();
        let src_link = setup_sync_link_test(tmp_path)?;

        let broken_link = &tmp_path.join("broken");
        create_link("no-such-file", broken_link)?;
        let outcome = sync_src_link(tmp_path, &src_link, "broken");
        assert_eq!(outcome.unwrap(), SyncOutcome::SymlinkUpdated);
        assert_links_to(tmp_path, "broken", "src");
        Ok(())
    }

    #[test]
    fn create_link_dest_doest_not_point_to_correct_location() -> Result<(), std::io::Error> {
        let tmp_dir = TempDir::new()?;
        let tmp_path = tmp_dir.path();
        let src_link = setup_sync_link_test(tmp_path)?;

        let old_dest = &tmp_path.join("old");
        create_file(old_dest)?;
        let existing_link = tmp_path.join("existing_link");
        create_link("old", &existing_link)?;
        let outcome = sync_src_link(tmp_path, &src_link, "existing_link");
        assert_eq!(outcome.unwrap(), SyncOutcome::SymlinkUpdated);
        assert_links_to(tmp_path, "existing_link", "src");
        Ok(())
    }

    #[test]
    fn create_link_dest_is_a_regular_file() -> Result<(), std::io::Error> {
        let tmp_dir = TempDir::new()?;
        let tmp_path = tmp_dir.path();
        let src_link = setup_sync_link_test(tmp_path)?;

        let existing_file = tmp_path.join("existing");
        create_file(&existing_file)?;
        let outcome = sync_src_link(tmp_path, &src_link, "existing");
        assert!(outcome.is_err());
        let err = outcome.err().unwrap();
        let desc = err.to_string();
        assert!(desc.contains("existing"));
        Ok(())
    }
}
