use crate::fsops::SyncOutcome;
use crate::sync::Stats;

#[doc(hidden)]
pub enum ProgressMessage {
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
    SyncError {
        entry: String,
        details: String,
    },
}

pub struct Progress {
    /// Name of the file being transferred
    pub current_file: String,
    /// Number of bytes transfered for the current file
    pub file_done: usize,
    /// Size of the current file (in bytes)
    pub file_size: usize,
    /// Number of bytes transfered since the start
    pub total_done: usize,
    /// Estimated total size of the transfer (this may change during transfer)
    pub total_size: usize,
    /// Index of the current file in the list of all files to transfer
    pub index: usize,
    /// Total number of files to transfer
    pub num_files: usize,
    /// Estimated time remaining for the transfer, in seconds
    pub eta: usize,
}

/// Trait for implementing rusync progress details
pub trait ProgressInfo {
    /// A new transfer has begun from the `source` directory to the `destination`
    /// directory
    #[allow(unused_variables)]
    fn start(&mut self, source: &str, destination: &str) {}

    /// A new file named `name` is being transfered
    #[allow(unused_variables)]
    fn new_file(&mut self, name: &str) {}

    /// The file transfer is done
    #[allow(unused_variables)]
    fn done_syncing(&mut self) {}

    /// Callback for the detailed progress
    #[allow(unused_variables)]
    fn progress(&mut self, progress: &Progress) {}

    /// The transfer between `source` and `destination` is done. Details
    /// of the transfer in the Stats struct
    #[allow(unused_variables)]
    fn end(&mut self, stats: &Stats) {}

    /// The entry could not be synced
    #[allow(unused_variables)]
    fn error(&mut self, entry: &str, details: &str) {}
}
