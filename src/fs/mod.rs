//! Filesystem operations

mod listing;

pub use listing::*;

use anyhow::Result;
use std::path::PathBuf;

/// Represents a file or directory entry
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<std::time::SystemTime>,
}

/// Filesystem state and operations
pub struct FileSystem {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
}

impl FileSystem {
    pub fn new(path: PathBuf) -> Self {
        Self {
            current_path: path,
            entries: Vec::new(),
        }
    }

    /// Load directory contents
    pub fn load_directory(&mut self) -> Result<()> {
        self.entries.clear();
        
        let read_dir = std::fs::read_dir(&self.current_path)?;
        
        for entry in read_dir.flatten() {
            let metadata = entry.metadata()?;
            let name = entry.file_name().to_string_lossy().to_string();
            
            // Skip hidden files for now (can be toggled later)
            if name.starts_with('.') {
                continue;
            }
            
            self.entries.push(FileEntry {
                name,
                path: entry.path(),
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                modified: metadata.modified().ok(),
            });
        }
        
        // Sort: directories first, then by name
        self.entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
        
        Ok(())
    }

    /// Navigate into a directory
    pub fn enter_directory(&mut self, name: &str) -> Result<()> {
        let new_path = self.current_path.join(name);
        if new_path.is_dir() {
            self.current_path = new_path;
            self.load_directory()?;
        }
        Ok(())
    }

    /// Navigate to parent directory
    pub fn go_up(&mut self) -> Result<()> {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.load_directory()?;
        }
        Ok(())
    }

    /// Get entry at index
    pub fn get_selected(&self, index: usize) -> Option<&FileEntry> {
        self.entries.get(index)
    }
}
