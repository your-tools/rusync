use std::fs;
use std::fs::DirEntry;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

use crate::entry::Entry;
use crate::error::Error;
use crate::fsops;
use crate::progress::ProgressMessage;

pub struct WalkWorker {
    entry_output: Sender<Entry>,
    progress_output: Sender<ProgressMessage>,
    source: PathBuf,
}

impl WalkWorker {
    pub fn new(
        source: &Path,
        entry_output: Sender<Entry>,
        progress_output: Sender<ProgressMessage>,
    ) -> WalkWorker {
        WalkWorker {
            entry_output,
            progress_output,
            source: source.to_path_buf(),
        }
    }

    fn walk(&self) -> Result<(), Error> {
        let mut num_files = 0;
        let mut total_size = 0;
        let mut subdirs: Vec<PathBuf> = vec![self.source.to_path_buf()];
        while !subdirs.is_empty() {
            // We just checked that subdirs is *not* empty, so calling pop() is safe
            let subdir = subdirs.pop().unwrap();
            let entries = fs::read_dir(&subdir).map_err(|e| {
                Error::new(&format!(
                    "While walking source, could not read directory {:?}: {}",
                    subdir, e
                ))
            })?;
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        subdirs.push(path);
                    } else {
                        let meta = self.process_file(&entry)?;
                        num_files += 1;
                        total_size += meta.len();
                        let sent = self.progress_output.send(ProgressMessage::Todo {
                            num_files,
                            total_size: total_size as usize,
                        });
                        if sent.is_err() {
                            return Err(Error::new("stats output chan is closed"));
                        }
                    }
                } else {
                    return Err(Error::new(&format!(
                        "While walking {:?} source dir, could not read entry: {}",
                        subdir,
                        entry.unwrap_err()
                    )));
                }
            }
        }
        Ok(())
    }

    fn process_file(&self, entry: &DirEntry) -> Result<fs::Metadata, Error> {
        let rel_path = fsops::get_rel_path(&entry.path(), &self.source)?;
        let desc = rel_path.to_string_lossy();
        let src_entry = Entry::new(&desc, &entry.path());
        let metadata = src_entry.metadata().ok_or_else(|| {
            Error::new(&format!("Could not read metadata from {:?}", entry.path()))
        })?;
        self.entry_output.send(src_entry.clone()).map_err(|e| {
            Error::new(&format!(
                "When walking source dir: could not send to progress worker: {}",
                e
            ))
        })?;
        Ok(metadata.clone())
    }

    pub fn start(&self) {
        let outcome = &self.walk();
        if outcome.is_err() {
            // Send err to output
        }
    }
}
