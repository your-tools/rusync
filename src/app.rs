extern crate pathdiff;
extern crate colored;
extern crate filetime;

use std::fs;
use std::fs::File;
use std::fs::DirEntry;
use std::path::Path;
use std::path::PathBuf;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;

use colored::Colorize;
use self::filetime::FileTime;

const BUFFER_SIZE: usize = 100 * 1024;

pub struct Stats {
    pub total: u64,
    pub copied: u64,
    pub up_to_date: u64
}

struct Syncer {
    pub source: PathBuf,
    pub destination: PathBuf,
    checked: u64,
    copied: u64,
}

impl Syncer {

    fn new(source: &Path, destination: &Path) -> Syncer {
        Syncer {
            source: source.to_path_buf(),
            destination: destination.to_path_buf(),
            checked: 0,
            copied: 0
        }
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

    fn get_rel_path(&self, entry: &Path) -> io::Result<PathBuf> {
        let rel_path = pathdiff::diff_paths(&entry, &self.source);
        if rel_path.is_none() {
            Err(to_io_error(format!("Could not get relative path from {} to {}",
                        &self.source.to_string_lossy(),
                        &entry.to_string_lossy())))
        } else {
            Ok(rel_path.unwrap())
        }
    }

    fn sync_file(&mut self, entry: &DirEntry) -> io::Result<()> {
        let rel_path = self.get_rel_path(&entry.path())?;

        let parent_rel_path = rel_path.parent();
        if let None = parent_rel_path {
            return Err(to_io_error(
                format!("Could not get parent path of {}", rel_path.to_string_lossy())
            ))
        }
        let parent_rel_path = parent_rel_path.unwrap();
        let to_create = self.destination.join(parent_rel_path);
        fs::create_dir_all(to_create)?;

        let dest_path = self.destination.join(&rel_path);
        let src_path = entry.path();
        self.copy_if_more_recent(&src_path, &dest_path)
    }

    fn copy_if_more_recent(&mut self, src: &Path, dest: &Path)  -> io::Result<()>{
        let more_recent = more_recent_than(&src, &dest)?;
        let rel_src = self.get_rel_path(&src)?;
        self.checked += 1;
        if more_recent {
            println!("{} {}", "->".color("blue"), rel_src.to_string_lossy().bold());
            self.copied += 1;
            return copy(&src, &dest);
        }
        Ok(())
    }

    fn sync(&mut self) -> io::Result<(Stats)> {
        let top_dir = &self.source.clone();
        self.walk_dir(top_dir)?;
        let up_to_date = self.checked - self.copied;
        Ok(Stats{copied: self.copied, total: self.checked, up_to_date: up_to_date})
    }
}


fn to_io_error(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, message)
}

fn more_recent_than(src: &Path, dest: &Path) -> io::Result<bool> {
    if !dest.exists() {
        return Ok(true);
    }

    let src_meta = fs::metadata(src)?;
    let dest_meta = fs::metadata(dest)?;

    let src_mtime = FileTime::from_last_modification_time(&src_meta);
    let dest_mtime = FileTime::from_last_modification_time(&dest_meta);

    let src_precise = src_mtime.seconds() * 1000 * 1000 * 1000 + src_mtime.nanoseconds() as u64;
    let dest_precise = dest_mtime.seconds() * 1000 * 1000 * 1000 + dest_mtime.nanoseconds() as u64;

    Ok(src_precise > dest_precise)
}

fn copy(source: &Path, destination: &Path) -> io::Result<()> {
    let src_path = File::open(source)?;
    let src_meta = fs::metadata(source)?;
    let src_size = src_meta.len();
    let mut done = 0;
    let mut buf_reader = BufReader::new(src_path);
    let dest_path = File::create(destination)?;
    let mut buf_writer = BufWriter::new(dest_path);
    let mut buffer = vec![0; BUFFER_SIZE];
    loop {
        let num_read = buf_reader.read(&mut buffer)?;
        if num_read == 0 {
            break;
        }
        done += num_read;
        let percent = ((done * 100) as u64) / src_size;
        print!("{number:>width$}%\r", number=percent, width=3);
        let _ = io::stdout().flush();
        buf_writer.write(&buffer[0..num_read])?;
    }
    Ok(())
}

pub fn sync(source: &Path, destination: &Path) -> io::Result<Stats> {
    println!("{} Syncing from {} to {} …",
             "::".color("blue"),
             source.to_string_lossy().bold(),
             destination.to_string_lossy().bold()
    );

    let mut syncer = Syncer::new(&source, &destination);
    syncer.sync()
}
