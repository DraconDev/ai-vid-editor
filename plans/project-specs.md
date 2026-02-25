# AI Video Editor - Project Specifications

## Overview

**Purpose**: Automation-first video editor for content creators. Drop in raw footage, get polished results without manual editing.

**Architecture**:
- CLI: Full-featured command-line interface for scripting/automation
- GUI: Visual management interface built with egui
- Backend: FFmpeg for video processing, Candle for ML (Whisper STT)

**Tech Stack**:
- Rust (edition 2024)
- egui/wgpu for GUI
- FFmpeg (external dependency)
- Candle (Whisper STT, face detection)
- TOML for configuration

---

## Current State

### Working Features

| Feature | CLI | GUI | Notes |
|---------|-----|-----|-------|
| Silence detection | ✅ | - | Core feature |
| Silence trimming | ✅ | - | Cut or speedup modes |
| Audio enhancement | ✅ | ✅ | Loudnorm + EQ |
| Noise reduction | ✅ | ✅ | FFmpeg afftdn |
| Music mixing | ✅ | - | Auto-ducking |
| Intro/outro | ✅ | - | Prepend/append videos |
| Video stabilization | ✅ | ✅ | vidstab filter |
| Color correction | ✅ | ✅ | Contrast, saturation, sharpen |
| Auto-reframe | ✅ | ✅ | 9:16 with face tracking |
| Background blur | ✅ | ✅ | Portrait mode effect |
| Filler word removal | ✅ | - | Whisper-based |
| Presets | ✅ | ✅ | youtube, shorts, podcast, minimal |
| Watch mode | ✅ | - | Auto-process new videos |
| Batch processing | ✅ | - | Directory processing |
| Export (SRT, FCPXML, EDL) | ✅ | - | Subtitles, timelines |

### GUI Structure

```
┌─────────────────────────────────────────────┐
│ █ AI Video Processor                        │  ← Accent bar + title
│ [All] [Folders] [Settings] [Activity]       │  ← Tab navigation
├─────────────────────────────────────────────┤
│ Watch Folders              ● Watching       │  ← Folder list
│ ┌─────────────────────────────────────────┐ │
│ │ [ON]                                     │ │
│ │     Input:  videos                       │ │
│ │     Output: videos/output         youtube│ │
│ └─────────────────────────────────────────┘ │
│ [+ Add Folder]                              │
├─────────────────────────────────────────────┤
│ Settings                                    │  ← Per-folder settings
│ [Folder selector]                           │
│ [x] Enhance Audio    [x] Remove Silence     │
│ Silence Threshold: ━━━━○━━━ -30 dB          │
├─────────────────────────────────────────────┤
│ Activity Log                        [Clear] │  ← Processing log
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

## UI Improvements

### 1. Pill-Style Folder Selector

**Current**: Dropdown/ComboBox
```
[ Configure Folder ▼ ]
```

**New**: Horizontal pill buttons
```
[ 1. videos ] [ 2. shorts ] [ 3. podcast ]
   ^selected    inactive       inactive
```

**Implementation**:
- Use `button_toggle()` style for each folder
- Wrap in horizontal scroll if too many folders
- Show folder number + truncated input path

### 2. Filled Track Sliders

**Current**: Default egui slider (thin line, small handle)

**New**: Filled track with accent color
```
Silence Threshold (dB)
███████████████████░░░░░░░○░░░  -30
-60                              -10
```

**Implementation**:
- Custom slider widget with filled background
- Use ACCENT_PRIMARY for fill color
- Add tick marks at range endpoints

### 3. Theme Polish

| Element | Current | New |
|---------|---------|-----|
| Folder selector | ComboBox | Pill buttons |
| Sliders | Default | Filled track |
| Section separators | `---` text | Subtle line |
| Hover states | None | Lighter background |

---

## Future Roadmap

### Phase 1: UI Polish (Current)
- [x] Two-line folder cards
- [x] Remove Load/Save buttons
- [x] Auto-save on changes
- [x] Per-folder settings
- [ ] Pill-style folder selector
- [ ] Filled track sliders
- [ ] Hover states for cards

### Phase 2: Watcher Integration
- [ ] Spawn watcher threads from GUI
- [ ] Real-time activity log updates
- [ ] Progress indicators per folder
- [ ] Processing queue visualization

### Phase 3: Custom Presets
- [ ] Save folder settings as new preset
- [ ] Preset management UI (rename, delete)
- [ ] Preset library in `~/.config/ai-vid-editor/presets/`

### Phase 4: Notifications
- [ ] Desktop notifications on complete/error
- [ ] Sound alerts (optional)
- [ ] System tray integration

### Phase 5: UX Enhancements
- [ ] Drag-drop folder paths
- [ ] File browser integration
- [ ] Keyboard shortcuts
- [ ] Search/filter folders

---

## Architecture

### Module Structure

```
src/
├── main.rs           # CLI entry point
├── gui_main.rs       # GUI entry point
├── gui.rs            # GUI components
├── gui/
│   └── theme.rs      # Styling constants
├── config.rs         # Configuration structs
├── analyzer.rs       # Silence detection
├── editor.rs         # FFmpeg edit commands
├── batch_processor.rs# Batch/watch processing
├── exporter.rs       # Export formats
├── ml.rs             # Face detection
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
2. `~/.config/ai-vid-editor/config.toml` (user-global)
3. `--config <path>` (explicit)

### Preset Files

Location: `presets/*.toml` (bundled) or `~/.config/ai-vid-editor/presets/*.toml` (custom)

---

## Design Principles

1. **Automation-first**: No manual mode, everything is watch-based
2. **Minimal clicks**: Changes auto-save, no explicit save button
3. **Clear hierarchy**: Tabs → Sections → Controls
4. **Dark theme**: Black background (#101010), red accents (#e63946)
5. **No color overload**: Primary palette is black/white/red only
6. **Compact but readable**: Small fonts, tight spacing, but clear labels

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
| SUCCESS | #b45a5f | Status badges (muted red) |
| ERROR | #ff4444 | Error states |

### Typography

| Element | Size | Weight |
|---------|------|--------|
| Panel title | 18px | Bold |
| Section label | 12px | Normal (muted) |
| Primary text | 14px | Normal |
| Secondary text | 13px | Normal |
| Muted text | 12px | Normal |
| Badge text | 11px | Bold |

### Spacing

| Element | Value |
|---------|-------|
| Panel padding | 20px |
| Card padding | 12px |
| Section gap | 16px |
| Control gap | 12px |
| Label gap | 4px |

### Corner Radius

| Element | Value |
|---------|-------|
| Panels | 12px |
| Buttons, cards | 8px |
| Pills | 24px |
| Badges | 4px |

---

## Build Commands

```bash
# Development
cargo run --features gui --bin ai-vid-editor-gui

# Release build
cargo build --release --features gui

# Run tests
cargo test

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
├── plans/
│   └── project-specs.md        # This file
├── presets/
│   ├── youtube.toml
│   ├── shorts.toml
│   ├── podcast.toml
│   └── minimal.toml
├── src/
│   ├── main.rs
│   ├── gui_main.rs
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
└── ai-vid-editor.example.toml
```
