# AI Video Editor - Feature Status

## ✅ Fully Working (Automated End-to-End)

### Silence Detection & Trimming
- [x] Silence detection via ffmpeg `silencedetect`
- [x] Automatic trimming of silent segments
- [x] Configurable threshold (dB), min duration, and padding
- [x] Speedup mode - speed through silences instead of cutting
- [x] TOML configuration file support

### Video Processing
- [x] Single file processing
- [x] Batch directory processing
- [x] Support for MP4, MOV, AVI, MKV, WebM formats

### Audio
- [x] Audio enhancement (loudnorm + EQ) - `--enhance`
- [x] Loudness normalization targeting -14 LUFS (YouTube standard)
- [x] Music mixing with auto-ducking - `--music <file>`
- [x] Music library folder - `--music-dir <dir>` (picks random track)

### Video Concatenation
- [x] Intro video support - `--intro <file>`
- [x] Outro video support - `--outro <file>`

### Export
- [x] FCPXML export for DaVinci Resolve / Premiere Pro - `--export-fcpxml`
- [x] EDL (Edit Decision List) export - `--export-edl`
- [x] SRT subtitle export - `--export-srt` (placeholder, needs STT)
- [x] YouTube chapters export - `--export-chapters`

### CLI Features
- [x] Preset profiles - `--preset youtube/shorts/podcast/minimal`
- [x] Dry run mode - `--dry-run` to preview without processing
- [x] JSON output - `--json` for scripting/automation
- [x] Watch folder mode - `--watch <dir>` daemon mode
- [x] Config file support - `--config <file>`

---

## ✅ Now Working

### Whisper STT
- [x] Model loading from HuggingFace Hub
- [x] Audio extraction via ffmpeg
- [x] Mel spectrogram calculation
- [x] Greedy decode loop

### Filler Word Removal
- [x] Config structure with filler words list
- [x] `calculate_keep_segments_from_transcript()`
- [x] CLI flag `--remove-fillers`

---

## Future Features (Automatable)

| Feature | Priority | Notes |
|---------|----------|-------|
| **Whisper STT** | HIGH | Unlocks subtitles + filler removal |
| **Filler word removal** | HIGH | Depends on STT |
| **Progress bar** | LOW | UX improvement |
| **GPU acceleration** | LOW | Speed improvement |

---

## Configuration

Save as `ai-vid-editor.toml` in project root or `~/.config/ai-vid-editor/config.toml`:

```toml
[silence]
threshold_db = -30.0
min_duration = 0.5
padding = 0.1
mode = "cut"
speedup_factor = 4.0
min_silence_for_speedup = 0.5

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

---

## CLI Usage

```bash
# Basic usage
ai-vid-editor -i input.mp4 -o output.mp4

# With preset
ai-vid-editor -i input.mp4 -o output.mp4 --preset youtube

# Full pipeline
ai-vid-editor -i input.mp4 -o output.mp4 \
  --preset youtube \
  --intro intro.mp4 \
  --outro outro.mp4 \
  --music-dir ./music

# Batch processing
ai-vid-editor -I ./raw_videos -O ./edited --preset youtube

# Watch mode
ai-vid-editor --watch ./incoming -O ./processed --preset youtube

# Preview
ai-vid-editor -i input.mp4 --dry-run
```

---

## Priority TODO

1. **Complete Whisper STT** - Implement mel spectrogram + decode loop
2. **Wire filler word removal** - Add `--remove-fillers` flag