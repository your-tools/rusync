//! console_info
//!
//! Display transfer progress to the command line

use crate::progress::{Progress, ProgressInfo};
use crate::sync;
use anyhow::{Context, Error};
use colored::Colorize;
use humansize::{file_size_opts as options, FileSize};
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::Path;
use terminal_size::{terminal_size, Width};

#[derive(Debug)]
pub struct ConsoleProgressInfo {
    err_file: Option<std::fs::File>,
}

impl ConsoleProgressInfo {
    pub fn new() -> Self {
        Self { err_file: None }
    }

    pub fn with_error_list_path(error_list_path: &Path) -> Result<Self, Error> {
        let err_file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(error_list_path)
            .with_context(|| {
                format!("Could not open errfile at '{}'", error_list_path.display())
            })?;
        Ok(Self {
            err_file: Some(err_file),
        })
    }
}

impl ProgressInfo for ConsoleProgressInfo {
    fn done_syncing(&mut self) {
        erase_line();
    }

    fn start(&mut self, source: &str, destination: &str) {
        println!(
            "{} Syncing from {} to {} …",
            "::".color("blue"),
            source.bold(),
            destination.bold()
        )
    }

    fn new_file(&mut self, _name: &str) {}

    fn progress(&mut self, progress: &Progress) {
        let eta_str = human_seconds(progress.eta);
        let percent_width = 3;
        let eta_width = eta_str.len();
        let index = progress.index;
        let index_width = index.to_string().len();
        let num_files = progress.num_files;
        let num_files_width = num_files.to_string().len();
        let widgets_width = percent_width + index_width + num_files_width + eta_width;
        let num_separators = 5;
        let line_width = get_terminal_width();
        let file_width = line_width - widgets_width - num_separators - 1;
        let current_file = progress.current_file.clone();
        let current_file = truncate_lossy(&current_file, file_width as usize);
        let current_file = format!(
            "{filename:<pad$}",
            pad = file_width as usize,
            filename = current_file
        );
        let file_percent = (progress.file_done * 100) / progress.file_size;
        print!(
            "{:>3}% {}/{} {} {:<}\r",
            file_percent, index, num_files, current_file, eta_str
        );
        let _ = io::stdout().flush();
    }

    fn error(&mut self, entry: &str, desc: &str) {
        eprintln!("Errror: {}", desc);
        if let Some(err_file) = &mut self.err_file {
            // Ignoring errrors when trying to log errors ...
            let _ = err_file.write(entry.as_bytes());
            let _ = err_file.write(b"\n");
            let _ = err_file.flush();
        }
    }

    fn end(&mut self, stats: &sync::Stats) {
        println!(
            "{} Synced {} files ({} up to date)",
            " ✓".color("green"),
            stats.num_synced,
            stats.up_to_date
        );
        println!(
            "{} files copied, {} symlinks created, {} symlinks updated",
            stats.copied, stats.symlink_created, stats.symlink_updated
        );
        let transfered = stats.total_transfered;
        // We know transfered cannot be negative
        let transfered = transfered.file_size(options::DECIMAL).unwrap();
        let duration = stats.duration();
        // Truncate below 1 second
        let duration = std::time::Duration::from_secs(duration.as_secs());
        let duration = humantime::format_duration(duration);
        println!("{} copied in {}", transfered, duration);
        if stats.errors != 0 {
            eprintln!("{} errors occurred", stats.errors);
        }
    }
}

impl Default for ConsoleProgressInfo {
    fn default() -> Self {
        Self::new()
    }
}

fn get_terminal_width() -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        return w as usize;
    }
    // We're likely not a tty here, so this is a good enough default:
    80
}

fn erase_line() {
    let line_width = get_terminal_width();
    let line = vec![32_u8; line_width];
    // We're calling from_utf8 on a string containing only spaces,
    // so calling unwrap() is safe
    print!("{}\r", String::from_utf8(line).unwrap());
}

fn human_seconds(s: usize) -> String {
    let hours = s / 3600;
    let minutes = (s / 60) % 60;
    let seconds = s % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn truncate_lossy(text: &str, maxsize: usize) -> String {
    // Our goal here is to make sure the text can be written
    // in the terminal without going over the `maxsize` length
    // Our approach is to first convert to bytes, then truncate
    // the vector of bytes, then convert to a lossy string
    // This way we *know* we won't cut at a char boundary
    let mut as_bytes = text.to_string().into_bytes();
    as_bytes.truncate(maxsize);
    String::from_utf8_lossy(&as_bytes).to_string()
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_truncate_string() {
        let new_text = truncate_lossy("ééé", 2);
        assert_eq!(new_text, "é");
    }

    #[test]
    fn test_human_seconds() {
        assert_eq!("00:00:05", human_seconds(5));
        assert_eq!("00:00:42", human_seconds(42));
        assert_eq!("00:03:05", human_seconds(185));
        assert_eq!("02:04:05", human_seconds(7445));
        assert_eq!("200:00:02", human_seconds(720_002));
    }
}
