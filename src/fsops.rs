extern crate pathdiff;
extern crate colored;
extern crate filetime;

use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::fs;
use std::fs::File;
use std::path::Path;

use self::colored::Colorize;
use self::filetime::FileTime;

const BUFFER_SIZE: usize = 100 * 1024;

pub fn to_io_error(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, message)
}

pub fn more_recent_than(src: &Path, dest: &Path) -> io::Result<bool> {
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

fn copy_perms(path: &Path, metadata: &fs::Metadata) -> io::Result<()> {
    let permissions = metadata.permissions();
    let file = File::create(path)?;
    file.set_permissions(permissions)?;
    Ok(())
}

pub fn copy(source: &Path, destination: &Path) -> io::Result<()> {
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
    // This is allowed to fail, for instance when
    // copying from an ext4 to a fat32 partition
    let copy_outcome = copy_perms(&destination, &src_meta);
    if let Err(err) = copy_outcome {
        println!("{} Failed to preserve permissions for {}: {}",
                 "Warning".yellow(),
                 destination.to_string_lossy().bold(),
                 err
      );
    }
    Ok(())
}
