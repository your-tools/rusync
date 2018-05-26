use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

use entry::Entry;
use fsops;
use fsops::SyncOutcome;
use progress::Progress;
use sync::SyncOptions;

pub struct SyncWorker {
    input: Receiver<Entry>,
    output: Sender<Progress>,
    source: PathBuf,
    destination: PathBuf,
}

impl SyncWorker {
    pub fn new(
        source: &Path,
        destination: &Path,
        input: Receiver<Entry>,
        output: Sender<Progress>,
    ) -> SyncWorker {
        SyncWorker {
            source: source.to_path_buf(),
            destination: destination.to_path_buf(),
            input,
            output,
        }
    }

    pub fn start(self, opts: SyncOptions) {
        for entry in self.input.iter() {
            // FIXME: handle errors
            let sync_outcome = self.sync(&entry, opts).unwrap();
            let progress = Progress::DoneSyncing(sync_outcome);
            self.output.send(progress).unwrap();
        }
    }

    fn sync(&self, src_entry: &Entry, opts: SyncOptions) -> io::Result<(SyncOutcome)> {
        let rel_path = fsops::get_rel_path(&src_entry.path(), &self.source)?;
        let parent_rel_path = rel_path.parent();
        if parent_rel_path.is_none() {
            return Err(fsops::to_io_error(&format!(
                "Could not get parent path of {}",
                rel_path.to_string_lossy()
            )));
        }
        let parent_rel_path = parent_rel_path.unwrap();
        let to_create = self.destination.join(parent_rel_path);
        fs::create_dir_all(to_create)?;

        let desc = rel_path.to_string_lossy();

        let dest_path = self.destination.join(&rel_path);
        let dest_entry = Entry::new(&desc, &dest_path);
        let outcome = fsops::sync_entries(&self.output, &src_entry, &dest_entry)?;
        if opts.preserve_permissions {
            fsops::copy_permissions(&src_entry, &dest_entry)?;
        }
        Ok(outcome)
    }
}
