use std::io;
use std::io::Write;
use std::sync::mpsc::Receiver;

use progress::Progress;
use sync::Stats;
use terminal_size::{terminal_size, Height, Width};

pub struct ProgressWorker {
    input: Receiver<Progress>,
}

fn get_terminal_width() -> u16 {
    let size = terminal_size();
    if let Some((Width(width), Height(_))) = size {
        return width;
    }
    // We're likely not a tty here, so this is a good enough
    // default:
    80
}

fn erase_line() {
    let line_width = get_terminal_width();
    let line = vec![32 as u8; line_width as usize];
    print!("{}\r", String::from_utf8(line).unwrap());
}

fn print_progress(percent: usize, index: u64, num_files: u64, current_file: &mut String) {
    let index_width = index.to_string().len();
    let num_files_width = num_files.to_string().len();
    let widgets_width = index_width + num_files_width;
    let separators_width = 6; // percent (padded at 3), space, slash, space
    let line_width = get_terminal_width();
    let remaining_size = line_width - (widgets_width as u16) - separators_width - 1;
    current_file.truncate(remaining_size as usize);
    print!(
        "{number:>width$}% {}/{} {}\r",
        index,
        num_files,
        current_file,
        number = percent,
        width = 3
    );
    let _ = io::stdout().flush();
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
                Progress::Todo { num_files } => stats.num_files = num_files,
                Progress::StartSync(x) => {
                    current_file = x;
                    index += 1;
                }
                Progress::DoneSyncing(x) => {
                    erase_line();
                    stats.add_outcome(&x);
                    file_done = 0;
                }
                Progress::Syncing { done, size, .. } => {
                    file_done += done;
                    let percent = ((file_done * 100) as usize) / size;
                    print_progress(percent, index, stats.num_files, &mut current_file);
                }
            }
        }
        stats
    }
}
