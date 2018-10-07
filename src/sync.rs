extern crate colored;

use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;

use entry::Entry;
use fsops;
use fsops::SyncOutcome::*;
use progress::{Progress, ProgressInfo};
use workers::ProgressWorker;
use workers::SyncWorker;
use workers::WalkWorker;

#[derive(Default)]
pub struct Stats {
    pub num_files: u64,
    pub total_size: usize,

    pub num_synced: u64,
    pub up_to_date: u64,
    pub copied: u64,

    pub symlink_created: u64,
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

#[derive(Copy, Clone)]
pub struct SyncOptions {
    pub preserve_permissions: bool,
}

impl SyncOptions {
    fn new() -> SyncOptions {
        SyncOptions {
            preserve_permissions: true,
        }
    }
}

pub struct Syncer {
    source: PathBuf,
    destination: PathBuf,
    options: SyncOptions,
    progress_info: Box<ProgressInfo + Send>,
}

impl Syncer {
    pub fn new(
        source: &Path,
        destination: &Path,
        progress_info: Box<ProgressInfo + Send>,
    ) -> Syncer {
        Syncer {
            source: source.to_path_buf(),
            destination: destination.to_path_buf(),
            options: SyncOptions::new(),
            progress_info,
        }
    }

    pub fn preserve_permissions(&mut self, preserve_permissions: bool) {
        self.options.preserve_permissions = preserve_permissions;
    }

    pub fn sync(self) -> Result<Stats, String> {
        let (walker_entry_output, syncer_input) = channel::<Entry>();
        let (walker_stats_output, progress_input) = channel::<Progress>();
        let progress_output = walker_stats_output.clone();

        let walk_worker = WalkWorker::new(&self.source, walker_entry_output, walker_stats_output);
        let sync_worker = SyncWorker::new(
            &self.source,
            &self.destination,
            syncer_input,
            progress_output,
        );
        let progress_worker = ProgressWorker::new(progress_input, self.progress_info);
        let options = self.options.clone();

        let walker_thread = thread::spawn(move || walk_worker.start());
        let syncer_thread = thread::spawn(move || sync_worker.start(options));
        let progress_thread = thread::spawn(|| progress_worker.start());

        let walker_outcome = walker_thread.join();
        let syncer_outcome = syncer_thread.join();
        let progress_outcome = progress_thread.join();

        if walker_outcome.is_err() {
            return Err("Could not join walker thread".to_string());
        }

        if syncer_outcome.is_err() {
            return Err("Could not join syncer thread".to_string());
        }
        let syncer_result = syncer_outcome.unwrap();
        if syncer_result.is_err() {
            return Err(format!("{}", syncer_result.err().unwrap()));
        }

        if progress_outcome.is_err() {
            return Err("Could not join progress thread".to_string());
        }
        Ok(progress_outcome.unwrap())
    }
}
