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
        for progress in self.input.iter() {
            match progress {
                Progress::DoneSyncing(x) => stats.add_outcome(&x),
                Progress::Syncing { done, size, .. } => {
                    let percent = ((done * 100) as usize) / size;
                    print!("{number:>width$}%\r", number = percent, width = 3);
                    let _ = io::stdout().flush();
                }
            }
        }
        stats
    }
}
