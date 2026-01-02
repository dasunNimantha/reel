use std::path::PathBuf;

/// Media type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MediaType {
    #[default]
    Unknown,
    Movie,
    TvShow,
}

impl MediaType {
    pub fn display_name(&self) -> &'static str {
        match self {
            MediaType::Unknown => "Unknown",
            MediaType::Movie => "Movie",
            MediaType::TvShow => "TV Show",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            MediaType::Unknown => "?",
            MediaType::Movie => "M",
            MediaType::TvShow => "TV",
        }
    }
}

/// Represents a media file to be processed
#[derive(Debug, Clone)]
pub struct MediaFile {
    pub path: PathBuf,
    pub filename: String,
    pub extension: String,
    pub size_bytes: u64,
    pub media_type: MediaType,
    pub parsed_info: Option<ParsedMediaInfo>,
    pub matched_metadata: Option<MediaMetadata>,
    pub new_filename: Option<String>,
    pub is_selected: bool,
}

impl MediaFile {
    pub fn new(path: PathBuf) -> Self {
        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let extension = path
            .extension()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        Self {
            path,
            filename,
            extension,
            size_bytes,
            media_type: MediaType::Unknown,
            parsed_info: None,
            matched_metadata: None,
            new_filename: None,
            is_selected: false,
        }
    }

    pub fn formatted_size(&self) -> String {
        let bytes = self.size_bytes as f64;
        if bytes >= 1_073_741_824.0 {
            format!("{:.2} GB", bytes / 1_073_741_824.0)
        } else if bytes >= 1_048_576.0 {
            format!("{:.1} MB", bytes / 1_048_576.0)
        } else if bytes >= 1024.0 {
            format!("{:.0} KB", bytes / 1024.0)
        } else {
            format!("{} B", bytes as u64)
        }
    }
}

/// Parsed information extracted from filename
#[derive(Debug, Clone, Default)]
pub struct ParsedMediaInfo {
    pub title: String,
    pub year: Option<u32>,
    pub season: Option<u32>,
    pub episode: Option<u32>,
    pub episode_title: Option<String>,
    pub quality: Option<String>,       // e.g., "1080p", "720p", "4K"
    pub source: Option<String>,        // e.g., "BluRay", "WEB-DL", "HDTV"
    pub codec: Option<String>,         // e.g., "x264", "x265", "HEVC"
    pub audio: Option<String>,         // e.g., "DTS", "AAC", "AC3"
    pub group: Option<String>,         // Release group
}

/// Metadata from online database (TMDB)
#[derive(Debug, Clone, Default)]
pub struct MediaMetadata {
    pub tmdb_id: u64,
    pub title: String,
    pub original_title: Option<String>,
    pub year: Option<u32>,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub vote_average: Option<f32>,
    pub genres: Vec<String>,
    
    // TV Show specific
    pub season_number: Option<u32>,
    pub episode_number: Option<u32>,
    pub episode_title: Option<String>,
    pub air_date: Option<String>,
    pub show_name: Option<String>,
}

/// Search result from TMDB
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub tmdb_id: u64,
    pub title: String,
    pub year: Option<u32>,
    pub media_type: MediaType,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub vote_average: Option<f32>,
}

/// Rename pattern template
#[derive(Debug, Clone)]
pub struct RenamePattern {
    pub name: String,
    pub movie_pattern: String,
    pub tv_pattern: String,
}

impl Default for RenamePattern {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            movie_pattern: "{title} ({year})".to_string(),
            tv_pattern: "{show} - S{season:02}E{episode:02} - {episode_title}".to_string(),
        }
    }
}

impl RenamePattern {
    pub fn plex() -> Self {
        Self {
            name: "Plex".to_string(),
            movie_pattern: "{title} ({year})".to_string(),
            tv_pattern: "{show} - s{season:02}e{episode:02} - {episode_title}".to_string(),
        }
    }

    pub fn jellyfin() -> Self {
        Self {
            name: "Jellyfin".to_string(),
            movie_pattern: "{title} ({year})".to_string(),
            tv_pattern: "{show} S{season:02}E{episode:02} {episode_title}".to_string(),
        }
    }

    pub fn all_patterns() -> Vec<RenamePattern> {
        vec![
            RenamePattern::default(),
            RenamePattern::plex(),
            RenamePattern::jellyfin(),
        ]
    }
}

/// Application state
#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub files: Vec<MediaFile>,
    pub selected_file_index: Option<usize>,
    pub search_query: String,
    pub loading: bool,
    pub status: String,
    
    // Metadata search
    pub search_results: Vec<SearchResult>,
    pub search_loading: bool,
    pub search_input: String,
    
    // TMDB API
    pub tmdb_api_key: String,           // User-entered key (empty if using default)
    pub using_default_key: bool,        // True = use built-in key, input shows placeholder
    pub api_key_valid: Option<bool>,    // None = not verified, Some(true) = valid, Some(false) = invalid
    pub api_key_verifying: bool,
    
    // Rename settings
    pub rename_pattern: RenamePattern,
    pub output_directory: Option<PathBuf>,
    
    // Confirmation modal
    pub show_rename_confirm: bool,
    pub rename_preview: Vec<(String, String)>, // (old_name, new_name)
}

/// Default API key - injected at build time via REEL_TMDB_API_KEY environment variable
pub fn get_default_api_key() -> String {
    // This is read at COMPILE TIME from the environment variable
    // Set REEL_TMDB_API_KEY in your build environment (e.g., GitHub Actions secrets)
    option_env!("REEL_TMDB_API_KEY")
        .unwrap_or("")
        .to_string()
}

/// Check if a default API key is available
pub fn has_default_api_key() -> bool {
    !get_default_api_key().is_empty()
}

impl AppState {
    pub fn new() -> Self {
        let has_default = has_default_api_key();
        Self {
            files: Vec::new(),
            selected_file_index: None,
            search_query: String::new(),
            loading: false,
            status: "Add files or folders to get started".to_string(),
            search_results: Vec::new(),
            search_loading: false,
            search_input: String::new(),
            tmdb_api_key: String::new(), // Empty - actual key is hidden
            using_default_key: has_default,
            api_key_valid: None,
            api_key_verifying: false,
            rename_pattern: RenamePattern::default(),
            output_directory: None,
            show_rename_confirm: false,
            rename_preview: Vec::new(),
        }
    }
    
    /// Get the effective API key (user-entered or default)
    pub fn effective_api_key(&self) -> String {
        if self.using_default_key {
            get_default_api_key()
        } else {
            self.tmdb_api_key.clone()
        }
    }

    pub fn filtered_files(&self) -> Vec<(usize, &MediaFile)> {
        if self.search_query.trim().is_empty() {
            self.files.iter().enumerate().collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.files
                .iter()
                .enumerate()
                .filter(|(_, f)| f.filename.to_lowercase().contains(&query))
                .collect()
        }
    }

    pub fn selected_file(&self) -> Option<&MediaFile> {
        self.selected_file_index
            .and_then(|idx| self.files.get(idx))
    }

    pub fn selected_file_mut(&mut self) -> Option<&mut MediaFile> {
        self.selected_file_index
            .and_then(|idx| self.files.get_mut(idx))
    }

    pub fn files_with_matches(&self) -> usize {
        self.files.iter().filter(|f| f.matched_metadata.is_some()).count()
    }

    pub fn files_ready_for_rename(&self) -> Vec<&MediaFile> {
        self.files
            .iter()
            .filter(|f| f.is_selected && f.new_filename.is_some())
            .collect()
    }
}

/// Supported video file extensions
pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mkv", "mp4", "avi", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "ts", "m2ts",
];

/// Check if a file extension is a video file
pub fn is_video_file(extension: &str) -> bool {
    VIDEO_EXTENSIONS.contains(&extension.to_lowercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ==================== MEDIA TYPE TESTS ====================

    #[test]
    fn test_media_type_display_name() {
        assert_eq!(MediaType::Unknown.display_name(), "Unknown");
        assert_eq!(MediaType::Movie.display_name(), "Movie");
        assert_eq!(MediaType::TvShow.display_name(), "TV Show");
    }

    #[test]
    fn test_media_type_short_name() {
        assert_eq!(MediaType::Unknown.short_name(), "?");
        assert_eq!(MediaType::Movie.short_name(), "M");
        assert_eq!(MediaType::TvShow.short_name(), "TV");
    }

    #[test]
    fn test_media_type_default() {
        let default: MediaType = Default::default();
        assert_eq!(default, MediaType::Unknown);
    }

    #[test]
    fn test_media_type_equality() {
        assert_eq!(MediaType::Movie, MediaType::Movie);
        assert_ne!(MediaType::Movie, MediaType::TvShow);
        assert_ne!(MediaType::TvShow, MediaType::Unknown);
    }

    #[test]
    fn test_media_type_clone() {
        let original = MediaType::TvShow;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    // ==================== VIDEO EXTENSIONS TESTS ====================

    #[test]
    fn test_is_video_file_mkv() {
        assert!(is_video_file("mkv"));
        assert!(is_video_file("MKV"));
        assert!(is_video_file("Mkv"));
    }

    #[test]
    fn test_is_video_file_mp4() {
        assert!(is_video_file("mp4"));
        assert!(is_video_file("MP4"));
    }

    #[test]
    fn test_is_video_file_avi() {
        assert!(is_video_file("avi"));
        assert!(is_video_file("AVI"));
    }

    #[test]
    fn test_is_video_file_mov() {
        assert!(is_video_file("mov"));
        assert!(is_video_file("MOV"));
    }

    #[test]
    fn test_is_video_file_wmv() {
        assert!(is_video_file("wmv"));
    }

    #[test]
    fn test_is_video_file_flv() {
        assert!(is_video_file("flv"));
    }

    #[test]
    fn test_is_video_file_webm() {
        assert!(is_video_file("webm"));
    }

    #[test]
    fn test_is_video_file_m4v() {
        assert!(is_video_file("m4v"));
    }

    #[test]
    fn test_is_video_file_mpeg_formats() {
        assert!(is_video_file("mpg"));
        assert!(is_video_file("mpeg"));
    }

    #[test]
    fn test_is_video_file_ts_formats() {
        assert!(is_video_file("ts"));
        assert!(is_video_file("m2ts"));
    }

    #[test]
    fn test_is_video_file_non_video() {
        assert!(!is_video_file("txt"));
        assert!(!is_video_file("jpg"));
        assert!(!is_video_file("png"));
        assert!(!is_video_file("pdf"));
        assert!(!is_video_file("doc"));
        assert!(!is_video_file("mp3"));
        assert!(!is_video_file("srt"));
        assert!(!is_video_file("sub"));
    }

    #[test]
    fn test_is_video_file_empty() {
        assert!(!is_video_file(""));
    }

    #[test]
    fn test_video_extensions_count() {
        assert_eq!(VIDEO_EXTENSIONS.len(), 12);
    }

    // ==================== MEDIA FILE TESTS ====================

    #[test]
    fn test_media_file_formatted_size_bytes() {
        let mut file = MediaFile::new(PathBuf::from("/test/small.mkv"));
        file.size_bytes = 500;
        assert_eq!(file.formatted_size(), "500 B");
    }

    #[test]
    fn test_media_file_formatted_size_kb() {
        let mut file = MediaFile::new(PathBuf::from("/test/medium.mkv"));
        file.size_bytes = 2048;
        assert_eq!(file.formatted_size(), "2 KB");
    }

    #[test]
    fn test_media_file_formatted_size_mb() {
        let mut file = MediaFile::new(PathBuf::from("/test/large.mkv"));
        file.size_bytes = 52_428_800; // 50 MB
        assert_eq!(file.formatted_size(), "50.0 MB");
    }

    #[test]
    fn test_media_file_formatted_size_gb() {
        let mut file = MediaFile::new(PathBuf::from("/test/huge.mkv"));
        file.size_bytes = 4_294_967_296; // 4 GB
        assert_eq!(file.formatted_size(), "4.00 GB");
    }

    #[test]
    fn test_media_file_extracts_filename() {
        let file = MediaFile::new(PathBuf::from("/path/to/movie.mkv"));
        assert_eq!(file.filename, "movie.mkv");
    }

    #[test]
    fn test_media_file_extracts_extension() {
        let file = MediaFile::new(PathBuf::from("/path/to/movie.mkv"));
        assert_eq!(file.extension, "mkv");
    }

    #[test]
    fn test_media_file_default_values() {
        let file = MediaFile::new(PathBuf::from("/test/file.mkv"));
        assert_eq!(file.media_type, MediaType::Unknown);
        assert!(file.parsed_info.is_none());
        assert!(file.matched_metadata.is_none());
        assert!(file.new_filename.is_none());
        assert!(!file.is_selected);
    }

    // ==================== PARSED MEDIA INFO TESTS ====================

    #[test]
    fn test_parsed_media_info_default() {
        let info: ParsedMediaInfo = Default::default();
        assert!(info.title.is_empty());
        assert!(info.year.is_none());
        assert!(info.season.is_none());
        assert!(info.episode.is_none());
        assert!(info.episode_title.is_none());
        assert!(info.quality.is_none());
        assert!(info.source.is_none());
        assert!(info.codec.is_none());
        assert!(info.audio.is_none());
        assert!(info.group.is_none());
    }

    #[test]
    fn test_parsed_media_info_with_values() {
        let info = ParsedMediaInfo {
            title: "Test Movie".to_string(),
            year: Some(2023),
            season: Some(1),
            episode: Some(5),
            episode_title: Some("Pilot".to_string()),
            quality: Some("1080p".to_string()),
            source: Some("BluRay".to_string()),
            codec: Some("x265".to_string()),
            audio: Some("DTS".to_string()),
            group: Some("SPARKS".to_string()),
        };
        assert_eq!(info.title, "Test Movie");
        assert_eq!(info.year, Some(2023));
        assert_eq!(info.quality, Some("1080p".to_string()));
    }

    // ==================== MEDIA METADATA TESTS ====================

    #[test]
    fn test_media_metadata_default() {
        let metadata: MediaMetadata = Default::default();
        assert_eq!(metadata.tmdb_id, 0);
        assert!(metadata.title.is_empty());
        assert!(metadata.original_title.is_none());
        assert!(metadata.year.is_none());
        assert!(metadata.overview.is_none());
        assert!(metadata.genres.is_empty());
    }

    #[test]
    fn test_media_metadata_movie() {
        let metadata = MediaMetadata {
            tmdb_id: 603,
            title: "The Matrix".to_string(),
            original_title: Some("The Matrix".to_string()),
            year: Some(1999),
            overview: Some("Description".to_string()),
            poster_path: Some("/path.jpg".to_string()),
            backdrop_path: None,
            vote_average: Some(8.7),
            genres: vec!["Action".to_string(), "Sci-Fi".to_string()],
            season_number: None,
            episode_number: None,
            episode_title: None,
            air_date: None,
            show_name: None,
        };
        assert_eq!(metadata.tmdb_id, 603);
        assert_eq!(metadata.title, "The Matrix");
        assert_eq!(metadata.genres.len(), 2);
    }

    #[test]
    fn test_media_metadata_tv_show() {
        let metadata = MediaMetadata {
            tmdb_id: 1396,
            title: "Pilot".to_string(),
            original_title: None,
            year: Some(2008),
            overview: None,
            poster_path: None,
            backdrop_path: None,
            vote_average: Some(9.5),
            genres: vec!["Drama".to_string()],
            season_number: Some(1),
            episode_number: Some(1),
            episode_title: Some("Pilot".to_string()),
            air_date: Some("2008-01-20".to_string()),
            show_name: Some("Breaking Bad".to_string()),
        };
        assert_eq!(metadata.show_name, Some("Breaking Bad".to_string()));
        assert_eq!(metadata.season_number, Some(1));
        assert_eq!(metadata.episode_number, Some(1));
    }

    // ==================== SEARCH RESULT TESTS ====================

    #[test]
    fn test_search_result() {
        let result = SearchResult {
            tmdb_id: 603,
            title: "The Matrix".to_string(),
            year: Some(1999),
            media_type: MediaType::Movie,
            overview: Some("A computer hacker learns...".to_string()),
            poster_path: Some("/path.jpg".to_string()),
            vote_average: Some(8.7),
        };
        assert_eq!(result.tmdb_id, 603);
        assert_eq!(result.title, "The Matrix");
        assert_eq!(result.media_type, MediaType::Movie);
    }

    // ==================== RENAME PATTERN TESTS ====================

    #[test]
    fn test_rename_pattern_default() {
        let pattern = RenamePattern::default();
        assert_eq!(pattern.name, "Default");
        assert_eq!(pattern.movie_pattern, "{title} ({year})");
        assert_eq!(pattern.tv_pattern, "{show} - S{season:02}E{episode:02} - {episode_title}");
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
    }

    #[test]
    fn test_all_patterns() {
        let patterns = RenamePattern::all_patterns();
        assert_eq!(patterns.len(), 3);
    }

    // ==================== APP STATE TESTS ====================

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert!(state.files.is_empty());
        assert!(state.selected_file_index.is_none());
        assert!(state.search_query.is_empty());
        assert!(!state.loading);
        assert!(!state.show_rename_confirm);
    }

    #[test]
    fn test_app_state_filtered_files_no_query() {
        let mut state = AppState::new();
        state.files.push(MediaFile::new(PathBuf::from("/test/movie1.mkv")));
        state.files.push(MediaFile::new(PathBuf::from("/test/movie2.mkv")));
        
        let filtered = state.filtered_files();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_app_state_filtered_files_with_query() {
        let mut state = AppState::new();
        let mut file1 = MediaFile::new(PathBuf::from("/test/matrix.mkv"));
        file1.filename = "matrix.mkv".to_string();
        let mut file2 = MediaFile::new(PathBuf::from("/test/inception.mkv"));
        file2.filename = "inception.mkv".to_string();
        state.files.push(file1);
        state.files.push(file2);
        
        state.search_query = "matrix".to_string();
        let filtered = state.filtered_files();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].1.filename, "matrix.mkv");
    }

    #[test]
    fn test_app_state_selected_file() {
        let mut state = AppState::new();
        state.files.push(MediaFile::new(PathBuf::from("/test/movie.mkv")));
        
        assert!(state.selected_file().is_none());
        
        state.selected_file_index = Some(0);
        assert!(state.selected_file().is_some());
    }

    #[test]
    fn test_app_state_selected_file_out_of_bounds() {
        let mut state = AppState::new();
        state.files.push(MediaFile::new(PathBuf::from("/test/movie.mkv")));
        state.selected_file_index = Some(5);
        
        assert!(state.selected_file().is_none());
    }

    #[test]
    fn test_app_state_files_with_matches() {
        let mut state = AppState::new();
        
        let mut matched = MediaFile::new(PathBuf::from("/test/movie1.mkv"));
        matched.matched_metadata = Some(MediaMetadata::default());
        
        let unmatched = MediaFile::new(PathBuf::from("/test/movie2.mkv"));
        
        state.files.push(matched);
        state.files.push(unmatched);
        
        assert_eq!(state.files_with_matches(), 1);
    }

    #[test]
    fn test_app_state_files_ready_for_rename() {
        let mut state = AppState::new();
        
        let mut ready = MediaFile::new(PathBuf::from("/test/movie1.mkv"));
        ready.is_selected = true;
        ready.new_filename = Some("New Name.mkv".to_string());
        
        let mut not_selected = MediaFile::new(PathBuf::from("/test/movie2.mkv"));
        not_selected.is_selected = false;
        not_selected.new_filename = Some("New Name 2.mkv".to_string());
        
        let mut no_new_name = MediaFile::new(PathBuf::from("/test/movie3.mkv"));
        no_new_name.is_selected = true;
        no_new_name.new_filename = None;
        
        state.files.push(ready);
        state.files.push(not_selected);
        state.files.push(no_new_name);
        
        assert_eq!(state.files_ready_for_rename().len(), 1);
    }

    #[test]
    fn test_app_state_effective_api_key_default() {
        let state = AppState::new();
        // When using_default_key is true (if default key exists), it should return the default key
        // When no default key, it returns empty string
        let key = state.effective_api_key();
        if has_default_api_key() {
            assert!(!key.is_empty());
        } else {
            assert!(key.is_empty());
        }
    }

    #[test]
    fn test_app_state_effective_api_key_custom() {
        let mut state = AppState::new();
        state.using_default_key = false;
        state.tmdb_api_key = "custom_key_12345".to_string();
        
        assert_eq!(state.effective_api_key(), "custom_key_12345");
    }
}

