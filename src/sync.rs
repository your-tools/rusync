extern crate filetime;
extern crate pathdiff;

use std::fs;
use std::fs::DirEntry;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use colored::Colorize;

use entry;
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

pub struct Syncer {
    pub stats: Stats,
    pub source: PathBuf,
    pub destination: PathBuf,
    pub preserve_permissions: bool,
}

fn get_rel_path(a: &Path, b: &Path) -> io::Result<PathBuf> {
    let rel_path = pathdiff::diff_paths(&a, &b);
    if rel_path.is_none() {
        Err(fsops::to_io_error(&format!(
            "Could not get relative path from {} to {}",
            &a.to_string_lossy(),
            &a.to_string_lossy()
        )))
    } else {
        Ok(rel_path.unwrap())
    }
}

impl Syncer {
    pub fn new(source: &Path, destination: &Path) -> Syncer {
        Syncer {
            source: source.to_path_buf(),
            destination: destination.to_path_buf(),
            stats: Stats::new(),
            preserve_permissions: true,
        }
    }

    pub fn stats(self) -> Stats {
        self.stats
    }

    pub fn preserve_permissions(&mut self, preserve_permissions: bool) {
        self.preserve_permissions = preserve_permissions;
    }

    fn walk_dir(&mut self, subdir: &Path) -> io::Result<()> {
        for entry in fs::read_dir(subdir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let subdir = path;
                self.walk_dir(&subdir)?;
            } else {
                self.sync_file(&entry)?;
            }
        }
        Ok(())
    }

    fn sync_file(&mut self, entry: &DirEntry) -> io::Result<()> {
        let rel_path = get_rel_path(&entry.path(), &self.source)?;
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
        let src_entry = entry::Entry::new(&desc, &entry.path());

        let dest_path = self.destination.join(&rel_path);
        let dest_entry = entry::Entry::new(&desc, &dest_path);
        let outcome = fsops::sync_entries(&src_entry, &dest_entry)?;
        self.stats.add_outcome(&outcome);
        if self.preserve_permissions {
            fsops::copy_permissions(&src_entry, &dest_entry)?;
        }
        Ok(())
    }

    pub fn sync(&mut self) -> io::Result<()> {
        if !self.preserve_permissions {
            println!("{} not preserving permissions", "Warning:".yellow());
        }
        let top_dir = &self.source.clone();
        self.walk_dir(top_dir)?;
        Ok(())
    }
}
