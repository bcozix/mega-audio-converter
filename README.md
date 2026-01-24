# mega-audio-converter

## Usage

```bash
sudo apt update
sudo apt install pipx
pipx ensurepath
pipx install "audio-separator[cpu]"

```

### List available quality profiles:
```bash
audioconv profiles
```

### Convert a single file (interactive):
```bash
audioconv convert -i input.mp3
```

### Convert to specific quality:
```bash
audioconv convert -i input.mp3 -q high
```

### Convert to all quality levels:
```bash
audioconv convert -i input.mp3 -a
```

### Batch convert directory:
```bash
audioconv batch -d ./music -q mid
```

## Quality Profiles

| Quality Level | Bitrate | Format | Use Case |
|--------------|---------|--------|----------|
| low | 48 kbps | OGG | Background play on slow mobile networks |
| mid | 160 kbps | MP3 | Default mobile streaming |
| high | 320 kbps | MP3 | High-end headphones, home systems |
| karaoke | Lossless | FLAC | Mastering, singing with live effects |
| preview | 96 kbps | OGG | Quick loading for discovery/search |

## Supported Input Formats
- MP3
- WAV
- FLAC
- OGG
- M4A
- AAC

## Requirements
- FFmpeg (automatically installed via ffmpeg-static)

## License
MIT

## Installation & Usage Instructions

1. **Install the tool:**

```bash
cargo run -- profiles
# This will trigger the prompts for path and quality
cargo run -- convert

cargo run -- convert -i song.wav -q high
cargo run -- convert -i song.wav -a
# Compiles with optimizations, then runs
cargo run --release -- convert -i song.wav -a

cargo run -- convert -i song.mp3 -q karaoke

# with ai remover
cargo run -- convert -i  audio_5b39782e-22f5-4ae5-860a-65f429446304.mp3 -q karaoke

# without ai
cargo run -- convert -i audio_5b39782e-22f5-4ae5-860a-65f429446304.mp3 -q karaoke --fast-karaoke

cargo run -- batch -d ./input_songs



cargo build --release
sudo cp target/release/audio_converter /usr/local/bin/audioconv
audioconv profiles
```

2. **Make it globally available (optional):**

3. **Run the tool:**
```bash
# Interactive mode
audioconv convert -i song.mp3

# Convert to high quality
audioconv convert -i song.mp3 -q high

# Convert directory of files
audioconv batch -d ./music -q mid
```

## Key Features

- ✅ Multiple quality profiles matching your specifications
- ✅ Batch processing for directories
- ✅ Interactive CLI with prompts
- ✅ Progress indicators
- ✅ Error handling and validation
- ✅ Automatic ffmpeg installation
- ✅ Support for MP3, WAV, FLAC, OGG, M4A, AAC input

The tool automatically installs ffmpeg-static, so users don't need to install FFmpeg separately. Each conversion preserves audio channels and uses appropriate codec settings for each format.