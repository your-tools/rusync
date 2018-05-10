extern crate pathdiff;
extern crate colored;
extern crate filetime;

use std;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::fs;
use std::os::unix;
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

fn is_link(path: &Path) -> io::Result<bool> {
    let metadata = std::fs::symlink_metadata(path)?;
    Ok(metadata.file_type().is_symlink())
}

fn copy_perms(path: &Path, metadata: &fs::Metadata) -> io::Result<()> {
    let permissions = metadata.permissions();
    let file = File::create(path)?;
    file.set_permissions(permissions)?;
    Ok(())
}

fn copy_link(name: &String, src_link: &Path, dest: &Path) -> io::Result<()> {
    let src_target =  std::fs::read_link(src_link)?;
    let is_link_outcome = is_link(dest);
    match is_link_outcome {
        Ok(true) => {
            let dest_target = std::fs::read_link(dest)?;
            if dest_target != src_target {
                println!("{} {}", "--".red(), name.bold());
                fs::remove_file(dest)?
            } else {
               return Ok(())
            }

        }
        Ok(false) => {
            // Never safe to delete
            return Err(to_io_error(String::from("Refusing to replace existing path by symlink")));
        }
        Err(_) => {
            // OK, dest does not exist
        }
    }
    println!("{} {} -> {}", "++".blue(), name.bold(), src_target.to_string_lossy());
    unix::fs::symlink(src_target, &dest)
}

pub fn copy(name: &String, source: &Path, destination: &Path) -> io::Result<()> {
    if is_link(source)? {
        return copy_link(&name, &source, destination)
    }
    let src_path = File::open(source)?;
    let src_meta = fs::metadata(source)?;
    let src_size = src_meta.len();
    let mut done = 0;
    let mut buf_reader = BufReader::new(src_path);
    let dest_path = File::create(destination)?;
    let mut buf_writer = BufWriter::new(dest_path);
    let mut buffer = vec![0; BUFFER_SIZE];
    println!("{} {}", "++".green(), name.bold());
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


#[cfg(test)]
mod tests {

extern crate tempdir;
use self::tempdir::TempDir;

use std;
use std::error::Error;
use std::os::unix;
use std::path::Path;
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;

use super::copy_link;


fn create_file(path: &Path) {
    let mut out = File::create(path).expect(&format!("could not open {:?} for writing", path));
    out.write_all(b"").expect("could not write old test");
}

fn create_link(src: &str, dest: &Path) {
    unix::fs::symlink(&src, &dest).expect(
        &format!("could not link {:?} -> {:?}",
                src, dest));
}

fn assert_links_to(src: &str, dest: &Path) {
    let link = std::fs::read_link(dest).expect(
        &format!("could not read link {:?}", src));
    assert_eq!(link.to_string_lossy(), src);
}

fn setup_copy_test(tmp_path: &Path) -> PathBuf {
    let src = &tmp_path.join("src");
    create_file(&src);
    let src_link = &tmp_path.join("src_link");
    create_link("src", &src_link);
    src_link.to_path_buf()
}

#[test]
fn copy_link_dest_does_not_exist() {
    let tmp_dir = TempDir::new("test-rusync-fsops").expect("failed to create temp dir");
    let tmp_path = tmp_dir.path();
    let src_link = setup_copy_test(tmp_path);

    let new_link = &tmp_path.join("new");
    copy_link(&String::from("src_link"), &src_link, &new_link).expect("");
    assert_links_to("src", &new_link);
}

#[test]
fn copy_link_dest_is_a_broken_link() {
    let tmp_dir = TempDir::new("test-rusync-fsops").expect("failed to create temp dir");
    let tmp_path = tmp_dir.path();
    let src_link = setup_copy_test(tmp_path);

    let broken_link = &tmp_path.join("broken");
    create_link("no-such-file", &broken_link);
    copy_link(&String::from("src_link"), &src_link, &broken_link).expect("");
    assert_links_to("src", &broken_link);
}

#[test]
fn copy_link_dest_doest_not_point_to_correct_location() {
    let tmp_dir = TempDir::new("test-rusync-fsops").expect("failed to create temp dir");
    let tmp_path = tmp_dir.path();
    let src_link = setup_copy_test(tmp_path);

    let old_dest = &tmp_path.join("old");
    create_file(&old_dest);
    let existing_link = tmp_path.join("existing");
    create_link("old", &existing_link);
    copy_link(&String::from("src_link"), &src_link, &existing_link).expect("");
    assert_links_to("src", &existing_link);
}

#[test]
fn copy_link_dest_is_a_regular_file() {
    let tmp_dir = TempDir::new("test-rusync-fsops").expect("failed to create temp dir");
    let tmp_path = tmp_dir.path();
    let src_link = setup_copy_test(tmp_path);

    let existing_file = tmp_path.join("existing");
    create_file(&existing_file);
    let outcome = copy_link(&String::from("src_link"), &src_link, &existing_file);
    assert!(outcome.is_err());
    let err = outcome.err().unwrap();
    let desc = err.description();
    assert!(desc.contains("existing"));
}

}
