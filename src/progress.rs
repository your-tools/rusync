use fsops::SyncOutcome;

pub enum Progress {
    DoneSyncing(SyncOutcome),
    Syncing {
        description: String,
        size: usize,
        done: usize,
    },
}
