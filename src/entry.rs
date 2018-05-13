use std::fs;
use std::option::Option;
use std::path::Path;
use std::path::PathBuf;

pub struct Entry {
    description: String,
    path: PathBuf,
    metadata: Option<fs::Metadata>,
    exists: bool,
}

impl Entry {
    pub fn new(description: &str, entry_path: &Path) -> Entry {
        let metadata = fs::metadata(entry_path).ok();
        Entry {
            description: String::from(description),
            metadata,
            path: entry_path.to_path_buf(),
            exists: entry_path.exists(),
        }
    }

    pub fn description(&self) -> &String {
        &self.description
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn metadata(&self) -> Option<&fs::Metadata> {
        self.metadata.as_ref()
    }
    pub fn exists(&self) -> bool {
        self.exists
    }
}

#[cfg(test)]
mod tests {

    use super::Entry;
    use super::Path;

    #[test]
    fn new_entry_with_non_existing_path() {
        let path = Path::new("/path/to/nosuch.txt");
        let entry = Entry::new("nosuch", &path);

        assert!(!entry.exists());
        assert!(entry.metadata.is_none());
    }

    #[test]
    fn new_entry_with_existing_path() {
        let path = Path::new(file!());
        let entry = Entry::new("entry.rs", &path);

        assert!(entry.exists());
        assert!(entry.metadata.is_some());
    }

}
