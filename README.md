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

### ✅ Now Working

- **Whisper STT** - Speech-to-text using Candle (HuggingFace model)
- **Filler Word Removal** - `--remove-fillers` flag

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
threshold_db = -30.0
min_duration = 0.5
padding = 0.1
mode = "cut"
speedup_factor = 4.0

[filler_words]
enabled = true
words = ["um", "uh", "ah", "er", "like"]
padding = 0.05

[audio]
enhance = true
target_lufs = -14.0
duck_volume = 0.2

[export]
subtitles = false
chapters = false
fcpxml = false
edl = false
```

## Project Status

| Feature | Status |
|---------|--------|
| Silence detection | ✅ Done |
| Silence trimming | ✅ Done |
| Speedup mode | ✅ Done |
| Batch processing | ✅ Done |
| TOML config | ✅ Done |
| Audio enhancement | ✅ Done |
| Music mixing | ✅ Done |
| Intro/Outro | ✅ Done |
| Preset profiles | ✅ Done |
| Watch mode | ✅ Done |
| Dry run | ✅ Done |
| JSON output | ✅ Done |
| Export formats | ✅ Done |
| Whisper STT | ❌ TODO |
| Filler word removal | ❌ TODO |

## License

MIT