extern crate colored;

use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;

use crate::entry::Entry;
use crate::error::Error;
use crate::fsops;
use crate::fsops::SyncOutcome::*;
use crate::progress::{ProgressInfo, ProgressMessage};
use crate::workers::ProgressWorker;
use crate::workers::SyncWorker;
use crate::workers::WalkWorker;

#[derive(Default)]
pub struct Stats {
    /// Number of files in the source
    pub num_files: u64,
    /// Sum of the sizes of all the files in the source
    pub total_size: usize,

    /// Number of files transfered (should match `num_files`
    /// if no error)
    pub num_synced: u64,
    /// Number of files for which the copy was skipped
    pub up_to_date: u64,
    /// Number of files that were copied
    pub copied: u64,

    /// Number of symlink created in the destination folder
    pub symlink_created: u64,
    /// Number of symlinks updated in the destination folder
    pub symlink_updated: u64,
}

impl Stats {
    pub fn new() -> Stats {
        Stats {
            num_files: 0,
            total_size: 0,

            num_synced: 0,
            up_to_date: 0,
            copied: 0,

            symlink_created: 0,
            symlink_updated: 0,
        }
    }

    #[doc(hidden)]
    pub fn add_outcome(&mut self, outcome: &fsops::SyncOutcome) {
        self.num_synced += 1;
        match outcome {
            FileCopied => self.copied += 1,
            UpToDate => self.up_to_date += 1,
            SymlinkUpdated => self.symlink_updated += 1,
            SymlinkCreated => self.symlink_created += 1,
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct SyncOptions {
    /// Wether to preserve permissions of the source file after the destination is written.
    pub preserve_permissions: bool,
}

impl SyncOptions {
    pub fn new() -> SyncOptions {
        SyncOptions {
            preserve_permissions: true,
        }
    }
}

pub struct Syncer {
    source: PathBuf,
    destination: PathBuf,
    options: SyncOptions,
    progress_info: Box<dyn ProgressInfo + Send>,
}

impl Syncer {
    pub fn new(
        source: &Path,
        destination: &Path,
        options: SyncOptions,
        progress_info: Box<dyn ProgressInfo + Send>,
    ) -> Syncer {
        Syncer {
            source: source.to_path_buf(),
            destination: destination.to_path_buf(),
            progress_info,
            options,
        }
    }

    pub fn sync(self) -> Result<Stats, Error> {
        let (walker_entry_output, syncer_input) = channel::<Entry>();
        let (walker_stats_output, progress_input) = channel::<ProgressMessage>();
        let progress_output = walker_stats_output.clone();

        let walk_worker = WalkWorker::new(&self.source, walker_entry_output, walker_stats_output);
        let sync_worker = SyncWorker::new(
            &self.source,
            &self.destination,
            syncer_input,
            progress_output,
        );
        let progress_worker = ProgressWorker::new(progress_input, self.progress_info);
        let options = self.options;

        let walker_thread = thread::spawn(move || walk_worker.start());
        let syncer_thread = thread::spawn(move || sync_worker.start(options));
        let progress_thread = thread::spawn(|| progress_worker.start());

        walker_thread
            .join()
            .map_err(|e| Error::new(&format!("Could not join walker thread: {:?}", e)))?;

        let syncer_result = syncer_thread
            .join()
            .map_err(|e| Error::new(&format!("Could not join syncer thread: {:?}", e)))?;
        let progress_result = progress_thread
            .join()
            .map_err(|e| Error::new(&format!("Could not join progress thread: {:?}", e)));

        syncer_result?;

        progress_result
    }
}
