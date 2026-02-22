# AI Video Editor (CLI)

A command-line tool for automated video editing using AI. Designed for content creators who want to drop in raw footage and get polished results without manual editing.

## Features

### ✅ Working Now

- **Silence Detection & Trimming** - Automatically detect and remove silent segments
- **Speedup Mode** - Speed through silences instead of cutting (4x default)
- **Batch Processing** - Process entire directories of videos
- **TOML Configuration** - Customizable settings via config files
- **Export Formats** - FCPXML, EDL, SRT subtitles, YouTube chapters

### 🔧 In Progress

- **Whisper STT** - Speech-to-text for filler word detection
- **Audio Enhancement** - Loudness normalization + EQ
- **Music Mixing** - Auto-ducking background music

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)
- [FFmpeg](https://ffmpeg.org/) (for video processing)

### Build from Source

```bash
git clone https://github.com/DraconDev/ai-vid-editor.git
cd ai-vid-editor
cargo build --release
```

The binary will be at `target/release/ai-vid-editor`.

## Usage

### Basic Commands

```bash
# Trim silences from a single video
ai-vid-editor -i input.mp4 -o output.mp4

# Speed through silences instead of cutting
ai-vid-editor -i input.mp4 -o output.mp4 --speedup

# Batch process a directory
ai-vid-editor -I ./raw_videos -O ./edited

# Use a config file
ai-vid-editor -i input.mp4 -o output.mp4 -c config.toml
```

### CLI Options

| Flag | Description |
|------|-------------|
| `-i, --input-file <FILE>` | Input video file |
| `-I, --input-dir <DIR>` | Input directory (batch mode) |
| `-o, --output-file <FILE>` | Output video file |
| `-O, --output-dir <DIR>` | Output directory (batch mode) |
| `-c, --config <FILE>` | Path to TOML config file |
| `-t, --threshold <dB>` | Silence threshold (default: -30.0) |
| `-d, --duration <SEC>` | Min silence duration (default: 0.5) |
| `-p, --padding <SEC>` | Padding around cuts (default: 0.1) |
| `-s, --speedup` | Speed up silences instead of cutting |
| `--generate-config` | Output a sample config file |

## Configuration

Create `ai-vid-editor.toml` in your project directory or `~/.config/ai-vid-editor/config.toml`:

```toml
[silence]
threshold_db = -30.0        # Silence detection threshold (dB)
min_duration = 0.5          # Min silence to detect (seconds)
padding = 0.1               # Padding around cuts (seconds)
mode = "cut"                # "cut" or "speedup"
speedup_factor = 4.0        # Speed multiplier for speedup mode
min_silence_for_speedup = 0.5

[filler_words]
enabled = true
words = ["um", "uh", "ah", "er", "like"]
padding = 0.05

[audio]
enhance = true              # Enable audio enhancement
target_lufs = -14.0         # Target loudness (YouTube: -14)
duck_volume = 0.2           # Music volume during speech

[export]
subtitles = false           # Generate SRT subtitles
chapters = false            # Generate YouTube chapters
fcpxml = false              # Generate FCPXML for DaVinci/Premiere
edl = false                 # Generate EDL
```

### Config Precedence

Settings are applied in this order (later overrides earlier):

1. Hardcoded defaults
2. Global config (`~/.config/ai-vid-editor/config.toml`)
3. Project config (`./ai-vid-editor.toml`)
4. CLI flags

## Project Status

| Feature | Status | Notes |
|---------|--------|-------|
| Silence detection | ✅ Done | Via ffmpeg silencedetect |
| Silence trimming | ✅ Done | With configurable padding |
| Speedup mode | ✅ Done | Speed through silences |
| Batch processing | ✅ Done | Recursive directory support |
| TOML config | ✅ Done | Full configuration support |
| Audio enhancement | 🔧 Partial | Implemented, needs CLI flag |
| Music mixing | 🔧 Partial | Implemented, needs CLI flag |
| Whisper STT | 🔧 Partial | Model loads, decode TODO |
| Filler word removal | 🔧 Partial | Needs STT completion |
| Export formats | ✅ Done | FCPXML, EDL, SRT, chapters |

## Architecture

```
src/
├── main.rs           # CLI entry point
├── config.rs         # TOML configuration
├── analyzer.rs       # Silence detection
├── editor.rs         # Video trimming & audio
├── batch_processor.rs # Single & batch processing
├── stt_analyzer.rs   # Whisper STT (WIP)
├── exporter.rs       # Export formats
└── utils.rs          # File discovery
```

## Development

```bash
# Run tests
cargo test

# Build with debug info
cargo build

# Build optimized release
cargo build --release

# Generate sample config
cargo run -- --generate-config
```

## License

MIT

## Contributing

Contributions welcome! Priority areas:
1. Complete Whisper STT implementation
2. Wire audio enhancement to CLI
3. Add music mixing CLI flags
4. Improve error messages