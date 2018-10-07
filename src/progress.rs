use fsops::SyncOutcome;
use sync::Stats;

/// Sent by the SyncWorker
/// DoneSyncing: when a file has been copied
/// StartSync: when starting the copy of a new file
/// Todo: the total number of files and the total size of
///       data to transfer (this is not constant, and is sent by
///       the walk_worker)
/// Syncing: during progress of *one* file: the total size of the file
///          and the size of the transfered data
pub enum Progress {
    DoneSyncing(SyncOutcome),
    StartSync(String),
    Todo {
        num_files: u64,
        total_size: usize,
    },
    Syncing {
        description: String,
        size: usize,
        done: usize,
    },
}

/// Sent by the ProgressWorker, which compiles the info from
/// the WalkWorker and the SyncWorker, and computes the ETA (estimated
/// number of seconds left for the transfer to finish)
/// Used by the ProgressInfo below
pub struct DetailedProgress {
    pub file_done: usize,
    pub file_size: usize,
    pub total_done: usize,
    pub total_size: usize,
    pub index: usize,
    pub num_files: usize,
    pub current_file: String,
    pub eta: usize,
}

pub trait ProgressInfo {
    fn start(&self, source: &str, destination: &str);
    fn new_file(&self, name: &str);
    fn done_syncing(&self);
    fn progress(&self, progress: &DetailedProgress);
    fn end(&self, stats: &Stats);
}
