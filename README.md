# AI Video Editor (CLI)

A command-line tool for automated video editing using AI. Designed for content creators who want to drop in raw footage and get polished results without manual editing.

## Features

### ✅ Fully Working

- **Silence Detection & Trimming** - Automatically detect and remove silent segments
- **Speedup Mode** - Speed through silences instead of cutting (configurable speed)
- **Batch Processing** - Process entire directories of videos
- **TOML Configuration** - Customizable settings via config files
- **Audio Enhancement** - Loudness normalization + EQ (`--enhance`)
- **Music Mixing** - Auto-ducking background music (`--music` or `--music-dir`)
- **Intro/Outro** - Prepend/append videos (`--intro`, `--outro`)
- **Export Formats** - FCPXML, EDL, SRT subtitles, YouTube chapters
- **Preset Profiles** - One-command setups for YouTube, Shorts, Podcasts
- **Watch Mode** - Daemon that auto-processes new videos
- **Dry Run** - Preview changes without processing
- **JSON Output** - For scripting and CI/CD integration

### 🔧 In Progress

- **Whisper STT** - Speech-to-text for filler word detection (model loads, decode TODO)
- **Filler Word Removal** - Cut "um", "uh", etc. (needs STT completion)

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

### Quick Examples

```bash
# Basic silence removal
ai-vid-editor -i input.mp4 -o output.mp4

# YouTube preset (cut silences + enhance + chapters)
ai-vid-editor -i input.mp4 -o output.mp4 --preset youtube

# Full production pipeline
ai-vid-editor -i input.mp4 -o output.mp4 \
  --preset youtube \
  --intro intro.mp4 \
  --outro outro.mp4 \
  --music-dir ./music

# Batch process a directory
ai-vid-editor -I ./raw_videos -O ./edited --preset youtube

# Watch folder (auto-process new videos)
ai-vid-editor --watch ./incoming -O ./processed

# Preview without processing
ai-vid-editor -i input.mp4 --dry-run

# JSON output for scripting
ai-vid-editor -i input.mp4 --dry-run --json
```

### CLI Options

| Flag | Description |
|------|-------------|
| `-i, --input-file <FILE>` | Input video file |
| `-I, --input-dir <DIR>` | Input directory (batch mode) |
| `-o, --output-file <FILE>` | Output video file |
| `-O, --output-dir <DIR>` | Output directory (batch mode) |
| `-P, --preset <PRESET>` | Preset: `youtube`, `shorts`, `podcast`, `minimal` |
| `-c, --config <FILE>` | Path to TOML config file |
| `-t, --threshold <dB>` | Silence threshold (default: -30.0) |
| `-d, --duration <SEC>` | Min silence duration (default: 0.5) |
| `-p, --padding <SEC>` | Padding around cuts (default: 0.1) |
| `-s, --speedup` | Speed up silences instead of cutting |
| `-E, --enhance` | Enable audio enhancement |
| `-m, --music <FILE>` | Background music file |
| `--music-dir <DIR>` | Music folder (picks random track) |
| `--intro <FILE>` | Video to prepend |
| `--outro <FILE>` | Video to append |
| `--export-srt` | Generate SRT subtitles |
| `--export-chapters` | Generate YouTube chapters |
| `--export-fcpxml` | Generate FCPXML |
| `--export-edl` | Generate EDL |
| `-n, --dry-run` | Preview without processing |
| `-j, --json` | JSON output for scripting |
| `-w, --watch <DIR>` | Watch folder for new videos |
| `--watch-interval <SEC>` | Polling interval (default: 5) |
| `--generate-config` | Output sample config |

### Presets

| Preset | Description |
|--------|-------------|
| `youtube` | Cut silences, enhance audio, export chapters + FCPXML |
| `shorts` | Speedup silences (3x), enhance audio, tight padding |
| `podcast` | Cut silences, enhance audio (-16 LUFS), export SRT |
| `minimal` | Just silence detection, no enhancement |

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
| Audio enhancement | ✅ Done | `--enhance` flag |
| Music mixing | ✅ Done | `--music` / `--music-dir` |
| Intro/Outro | ✅ Done | `--intro` / `--outro` |
| Preset profiles | ✅ Done | youtube, shorts, podcast, minimal |
| Watch mode | ✅ Done | `--watch` daemon |
| Dry run | ✅ Done | `--dry-run` preview |
| JSON output | ✅ Done | `--json` for scripting |
| Export formats | ✅ Done | FCPXML, EDL, SRT, chapters |
| Whisper STT | 🔧 Partial | Model loads, decode TODO |
| Filler word removal | 🔧 Partial | Needs STT completion |

## Architecture

```
src/
├── main.rs           # CLI entry point
├── config.rs         # TOML configuration + presets
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

# Run with options
cargo run -- -i input.mp4 -o output.mp4 --dry-run
```

## Future Features

Potential features for future development:

- **Video cropping** - Auto-crop horizontal to vertical for shorts
- **Scene detection** - Detect scene changes for smarter cuts
- **Progress bar** - Visual feedback during processing
- **Thumbnail extraction** - Auto-suggest best thumbnails
- **Video stabilization** - Stabilize shaky footage
- **Noise reduction** - Audio noise reduction
- **GPU acceleration** - Use GPU for faster processing

## License

MIT

## Contributing

Contributions welcome! Priority areas:
1. Complete Whisper STT implementation (mel spectrogram + decode loop)
2. Wire filler word removal to CLI (needs STT)
3. Add progress bar during processing
4. Video cropping for shorts