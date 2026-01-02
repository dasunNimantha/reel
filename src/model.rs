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

