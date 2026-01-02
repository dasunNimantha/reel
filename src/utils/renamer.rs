use crate::model::{MediaFile, MediaMetadata, MediaType, RenamePattern};
use std::path::PathBuf;

/// Generate a new filename based on metadata and pattern
pub fn generate_filename(
    file: &MediaFile,
    metadata: &MediaMetadata,
    pattern: &RenamePattern,
) -> String {
    let template = match file.media_type {
        MediaType::Movie => &pattern.movie_pattern,
        MediaType::TvShow => &pattern.tv_pattern,
        MediaType::Unknown => &pattern.movie_pattern,
    };

    let mut result = template.clone();

    // Replace placeholders
    result = result.replace("{title}", &sanitize_filename(&metadata.title));
    
    // Year - prefer metadata, fallback to parsed info
    let year = metadata.year.or_else(|| {
        file.parsed_info.as_ref().and_then(|p| p.year)
    });
    if let Some(y) = year {
        result = result.replace("{year}", &y.to_string());
    } else {
        result = result.replace("{year}", "");
        result = result.replace(" ()", ""); // Clean up empty parentheses
    }

    // TV show specific - use show_name from metadata or title
    if let Some(show) = &metadata.show_name {
        result = result.replace("{show}", &sanitize_filename(show));
    } else {
        result = result.replace("{show}", &sanitize_filename(&metadata.title));
    }

    // Season - prefer metadata, fallback to parsed info
    let season = metadata.season_number.or_else(|| {
        file.parsed_info.as_ref().and_then(|p| p.season)
    });
    if let Some(s) = season {
        result = result.replace("{season:02}", &format!("{:02}", s));
        result = result.replace("{season}", &s.to_string());
    } else {
        // Remove unreplaced placeholders
        result = result.replace("{season:02}", "");
        result = result.replace("{season}", "");
    }

    // Episode - prefer metadata, fallback to parsed info
    let episode = metadata.episode_number.or_else(|| {
        file.parsed_info.as_ref().and_then(|p| p.episode)
    });
    if let Some(e) = episode {
        result = result.replace("{episode:02}", &format!("{:02}", e));
        result = result.replace("{episode}", &e.to_string());
    } else {
        result = result.replace("{episode:02}", "");
        result = result.replace("{episode}", "");
    }

    // Episode title - prefer metadata, fallback to parsed info
    let ep_title = metadata.episode_title.clone().or_else(|| {
        file.parsed_info.as_ref().and_then(|p| p.episode_title.clone())
    });
    if let Some(title) = ep_title {
        result = result.replace("{episode_title}", &sanitize_filename(&title));
    } else {
        result = result.replace("{episode_title}", "");
    }

    // Clean up the result
    result = result.trim().to_string();
    
    // Clean up orphan separators and dashes
    result = result.replace(" - .", "."); // Remove trailing " - " before extension
    result = result.replace(" -.", ".");
    result = result.replace("  - ", " ");
    result = result.replace(" -  ", " ");
    
    // Remove duplicate spaces
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }
    
    // Remove trailing dash or hyphen before extension
    if result.ends_with(" -") {
        result = result[..result.len()-2].to_string();
    }

    // Add extension
    format!("{}.{}", result.trim(), file.extension)
}

/// Sanitize a string for use in a filename
fn sanitize_filename(s: &str) -> String {
    let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    let mut result = s.to_string();
    
    for c in invalid_chars {
        result = result.replace(c, "");
    }
    
    // Replace multiple spaces with single space
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }
    
    result.trim().to_string()
}

/// Execute the rename operation
pub async fn rename_files(
    files: Vec<(PathBuf, String)>, // (old_path, new_filename)
    output_dir: Option<PathBuf>,
) -> Result<Vec<(String, String)>, String> {
    let mut results = Vec::new();

    for (old_path, new_filename) in files {
        let new_path = if let Some(ref dir) = output_dir {
            dir.join(&new_filename)
        } else {
            old_path.parent().unwrap_or(&old_path).join(&new_filename)
        };

        // Skip if same path
        if old_path == new_path {
            continue;
        }

        // Check if destination exists
        if new_path.exists() {
            return Err(format!(
                "Destination already exists: {}",
                new_path.display()
            ));
        }

        // Create output directory if needed
        if let Some(parent) = new_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }
        }

        // Rename/move the file
        std::fs::rename(&old_path, &new_path)
            .map_err(|e| format!("Failed to rename {}: {}", old_path.display(), e))?;

        results.push((
            old_path.file_name().unwrap_or_default().to_string_lossy().to_string(),
            new_filename,
        ));
    }

    Ok(results)
}

/// Generate rename preview
pub fn generate_preview(files: &[MediaFile], pattern: &RenamePattern) -> Vec<(String, String)> {
    files
        .iter()
        .filter_map(|f| {
            if let Some(metadata) = &f.matched_metadata {
                let new_name = generate_filename(f, metadata, pattern);
                Some((f.filename.clone(), new_name))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Test: Movie"), "Test Movie");
        assert_eq!(sanitize_filename("Movie/Title"), "MovieTitle");
        assert_eq!(sanitize_filename("What?"), "What");
    }
}

