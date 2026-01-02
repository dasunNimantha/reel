use crate::message::Message;
use crate::model::{AppState, MediaType};
use crate::settings::AppSettings;
use crate::theme::{reel_theme, ThemeMode};
use crate::utils::{file_scanner, filename_parser, renamer, tmdb};
use crate::view::build_view;
use iced::event::{self, Event};
use iced::window;
use iced::{Application, Command, Subscription, Theme};
use std::time::Duration;

pub struct ReelApp {
    state: AppState,
    theme_mode: ThemeMode,
    settings: AppSettings,
}

impl Application for ReelApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let settings = AppSettings::load();
        let mut state = AppState::new();

        // Load saved API key if exists (overrides default)
        if let Some(key) = settings.get_api_key() {
            if !key.is_empty() {
                state.tmdb_api_key = key;
                state.using_default_key = false;
            }
        }

        // Check if we have any API key (user-entered or default)
        let has_api_key = !state.effective_api_key().is_empty();

        let app = Self {
            state,
            theme_mode: ThemeMode::Dark,
            settings,
        };

        // Automatically verify API key on startup
        let command = if has_api_key {
            Command::perform(
                async move {
                    // Small delay to let the UI initialize first
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                },
                |_| Message::VerifyApiKey,
            )
        } else {
            Command::none()
        };

        (app, command)
    }

    fn title(&self) -> String {
        let file_count = self.state.files.len();
        if file_count > 0 {
            format!("Reel - {} file(s)", file_count)
        } else {
            "Reel - Media Organizer".to_string()
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ToggleTheme => {
                self.theme_mode = match self.theme_mode {
                    ThemeMode::Dark => ThemeMode::Light,
                    ThemeMode::Light => ThemeMode::Dark,
                };
                Command::none()
            }

            // File management
            Message::AddFiles => {
                let last_dir = self.settings.last_input_directory.clone();
                Command::perform(
                    async move {
                        let mut dialog = rfd::AsyncFileDialog::new().add_filter(
                            "Video Files",
                            &["mkv", "mp4", "avi", "mov", "wmv", "flv", "webm", "m4v"],
                        );
                        if let Some(dir) = last_dir {
                            dialog = dialog.set_directory(&dir);
                        }
                        match dialog.pick_files().await {
                            Some(handles) => {
                                let paths: Vec<_> =
                                    handles.iter().map(|h| h.path().to_path_buf()).collect();
                                file_scanner::scan_files(paths).await
                            }
                            None => Ok(Vec::new()),
                        }
                    },
                    Message::FilesAdded,
                )
            }

            Message::AddFolder => {
                let last_dir = self.settings.last_input_directory.clone();
                Command::perform(
                    async move {
                        let mut dialog = rfd::AsyncFileDialog::new();
                        if let Some(dir) = last_dir {
                            dialog = dialog.set_directory(&dir);
                        }
                        match dialog.pick_folder().await {
                            Some(handle) => {
                                let path = handle.path().to_path_buf();
                                file_scanner::scan_directory(path).await
                            }
                            None => Ok(Vec::new()),
                        }
                    },
                    Message::FolderAdded, // Different message for folder
                )
            }

            Message::RefreshFiles => {
                // Rescan the last used input directory
                if let Some(dir) = self.settings.last_input_directory.clone() {
                    self.state.status = "Refreshing files...".to_string();
                    Command::perform(
                        async move { file_scanner::scan_directory(dir).await },
                        Message::FolderAdded,
                    )
                } else {
                    self.state.status = "No folder to refresh".to_string();
                    Command::none()
                }
            }

            Message::FilesAdded(result) => {
                match result {
                    Ok(mut new_files) => {
                        if !new_files.is_empty() {
                            // Save last input directory
                            if let Some(first) = new_files.first() {
                                if let Some(parent) = first.path.parent() {
                                    self.settings.last_input_directory = Some(parent.to_path_buf());
                                    let _ = self.settings.save();
                                }
                            }

                            // Auto-detect media types, parse filenames, and select all by default
                            for file in &mut new_files {
                                let (media_type, parsed_info) =
                                    filename_parser::parse_filename(&file.filename);
                                file.media_type = media_type;
                                file.parsed_info = Some(parsed_info);
                                file.is_selected = true; // Select by default
                            }

                            // Append to existing files
                            self.state.files.append(&mut new_files);
                            self.state.status = format!("Added {} file(s)", self.state.files.len());
                        }
                    }
                    Err(e) => {
                        self.state.status = format!("Error: {}", e);
                    }
                }
                Command::none()
            }

            Message::FolderAdded(result) => {
                match result {
                    Ok(mut new_files) => {
                        if !new_files.is_empty() {
                            // Save last input directory
                            if let Some(first) = new_files.first() {
                                if let Some(parent) = first.path.parent() {
                                    self.settings.last_input_directory = Some(parent.to_path_buf());
                                    let _ = self.settings.save();
                                }
                            }

                            // Auto-detect media types, parse filenames, and select all by default
                            for file in &mut new_files {
                                let (media_type, parsed_info) =
                                    filename_parser::parse_filename(&file.filename);
                                file.media_type = media_type;
                                file.parsed_info = Some(parsed_info);
                                file.is_selected = true; // Select by default
                            }

                            // REPLACE existing files (not append)
                            self.state.files = new_files;

                            // Clear all search/selection state
                            self.state.search_query.clear();
                            self.state.search_input.clear();
                            self.state.search_results.clear();
                            self.state.selected_file_index = None;

                            self.state.status =
                                format!("Loaded {} file(s)", self.state.files.len());
                        }
                    }
                    Err(e) => {
                        self.state.status = format!("Error: {}", e);
                    }
                }
                Command::none()
            }

            Message::FileSelected(index) => {
                let filtered = self.state.filtered_files();
                if index < filtered.len() {
                    let (actual_index, file) = filtered[index];
                    let parsed_title = file.parsed_info.as_ref().map(|p| p.title.clone());
                    let filename = file.filename.clone();

                    self.state.selected_file_index = Some(actual_index);

                    // Update search input with parsed title
                    if let Some(title) = parsed_title {
                        self.state.search_input = title;
                    }

                    // Update status to show selected file
                    self.state.status = format!("Selected: {}", filename);
                }
                Command::none()
            }

            Message::FileSearchChanged(query) => {
                self.state.search_query = query;
                Command::none()
            }

            Message::RemoveFile(index) => {
                if index < self.state.files.len() {
                    self.state.files.remove(index);
                    self.state.selected_file_index = None;
                    self.state.status = format!("{} file(s)", self.state.files.len());
                }
                Command::none()
            }

            Message::RemoveAllFiles => {
                self.state.files.clear();
                self.state.selected_file_index = None;
                self.state.search_query.clear();
                self.state.search_input.clear();
                self.state.search_results.clear();
                self.state.status = "All files removed".to_string();
                Command::none()
            }

            Message::SelectAllFiles => {
                for file in &mut self.state.files {
                    file.is_selected = true;
                }
                Command::none()
            }

            Message::DeselectAllFiles => {
                for file in &mut self.state.files {
                    file.is_selected = false;
                }
                Command::none()
            }

            Message::ToggleFileSelection(index) => {
                if let Some(file) = self.state.files.get_mut(index) {
                    file.is_selected = !file.is_selected;
                }
                Command::none()
            }

            // Media type
            Message::SetMediaType(index, media_type) => {
                if let Some(file) = self.state.files.get_mut(index) {
                    file.media_type = media_type;
                }
                Command::none()
            }

            Message::AutoDetectMediaTypes => {
                for file in &mut self.state.files {
                    let (media_type, parsed_info) = filename_parser::parse_filename(&file.filename);
                    file.media_type = media_type;
                    file.parsed_info = Some(parsed_info);
                }
                self.state.status = "Media types detected".to_string();
                Command::none()
            }

            // TMDB Search
            Message::TmdbApiKeyChanged(key) => {
                self.state.tmdb_api_key = key.clone();
                self.state.using_default_key = false; // User is entering their own key
                self.state.api_key_valid = None; // Reset validation status
                self.settings.set_api_key(Some(key));
                let _ = self.settings.save();
                Command::none()
            }

            Message::UseDefaultApiKey => {
                self.state.tmdb_api_key = String::new();
                self.state.using_default_key = true;
                self.state.api_key_valid = None;
                // Clear saved key
                self.settings.set_api_key(None);
                let _ = self.settings.save();
                // Verify the default key
                self.update(Message::VerifyApiKey)
            }

            Message::VerifyApiKey => {
                let api_key = self.state.effective_api_key();
                if api_key.is_empty() {
                    self.state.api_key_valid = None;
                    return Command::none();
                }

                self.state.api_key_verifying = true;

                Command::perform(
                    async move { tmdb::verify_api_key(&api_key).await },
                    Message::ApiKeyVerified,
                )
            }

            Message::ApiKeyVerified(is_valid) => {
                self.state.api_key_verifying = false;
                self.state.api_key_valid = Some(is_valid);
                if is_valid {
                    self.state.status = "API key verified successfully".to_string();
                } else {
                    self.state.status = "Invalid API key".to_string();
                }
                Command::none()
            }

            Message::SearchTmdb(query) => {
                let api_key = self.state.effective_api_key();
                if api_key.is_empty() {
                    self.state.status = "Please set your TMDB API key".to_string();
                    return Command::none();
                }

                self.state.search_loading = true;
                self.state.search_results.clear();
                let media_type = self
                    .state
                    .selected_file()
                    .map(|f| f.media_type)
                    .unwrap_or(MediaType::Unknown);
                let year = self
                    .state
                    .selected_file()
                    .and_then(|f| f.parsed_info.as_ref())
                    .and_then(|p| p.year);

                Command::perform(
                    async move { tmdb::search_media(&api_key, &query, media_type, year).await },
                    Message::TmdbSearchCompleted,
                )
            }

            Message::TmdbSearchCompleted(result) => {
                self.state.search_loading = false;
                match result {
                    Ok(results) => {
                        let count = results.len();
                        self.state.search_results = results;
                        self.state.status = format!("Found {} result(s)", count);
                    }
                    Err(e) => {
                        self.state.status = format!("Search error: {}", e);
                    }
                }
                Command::none()
            }

            Message::TmdbSearchInputChanged(input) => {
                self.state.search_input = input;
                Command::none()
            }

            Message::ApplySearchResult(result_index) => {
                if let Some(result) = self.state.search_results.get(result_index).cloned() {
                    // Get all selected files to apply this result (regardless of existing metadata)
                    let selected_files: Vec<_> = self
                        .state
                        .files
                        .iter()
                        .enumerate()
                        .filter(|(_, f)| f.is_selected)
                        .map(|(i, f)| {
                            let season = f.parsed_info.as_ref().and_then(|p| p.season);
                            let episode = f.parsed_info.as_ref().and_then(|p| p.episode);
                            (i, season, episode)
                        })
                        .collect();

                    // If no selected files, try to use the focused file
                    let files_to_apply: Vec<(usize, Option<u32>, Option<u32>)> =
                        if selected_files.is_empty() {
                            if let Some(idx) = self.state.selected_file_index {
                                if let Some(file) = self.state.files.get(idx) {
                                    let season = file.parsed_info.as_ref().and_then(|p| p.season);
                                    let episode = file.parsed_info.as_ref().and_then(|p| p.episode);
                                    vec![(idx, season, episode)]
                                } else {
                                    self.state.status = "No files to apply".to_string();
                                    return Command::none();
                                }
                            } else {
                                self.state.status = "Select files to apply this result".to_string();
                                return Command::none();
                            }
                        } else {
                            selected_files
                        };

                    // Update media type for all files
                    for (idx, _, _) in &files_to_apply {
                        if let Some(file) = self.state.files.get_mut(*idx) {
                            file.media_type = result.media_type;
                        }
                    }

                    let api_key = self.state.effective_api_key();
                    let count = files_to_apply.len();
                    self.state.search_loading = true;
                    self.state.status =
                        format!("Applying {} to {} file(s)...", result.title, count);

                    // Batch fetch metadata for all files
                    return Command::perform(
                        async move {
                            use futures::future::join_all;

                            let futures: Vec<_> = files_to_apply
                                .iter()
                                .map(|(idx, season, episode)| {
                                    let api_key = api_key.clone();
                                    let result = result.clone();
                                    let idx = *idx;
                                    let season = *season;
                                    let episode = *episode;

                                    async move {
                                        match tmdb::fetch_metadata(
                                            &api_key, &result, season, episode,
                                        )
                                        .await
                                        {
                                            Ok(metadata) => (idx, Ok(metadata)),
                                            Err(e) => (idx, Err(e)),
                                        }
                                    }
                                })
                                .collect();

                            join_all(futures).await
                        },
                        Message::BatchMetadataFetched,
                    );
                }
                Command::none()
            }

            Message::BatchMetadataFetched(results) => {
                self.state.search_loading = false;
                let mut success_count = 0;
                for (index, result) in results {
                    if let Ok(metadata) = result {
                        if let Some(file) = self.state.files.get_mut(index) {
                            let new_name = renamer::generate_filename(
                                file,
                                &metadata,
                                &self.state.rename_pattern,
                            );
                            file.new_filename = Some(new_name);
                            file.matched_metadata = Some(metadata);
                            success_count += 1;
                        }
                    }
                }
                self.state.status = format!("Applied metadata to {} file(s)", success_count);
                Command::none()
            }

            Message::FetchMetadataForSelected => {
                // Trigger search for selected file's parsed title
                if let Some(file) = self.state.selected_file() {
                    if let Some(parsed) = &file.parsed_info {
                        let query = parsed.title.clone();
                        self.state.search_input = query.clone();
                        return Command::perform(async {}, move |_| Message::SearchTmdb(query));
                    }
                }
                Command::none()
            }

            Message::MetadataFetched(file_index, result) => {
                self.state.search_loading = false;
                match result {
                    Ok(metadata) => {
                        if let Some(file) = self.state.files.get_mut(file_index) {
                            // Generate new filename
                            let new_name = renamer::generate_filename(
                                file,
                                &metadata,
                                &self.state.rename_pattern,
                            );
                            file.new_filename = Some(new_name);
                            file.matched_metadata = Some(metadata);
                        }
                        self.state.status = "Metadata applied".to_string();
                    }
                    Err(e) => {
                        self.state.status = format!("Metadata error: {}", e);
                    }
                }
                Command::none()
            }

            Message::AutoMatchAll => {
                let api_key = self.state.effective_api_key();
                if api_key.is_empty() {
                    self.state.status = "Please set your TMDB API key".to_string();
                    return Command::none();
                }

                self.state.loading = true;
                self.state.status = "Matching files (optimized batch mode)...".to_string();
                let files_info: Vec<_> = self
                    .state
                    .files
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| f.matched_metadata.is_none())
                    .map(|(i, f)| {
                        let mut title = f
                            .parsed_info
                            .as_ref()
                            .map(|p| p.title.clone())
                            .unwrap_or_default();

                        // If title is empty but it's a TV show, try to use parent folder name
                        if title.is_empty() && f.media_type == MediaType::TvShow {
                            if let Some(parent) = f.path.parent() {
                                if let Some(folder_name) = parent.file_name() {
                                    title = folder_name.to_string_lossy().to_string();
                                }
                            }
                        }

                        let year = f.parsed_info.as_ref().and_then(|p| p.year);
                        let season = f.parsed_info.as_ref().and_then(|p| p.season);
                        let episode = f.parsed_info.as_ref().and_then(|p| p.episode);
                        let media_type = f.media_type;

                        tmdb::BatchFileInfo {
                            index: i,
                            title,
                            year,
                            season,
                            episode,
                            media_type,
                        }
                    })
                    .collect();

                // Use optimized batch matching
                Command::perform(
                    async move { tmdb::batch_match_files(&api_key, files_info).await },
                    Message::AutoMatchCompleted,
                )
            }

            Message::AutoMatchCompleted(results) => {
                self.state.loading = false;
                let mut success_count = 0;
                let total = results.len();
                for (index, result) in results {
                    if let Ok(metadata) = result {
                        if let Some(file) = self.state.files.get_mut(index) {
                            let new_name = renamer::generate_filename(
                                file,
                                &metadata,
                                &self.state.rename_pattern,
                            );
                            file.new_filename = Some(new_name);
                            file.matched_metadata = Some(metadata);
                            success_count += 1;
                        }
                    }
                }
                let failed = total - success_count;
                if failed > 0 {
                    self.state.status = format!(
                        "Matched {} of {} files ({} need manual search)",
                        success_count, total, failed
                    );
                } else {
                    self.state.status =
                        format!("Successfully matched all {} files!", success_count);
                }
                Command::none()
            }

            // Rename
            Message::PatternChanged(pattern) => {
                self.state.rename_pattern = pattern;
                // Regenerate filenames for matched files
                for file in &mut self.state.files {
                    if let Some(metadata) = &file.matched_metadata {
                        let new_name =
                            renamer::generate_filename(file, metadata, &self.state.rename_pattern);
                        file.new_filename = Some(new_name);
                    }
                }
                Command::none()
            }

            Message::GenerateNewFilenames => {
                for file in &mut self.state.files {
                    if let Some(metadata) = &file.matched_metadata {
                        let new_name =
                            renamer::generate_filename(file, metadata, &self.state.rename_pattern);
                        file.new_filename = Some(new_name);
                    }
                }
                self.state.status = "Filenames generated".to_string();
                Command::none()
            }

            Message::FilenamesGenerated(names) => {
                for (index, new_name) in names {
                    if let Some(file) = self.state.files.get_mut(index) {
                        file.new_filename = Some(new_name);
                    }
                }
                Command::none()
            }

            Message::SelectOutputDirectory => {
                let last_dir = self.settings.last_output_directory.clone();
                Command::perform(
                    async move {
                        let mut dialog = rfd::AsyncFileDialog::new();
                        if let Some(dir) = last_dir {
                            dialog = dialog.set_directory(&dir);
                        }
                        dialog.pick_folder().await.map(|h| h.path().to_path_buf())
                    },
                    Message::OutputDirectorySelected,
                )
            }

            Message::OutputDirectorySelected(path) => {
                if let Some(dir) = path {
                    self.settings.last_output_directory = Some(dir.clone());
                    let _ = self.settings.save();
                    self.state.output_directory = Some(dir);
                }
                Command::none()
            }

            Message::ShowRenamePreview => {
                // Only preview selected files that have metadata
                let selected_files: Vec<_> = self
                    .state
                    .files
                    .iter()
                    .filter(|f| f.is_selected)
                    .cloned()
                    .collect();
                let preview =
                    renamer::generate_preview(&selected_files, &self.state.rename_pattern);
                self.state.rename_preview = preview;
                self.state.show_rename_confirm = true;
                Command::none()
            }

            Message::HideRenamePreview => {
                self.state.show_rename_confirm = false;
                self.state.rename_preview.clear();
                Command::none()
            }

            Message::ExecuteRename => {
                self.state.show_rename_confirm = false;

                // Only rename selected files that have a new filename
                let files_to_rename: Vec<_> = self
                    .state
                    .files
                    .iter()
                    .filter(|f| f.is_selected)
                    .filter_map(|f| {
                        f.new_filename
                            .as_ref()
                            .map(|new| (f.path.clone(), new.clone()))
                    })
                    .collect();

                if files_to_rename.is_empty() {
                    self.state.status = "No files to rename".to_string();
                    return Command::none();
                }

                let output_dir = self.state.output_directory.clone();
                self.state.loading = true;
                self.state.status = "Renaming files...".to_string();

                Command::perform(
                    async move { renamer::rename_files(files_to_rename, output_dir).await },
                    Message::RenameCompleted,
                )
            }

            Message::RenameCompleted(result) => {
                self.state.loading = false;
                match result {
                    Ok(renamed) => {
                        let count = renamed.len();

                        // Update renamed files with their new names
                        for (old_name, new_name) in &renamed {
                            if let Some(file) = self
                                .state
                                .files
                                .iter_mut()
                                .find(|f| f.filename == *old_name)
                            {
                                // Update the filename to the new name
                                file.filename = new_name.clone();

                                // Update the path to the new location
                                if let Some(ref output_dir) = self.state.output_directory {
                                    file.path = output_dir.join(new_name);
                                } else if let Some(parent) = file.path.parent() {
                                    file.path = parent.join(new_name);
                                }

                                // Clear the new_filename since it's now applied
                                file.new_filename = None;

                                // Keep the metadata but mark as no longer needing rename
                                file.is_selected = false;
                            }
                        }

                        self.state.status = format!("Successfully renamed {} file(s)", count);
                    }
                    Err(e) => {
                        self.state.status = format!("Rename error: {}", e);
                    }
                }
                Command::none()
            }

            // Settings
            Message::SaveApiKey => {
                // Only save if user entered their own key
                if !self.state.using_default_key {
                    self.settings
                        .set_api_key(Some(self.state.tmdb_api_key.clone()));
                    let _ = self.settings.save();
                    self.state.status = "API key saved".to_string();
                }
                Command::none()
            }

            Message::LoadApiKey => Command::perform(
                async { AppSettings::load().get_api_key() },
                Message::ApiKeyLoaded,
            ),

            Message::ApiKeyLoaded(key) => {
                if let Some(k) = key {
                    if !k.is_empty() {
                        self.state.tmdb_api_key = k;
                        self.state.using_default_key = false;
                        // Automatically verify the loaded API key
                        return self.update(Message::VerifyApiKey);
                    }
                }
                // If no saved key, verify default key
                if self.state.using_default_key {
                    return self.update(Message::VerifyApiKey);
                }
                Command::none()
            }

            Message::Tick(_) => Command::none(),

            Message::CloseRequested => window::close(window::Id::MAIN),

            Message::ClearMatchedMetadata => {
                for file in &mut self.state.files {
                    file.matched_metadata = None;
                    file.new_filename = None;
                }
                self.state.status = "Cleared all matches - ready to re-match".to_string();
                Command::none()
            }

            Message::CopyFilename(filename) => {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(&filename);
                    self.state.status = format!("Copied: {}", filename);
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message> {
        build_view(&self.state, self.theme_mode)
    }

    fn theme(&self) -> Theme {
        reel_theme(self.theme_mode)
    }

    fn subscription(&self) -> Subscription<Message> {
        use iced::keyboard::{self, Key};

        let mut subscriptions = vec![event::listen_with(|event, _| match event {
            Event::Window(_, window::Event::CloseRequested) => Some(Message::CloseRequested),
            // Keyboard shortcuts
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                if modifiers.control() || modifiers.command() {
                    match key.as_ref() {
                        Key::Character("o") => Some(Message::AddFolder),
                        Key::Character("m") => Some(Message::AutoMatchAll),
                        Key::Character("r") => Some(Message::ShowRenamePreview),
                        Key::Character("a") => Some(Message::SelectAllFiles),
                        Key::Character("d") => Some(Message::DeselectAllFiles),
                        _ => None,
                    }
                } else {
                    match key.as_ref() {
                        Key::Named(keyboard::key::Named::F5) => Some(Message::RefreshFiles),
                        Key::Named(keyboard::key::Named::Delete) => Some(Message::RemoveAllFiles),
                        _ => None,
                    }
                }
            }
            _ => None,
        })];

        if self.state.loading || self.state.search_loading {
            subscriptions.push(iced::time::every(Duration::from_millis(100)).map(Message::Tick));
        }

        Subscription::batch(subscriptions)
    }
}
