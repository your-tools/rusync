use std::fs;
use std::fs::DirEntry;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

use entry::Entry;
use fsops;

pub struct WalkWorker {
    output: Sender<Entry>,
    source: PathBuf,
}

impl WalkWorker {
    pub fn new(source: &Path, output: Sender<Entry>) -> WalkWorker {
        WalkWorker {
            output,
            source: source.to_path_buf(),
        }
    }

    fn walk_dir(&self, subdir: &Path) -> io::Result<()> {
        for entry in fs::read_dir(subdir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let subdir = path;
                self.walk_dir(&subdir)?;
            } else {
                self.process_file(&entry)?;
            }
        }
        Ok(())
    }

    fn process_file(&self, entry: &DirEntry) -> io::Result<()> {
        let rel_path = fsops::get_rel_path(&entry.path(), &self.source)?;
        let parent_rel_path = rel_path.parent();
        if parent_rel_path.is_none() {
            return Err(fsops::to_io_error(&format!(
                "Could not get parent path of {}",
                rel_path.to_string_lossy()
            )));
        }

        let desc = rel_path.to_string_lossy();
        let src_entry = Entry::new(&desc, &entry.path());
        self.output.send(src_entry).unwrap();
        Ok(())
    }

    pub fn start(&self) {
        let top_dir = &self.source.clone();
        let outcome = self.walk_dir(top_dir);
        if outcome.is_err() {
            // Send err to output
        }
    }
}
