use crate::message::Message;
use crate::model::{AppState, MediaType, RenamePattern};
use crate::theme::{
    get_colors, CardStyle, DangerButtonStyle, FileItemStyle, PanelStyle, PrimaryButtonStyle,
    SecondaryButtonStyle, SuccessButtonStyle, TextInputStyle, ThemeMode, ToggleStyle, TooltipStyle,
};
use iced::widget::{
    button, checkbox, column, container, row, scrollable, text, text_input, tooltip, Column, Row,
    Space,
};
use iced::{Alignment, Color, Element, Font, Length, Theme};
use iced_aw::core::icons::bootstrap::{icon_to_text, Bootstrap};
use iced_aw::modal;

const JETBRAINS_MONO: Font = Font::with_name("JetBrains Mono");

pub fn build_view(state: &AppState, theme_mode: ThemeMode) -> Element<'_, Message> {
    let colors = get_colors(theme_mode);

    // Header with status
    let header = build_header(state, theme_mode);

    // Main content - three panel layout
    let content = row![
        // Left panel - File list
        build_file_list_panel(state, theme_mode),
        Space::with_width(12),
        // Middle panel - Actions (Auto-match focused)
        build_actions_panel(state, theme_mode),
        Space::with_width(12),
        // Right panel - Rename preview
        build_rename_panel(state, theme_mode),
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(Length::Fill);

    let main_content = column![header, Space::with_height(12), content,]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill);

    // Wrap in modal if showing rename preview
    let base: Element<Message> = container(main_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([12, 16])
        .style(iced::theme::Container::Custom(Box::new(
            move |_: &Theme| iced::widget::container::Appearance {
                text_color: Some(colors.text_primary),
                background: Some(iced::Background::Color(colors.bg_primary)),
                border: iced::Border::default(),
                shadow: Default::default(),
            },
        )))
        .into();

    if state.show_rename_confirm {
        modal(base, Some(build_rename_modal(state, theme_mode)))
            .backdrop(Message::HideRenamePreview)
            .on_esc(Message::HideRenamePreview)
            .into()
    } else {
        base
    }
}

fn build_header(state: &AppState, theme_mode: ThemeMode) -> Element<'_, Message> {
    let colors = get_colors(theme_mode);

    container(
        row![
            row![
                icon_to_text(Bootstrap::Film)
                    .size(26.0)
                    .style(iced::theme::Text::Color(colors.accent_primary)),
                Space::with_width(12),
                text("Reel")
                    .size(24)
                    .style(iced::theme::Text::Color(colors.text_primary)),
            ]
            .spacing(0)
            .align_items(Alignment::Center),
            Space::with_width(24),
            // Status text in header
            text(&state.status)
                .size(13)
                .style(iced::theme::Text::Color(colors.text_secondary)),
            Space::with_width(Length::Fill),
            // Theme toggle
            row![
                text(if theme_mode == ThemeMode::Dark {
                    "Dark"
                } else {
                    "Light"
                })
                .size(12)
                .style(iced::theme::Text::Color(colors.text_secondary)),
                Space::with_width(8),
                checkbox("", theme_mode == ThemeMode::Light)
                    .on_toggle(|_| Message::ToggleTheme)
                    .style(iced::theme::Checkbox::Custom(Box::new(ToggleStyle {
                        mode: theme_mode
                    }))),
            ]
            .spacing(0)
            .align_items(Alignment::Center),
        ]
        .spacing(0)
        .align_items(Alignment::Center)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .padding([10, 16])
    .style(iced::theme::Container::Custom(Box::new(CardStyle {
        mode: theme_mode,
    })))
    .into()
}

// ============== FILE LIST PANEL ==============

fn build_file_list_panel(state: &AppState, theme_mode: ThemeMode) -> Element<'_, Message> {
    let colors = get_colors(theme_mode);

    // Header with add buttons
    let header = row![
        text("Files")
            .size(16)
            .style(iced::theme::Text::Color(colors.text_primary)),
        Space::with_width(Length::Fill),
        // Refresh button
        button(icon_to_text(Bootstrap::ArrowClockwise).size(13.0))
            .style(iced::theme::Button::Custom(Box::new(
                SecondaryButtonStyle { mode: theme_mode }
            )))
            .padding([5, 8])
            .on_press(Message::RefreshFiles),
        Space::with_width(6),
        button(
            row![
                icon_to_text(Bootstrap::FileEarmarkPlus).size(13.0),
                Space::with_width(5),
                text("Files").size(11),
            ]
            .align_items(Alignment::Center)
        )
        .style(iced::theme::Button::Custom(Box::new(
            SecondaryButtonStyle { mode: theme_mode }
        )))
        .padding([5, 10])
        .on_press(Message::AddFiles),
        Space::with_width(6),
        button(
            row![
                icon_to_text(Bootstrap::FolderPlus).size(13.0),
                Space::with_width(5),
                text("Folder").size(11),
            ]
            .align_items(Alignment::Center)
        )
        .style(iced::theme::Button::Custom(Box::new(PrimaryButtonStyle {
            mode: theme_mode
        })))
        .padding([5, 10])
        .on_press(Message::AddFolder),
    ]
    .spacing(0)
    .align_items(Alignment::Center)
    .width(Length::Fill);

    // Search bar
    let search_bar = text_input("Filter files...", &state.search_query)
        .on_input(Message::FileSearchChanged)
        .style(iced::theme::TextInput::Custom(Box::new(TextInputStyle {
            mode: theme_mode,
        })))
        .padding(8)
        .size(13);

    // File list
    let filtered_files = state.filtered_files();
    let selected_count = state.files.iter().filter(|f| f.is_selected).count();

    let file_list: Element<Message> = if filtered_files.is_empty() {
        container(
            column![
                icon_to_text(Bootstrap::CollectionPlay)
                    .size(48.0)
                    .style(iced::theme::Text::Color(colors.text_disabled)),
                Space::with_height(16),
                text("No files added")
                    .size(14)
                    .style(iced::theme::Text::Color(colors.text_secondary)),
                Space::with_height(8),
                text("Click 'Folder' to add a folder with videos")
                    .size(12)
                    .style(iced::theme::Text::Color(colors.text_disabled)),
            ]
            .align_items(Alignment::Center)
            .spacing(0),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    } else {
        let items: Vec<Element<Message>> = filtered_files
            .into_iter()
            .enumerate()
            .map(|(display_idx, (actual_idx, file))| {
                let is_focused = state.selected_file_index == Some(actual_idx);

                // Media type badge with text (more reliable than icons)
                let type_text = match file.media_type {
                    MediaType::Movie => "MOV",
                    MediaType::TvShow => "TV",
                    MediaType::Unknown => "?",
                };
                let type_color = match file.media_type {
                    MediaType::Movie => colors.movie_color,
                    MediaType::TvShow => colors.tv_color,
                    MediaType::Unknown => colors.text_disabled,
                };

                let type_badge = container(
                    text(type_text)
                        .size(9)
                        .style(iced::theme::Text::Color(Color::WHITE)),
                )
                .padding([2, 5])
                .style(iced::theme::Container::Custom(Box::new(BadgeStyle {
                    color: type_color,
                })));

                // Match indicator - use colored dot
                let match_indicator: Element<Message> = if file.matched_metadata.is_some() {
                    container(
                        text("OK")
                            .size(9)
                            .style(iced::theme::Text::Color(Color::WHITE)),
                    )
                    .padding([2, 5])
                    .style(iced::theme::Container::Custom(Box::new(BadgeStyle {
                        color: colors.success,
                    })))
                    .into()
                } else {
                    container(
                        text("--")
                            .size(9)
                            .style(iced::theme::Text::Color(colors.text_disabled)),
                    )
                    .padding([2, 5])
                    .into()
                };

                // Selection checkbox
                let selection_cb = checkbox("", file.is_selected)
                    .on_toggle(move |_| Message::ToggleFileSelection(actual_idx))
                    .size(14)
                    .style(iced::theme::Checkbox::Custom(Box::new(
                        SmallCheckboxStyle { mode: theme_mode },
                    )));

                let file_row = button(
                    container(
                        row![
                            selection_cb,
                            Space::with_width(8),
                            type_badge,
                            Space::with_width(8),
                            column![
                                text(&file.filename)
                                    .size(14)
                                    .style(iced::theme::Text::Color(colors.text_primary)),
                                text(file.formatted_size())
                                    .size(12)
                                    .style(iced::theme::Text::Color(colors.text_secondary)),
                            ]
                            .spacing(3)
                            .width(Length::FillPortion(10)), // Take most space but not all
                            Space::with_width(8),
                            container(match_indicator).width(Length::Fixed(30.0)), // Fixed width for badge
                        ]
                        .align_items(Alignment::Center)
                        .spacing(0)
                        .width(Length::Fill),
                    )
                    .padding([8, 10])
                    .style(iced::theme::Container::Custom(Box::new(FileItemStyle {
                        mode: theme_mode,
                        is_selected: is_focused,
                    }))),
                )
                .style(iced::theme::Button::Custom(Box::new(
                    TransparentButtonStyle,
                )))
                .padding(0)
                .on_press(Message::FileSelected(display_idx));

                file_row.into()
            })
            .collect();

        scrollable(
            Column::with_children(items)
                .spacing(6) // More spacing between files
                .width(Length::Fill)
                .padding([0, 10, 0, 0]),
        )
        .height(Length::Fill)
        .into()
    };

    // Footer with selection controls
    let footer = column![
        // Selection buttons
        if !state.files.is_empty() {
            row![
                button(text("Select All").size(11))
                    .style(iced::theme::Button::Custom(Box::new(
                        SecondaryButtonStyle { mode: theme_mode }
                    )))
                    .padding([5, 10])
                    .on_press(Message::SelectAllFiles),
                Space::with_width(8),
                button(text("Deselect All").size(11))
                    .style(iced::theme::Button::Custom(Box::new(
                        SecondaryButtonStyle { mode: theme_mode }
                    )))
                    .padding([5, 10])
                    .on_press(Message::DeselectAllFiles),
                Space::with_width(Length::Fill),
                button(icon_to_text(Bootstrap::Trash).size(13.0))
                    .style(iced::theme::Button::Custom(Box::new(DangerButtonStyle {
                        mode: theme_mode
                    })))
                    .padding([4, 8])
                    .on_press(Message::RemoveAllFiles),
            ]
            .align_items(Alignment::Center)
        } else {
            row![]
        },
        Space::with_height(8),
        // Stats
        row![
            text(format!("{} files", state.files.len()))
                .size(11)
                .style(iced::theme::Text::Color(colors.text_secondary)),
            text(format!(" • {} selected", selected_count))
                .size(11)
                .style(iced::theme::Text::Color(colors.accent_primary)),
            if state.files_with_matches() > 0 {
                text(format!(" • {} matched", state.files_with_matches()))
                    .size(11)
                    .style(iced::theme::Text::Color(colors.success))
            } else {
                text("").size(11)
            },
        ]
        .align_items(Alignment::Center),
    ]
    .spacing(0);

    container(
        column![
            header,
            Space::with_height(14),
            search_bar,
            Space::with_height(14),
            file_list,
            Space::with_height(14),
            footer,
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::FillPortion(3))
    .height(Length::Fill)
    .padding(16)
    .style(iced::theme::Container::Custom(Box::new(PanelStyle {
        mode: theme_mode,
    })))
    .into()
}

// ============== ACTIONS PANEL (Improved UX) ==============

fn build_actions_panel(state: &AppState, theme_mode: ThemeMode) -> Element<'_, Message> {
    let colors = get_colors(theme_mode);
    let has_files = !state.files.is_empty();
    // Check effective API key (includes default if using_default_key is true)
    let has_api_key = !state.effective_api_key().is_empty();
    let unmatched_count = state
        .files
        .iter()
        .filter(|f| f.matched_metadata.is_none())
        .count();

    // API Key Section - with verification status in header and verify button next to input
    let api_status_text: Element<Message> = if state.api_key_verifying {
        text("Verifying...")
            .size(10)
            .style(iced::theme::Text::Color(colors.text_secondary))
            .into()
    } else if !has_api_key {
        text("Required")
            .size(10)
            .style(iced::theme::Text::Color(colors.warning))
            .into()
    } else {
        match state.api_key_valid {
            Some(true) => text("Connected")
                .size(10)
                .style(iced::theme::Text::Color(colors.success))
                .into(),
            Some(false) => text("Invalid")
                .size(10)
                .style(iced::theme::Text::Color(colors.error))
                .into(),
            None => text("Not verified")
                .size(10)
                .style(iced::theme::Text::Color(colors.text_secondary))
                .into(),
        }
    };

    // Verify button - enabled when key exists and not yet verified or invalid
    let can_verify = has_api_key && !state.api_key_verifying && state.api_key_valid != Some(true);

    // Always show the same structure to avoid focus loss
    let verify_button = if can_verify {
        button(text("Verify").size(12))
            .style(iced::theme::Button::Custom(Box::new(
                SecondaryButtonStyle { mode: theme_mode },
            )))
            .padding([10, 16])
            .on_press(Message::VerifyApiKey)
    } else if state.api_key_verifying {
        button(text("...").size(12))
            .style(iced::theme::Button::Custom(Box::new(
                SecondaryButtonStyle { mode: theme_mode },
            )))
            .padding([10, 16])
    } else {
        // Key is verified/valid - show disabled verify button
        button(text("Verify").size(12))
            .style(iced::theme::Button::Custom(Box::new(
                SecondaryButtonStyle { mode: theme_mode },
            )))
            .padding([10, 16])
    };

    // When using default key, show placeholder text (not copyable)
    // When user enters their own key, show secure input
    let api_input_row: Element<Message> = if state.using_default_key {
        row![
            container(
                text("Using built-in API key")
                    .size(12)
                    .style(iced::theme::Text::Color(colors.text_secondary))
            )
            .style(iced::theme::Container::Custom(Box::new(
                DefaultKeyContainerStyle { mode: theme_mode }
            )))
            .padding(10)
            .width(Length::Fill),
            Space::with_width(8),
            button(text("Use Own Key").size(12))
                .style(iced::theme::Button::Custom(Box::new(
                    SecondaryButtonStyle { mode: theme_mode }
                )))
                .padding([10, 16])
                .on_press(Message::TmdbApiKeyChanged(String::new())),
            Space::with_width(8),
            verify_button,
        ]
        .align_items(Alignment::Center)
        .into()
    } else {
        // User is using their own key - show input with option to switch back to default
        let has_default = crate::model::has_default_api_key();

        let mut input_row =
            row![
                text_input("Paste your TMDB API key here...", &state.tmdb_api_key)
                    .on_input(Message::TmdbApiKeyChanged)
                    .style(iced::theme::TextInput::Custom(Box::new(TextInputStyle {
                        mode: theme_mode
                    })))
                    .padding(10)
                    .size(12)
                    .secure(true)
                    .width(Length::Fill),
            ]
            .align_items(Alignment::Center);

        // Add "Use Default" button if a default key is available
        if has_default {
            input_row = input_row.push(Space::with_width(8)).push(
                button(text("Use Default").size(12))
                    .style(iced::theme::Button::Custom(Box::new(
                        SecondaryButtonStyle { mode: theme_mode },
                    )))
                    .padding([10, 16])
                    .on_press(Message::UseDefaultApiKey),
            );
        }

        input_row = input_row.push(Space::with_width(8)).push(verify_button);

        input_row.into()
    };

    let api_help_tooltip = tooltip(
        container(
            text("?")
                .size(11)
                .style(iced::theme::Text::Color(colors.text_secondary))
        )
        .style(iced::theme::Container::Custom(Box::new(HelpBadgeStyle {
            mode: theme_mode,
        })))
        .padding([2, 7]),
        "A default API key is included. You can optionally replace it with your own key from themoviedb.org",
        tooltip::Position::Top,
    )
    .style(iced::theme::Container::Custom(Box::new(TooltipStyle { mode: theme_mode })));

    let api_section = column![
        row![
            text("TMDB API Key")
                .size(13)
                .style(iced::theme::Text::Color(colors.text_primary)),
            Space::with_width(6),
            api_help_tooltip,
            Space::with_width(Length::Fill),
            api_status_text,
        ]
        .align_items(Alignment::Center),
        Space::with_height(8),
        api_input_row,
    ]
    .spacing(0);

    // Find and Match Section with loading state in button
    let is_matching = state.loading;
    let matched_count = state
        .files
        .iter()
        .filter(|f| f.matched_metadata.is_some())
        .count();
    let match_section = column![row![
        text("Find & Match")
            .size(13)
            .style(iced::theme::Text::Color(colors.text_primary)),
        Space::with_width(Length::Fill),
        if unmatched_count > 0 {
            text(format!("{} unmatched", unmatched_count))
                .size(11)
                .style(iced::theme::Text::Color(colors.warning))
        } else if has_files {
            text("All matched")
                .size(11)
                .style(iced::theme::Text::Color(colors.success))
        } else {
            text("").size(11)
        },
        // Clear matches button (only show if there are matched files)
        if matched_count > 0 {
            Space::with_width(8)
        } else {
            Space::with_width(0)
        },
        if matched_count > 0 {
            button(icon_to_text(Bootstrap::ArrowCounterclockwise).size(11.0))
                .style(iced::theme::Button::Custom(Box::new(
                    SecondaryButtonStyle { mode: theme_mode },
                )))
                .padding([4, 6])
                .on_press(Message::ClearMatchedMetadata)
        } else {
            button(text("").size(1))
                .style(iced::theme::Button::Custom(Box::new(
                    SecondaryButtonStyle { mode: theme_mode },
                )))
                .padding(0)
        },
    ]
    .align_items(Alignment::Center),]
    .spacing(0);

    // Find and Match Button with visible loading indicator (separate element)
    let match_button: Element<Message> = if is_matching {
        container(
            row![
                icon_to_text(Bootstrap::Magic).size(14.0),
                Space::with_width(8),
                text("Matching files...").size(13),
            ]
            .align_items(Alignment::Center),
        )
        .padding([12, 16])
        .width(Length::Fill)
        .center_x()
        .style(iced::theme::Container::Custom(Box::new(
            LoadingButtonStyle { mode: theme_mode },
        )))
        .into()
    } else if has_files && has_api_key {
        tooltip(
            button(
                container(
                    row![
                        icon_to_text(Bootstrap::Magic).size(14.0),
                        Space::with_width(8),
                        text("Find & Match All").size(13),
                    ]
                    .align_items(Alignment::Center),
                )
                .width(Length::Fill)
                .center_x(),
            )
            .style(iced::theme::Button::Custom(Box::new(PrimaryButtonStyle {
                mode: theme_mode,
            })))
            .padding([10, 16])
            .on_press(Message::AutoMatchAll)
            .width(Length::Fill),
            "Ctrl+M",
            tooltip::Position::Bottom,
        )
        .style(iced::theme::Container::Custom(Box::new(TooltipStyle {
            mode: theme_mode,
        })))
        .into()
    } else {
        button(
            container(
                row![
                    icon_to_text(Bootstrap::Magic).size(14.0),
                    Space::with_width(8),
                    text("Find & Match All").size(13),
                ]
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .center_x(),
        )
        .style(iced::theme::Button::Custom(Box::new(
            SecondaryButtonStyle { mode: theme_mode },
        )))
        .padding([10, 16])
        .width(Length::Fill)
        .into()
    };

    // Count selected files for manual search
    let selected_count = state
        .files
        .iter()
        .filter(|f| f.is_selected && f.matched_metadata.is_none())
        .count();

    // Manual Search Section - Always visible for searching any title
    let manual_section: Element<Message> = column![
        Space::with_height(20),
        container(Space::new(Length::Fill, Length::Fixed(1.0))).style(
            iced::theme::Container::Custom(Box::new(DividerStyle { mode: theme_mode }))
        ),
        Space::with_height(16),
        row![
            text("Manual Search")
                .size(13)
                .style(iced::theme::Text::Color(colors.text_primary)),
            Space::with_width(Length::Fill),
            if selected_count > 0 {
                text(format!("Apply to {} selected", selected_count))
                    .size(10)
                    .style(iced::theme::Text::Color(colors.accent_primary))
            } else if let Some(file) = state.selected_file() {
                text(format!(
                    "Apply to: {}",
                    truncate_filename(&file.filename, 18)
                ))
                .size(10)
                .style(iced::theme::Text::Color(colors.text_secondary))
            } else {
                text("Click a file to apply")
                    .size(10)
                    .style(iced::theme::Text::Color(colors.text_disabled))
            },
        ]
        .align_items(Alignment::Center),
        Space::with_height(10),
        row![
            text_input("Search movie or TV show...", &state.search_input)
                .on_input(Message::TmdbSearchInputChanged)
                .on_submit(Message::SearchTmdb(state.search_input.clone()))
                .style(iced::theme::TextInput::Custom(Box::new(TextInputStyle {
                    mode: theme_mode
                })))
                .padding(10)
                .size(12)
                .width(Length::Fill),
            Space::with_width(8),
            button(icon_to_text(Bootstrap::Search).size(14.0))
                .style(iced::theme::Button::Custom(Box::new(
                    SecondaryButtonStyle { mode: theme_mode }
                )))
                .padding([10, 14])
                .on_press(Message::SearchTmdb(state.search_input.clone())),
        ]
        .align_items(Alignment::Center),
        Space::with_height(12),
        build_search_results(state, theme_mode),
    ]
    .spacing(0)
    .into();

    container(
        column![
            text("Match & Organize")
                .size(16)
                .style(iced::theme::Text::Color(colors.text_primary)),
            Space::with_height(16),
            api_section,
            Space::with_height(20),
            match_section,
            Space::with_height(10),
            match_button,
            manual_section,
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::FillPortion(4))
    .height(Length::Fill)
    .padding(14)
    .style(iced::theme::Container::Custom(Box::new(PanelStyle {
        mode: theme_mode,
    })))
    .into()
}

fn truncate_filename(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

fn build_search_results(state: &AppState, theme_mode: ThemeMode) -> Element<'_, Message> {
    let colors = get_colors(theme_mode);

    if state.search_results.is_empty() {
        if state.search_loading {
            container(
                row![
                    icon_to_text(Bootstrap::ArrowRepeat).size(12.0),
                    Space::with_width(8),
                    text("Searching...").size(12),
                ]
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .center_x()
            .into()
        } else {
            container(
                text("Search for movies or TV shows above")
                    .size(11)
                    .style(iced::theme::Text::Color(colors.text_disabled)),
            )
            .width(Length::Fill)
            .center_x()
            .padding([20, 0])
            .into()
        }
    } else {
        let result_items: Vec<Element<Message>> = state
            .search_results
            .iter()
            .enumerate()
            .take(12) // Show more results
            .map(|(idx, result)| {
                // Use text badges instead of Bootstrap icons
                let type_text = match result.media_type {
                    MediaType::Movie => "MOV",
                    MediaType::TvShow => "TV",
                    MediaType::Unknown => "?",
                };
                let type_color = match result.media_type {
                    MediaType::Movie => colors.movie_color,
                    MediaType::TvShow => colors.tv_color,
                    MediaType::Unknown => colors.text_disabled,
                };

                let type_badge = container(
                    text(type_text)
                        .size(9)
                        .style(iced::theme::Text::Color(Color::WHITE)),
                )
                .padding([2, 5])
                .style(iced::theme::Container::Custom(Box::new(BadgeStyle {
                    color: type_color,
                })));

                button(
                    container(
                        row![
                            type_badge,
                            Space::with_width(8),
                            column![
                                text(&result.title)
                                    .size(12)
                                    .style(iced::theme::Text::Color(colors.text_primary)),
                                row![
                                    if let Some(year) = result.year {
                                        text(year.to_string())
                                            .size(10)
                                            .style(iced::theme::Text::Color(colors.text_secondary))
                                    } else {
                                        text("").size(10)
                                    },
                                    if let Some(rating) = result.vote_average {
                                        text(format!(" • {:.1}", rating))
                                            .size(10)
                                            .style(iced::theme::Text::Color(colors.warning))
                                    } else {
                                        text("").size(10)
                                    },
                                ],
                            ]
                            .spacing(1),
                            Space::with_width(Length::Fill),
                            text("Apply →")
                                .size(10)
                                .style(iced::theme::Text::Color(colors.accent_primary)),
                        ]
                        .align_items(Alignment::Center)
                        .spacing(0),
                    )
                    .padding([6, 10])
                    .style(iced::theme::Container::Custom(Box::new(FileItemStyle {
                        mode: theme_mode,
                        is_selected: false,
                    }))),
                )
                .style(iced::theme::Button::Custom(Box::new(
                    TransparentButtonStyle,
                )))
                .padding(0)
                .on_press(Message::ApplySearchResult(idx))
                .into()
            })
            .collect();

        scrollable(
            Column::with_children(result_items)
                .spacing(4)
                .width(Length::Fill),
        )
        .height(Length::Fill) // Use remaining space
        .into()
    }
}

// ============== RENAME PANEL ==============

fn build_rename_panel(state: &AppState, theme_mode: ThemeMode) -> Element<'_, Message> {
    let colors = get_colors(theme_mode);

    // Pattern selection
    let patterns = RenamePattern::all_patterns();
    let pattern_buttons: Vec<Element<Message>> = patterns
        .into_iter()
        .map(|p| {
            let is_selected = p.name == state.rename_pattern.name;
            button(
                text(&p.name)
                    .size(11)
                    .style(iced::theme::Text::Color(if is_selected {
                        colors.accent_primary
                    } else {
                        colors.text_secondary
                    })),
            )
            .style(iced::theme::Button::Custom(Box::new(PatternButtonStyle {
                mode: theme_mode,
                is_selected,
            })))
            .padding([5, 10])
            .on_press(Message::PatternChanged(p))
            .into()
        })
        .collect();

    let pattern_selector = column![
        text("Naming Pattern")
            .size(13)
            .style(iced::theme::Text::Color(colors.text_primary)),
        Space::with_height(10),
        Row::with_children(pattern_buttons).spacing(8),
        Space::with_height(10),
        container(
            column![
                row![
                    text("Movie: ")
                        .size(11)
                        .style(iced::theme::Text::Color(colors.text_disabled)),
                    text(&state.rename_pattern.movie_pattern)
                        .size(11)
                        .font(JETBRAINS_MONO)
                        .style(iced::theme::Text::Color(colors.text_secondary)),
                ]
                .width(Length::Fill),
                Space::with_height(6),
                row![
                    text("TV: ")
                        .size(11)
                        .style(iced::theme::Text::Color(colors.text_disabled)),
                    text(&state.rename_pattern.tv_pattern)
                        .size(11)
                        .font(JETBRAINS_MONO)
                        .style(iced::theme::Text::Color(colors.text_secondary)),
                ]
                .width(Length::Fill),
            ]
            .spacing(0)
        )
        .padding([8, 10])
        .style(iced::theme::Container::Custom(Box::new(CodeBlockStyle {
            mode: theme_mode
        }))),
    ]
    .spacing(0);

    // Output directory
    let output_section = column![
        text("Output Directory")
            .size(13)
            .style(iced::theme::Text::Color(colors.text_primary)),
        Space::with_height(8),
        row![
            container(
                text(
                    state
                        .output_directory
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Same as source".to_string())
                )
                .size(11)
                .style(iced::theme::Text::Color(colors.text_secondary))
            )
            .width(Length::Fill),
            button(icon_to_text(Bootstrap::FolderFill).size(12.0))
                .style(iced::theme::Button::Custom(Box::new(
                    SecondaryButtonStyle { mode: theme_mode }
                )))
                .padding([8, 12])
                .on_press(Message::SelectOutputDirectory),
        ]
        .align_items(Alignment::Center)
        .spacing(8),
    ]
    .spacing(0);

    // Preview list
    let files_ready = state.files_ready_for_rename();
    let preview_section: Element<Message> = if files_ready.is_empty() {
        container(
            column![
                icon_to_text(Bootstrap::ArrowLeftRight)
                    .size(28.0)
                    .style(iced::theme::Text::Color(colors.text_disabled)),
                Space::with_height(10),
                text("No files ready to rename")
                    .size(12)
                    .style(iced::theme::Text::Color(colors.text_secondary)),
                Space::with_height(4),
                text("Match files with metadata first")
                    .size(10)
                    .style(iced::theme::Text::Color(colors.text_disabled)),
            ]
            .align_items(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    } else {
        let preview_items: Vec<Element<Message>> = files_ready
            .iter()
            .map(|file| {
                container(
                    column![
                        text(&file.filename)
                            .size(11)
                            .font(JETBRAINS_MONO)
                            .style(iced::theme::Text::Color(colors.text_secondary)),
                        Space::with_height(4),
                        row![
                            text("→")
                                .size(12)
                                .style(iced::theme::Text::Color(colors.accent_primary)),
                            Space::with_width(6),
                            text(file.new_filename.as_ref().unwrap_or(&String::new()))
                                .size(11)
                                .font(JETBRAINS_MONO)
                                .style(iced::theme::Text::Color(colors.success)),
                        ]
                        .align_items(Alignment::Center),
                    ]
                    .spacing(0)
                    .width(Length::Fill),
                )
                .padding([10, 12])
                .width(Length::Fill)
                .style(iced::theme::Container::Custom(Box::new(FileItemStyle {
                    mode: theme_mode,
                    is_selected: false,
                })))
                .into()
            })
            .collect();

        scrollable(
            Column::with_children(preview_items)
                .spacing(6)
                .width(Length::Fill),
        )
        .height(Length::Fill)
        .into()
    };

    // Rename button
    let rename_btn = if !files_ready.is_empty() {
        button(
            row![
                icon_to_text(Bootstrap::CheckLg).size(15.0),
                Space::with_width(8),
                text(format!("Rename {} File(s)", files_ready.len())).size(13),
            ]
            .align_items(Alignment::Center),
        )
        .style(iced::theme::Button::Custom(Box::new(SuccessButtonStyle {
            mode: theme_mode,
        })))
        .padding([11, 18])
        .on_press(Message::ShowRenamePreview)
        .width(Length::Fill)
    } else {
        button(
            row![
                icon_to_text(Bootstrap::CheckLg).size(15.0),
                Space::with_width(8),
                text("Rename Files").size(13),
            ]
            .align_items(Alignment::Center),
        )
        .style(iced::theme::Button::Custom(Box::new(
            SecondaryButtonStyle { mode: theme_mode },
        )))
        .padding([11, 18])
        .width(Length::Fill)
    };

    container(
        column![
            text("Rename")
                .size(16)
                .style(iced::theme::Text::Color(colors.text_primary)),
            Space::with_height(18),
            pattern_selector,
            Space::with_height(18),
            output_section,
            Space::with_height(18),
            row![
                text("Preview")
                    .size(13)
                    .style(iced::theme::Text::Color(colors.text_primary)),
                Space::with_width(Length::Fill),
                text(format!("{} ready", files_ready.len()))
                    .size(11)
                    .style(iced::theme::Text::Color(colors.text_secondary)),
            ]
            .align_items(Alignment::Center),
            Space::with_height(12),
            preview_section,
            Space::with_height(14),
            rename_btn,
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::FillPortion(3))
    .height(Length::Fill)
    .padding(16)
    .style(iced::theme::Container::Custom(Box::new(PanelStyle {
        mode: theme_mode,
    })))
    .into()
}

// ============== RENAME MODAL ==============

fn build_rename_modal(state: &AppState, theme_mode: ThemeMode) -> Element<'_, Message> {
    let colors = get_colors(theme_mode);

    let preview_items: Vec<Element<Message>> = state
        .rename_preview
        .iter()
        .map(|(old, new)| {
            column![
                text(old)
                    .size(11)
                    .font(JETBRAINS_MONO)
                    .style(iced::theme::Text::Color(colors.text_secondary)),
                row![
                    text("→")
                        .size(11)
                        .style(iced::theme::Text::Color(colors.accent_primary)),
                    Space::with_width(6),
                    text(new)
                        .font(JETBRAINS_MONO)
                        .size(11)
                        .style(iced::theme::Text::Color(colors.success)),
                ]
                .align_items(Alignment::Center),
            ]
            .spacing(3)
            .into()
        })
        .collect();

    let preview_list = scrollable(
        Column::with_children(preview_items)
            .spacing(12)
            .width(Length::Fill),
    )
    .height(Length::Fixed(280.0));

    container(
        column![
            text("Confirm Rename")
                .size(17)
                .style(iced::theme::Text::Color(colors.text_primary)),
            Space::with_height(6),
            text(format!(
                "{} file(s) will be renamed:",
                state.rename_preview.len()
            ))
            .size(12)
            .style(iced::theme::Text::Color(colors.text_secondary)),
            Space::with_height(14),
            preview_list,
            Space::with_height(14),
            row![
                button(text("Cancel").size(12))
                    .style(iced::theme::Button::Custom(Box::new(
                        SecondaryButtonStyle { mode: theme_mode }
                    )))
                    .padding([8, 16])
                    .on_press(Message::HideRenamePreview),
                Space::with_width(10),
                button(
                    row![
                        icon_to_text(Bootstrap::CheckLg).size(13.0),
                        Space::with_width(6),
                        text("Rename").size(12),
                    ]
                    .align_items(Alignment::Center)
                )
                .style(iced::theme::Button::Custom(Box::new(SuccessButtonStyle {
                    mode: theme_mode
                })))
                .padding([8, 16])
                .on_press(Message::ExecuteRename),
            ]
            .spacing(0),
        ]
        .spacing(0)
        .align_items(Alignment::Center),
    )
    .padding(20)
    .max_width(550)
    .style(iced::theme::Container::Custom(Box::new(CardStyle {
        mode: theme_mode,
    })))
    .into()
}

// ============== HELPER STYLES ==============

struct TransparentButtonStyle;

impl iced::widget::button::StyleSheet for TransparentButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: None,
            border: iced::Border::default(),
            text_color: Color::WHITE,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        self.active(style)
    }

    fn pressed(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        self.active(style)
    }
}

struct PatternButtonStyle {
    mode: ThemeMode,
    is_selected: bool,
}

impl iced::widget::button::StyleSheet for PatternButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(if self.is_selected {
                colors.surface_active
            } else {
                colors.surface
            })),
            border: iced::Border {
                color: if self.is_selected {
                    colors.accent_primary
                } else {
                    colors.border
                },
                width: 1.0,
                radius: 5.0.into(),
            },
            text_color: colors.text_primary,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(colors.surface_hover)),
            border: iced::Border {
                color: colors.accent_primary,
                width: 1.0,
                radius: 5.0.into(),
            },
            text_color: colors.text_primary,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn pressed(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        self.hovered(style)
    }
}

struct BadgeStyle {
    color: Color,
}

impl iced::widget::container::StyleSheet for BadgeStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            text_color: Some(Color::WHITE),
            background: Some(iced::Background::Color(self.color)),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 3.0.into(),
            },
            shadow: Default::default(),
        }
    }
}

struct HelpBadgeStyle {
    mode: ThemeMode,
}

impl iced::widget::container::StyleSheet for HelpBadgeStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::container::Appearance {
            text_color: Some(colors.text_secondary),
            background: Some(iced::Background::Color(colors.surface_hover)),
            border: iced::Border {
                color: colors.border_light,
                width: 1.0,
                radius: 10.0.into(),
            },
            shadow: Default::default(),
        }
    }
}

struct DefaultKeyContainerStyle {
    mode: ThemeMode,
}

impl iced::widget::container::StyleSheet for DefaultKeyContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::container::Appearance {
            text_color: Some(colors.text_secondary),
            background: Some(iced::Background::Color(colors.surface)),
            border: iced::Border {
                color: colors.border,
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: Default::default(),
        }
    }
}

struct DividerStyle {
    mode: ThemeMode,
}

impl iced::widget::container::StyleSheet for DividerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::container::Appearance {
            text_color: None,
            background: Some(iced::Background::Color(colors.border)),
            border: iced::Border::default(),
            shadow: Default::default(),
        }
    }
}

struct LoadingButtonStyle {
    mode: ThemeMode,
}

impl iced::widget::container::StyleSheet for LoadingButtonStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::container::Appearance {
            text_color: Some(colors.text_primary),
            background: Some(iced::Background::Color(colors.surface_active)),
            border: iced::Border {
                color: colors.accent_primary,
                width: 2.0,
                radius: 6.0.into(),
            },
            shadow: Default::default(),
        }
    }
}

struct CodeBlockStyle {
    mode: ThemeMode,
}

impl iced::widget::container::StyleSheet for CodeBlockStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::container::Appearance {
            text_color: Some(colors.text_secondary),
            background: Some(iced::Background::Color(colors.bg_tertiary)),
            border: iced::Border {
                color: colors.border_light,
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: Default::default(),
        }
    }
}

struct SmallCheckboxStyle {
    mode: ThemeMode,
}

impl iced::widget::checkbox::StyleSheet for SmallCheckboxStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style, is_checked: bool) -> iced::widget::checkbox::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::checkbox::Appearance {
            background: if is_checked {
                iced::Background::Color(colors.accent_primary)
            } else {
                iced::Background::Color(colors.bg_tertiary)
            },
            icon_color: if is_checked {
                Color::from_rgb(0.1, 0.1, 0.1)
            } else {
                Color::WHITE
            },
            border: iced::Border {
                color: if is_checked {
                    colors.accent_primary
                } else {
                    colors.border
                },
                width: 1.0,
                radius: 3.0.into(),
            },
            text_color: Some(colors.text_primary),
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> iced::widget::checkbox::Appearance {
        self.active(style, is_checked)
    }
}
