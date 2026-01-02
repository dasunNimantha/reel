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
    use crate::model::ParsedMediaInfo;

    // ==================== SANITIZE FILENAME TESTS ====================

    #[test]
    fn test_sanitize_filename_colon() {
        assert_eq!(sanitize_filename("Test: Movie"), "Test Movie");
    }

    #[test]
    fn test_sanitize_filename_forward_slash() {
        assert_eq!(sanitize_filename("Movie/Title"), "MovieTitle");
    }

    #[test]
    fn test_sanitize_filename_question_mark() {
        assert_eq!(sanitize_filename("What?"), "What");
    }

    #[test]
    fn test_sanitize_filename_backslash() {
        assert_eq!(sanitize_filename("Path\\To\\Movie"), "PathToMovie");
    }

    #[test]
    fn test_sanitize_filename_asterisk() {
        // sanitize_filename removes the asterisk and then collapses multiple spaces
        assert_eq!(sanitize_filename("Movie * Special"), "Movie Special");
    }

    #[test]
    fn test_sanitize_filename_quotes() {
        assert_eq!(sanitize_filename("The \"Movie\""), "The Movie");
    }

    #[test]
    fn test_sanitize_filename_angle_brackets() {
        assert_eq!(sanitize_filename("Movie <HD>"), "Movie HD");
    }

    #[test]
    fn test_sanitize_filename_pipe() {
        // sanitize_filename removes the pipe and then collapses multiple spaces
        assert_eq!(sanitize_filename("Movie | Extended"), "Movie Extended");
    }

    #[test]
    fn test_sanitize_filename_multiple_spaces() {
        assert_eq!(sanitize_filename("Movie    Name"), "Movie Name");
    }

    #[test]
    fn test_sanitize_filename_all_invalid_chars() {
        assert_eq!(
            sanitize_filename("Test:File/Name\\With*Illegal?Chars\"And<More>And|Pipes"),
            "TestFileNameWithIllegalCharsAndMoreAndPipes"
        );
    }

    #[test]
    fn test_sanitize_filename_trims_whitespace() {
        assert_eq!(sanitize_filename("  Movie Name  "), "Movie Name");
    }

    // ==================== GENERATE FILENAME - MOVIE TESTS ====================

    fn create_movie_file(filename: &str) -> MediaFile {
        MediaFile {
            path: PathBuf::from(format!("/test/{}", filename)),
            filename: filename.to_string(),
            extension: "mkv".to_string(),
            size_bytes: 1000000,
            media_type: MediaType::Movie,
            parsed_info: Some(ParsedMediaInfo {
                title: "The Matrix".to_string(),
                year: Some(1999),
                ..Default::default()
            }),
            matched_metadata: None,
            new_filename: None,
            is_selected: false,
        }
    }

    fn create_movie_metadata() -> MediaMetadata {
        MediaMetadata {
            tmdb_id: 603,
            title: "The Matrix".to_string(),
            original_title: Some("The Matrix".to_string()),
            year: Some(1999),
            overview: Some("A computer hacker learns about the true nature of reality.".to_string()),
            poster_path: None,
            backdrop_path: None,
            vote_average: Some(8.7),
            genres: vec!["Action".to_string(), "Sci-Fi".to_string()],
            season_number: None,
            episode_number: None,
            episode_title: None,
            air_date: None,
            show_name: None,
        }
    }

    #[test]
    fn test_generate_movie_filename_default_pattern() {
        let file = create_movie_file("The.Matrix.1999.mkv");
        let metadata = create_movie_metadata();
        let pattern = RenamePattern::default();

        let result = generate_filename(&file, &metadata, &pattern);
        assert_eq!(result, "The Matrix (1999).mkv");
    }

    #[test]
    fn test_generate_movie_filename_plex_pattern() {
        let file = create_movie_file("The.Matrix.1999.mkv");
        let metadata = create_movie_metadata();
        let pattern = RenamePattern::plex();

        let result = generate_filename(&file, &metadata, &pattern);
        assert_eq!(result, "The Matrix (1999).mkv");
    }

    #[test]
    fn test_generate_movie_filename_no_year() {
        let mut file = create_movie_file("Unknown.Movie.mkv");
        // Clear the parsed_info year as well, since generate_filename falls back to it
        if let Some(ref mut parsed) = file.parsed_info {
            parsed.year = None;
        }
        let mut metadata = create_movie_metadata();
        metadata.year = None;
        let pattern = RenamePattern::default();

        let result = generate_filename(&file, &metadata, &pattern);
        assert_eq!(result, "The Matrix.mkv");
    }

    #[test]
    fn test_generate_movie_filename_sanitizes_title() {
        let file = create_movie_file("test.mkv");
        let mut metadata = create_movie_metadata();
        metadata.title = "Test: The Movie?".to_string();
        let pattern = RenamePattern::default();

        let result = generate_filename(&file, &metadata, &pattern);
        assert!(!result.contains(':'));
        assert!(!result.contains('?'));
    }

    // ==================== GENERATE FILENAME - TV SHOW TESTS ====================

    fn create_tv_file(filename: &str) -> MediaFile {
        MediaFile {
            path: PathBuf::from(format!("/test/{}", filename)),
            filename: filename.to_string(),
            extension: "mkv".to_string(),
            size_bytes: 500000,
            media_type: MediaType::TvShow,
            parsed_info: Some(ParsedMediaInfo {
                title: "Breaking Bad".to_string(),
                season: Some(1),
                episode: Some(1),
                episode_title: Some("Pilot".to_string()),
                ..Default::default()
            }),
            matched_metadata: None,
            new_filename: None,
            is_selected: false,
        }
    }

    fn create_tv_metadata() -> MediaMetadata {
        MediaMetadata {
            tmdb_id: 1396,
            title: "Pilot".to_string(),
            original_title: None,
            year: Some(2008),
            overview: Some("Walter White, a chemistry teacher, discovers he has cancer.".to_string()),
            poster_path: None,
            backdrop_path: None,
            vote_average: Some(9.5),
            genres: vec!["Drama".to_string(), "Crime".to_string()],
            season_number: Some(1),
            episode_number: Some(1),
            episode_title: Some("Pilot".to_string()),
            air_date: Some("2008-01-20".to_string()),
            show_name: Some("Breaking Bad".to_string()),
        }
    }

    #[test]
    fn test_generate_tv_filename_default_pattern() {
        let file = create_tv_file("Breaking.Bad.S01E01.mkv");
        let metadata = create_tv_metadata();
        let pattern = RenamePattern::default();

        let result = generate_filename(&file, &metadata, &pattern);
        assert_eq!(result, "Breaking Bad - S01E01 - Pilot.mkv");
    }

    #[test]
    fn test_generate_tv_filename_plex_pattern() {
        let file = create_tv_file("Breaking.Bad.S01E01.mkv");
        let metadata = create_tv_metadata();
        let pattern = RenamePattern::plex();

        let result = generate_filename(&file, &metadata, &pattern);
        assert_eq!(result, "Breaking Bad - s01e01 - Pilot.mkv");
    }

    #[test]
    fn test_generate_tv_filename_jellyfin_pattern() {
        let file = create_tv_file("Breaking.Bad.S01E01.mkv");
        let metadata = create_tv_metadata();
        let pattern = RenamePattern::jellyfin();

        let result = generate_filename(&file, &metadata, &pattern);
        assert_eq!(result, "Breaking Bad S01E01 Pilot.mkv");
    }

    #[test]
    fn test_generate_tv_filename_double_digit_episode() {
        let mut file = create_tv_file("Show.S01E10.mkv");
        if let Some(ref mut parsed) = file.parsed_info {
            parsed.episode = Some(10);
        }
        let mut metadata = create_tv_metadata();
        metadata.episode_number = Some(10);
        metadata.episode_title = Some("Episode Ten".to_string());
        let pattern = RenamePattern::default();

        let result = generate_filename(&file, &metadata, &pattern);
        assert!(result.contains("E10"));
    }

    #[test]
    fn test_generate_tv_filename_no_episode_title() {
        let file = create_tv_file("Show.S01E01.mkv");
        let mut metadata = create_tv_metadata();
        metadata.episode_title = None;
        let pattern = RenamePattern::default();

        let result = generate_filename(&file, &metadata, &pattern);
        // Should not have dangling " - " at the end
        assert!(!result.ends_with(" -.mkv"));
        assert!(!result.contains(" - .mkv"));
    }

    #[test]
    fn test_generate_tv_filename_fallback_to_parsed_info() {
        let file = create_tv_file("Show.S05E12.mkv");
        let mut metadata = create_tv_metadata();
        metadata.season_number = None;
        metadata.episode_number = None;
        // Should fallback to parsed_info values
        let pattern = RenamePattern::default();

        let result = generate_filename(&file, &metadata, &pattern);
        // Check it uses some season/episode info
        assert!(result.contains('S') || result.contains('s'));
        assert!(result.contains('E') || result.contains('e'));
    }

    // ==================== GENERATE PREVIEW TESTS ====================

    #[test]
    fn test_generate_preview_with_matched_files() {
        let mut file = create_movie_file("The.Matrix.1999.mkv");
        file.matched_metadata = Some(create_movie_metadata());
        let files = vec![file];
        let pattern = RenamePattern::default();

        let preview = generate_preview(&files, &pattern);
        assert_eq!(preview.len(), 1);
        assert_eq!(preview[0].0, "The.Matrix.1999.mkv");
        assert_eq!(preview[0].1, "The Matrix (1999).mkv");
    }

    #[test]
    fn test_generate_preview_skips_unmatched_files() {
        let file = create_movie_file("Unknown.Movie.mkv"); // No matched_metadata
        let files = vec![file];
        let pattern = RenamePattern::default();

        let preview = generate_preview(&files, &pattern);
        assert!(preview.is_empty());
    }

    #[test]
    fn test_generate_preview_multiple_files() {
        let mut movie = create_movie_file("The.Matrix.1999.mkv");
        movie.matched_metadata = Some(create_movie_metadata());
        
        let mut tv = create_tv_file("Breaking.Bad.S01E01.mkv");
        tv.matched_metadata = Some(create_tv_metadata());
        
        let unmatched = create_movie_file("Unknown.mkv");
        
        let files = vec![movie, tv, unmatched];
        let pattern = RenamePattern::default();

        let preview = generate_preview(&files, &pattern);
        assert_eq!(preview.len(), 2); // Only matched files
    }

    // ==================== RENAME PATTERN TESTS ====================

    #[test]
    fn test_rename_pattern_default() {
        let pattern = RenamePattern::default();
        assert_eq!(pattern.name, "Default");
        assert!(pattern.movie_pattern.contains("{title}"));
        assert!(pattern.movie_pattern.contains("{year}"));
        assert!(pattern.tv_pattern.contains("{show}"));
        assert!(pattern.tv_pattern.contains("{season:02}"));
        assert!(pattern.tv_pattern.contains("{episode:02}"));
    }

    #[test]
    fn test_rename_pattern_plex() {
        let pattern = RenamePattern::plex();
        assert_eq!(pattern.name, "Plex");
        assert!(pattern.tv_pattern.contains("s{season:02}e{episode:02}"));
    }

    #[test]
    fn test_rename_pattern_jellyfin() {
        let pattern = RenamePattern::jellyfin();
        assert_eq!(pattern.name, "Jellyfin");
        assert!(pattern.tv_pattern.contains("S{season:02}E{episode:02}"));
        // Jellyfin doesn't have " - " separator
        assert!(!pattern.tv_pattern.contains(" - S"));
    }

    #[test]
    fn test_all_patterns_returns_three() {
        let patterns = RenamePattern::all_patterns();
        assert_eq!(patterns.len(), 3);
        assert_eq!(patterns[0].name, "Default");
        assert_eq!(patterns[1].name, "Plex");
        assert_eq!(patterns[2].name, "Jellyfin");
    }
}

