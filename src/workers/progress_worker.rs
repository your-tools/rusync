use std::io;
use std::io::Write;
use std::sync::mpsc::Receiver;

use progress::Progress;
use sync::Stats;

pub struct ProgressWorker {
    input: Receiver<Progress>,
}

impl ProgressWorker {
    pub fn new(input: Receiver<Progress>) -> ProgressWorker {
        ProgressWorker { input }
    }

    pub fn start(self) -> Stats {
        let mut stats = Stats::new();
        let mut total_done = 0;
        let mut file_done = 0;
        for progress in self.input.iter() {
            match progress {
                Progress::Todo {
                    num_files,
                    total_size,
                } => {
                    stats.num_files = num_files;
                    stats.total_size = total_size;
                }
                Progress::DoneSyncing(x) => {
                    stats.add_outcome(&x);
                    file_done = 0;
                }
                Progress::Syncing { done, size, .. } => {
                    file_done += done;
                    let percent = ((file_done * 100) as usize) / size;
                    // TODO: two lines
                    println!("{number:>width$}%", number = percent, width = 3);
                    println!("File {} on {}", stats.num_synced, stats.num_files);
                    println!("{} on {}", total_done, stats.total_size);
                    let _ = io::stdout().flush();
                    total_done += done;
                }
            }
        }
        stats
    }
}
