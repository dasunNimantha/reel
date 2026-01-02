// Hide console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{Application, Font, Pixels, Settings};
use reel::ReelApp;

// Embed JetBrains Mono font for file lists
const JETBRAINS_MONO: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");

// Embed app icon (PNG format, works on all platforms)
const APP_ICON: &[u8] = include_bytes!("../assets/icon.png");

fn load_icon() -> Option<iced::window::Icon> {
    let image = image::load_from_memory(APP_ICON).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    iced::window::icon::from_rgba(image.into_raw(), width, height).ok()
}

fn main() -> iced::Result {
    let fira_sans_font = Font::with_name("Fira Sans");

    ReelApp::run(Settings {
        window: iced::window::Settings {
            // Smaller default for Windows DPI scaling compatibility
            size: iced::Size::new(1100.0, 650.0),
            min_size: Some(iced::Size::new(850.0, 500.0)),
            icon: load_icon(),
            ..Default::default()
        },
        fonts: vec![iced_aw::BOOTSTRAP_FONT_BYTES.into(), JETBRAINS_MONO.into()],
        default_font: fira_sans_font,
        default_text_size: Pixels(15.0),
        ..Default::default()
    })
}
