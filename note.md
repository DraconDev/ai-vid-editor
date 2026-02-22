# AI Video Editor - Feature Status

## ✅ Implemented

### Silence Detection & Trimming
- [x] Silence detection via ffmpeg `silencedetect`
- [x] Automatic trimming of silent segments
- [x] Configurable threshold (dB), min duration, and padding
- [x] Speedup mode - speed through silences instead of cutting
- [x] TOML configuration file support

### Video Processing
- [x] Single file processing
- [x] Batch directory processing
- [x] Support for MP4, MOV, AVI formats

### Audio
- [x] Audio enhancement (loudnorm + EQ) - `--enhance` flag
- [x] Loudness normalization targeting -14 LUFS (YouTube standard)
- [x] Music mixing with auto-ducking - `--music <file>` flag

### Export
- [x] FCPXML export for DaVinci Resolve / Premiere Pro - `--export-fcpxml`
- [x] EDL (Edit Decision List) export - `--export-edl`
- [x] SRT subtitle export - `--export-srt` (placeholder, needs STT)
- [x] YouTube chapters export - `--export-chapters` (placeholder, needs STT)

---

## 🔧 Partially Implemented (Needs Work)

### STT / Whisper Integration
- [x] Model loading from HuggingFace Hub
- [x] Audio extraction via ffmpeg
- [ ] **TODO: Mel spectrogram calculation** (placeholder exists)
- [ ] **TODO: Decode loop (greedy/beam search)** (returns dummy text)

### Filler Word Removal
- [x] Config structure with filler words list
- [x] `calculate_keep_segments_from_transcript()` implemented
- [ ] **TODO: Wire to CLI** (needs STT to work first)

---

## ❌ Not Yet Implemented

### CLI Flags Needed
- [ ] `--remove-fillers` - Enable filler word removal (needs STT)

### Pipeline Composition
- [ ] "Full auto" mode that chains all operations
- [ ] Preset profiles (e.g., "youtube-podcast", "tiktok-fast")

---

## Configuration

Save as `ai-vid-editor.toml` in project root or `~/.config/ai-vid-editor/config.toml`:

```toml
[silence]
threshold_db = -30.0        # Silence detection threshold
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
enhance = true
target_lufs = -14.0
# music_file = "/path/to/music.mp3"
duck_volume = 0.2

[export]
subtitles = false
chapters = false
fcpxml = false
edl = false
```

---

## CLI Usage

```bash
# Basic usage (cut silences)
ai-vid-editor -i input.mp4 -o output.mp4

# Speedup silences instead of cutting
ai-vid-editor -i input.mp4 -o output.mp4 --speedup

# Use config file
ai-vid-editor -i input.mp4 -o output.mp4 -c config.toml

# Override settings via CLI
ai-vid-editor -i input.mp4 -o output.mp4 --threshold -35 --padding 0.2

# Batch processing
ai-vid-editor -I ./raw_videos -O ./edited

# Generate sample config
ai-vid-editor --generate-config > ai-vid-editor.toml
```

---

## Priority TODO

1. **Complete Whisper STT** - Implement mel spectrogram + decode loop
2. **Wire filler word removal** - Add `--remove-fillers` flag (needs STT)
3. **Preset profiles** - Add "youtube-podcast", "tiktok-fast" presets
