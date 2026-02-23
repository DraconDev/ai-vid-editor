# AI Video Editor - Feature Status & Roadmap

## ✅ Fully Working

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
- [x] SRT subtitle export - `--export-srt`
- [x] YouTube chapters export - `--export-chapters`

### CLI Features
- [x] Preset profiles - `--preset youtube/shorts/podcast/minimal`
- [x] Dry run mode - `--dry-run` to preview without processing
- [x] JSON output - `--json` for scripting/automation
- [x] Watch folder mode - `--watch <dir>` daemon mode
- [x] Config file support - `--config <file>`

### Whisper STT
- [x] Model loading from HuggingFace Hub
- [x] Audio extraction via ffmpeg
- [x] Mel spectrogram calculation
- [x] Greedy decode loop

### Filler Word Removal
- [x] Config structure with filler words list
- [x] `calculate_keep_segments_from_transcript()`
- [x] CLI flag `--remove-fillers`
- [x] Safe defaults: `um`, `uh`, `ah`, `er` (no "like")

---

## 🚧 In Progress

### Phase 1: Quick Fixes
- [ ] Fix `sample_rate` unused variable warning
- [ ] Show help when no args provided (instead of error)

### Phase 2: Noise Reduction
- [ ] Add `--noise-reduction` flag using ffmpeg `afftdn`

### Phase 3: Project System
- [ ] `--project <dir>` flag to load project config
- [ ] Auto-detect subfolders: watch/, output/, music/
- [ ] Project-level intro/outro

### Phase 4: Video Joining
- [ ] `--join` flag to concatenate multiple input files

---

## 📋 Later (Complex Features)

### Face Detection
- [ ] Research best approach for Rust (OpenCV vs mediapipe vs ffmpeg)
- [ ] Implement face detection

### Background Blur
- [ ] Detect faces, blur background
- [ ] Requires face detection

### Auto-Reframe
- [ ] Horizontal → Vertical crop following speaker
- [ ] Requires face detection

### Auto-Zoom
- [ ] Zoom in on speaker during active speech
- [ ] Requires face detection + timing

---

## ❌ Not Doing

### Parallel Processing
- FFmpeg already uses multiple threads internally
- Running multiple processes = resource contention
- Not worth the complexity

### Webhooks/Notifications
- Can just output to folder
- User can script their own notifications

### Burned-in Captions
- SRT export is sufficient
- Styling is manual work

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
words = ["um", "uh", "ah", "er"]
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

## Competitor Research

| Feature | Descript | Adobe Podcast | Auphonic | Gling | Us |
|---------|----------|---------------|----------|-------|-----|
| Silence removal | ✅ | ✅ | ✅ | ✅ | ✅ |
| Filler removal | ✅ | ✅ | ✅ | ✅ | ✅ |
| Audio enhancement | ✅ | ✅ | ✅ | ❌ | ✅ |
| Noise reduction | ✅ | ✅ | ✅ | ❌ | TODO |
| Auto-captions | ✅ | ❌ | ❌ | ✅ | ✅ |
| Background blur | ✅ | ❌ | ❌ | ❌ | LATER |
| Auto-reframe | ✅ | ❌ | ❌ | ✅ | LATER |