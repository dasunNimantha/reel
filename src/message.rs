use crate::model::{MediaFile, MediaMetadata, RenamePattern, SearchResult};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum Message {
    // Theme
    ToggleTheme,

    // File management
    AddFiles,
    AddFolder,
    RefreshFiles, // Rescan current directory
    FilesAdded(Result<Vec<MediaFile>, String>),
    FolderAdded(Result<Vec<MediaFile>, String>), // Replaces existing files
    FileSelected(usize),
    FileSearchChanged(String),
    RemoveFile(usize),
    RemoveAllFiles,
    SelectAllFiles,
    DeselectAllFiles,
    ToggleFileSelection(usize),

    // Media type
    SetMediaType(usize, crate::model::MediaType),
    AutoDetectMediaTypes,

    // TMDB Search
    TmdbApiKeyChanged(String),
    UseDefaultApiKey, // Switch back to built-in API key
    VerifyApiKey,
    ApiKeyVerified(bool), // true = valid, false = invalid
    SearchTmdb(String),
    TmdbSearchCompleted(Result<Vec<SearchResult>, String>),
    TmdbSearchInputChanged(String),
    ApplySearchResult(usize), // index in search_results
    FetchMetadataForSelected,
    MetadataFetched(usize, Result<MediaMetadata, String>), // file_index, result
    BatchMetadataFetched(Vec<(usize, Result<MediaMetadata, String>)>), // for applying search result to multiple files
    AutoMatchAll,
    AutoMatchCompleted(Vec<(usize, Result<MediaMetadata, String>)>),

    // Rename
    PatternChanged(RenamePattern),
    GenerateNewFilenames,
    FilenamesGenerated(Vec<(usize, String)>), // (file_index, new_filename)
    SelectOutputDirectory,
    OutputDirectorySelected(Option<PathBuf>),
    ShowRenamePreview,
    HideRenamePreview,
    ExecuteRename,
    RenameCompleted(Result<Vec<(String, String)>, String>), // (old, new) pairs

    // Settings
    SaveApiKey,
    LoadApiKey,
    ApiKeyLoaded(Option<String>),

    // Animation
    Tick(Instant),

    // Window
    CloseRequested,
}
