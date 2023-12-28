use std::fs;
use std::option::Option;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Entry {
    description: String,
    path: PathBuf,
    metadata: Option<fs::Metadata>,
    exists: bool,
    is_link: Option<bool>,
}

impl Entry {
    pub fn new(description: &str, entry_path: &Path) -> Entry {
        let mut metadata = fs::metadata(entry_path).ok();
        let is_link;
        let symlink_metadata = fs::symlink_metadata(entry_path);
        if let Ok(data) = symlink_metadata {
            is_link = Some(data.file_type().is_symlink());
            metadata = Some(data);
        } else {
            is_link = None;
        }

        Entry {
            description: String::from(description),
            metadata,
            path: entry_path.to_path_buf(),
            exists: entry_path.exists(),
            is_link,
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

    pub fn is_link(&self) -> Option<bool> {
        self.is_link
    }
}

#[cfg(test)]
mod tests {

    use super::Entry;
    use super::Path;

    #[test]
    fn new_entry_with_non_existing_path() {
        let path = Path::new("/path/to/nosuch.txt");
        let entry = Entry::new("nosuch", path);

        assert!(!entry.exists());
        assert!(entry.metadata.is_none());
    }

    #[test]
    fn new_entry_with_existing_path() {
        let path = Path::new(file!());
        let entry = Entry::new("entry.rs", path);

        assert!(entry.exists());
        assert!(entry.metadata.is_some());
        let is_link = entry.is_link();
        assert!(is_link.is_some());
        assert!(!is_link.unwrap());
    }
}
