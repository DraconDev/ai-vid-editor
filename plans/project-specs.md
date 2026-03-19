# AI Video Editor - Project Specifications

## Overview

**Purpose**: Automation-first video editor for content creators. Drop in raw footage, get polished results without manual editing.

**Architecture**:
- CLI: Full-featured command-line interface for scripting/automation
- GUI: Visual management interface built with egui (single binary with `--gui` flag)
- Backend: FFmpeg for video processing, Candle/tract for ML

**Tech Stack**:
- Rust (edition 2024)
- egui/wgpu for GUI
- FFmpeg (external dependency)
- tract (ONNX runtime for ML inference)
- TOML for configuration

---

## Current State

### Working Features

| Feature | CLI | GUI | Notes |
|---------|-----|-----|-------|
| Silence detection | ✅ | - | Core feature |
| Silence trimming | ✅ | - | Cut or speedup modes |
| Audio enhancement | ✅ | ✅ | Two-pass loudnorm + gentle EQ |
| Noise reduction | ✅ | ✅ | FFmpeg afftdn |
| Music mixing | ✅ | - | Auto-ducking |
| Intro/outro | ✅ | - | Prepend/append videos |
| Video stabilization | ✅ | ✅ | vidstab filter |
| Color correction | ✅ | ✅ | Contrast, saturation, sharpen |
| Auto-reframe | ✅ | ✅ | 9:16 with face tracking |
| Background blur | ✅ | ✅ | Portrait mode effect |
| Filler word removal | ✅ | - | Whisper-based |
| Presets | ✅ | ✅ | youtube, shorts, podcast, minimal |
| Watch mode | ✅ | ✅ | Shared `[[paths.watch_folders]]` config |
| Batch processing | ✅ | - | Directory processing |
| Export (SRT, FCPXML, EDL) | ✅ | - | Subtitles, timelines |
| Desktop notifications | ✅ | - | `--notify` flag |
| Unified binary | ✅ | ✅ | `--gui` flag launches GUI |
| Config auto-loading | ✅ | ✅ | `~/.config/ai-vid-editor/config.toml` loaded automatically |
| Progress feedback | ✅ | - | Timestamped stage output during processing |

### GUI Structure

```
┌─────────────────────────────────────────────┐
│ █ AI Video Processor                        │  ← Accent bar + title
│ [All] [Folders] [Settings] [Activity]       │  ← Tab navigation
├─────────────────────────────────────────────┤
│ Watch Folders              ● Watching       │  ← Folder list
│ ┌─────────────────────────────────────────┐ │
│ │ [ON]                              youtube│ │
│ │     Input:  videos/youtube               │ │
│ │     Output: videos/youtube/output        │ │
│ │                              [Remove]    │ │
│ └─────────────────────────────────────────┘ │
│ [+ Add Folder]                              │
├─────────────────────────────────────────────┤
│ Processing                                  │  ← Per-folder settings
│ [youtube] [shorts] [podcast]                │  ← Folder pills
│ [x] Enhance Audio    [x] Remove Silence     │
│ Silence Threshold: ━━━━○━━━ -30 dB          │
├─────────────────────────────────────────────┤
│ Activity                            [Clear] │  ← Processing log
│ ✓ 14:32 Added new watch folder              │
└─────────────────────────────────────────────┘
```

### Data Model

```
Config
├── audio: AudioConfig
│   ├── enhance: bool
│   ├── target_lufs: f32
│   └── duck_volume: f32
├── silence: SilenceConfig
│   ├── threshold_db: f32
│   ├── min_duration: f32
│   ├── padding: f32
│   └── mode: Cut | Speedup
├── video: VideoConfig
│   ├── stabilize: bool
│   ├── color_correct: bool
│   ├── reframe: bool
│   └── blur_background: bool
├── paths: PathsConfig
│   └── watch_folders: Vec<WatchFolder>
│       ├── input: PathBuf
│       ├── output: PathBuf
│       ├── preset: String
│       ├── enabled: bool
│       └── settings: FolderSettings
│           ├── enhance_audio: Option<bool>
│           ├── remove_silence: Option<bool>
│           ├── silence_threshold_db: Option<f32>
│           └── ...
└── processing: ProcessingConfig
    └── join_mode: Off | ByDate | ByName | AfterCount
```

---

## Completed Phases

### Phase 1: UI Polish ✅
- [x] Two-line folder cards
- [x] Remove Load/Save buttons
- [x] Auto-save on changes
- [x] Per-folder settings
- [x] Pill-style folder selector
- [x] Filled track sliders
- [x] Bigger fonts
- [x] Delete confirmation modal
- [x] Default folder paths by preset

### Phase 2: Unified Binary ✅
- [x] Single binary with `--gui` flag
- [x] Desktop notifications (`--notify`)
- [x] Fixed ML stubs (face detection, segmentation)

### Phase 3: Integration Tests ✅
- [x] ML feature tests
- [x] Video processing pipeline tests

### Phase 4: Desktop Integration ✅
- [x] Application icon (SVG)
- [x] Desktop entry for Linux app menu
- [x] Install script handles icon/desktop installation
- [x] GUI launches by default when not from terminal
- [x] Window app ID for desktop tracking

---

## Future Roadmap

### Phase 4: Watcher Integration
- [x] CLI watches `[[paths.watch_folders]]` from config (same as GUI)
- [x] Timestamped progress output during processing
- [x] Config auto-loading from `~/.config/ai-vid-editor/config.toml`
- [ ] Spawn watcher threads from GUI
- [ ] Real-time activity log updates in GUI
- [ ] Processing queue visualization

### Phase 5: Custom Presets
- [ ] Save folder settings as new preset
- [ ] Preset management UI (rename, delete)
- [ ] Preset library in `~/.config/ai-vid-editor/presets/`

### Phase 6: UX Enhancements
- [ ] Drag-drop folder paths
- [ ] File browser integration
- [ ] Keyboard shortcuts
- [ ] Search/filter folders

---

## Architecture

### Module Structure

```
src/
├── main.rs           # Entry point (CLI + GUI)
├── lib.rs            # Library exports
├── gui.rs            # GUI components
├── gui/
│   └── theme.rs      # Styling constants
├── config.rs         # Configuration structs
├── analyzer.rs       # Silence detection
├── editor.rs         # FFmpeg edit commands
├── batch_processor.rs# Batch/watch processing
├── exporter.rs       # Export formats
├── ml.rs             # Face detection, segmentation
├── stt_analyzer.rs   # Whisper STT
└── utils.rs          # Utilities
```

### Data Flow

```
GUI                    CLI
  │                      │
  ▼                      ▼
AppState              Config
  │                      │
  ├── folders[]         ├── input/output paths
  ├── settings          ├── processing config
  └── activity_log      └── preset
  │
  ▼ (on change)
auto_save_config()
  │
  ▼
TOML file (~/.config/ai-vid-editor/config.toml)
```

### Config File Locations

1. `./ai-vid-editor.toml` (project-local, highest priority)
2. `~/.config/ai-vid-editor/config.toml` (user-global, **auto-loaded**)
3. `--config <path>` (explicit, overrides both)

The global config (`~/.config/ai-vid-editor/config.toml`) is loaded automatically by both CLI and GUI — no `--config` flag needed. The `[[paths.watch_folders]]` section is shared between both modes.

---

## Design Principles

1. **Automation-first**: No manual mode, everything is watch-based
2. **Minimal clicks**: Changes auto-save, no explicit save button
3. **Clear hierarchy**: Tabs → Sections → Controls
4. **Dark theme**: Black background (#101010), red accents (#e63946)
5. **No color overload**: Primary palette is black/white/red only
6. **Compact but readable**: Clear labels, consistent spacing

---

## Style Guide

### Colors

| Name | Hex | Usage |
|------|-----|-------|
| PANEL_BG | #101010 | Main background |
| PANEL_BG_LIGHT | #181818 | Cards, inputs |
| PANEL_BG_LIGHTER | #222222 | Hover states |
| ACCENT_PRIMARY | #e63946 | Buttons, highlights |
| ACCENT_DARK | #b42d37 | Button borders |
| TEXT_PRIMARY | #fafafa | Main text |
| TEXT_SECONDARY | #aaaaaa | Labels |
| TEXT_MUTED | #646464 | Disabled text |
| SUCCESS | #b45a5f | Status badges |
| ERROR | #ff4444 | Error states |

### Typography

| Element | Size | Weight |
|---------|------|--------|
| Panel title | 18px | Bold |
| Primary text | 16px | Normal |
| Secondary text | 15px | Normal |
| Muted text | 14px | Normal |

### Spacing

| Element | Value |
|---------|-------|
| Panel padding | 20px |
| Card padding | 14px |
| Section gap | 12-16px |
| Control gap | 6-12px |

---

## Build Commands

```bash
# Development
cargo run                    # Launch GUI (or CLI with args)
cargo run -- -i in.mp4 ...   # CLI mode

# Release build
cargo build --release

# Run tests
cargo test --lib

# Install (with desktop integration)
./install.sh --user          # User install
sudo ./install.sh            # System install

# Using just
just gui
just build
just test
```

---

## File Structure

```
ai-vid-editor/
├── Cargo.toml
├── README.md
├── justfile
├── install.sh               # Install script with desktop integration
├── assets/
│   ├── icon.svg             # Application icon
│   └── ai-vid-editor.desktop # Desktop entry
├── plans/
│   └── project-specs.md     # This file
├── presets/
│   ├── youtube.toml
│   ├── shorts.toml
│   ├── podcast.toml
│   └── minimal.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── gui.rs
│   ├── gui/
│   │   └── theme.rs
│   ├── config.rs
│   ├── analyzer.rs
│   ├── editor.rs
│   ├── batch_processor.rs
│   ├── exporter.rs
│   ├── ml.rs
│   ├── stt_analyzer.rs
│   └── utils.rs
├── tests/
│   ├── common/mod.rs
│   ├── ml_integration.rs
│   └── pipeline_integration.rs
└── ai-vid-editor.example.toml  # Example config file
```
