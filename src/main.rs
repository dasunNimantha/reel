// Hide console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{Application, Font, Pixels, Settings};
use reel::ReelApp;

// Embed JetBrains Mono font for file lists
const JETBRAINS_MONO: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");

fn main() -> iced::Result {
    let fira_sans_font = Font::with_name("Fira Sans");

    ReelApp::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(1400.0, 800.0),
            min_size: Some(iced::Size::new(1000.0, 600.0)),
            ..Default::default()
        },
        fonts: vec![iced_aw::BOOTSTRAP_FONT_BYTES.into(), JETBRAINS_MONO.into()],
        default_font: fira_sans_font,
        default_text_size: Pixels(15.0),
        ..Default::default()
    })
}
