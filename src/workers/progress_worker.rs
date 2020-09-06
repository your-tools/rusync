use std::sync::mpsc::Receiver;
use std::time::Instant;

use crate::progress::{Progress, ProgressInfo, ProgressMessage};
use crate::sync::Stats;

pub struct ProgressWorker {
    input: Receiver<ProgressMessage>,
    progress_info: Box<dyn ProgressInfo + Send>,
}

impl ProgressWorker {
    pub fn new(
        input: Receiver<ProgressMessage>,
        progress_info: Box<dyn ProgressInfo + Send>,
    ) -> ProgressWorker {
        ProgressWorker {
            input,
            progress_info,
        }
    }

    pub fn start(mut self) -> Stats {
        let mut stats = Stats::new();
        let mut file_done = 0;
        let mut current_file = String::from("");
        let mut index = 0;
        let mut total_done = 0;
        let now = Instant::now();
        stats.start();
        for progress in self.input.iter() {
            match progress {
                ProgressMessage::Todo {
                    num_files,
                    total_size,
                } => {
                    stats.num_files = num_files;
                    stats.total_size = total_size;
                }
                ProgressMessage::StartSync(x) => {
                    self.progress_info.new_file(&x);
                    current_file = x;
                    index += 1;
                }
                ProgressMessage::DoneSyncing(x) => {
                    self.progress_info.done_syncing();
                    stats.add_outcome(&x);
                    file_done = 0;
                }
                ProgressMessage::SyncError { entry, details } => {
                    self.progress_info.error(&entry, &details);
                    stats.add_error();
                }
                ProgressMessage::Syncing { done, size, .. } => {
                    file_done += done;
                    total_done += done;
                    let elapsed = now.elapsed().as_secs() as usize;
                    let eta = ((elapsed * stats.total_size) / total_done) - elapsed;
                    let detailed_progress = Progress {
                        file_done,
                        file_size: size,
                        total_done,
                        total_size: stats.total_size,
                        index,
                        num_files: stats.num_files as usize,
                        current_file: current_file.clone(),
                        eta,
                    };
                    self.progress_info.progress(&detailed_progress);
                }
            }
        }
        stats.stop();
        self.progress_info.end(&stats);
        stats
    }
}
