# Reel

A modern media file organizer and renamer built with Rust and Iced GUI framework. Similar to FileBot, Reel helps you organize your movie and TV show files with proper naming using online metadata from TMDB.

## Features

- 🎬 **TMDB Integration** - Fetch movie and TV show metadata automatically
- 📁 **Smart Renaming** - Rename files using customizable patterns
- 🔍 **Auto-Match** - Automatically match files to TMDB entries
- 🎯 **Manual Search** - Search and apply metadata manually when needed
- 🌓 **Dark/Light Theme** - Beautiful cinema-inspired UI
- 📺 **TV Show Support** - Handle seasons and episodes with episode titles
- 🎥 **Movie Support** - Rename movies with title and year

## Installation

### Pre-built Binaries

Download the latest release for your platform from the [Releases](../../releases) page:

- **Linux**: `reel-linux-x86_64`
- **Windows**: `reel-windows-x86_64.exe`
- **macOS Intel**: `reel-macos-x86_64`
- **macOS Apple Silicon**: `reel-macos-arm64`

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/reel.git
cd reel

# Build release version
cargo build --release

# Run
./target/release/reel
```

## Usage

1. **Add Files** - Click "Files" or "Folder" to add media files
2. **API Key** - A built-in TMDB API key is included, or use your own
3. **Find & Match** - Click "Find & Match All" to auto-match files
4. **Review** - Check the preview on the right panel
5. **Rename** - Click "Rename" to apply the new names

### Naming Patterns

**Movies:**
```
{title} ({year})
```

**TV Shows:**
```
{show} - S{season:02}E{episode:02} - {episode_title}
```

## Development

### Prerequisites

- Rust 1.70+
- Linux: `libgtk-3-dev libxkbcommon-dev libwayland-dev libvulkan-dev libssl-dev pkg-config`

### Building with Custom API Key

```bash
REEL_TMDB_API_KEY="your-api-key" cargo build --release
```

## License

MIT License

## Credits

- [TMDB](https://www.themoviedb.org/) for movie and TV metadata
- [Iced](https://github.com/iced-rs/iced) for the GUI framework

