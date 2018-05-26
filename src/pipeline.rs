use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

use sync::Stats;

#[derive(Debug)]
struct Entry {
    src: String,
    size: u32,
}

#[derive(Debug)]
struct Progress {
    src: String,
    size: u32,
    done: u32,
}

struct SyncWorker {
    input: Receiver<Entry>,
    output: Sender<Progress>,
}

impl SyncWorker {
    fn new(input: Receiver<Entry>, output: Sender<Progress>) -> SyncWorker {
        SyncWorker { input, output }
    }

    fn start(self) {
        for entry in self.input.iter() {
            let progress = Progress {
                src: entry.src.clone(),
                size: entry.size,
                done: 0,
            };
            self.output.send(progress).unwrap();
            let progress = Progress {
                src: entry.src.clone(),
                size: entry.size,
                done: entry.size / 2,
            };
            thread::sleep(Duration::from_secs(1));
            self.output.send(progress).unwrap();
            let progress = Progress {
                src: entry.src.clone(),
                size: entry.size,
                done: entry.size,
            };
            thread::sleep(Duration::from_secs(1));
            self.output.send(progress).unwrap();
        }
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

    fn start(self) {
        let foo_entry = Entry {
            src: String::from("foo.txt"),
            size: 100,
        };
        let bar_entry = Entry {
            src: String::from("bar.txt"),
            size: 100,
        };
        self.output.send(foo_entry).expect("could not send foo");
        thread::sleep(Duration::from_millis(500));
        self.output.send(bar_entry).expect("could not send bar");
    }
}

struct ProgressWorker {
    input: Receiver<Progress>,
}

impl ProgressWorker {
    fn new(input: Receiver<Progress>) -> ProgressWorker {
        ProgressWorker { input }
    }

    fn start(self) {
        for progress in self.input.iter() {
            println!("{} {}/{}", progress.src, progress.done, progress.size);
        }
    }
}

struct Pipeline {
    source: PathBuf,
    destination: PathBuf,
}

impl Pipeline {
    fn new(source: &Path, destination: &Path) -> Pipeline {
        Pipeline {
            source: source.to_path_buf(),
            destination: source.to_path_buf(),
        }
    }

    pub fn run(self) -> Result<Stats, String> {
        let (walker_output, syncer_input) = channel::<Entry>();
        let (syncer_output, progress_input) = channel::<Progress>();
        let walk_worker = WalkWorker::new(&self.source, walker_output);
        let sync_worker = SyncWorker::new(syncer_input, syncer_output);
        let progress_worker = ProgressWorker::new(progress_input);

        let walker_thread = thread::spawn(|| {
            walk_worker.start();
        });

        let syncer_thread = thread::spawn(|| {
            sync_worker.start();
        });

        let progress_thread = thread::spawn(|| {
            progress_worker.start();
        });

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
        Ok(Stats::new())
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
        pipeline.run();
    }

}
