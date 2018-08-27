use std::io;
use std::io::Write;
use std::sync::mpsc::Receiver;
use std::time::Instant;

use progress::Progress;
use sync::Stats;
use term_size;

pub struct ProgressWorker {
    input: Receiver<Progress>,
}

fn get_terminal_width() -> usize {
    if let Some((w, _)) = term_size::dimensions() {
        return w;
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

fn human_seconds(s: usize) -> String {
    let hours = s / 3600;
    let minutes = (s / 60) % 60;
    let seconds = s % 60;
    return format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
}

fn print_progress(
    percent: usize,
    index: u64,
    num_files: u64,
    current_file: &mut String,
    eta: usize,
) {
    let eta_str = human_seconds(eta);
    let percent_width = 3;
    let eta_width = eta_str.len();
    let index_width = index.to_string().len();
    let num_files_width = num_files.to_string().len();
    let widgets_width = percent_width + index_width + num_files_width + eta_width;
    let num_separators = 5;
    let line_width = get_terminal_width();
    let file_width = line_width - widgets_width - num_separators - 1;
    current_file.truncate(file_width as usize);
    let current_file = format!(
        "{filename:<pad$}",
        pad = file_width as usize,
        filename = current_file
    );
    print!(
        "{:>3}% {}/{} {} {:<}\r",
        percent, index, num_files, current_file, eta_str
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
                    total_done += done;
                    let file_percent = ((file_done * 100) as usize) / size;
                    let elapsed = now.elapsed().as_secs() as usize;
                    let eta = ((elapsed * stats.total_size) / total_done) - elapsed;
                    print_progress(file_percent, index, stats.num_files, &mut current_file, eta);
                }
            }
        }
        stats
    }
}

#[cfg(test)]
mod test {

    use super::human_seconds;

    #[test]
    fn test_human_seconds() {
        assert_eq!("00:00:05", human_seconds(5));
        assert_eq!("00:00:42", human_seconds(42));
        assert_eq!("00:03:05", human_seconds(185));
        assert_eq!("02:04:05", human_seconds(7445));
        assert_eq!("200:00:02", human_seconds(720002));
    }

}
