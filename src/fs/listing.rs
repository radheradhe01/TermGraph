//! Directory listing utilities

use std::path::Path;
use anyhow::Result;

/// Get directory size (recursive)
pub fn get_directory_size(path: &Path) -> Result<u64> {
    let mut size = 0u64;
    
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            
            if metadata.is_dir() {
                size += get_directory_size(&entry.path())?;
            } else {
                size += metadata.len();
            }
        }
    }
    
    Ok(size)
}

/// Format file size for display
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
