# AI Video Editor (CLI + GUI)

A command-line and GUI tool for automated video editing using AI. Designed for content creators who want to drop in raw footage and get polished results without manual editing.

## Quick Start

**GUI (recommended):**
```bash
cargo run --features gui --bin ai-vid-editor-gui
```

**CLI:**
```bash
cargo run --release -- -i input.mp4 -o output.mp4 --preset youtube
```

**Using just (optional):**
```bash
just gui      # Run GUI
just build    # Build release
just test     # Run tests
```

## Installation

```bash
git clone https://github.com/DraconDev/ai-vid-editor.git
cd ai-vid-editor
```

### Requirements

- [Rust](https://rustup.rs/) (edition 2024)
- [FFmpeg](https://ffmpeg.org/) (for video processing)

### NixOS

```bash
nix-shell  # or: nix develop
```

## Features

### ✅ Fully Working

- **Silence Detection & Trimming** - Automatically detect and remove silent segments
- **Speedup Mode** - Speed through silences instead of cutting (configurable speed)
- **Batch Processing** - Process entire directories of videos
- **TOML Configuration** - Customizable settings via config files
- **Audio Enhancement** - Loudness normalization + EQ (`--enhance`)
- **Noise Reduction** - Remove background noise (`--noise-reduction`)
- **Music Mixing** - Auto-ducking background music (`--music` or `--music-dir`)
- **Intro/Outro** - Prepend/append videos (`--intro`, `--outro`)
- **Video Stabilization** - Remove camera shake (`--stabilize`)
- **Auto Color Correction** - Enhance contrast, brightness, saturation (`--color-correct`)
- **Auto-Reframe** - Convert horizontal to vertical (9:16) following speaker's face (`--reframe`)
- **Background Blur** - Blur background while keeping speaker sharp (`--blur-background`)
- **Export Formats** - FCPXML, EDL, SRT subtitles, YouTube chapters
- **Preset Profiles** - One-command setups for YouTube, Shorts, Podcasts
- **Watch Mode** - Daemon that auto-processes new videos
- **Dry Run** - Preview changes without processing
- **JSON Output** - For scripting and CI/CD integration
- **Whisper STT** - Speech-to-text using Candle (HuggingFace model)
- **Filler Word Removal** - Remove "um", "uh", etc. (`--remove-fillers`)

## GUI

The GUI provides a visual interface for managing watch folders and settings:

**Navigation:**
- **All** - Shows Folders, Settings, and Activity stacked
- **Folders** - Manage watch folders only
- **Settings** - Configure processing options
- **Activity** - View processing log

**Watch Folders:**
- Add multiple input/output folder pairs
- Each folder has its own preset (YouTube, Shorts, Podcast, Minimal)
- Toggle folders on/off without deleting
- Click any folder card to edit

**Layout:**
```
┌─────────────────────────────────────────────┐
│ █ AI Video Processor      [Save] [Load]     │
│ [All] [Folders] [Settings] [Activity]       │
├─────────────────────────────────────────────┤
│ Watch Folders              ● Watching       │
│ ┌─────────────────────────────────────────┐ │
│ │ [ON] videos → videos/output    youtube  │ │
│ └─────────────────────────────────────────┘ │
│ [+ Add Folder]                              │
├─────────────────────────────────────────────┤
│ Settings                                    │
│ [x] Enhance Audio    [x] Remove Silence     │
│ ...                                         │
├─────────────────────────────────────────────┤
│ Activity Log                        [Clear] │
│ ✓ 14:32 Added new watch folder              │
└─────────────────────────────────────────────┘
```

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
  --music-dir ./music \
  --stabilize \
  --color-correct

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
| `--noise-reduction` | Enable noise reduction |
| `-m, --music <FILE>` | Background music file |
| `--music-dir <DIR>` | Music folder (picks random track) |
| `--intro <FILE>` | Video to prepend |
| `--outro <FILE>` | Video to append |
| `--stabilize` | Enable video stabilization |
| `--color-correct` | Enable auto color correction |
| `--reframe` | Auto-reframe to vertical (9:16) |
| `--blur-background` | Blur background behind speaker |
| `--remove-fillers` | Remove filler words (um, uh, etc.) |
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
words = ["um", "uh", "ah", "er"]
padding = 0.05

[audio]
enhance = true
noise_reduction = false
target_lufs = -14.0
duck_volume = 0.2

[video]
stabilize = false
color_correct = false

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
| Noise reduction | ✅ Done |
| Music mixing | ✅ Done |
| Intro/Outro | ✅ Done |
| Video stabilization | ✅ Done |
| Auto color correction | ✅ Done |
| Preset profiles | ✅ Done |
| Watch mode | ✅ Done |
| Dry run | ✅ Done |
| JSON output | ✅ Done |
| Export formats | ✅ Done |
| Whisper STT | ✅ Done |
| Filler word removal | ✅ Done |
| Auto-reframe | ✅ Done |
| Background blur | ✅ Done |
| GUI (egui) | ✅ Done |

## License

MIT