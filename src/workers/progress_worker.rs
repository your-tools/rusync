use std::sync::mpsc::Receiver;
use std::time::Instant;

use progress::{DetailedProgress, Progress, ProgressInfo};
use sync::Stats;

pub struct ProgressWorker {
    input: Receiver<Progress>,
    progress_info: Box<ProgressInfo + Send>,
}

impl ProgressWorker {
    pub fn new(
        input: Receiver<Progress>,
        progress_info: Box<ProgressInfo + Send>,
    ) -> ProgressWorker {
        ProgressWorker {
            input,
            progress_info,
        }
    }

    pub fn start(self) -> Stats {
        let mut stats = Stats::new();
        let mut file_done = 0;
        let mut current_file = String::from("");
        let mut index = 0;
        let mut total_done = 0;
        let now = Instant::now();
        for progress in self.input.iter() {
            match progress {
                Progress::Todo {
                    num_files,
                    total_size,
                } => {
                    stats.num_files = num_files;
                    stats.total_size = total_size;
                }
                Progress::StartSync(x) => {
                    self.progress_info.new_file(&x);
                    current_file = x;
                    index += 1;
                }
                Progress::DoneSyncing(x) => {
                    self.progress_info.done_syncing();
                    stats.add_outcome(&x);
                    file_done = 0;
                }
                Progress::Syncing { done, size, .. } => {
                    file_done += done;
                    total_done += done;
                    let elapsed = now.elapsed().as_secs() as usize;
                    let eta = ((elapsed * stats.total_size) / total_done) - elapsed;
                    let detailed_progress = DetailedProgress {
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
        self.progress_info.end(&stats);
        stats
    }
}
