use crate::model::{is_video_file, MediaFile};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Scan a directory recursively for video files
pub async fn scan_directory(path: PathBuf) -> Result<Vec<MediaFile>, String> {
    tokio::task::spawn_blocking(move || scan_directory_sync(&path))
        .await
        .map_err(|e| format!("Task error: {}", e))?
}

fn scan_directory_sync(path: &PathBuf) -> Result<Vec<MediaFile>, String> {
    let mut files = Vec::new();

    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let entry_path = entry.path();
        if entry_path.is_file() {
            if let Some(ext) = entry_path.extension() {
                if is_video_file(&ext.to_string_lossy()) {
                    files.push(MediaFile::new(entry_path.to_path_buf()));
                }
            }
        }
    }

    // Sort by filename
    files.sort_by(|a, b| a.filename.to_lowercase().cmp(&b.filename.to_lowercase()));

    Ok(files)
}

/// Scan multiple files (from file picker)
pub async fn scan_files(paths: Vec<PathBuf>) -> Result<Vec<MediaFile>, String> {
    let mut files = Vec::new();

    for path in paths {
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if is_video_file(&ext.to_string_lossy()) {
                    files.push(MediaFile::new(path));
                }
            }
        }
    }

    // Sort by filename
    files.sort_by(|a, b| a.filename.to_lowercase().cmp(&b.filename.to_lowercase()));

    Ok(files)
}
