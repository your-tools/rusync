extern crate colored;

use std::fs;
use std::fs::DirEntry;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use entry::Entry;
use fsops;
use fsops::SyncOutcome;
use fsops::SyncOutcome::*;

pub struct Stats {
    pub total: u64,
    pub up_to_date: u64,
    pub copied: u64,
    pub symlink_created: u64,
    pub symlink_updated: u64,
}

impl Stats {
    pub fn new() -> Stats {
        Stats {
            total: 0,
            up_to_date: 0,
            copied: 0,
            symlink_created: 0,
            symlink_updated: 0,
        }
    }

    pub fn add_outcome(&mut self, outcome: &SyncOutcome) {
        self.total += 1;
        match outcome {
            FileCopied => self.copied += 1,
            UpToDate => self.up_to_date += 1,
            SymlinkUpdated => self.symlink_updated += 1,
            SymlinkCreated => self.symlink_created += 1,
        }
    }
}

pub enum Progress {
    DoneSyncing(SyncOutcome),
    Syncing {
        description: String,
        size: u64,
        done: u64,
    },
}

struct SyncWorker {
    input: Receiver<Entry>,
    output: Sender<Progress>,
    source: PathBuf,
    destination: PathBuf,
    preserve_permissions: bool,
}

impl SyncWorker {
    fn new(
        source: &Path,
        destination: &Path,
        input: Receiver<Entry>,
        output: Sender<Progress>,
    ) -> SyncWorker {
        SyncWorker {
            source: source.to_path_buf(),
            preserve_permissions: true,
            destination: destination.to_path_buf(),
            input,
            output,
        }
    }

    fn start(self) {
        for entry in self.input.iter() {
            // FIXME: handle errors
            let sync_outcome = self.sync(&entry).unwrap();
            let progress = Progress::DoneSyncing(sync_outcome);
            self.output.send(progress).unwrap();
        }
    }

    fn sync(&self, src_entry: &Entry) -> io::Result<(SyncOutcome)> {
        let rel_path = fsops::get_rel_path(&src_entry.path(), &self.source)?;
        let parent_rel_path = rel_path.parent();
        if parent_rel_path.is_none() {
            return Err(fsops::to_io_error(&format!(
                "Could not get parent path of {}",
                rel_path.to_string_lossy()
            )));
        }
        let parent_rel_path = parent_rel_path.unwrap();
        let to_create = self.destination.join(parent_rel_path);
        fs::create_dir_all(to_create)?;

        let desc = rel_path.to_string_lossy();

        let dest_path = self.destination.join(&rel_path);
        let dest_entry = Entry::new(&desc, &dest_path);
        let outcome = fsops::sync_entries(&self.output, &src_entry, &dest_entry)?;
        if self.preserve_permissions {
            fsops::copy_permissions(&src_entry, &dest_entry)?;
        }
        Ok(outcome)
    }
}

struct WalkWorker {
    output: Sender<Entry>,
    source: PathBuf,
}

impl WalkWorker {
    fn new(source: &Path, output: Sender<Entry>) -> WalkWorker {
        WalkWorker {
            output,
            source: source.to_path_buf(),
        }
    }

    fn walk_dir(&self, subdir: &Path) -> io::Result<()> {
        for entry in fs::read_dir(subdir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let subdir = path;
                self.walk_dir(&subdir)?;
            } else {
                self.process_file(&entry)?;
            }
        }
        Ok(())
    }

    fn process_file(&self, entry: &DirEntry) -> io::Result<()> {
        let rel_path = fsops::get_rel_path(&entry.path(), &self.source)?;
        let parent_rel_path = rel_path.parent();
        if parent_rel_path.is_none() {
            return Err(fsops::to_io_error(&format!(
                "Could not get parent path of {}",
                rel_path.to_string_lossy()
            )));
        }

        let desc = rel_path.to_string_lossy();
        let src_entry = Entry::new(&desc, &entry.path());
        self.output.send(src_entry).unwrap();
        Ok(())
    }

    fn start(&self) {
        let top_dir = &self.source.clone();
        let outcome = self.walk_dir(top_dir);
        if outcome.is_err() {
            // Send err to output
        }
    }
}

struct ProgressWorker {
    input: Receiver<Progress>,
}

impl ProgressWorker {
    fn new(input: Receiver<Progress>) -> ProgressWorker {
        ProgressWorker { input }
    }

    fn start(self) -> Stats {
        let mut stats = Stats::new();
        for progress in self.input.iter() {
            match progress {
                Progress::DoneSyncing(x) => stats.add_outcome(&x),
                Progress::Syncing {
                    description,
                    done,
                    size,
                } => println!("{} {} {}", description, done, size),
            }
        }
        stats
    }
}

pub struct Pipeline {
    source: PathBuf,
    destination: PathBuf,
}

impl Pipeline {
    pub fn new(source: &Path, destination: &Path) -> Pipeline {
        Pipeline {
            source: source.to_path_buf(),
            destination: destination.to_path_buf(),
        }
    }

    pub fn run(self) -> Result<Stats, String> {
        let (walker_output, syncer_input) = channel::<Entry>();
        let (syncer_output, progress_input) = channel::<Progress>();
        let walk_worker = WalkWorker::new(&self.source, walker_output);
        let sync_worker =
            SyncWorker::new(&self.source, &self.destination, syncer_input, syncer_output);
        let progress_worker = ProgressWorker::new(progress_input);

        let walker_thread = thread::spawn(move || walk_worker.start());
        let syncer_thread = thread::spawn(move || sync_worker.start());
        let progress_thread = thread::spawn(|| progress_worker.start());

        let walker_outcome = walker_thread.join();
        let syncer_outcome = syncer_thread.join();
        let progress_outcome = progress_thread.join();

        if walker_outcome.is_err() {
            return Err(format!("Could not join walker thread"));
        }

        if syncer_outcome.is_err() {
            return Err(format!("Could not join syncer thread"));
        }

        if progress_outcome.is_err() {
            return Err(format!("Could not join progress thread"));
        }
        Ok(progress_outcome.unwrap())
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use self::tempdir::TempDir;
    use super::*;
    use std::process::Command;

    fn setup_test(tmp_path: &Path) -> (PathBuf, PathBuf) {
        let src_path = tmp_path.join("src");
        let dest_path = tmp_path.join("dest");
        let status = Command::new("cp")
            .args(&["-r", "tests/data", &src_path.to_string_lossy()])
            .status()
            .expect("Failed to execute process");
        assert!(status.success());
        (src_path, dest_path)
    }

    #[test]
    fn test_pipeline() {
        let tmp_dir = TempDir::new("test-rusync").expect("failed to create temp dir");
        let (src_path, dest_path) = setup_test(&tmp_dir.path());
        let pipeline = Pipeline::new(&src_path, &dest_path);
        let stats = pipeline.run();
        assert!(stats.is_ok());
    }

}
