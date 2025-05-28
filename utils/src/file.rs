//file structure
use std::path::PathBuf;

pub struct FileEntry {
    pub file_path: PathBuf,
    pub is_dir: bool,
    pub file_content: Option<Vec<u8>>
}