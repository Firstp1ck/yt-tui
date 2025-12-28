# YouTube TUI

A terminal-based user interface for browsing YouTube recommendations and playing videos using MPV.

## Features

- 🎬 **Browse YouTube Recommendations** - View your YouTube recommended videos in a beautiful terminal interface
- 🔍 **Search & Filter** - Search videos by title, channel, or description. Filter by channel, duration, and date
- 📊 **Sort Options** - Sort videos by date (newest/oldest), views, or creator name
- 📺 **Video Playback** - Play videos directly in MPV with automatic audio/video configuration
- 📝 **Watch History** - Track watched videos and optionally hide them from the list
- 🖱️ **Mouse Support** - Navigate with mouse scroll and click to play videos
- 🎨 **Modern TUI** - Built with Ratatui for a responsive and beautiful terminal interface

## Requirements

- **Rust** (latest stable version)
- **MPV** - Video player (must be installed and available in PATH)
- **yt-dlp** - YouTube downloader (required by MPV for YouTube support)
- **YouTube Data API v3 Key** - Get one from [Google Cloud Console](https://console.cloud.google.com/apis/credentials)

### Installing Dependencies

#### MPV
- **Arch Linux**: `sudo pacman -S mpv`
- **Ubuntu/Debian**: `sudo apt install mpv`
- **Fedora**: `sudo dnf install mpv`
- **macOS**: `brew install mpv`

#### yt-dlp
- **Arch Linux**: `sudo pacman -S yt-dlp`
- **Ubuntu/Debian**: `sudo apt install yt-dlp`
- **Fedora**: `sudo dnf install yt-dlp`
- **macOS**: `brew install yt-dlp`

## Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/yt-tui.git
cd yt-tui
```

2. Build the project:
```bash
cargo build --release
```

3. Run the application:
```bash
cargo run --release
```

Or install it:
```bash
cargo install --path .
```

## Configuration

### Initial Setup

1. Copy the example configuration file:
```bash
cp config.jsonc.example ~/.config/yt-tui/config.jsonc
```

2. Edit the configuration file and add your YouTube API key:
```jsonc
{
    "api_key": "YOUR_API_KEY_HERE",
    // ... other settings
}
```

### Getting a YouTube API Key

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select an existing one
3. Enable the **YouTube Data API v3**
4. Go to **Credentials** → **Create Credentials** → **API Key**
5. Copy the API key and paste it into your `config.jsonc` file

### Configuration Options

The configuration file supports JSONC format (JSON with comments). Available options:

- `api_key` (required): Your YouTube Data API v3 key
- `oauth_client_id`, `oauth_client_secret`, `oauth_access_token`, `oauth_refresh_token` (optional): OAuth2 credentials for personalized recommendations
- `default_filters`: Default filter settings
  - `channel`: Filter by channel name (partial match, case-insensitive)
  - `min_duration`: Minimum video duration in seconds
  - `max_duration`: Maximum video duration in seconds
  - `after_date`: Filter videos after this date (RFC3339 format)
- `hide_watched`: Whether to hide watched videos by default
- `history_path`: Path to the history file (relative to config directory or absolute)

## Usage

### Starting the Application

Simply run:
```bash
yt-tui
```

The application will:
1. Load your configuration
2. Fetch YouTube recommendations (or trending videos if OAuth is not configured)
3. Display them in an interactive terminal interface

### Keyboard Shortcuts

#### General Navigation
- `q`, `Q`, or `Esc` - Quit the application
- `↑` / `k` - Move selection up
- `↓` / `j` - Move selection down
- `Enter` - Play selected video
- `Ctrl+C` - Quit the application

#### Search Mode (press `/`)
- Type to search videos by title, channel, or description
- `Enter` or `Esc` - Exit search mode
- `Backspace` - Delete last character

#### Filters Mode (press `f`)
- `h` - Toggle hide watched videos
- `s` - Cycle through sort modes
- `Esc` or `f` - Exit filters mode

#### List Mode
- `/` - Enter search mode
- `f` - Enter filters mode
- `h` - Toggle hide watched videos
- `s` - Cycle through sort modes

### Mouse Support

- **Scroll Up/Down** - Navigate through the video list
- **Left Click** - Select and play a video

### Sort Modes

Press `s` to cycle through sort modes:
1. **Date (newest)** - Sort by upload date, newest first
2. **Views (highest)** - Sort by view count, highest first
3. **Upload Date (oldest)** - Sort by upload date, oldest first
4. **Creator (A-Z)** - Sort alphabetically by channel name

## Video Display

Each video is displayed with the following information:
- **Title** (bold, highlighted when selected)
- **Creator/Channel** name
- **Duration** (formatted as HH:MM:SS or MM:SS)
- **Upload Date** (formatted as "Day. DD.MM.YYYY")
- **View Count** (formatted with K/M suffixes)
- **Watched Indicator** - Shows `[WATCHED]` for videos you've already watched

## Video Playback

Videos are played using MPV with automatic configuration:
- **Wayland**: Uses PipeWire, PulseAudio, or ALSA for audio
- **X11**: Uses PulseAudio, ALSA, or auto-detection for audio
- **Video Output**: Automatically selects the best video output driver for your system

The application automatically marks videos as watched when you play them.

## History

Watched videos are tracked in a JSON file (default: `~/.config/yt-tui/history.json`). You can:
- Toggle hiding watched videos with `h`
- The history is automatically saved when you play a video

## Troubleshooting

### No videos showing up
- Check that your API key is correct and has YouTube Data API v3 enabled
- Verify your API key has the necessary permissions
- Check the status message at the bottom of the screen for error details

### Video plays but no audio
- Ensure MPV is properly installed
- Check that your audio system (PipeWire/PulseAudio/ALSA) is running
- Try running MPV manually: `mpv --ao=pipewire "https://www.youtube.com/watch?v=VIDEO_ID"`

### Video plays but no video (only audio)
- Check that your video drivers are properly configured
- On Wayland, ensure you have the correct video output drivers installed
- Try running MPV manually to see if it works: `mpv "https://www.youtube.com/watch?v=VIDEO_ID"`

### Application freezes
- Make sure you're using a terminal that supports the required features
- Try resizing your terminal window
- Check that MPV and yt-dlp are properly installed

## Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/yt-tui.git
cd yt-tui

# Build in release mode
cargo build --release

# The binary will be at: target/release/yt-tui
```

## Development

### Running Tests
```bash
cargo test
```

### Code Formatting
```bash
cargo fmt
```

### Linting
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

## License

[Add your license here]

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Built with [Ratatui](https://github.com/ratatui-org/ratatui) - A Rust library for building TUI applications
- Uses [MPV](https://mpv.io/) - A free, open source, and cross-platform media player
- Uses [yt-dlp](https://github.com/yt-dlp/yt-dlp) - A youtube-dl fork with additional features

