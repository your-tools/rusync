use std::option::Option;
use std::path::Path;
use std::path::PathBuf;
use std::fs;

pub struct Entry {
    description: String,
    path: PathBuf,
    metadata: Option<fs::Metadata>,
    exists: bool,
}

impl Entry {
    pub fn new(description: String, entry_path: &Path) -> Entry {
        let metadata = fs::metadata(entry_path).ok();
        Entry {
            description: description,
            metadata: metadata,
            path: entry_path.to_path_buf(),
            exists: entry_path.exists(),
        }
    }

    pub fn description(&self) -> &String { &self.description }
    pub fn path(&self) -> &PathBuf { &self.path }
    pub fn metadata(&self) -> Option<&fs::Metadata> { self.metadata.as_ref() }
    pub fn exists(&self) -> bool { self.exists }
}

#[cfg(test)]
mod tests {

use super::Path;
use super::Entry;

#[test]
fn new_entry_with_non_existing_path() {
    let path = Path::new("/path/to/nosuch.txt");
    let entry = Entry::new(String::from("nosuch"), &path);

    assert!(!entry.exists());
    assert!(entry.metadata.is_none());
}

#[test]
fn new_entry_with_existing_path() {
    let path = Path::new(file!());
    let entry = Entry::new(String::from("entry.rs"), &path);

    assert!(entry.exists());
    assert!(entry.metadata.is_some());
}

}
