use fsops::SyncOutcome;

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
