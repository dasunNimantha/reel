//! Reel - Media File Organizer
//!
//! A FileBot-like application for organizing and renaming media files
//! with online metadata fetching from TMDB.
//!
//! Features:
//! - Scan folders for media files
//! - Fetch metadata from TMDB (The Movie Database)
//! - Rename files with customizable patterns
//! - Organize into folder structures
//! - Support for TV shows and movies

pub mod app;
pub mod message;
pub mod model;
pub mod settings;
pub mod theme;
pub mod utils;
pub mod view;

// Re-export main types
pub use app::ReelApp;
pub use message::Message;
pub use model::{AppState, MediaFile, MediaType};
pub use theme::ThemeMode;
