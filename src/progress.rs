use fsops::SyncOutcome;

pub enum Progress {
    DoneSyncing(SyncOutcome),
    StartSync(String),
    Todo {
        num_files: u64,
    },
    Syncing {
        description: String,
        size: usize,
        done: usize,
    },
}
