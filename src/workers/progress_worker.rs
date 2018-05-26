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
        let mut file_done = 0;
        let mut current_file = String::from("");
        let mut index = 0;
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
                    current_file = x;
                    index += 1;
                }
                Progress::DoneSyncing(x) => {
                    stats.add_outcome(&x);
                    file_done = 0;
                }
                Progress::Syncing { done, size, .. } => {
                    file_done += done;
                    let percent = ((file_done * 100) as usize) / size;
                    current_file.truncate(50);
                    print!(
                        "{number:>width$}% {}/{} {}\r",
                        index,
                        stats.num_files,
                        current_file,
                        number = percent,
                        width = 3
                    );
                    let _ = io::stdout().flush();
                }
            }
        }
        stats
    }
}
