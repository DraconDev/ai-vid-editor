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
- [x] Noise reduction - `--noise-reduction`
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
- [x] Project mode - `--project <dir>`
- [x] Minimal CLI (config-first architecture)

### Installation
- [x] Install script (`install.sh`)
- [x] User install (`./install.sh --user`)
- [x] System install (`sudo ./install.sh`)
- [x] Systemd service support

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

## 🚧 Phase 1: Easy Wins (ffmpeg-native)

### Video Stabilization
- [ ] Add `--stabilize` flag
- [ ] Use ffmpeg `vidstab` filter (two-pass)
- [ ] Add to config: `[video] stabilize = true`

### Auto-Color Correction
- [ ] Add `--color-correct` flag
- [ ] Use ffmpeg `eq` filter for auto-levels
- [ ] Add to config: `[video] color_correct = true`

### Tests
- [ ] Integration tests for silence detection
- [ ] Integration tests for audio enhancement
- [ ] Integration tests for batch processing
- [ ] Performance benchmarks

---

## 📋 Phase 2: Thumbnail Guide from Transcript

### Thumbnail Guide Export
- [ ] Add `--export-thumbnail-guide` flag
- [ ] Analyze transcript for highlight moments
- [ ] Detect hooks (questions, bold statements)
- [ ] Detect chapter starts
- [ ] Extract key quotes for overlay text
- [ ] Generate title suggestions
- [ ] Output as JSON

Example output (`thumbnail-guide.json`):
```json
{
  "title_suggestions": [
    "How to Edit Videos Like a Pro",
    "3 Secrets to Viral Content"
  ],
  "thumbnail_moments": [
    {"time": 12.5, "text": "The secret is...", "type": "hook"},
    {"time": 45.0, "text": "Watch this!", "type": "action"},
    {"time": 120.0, "text": "The results were amazing", "type": "climax"}
  ],
  "key_quotes": [
    "This one trick changed everything",
    "Most people don't know this"
  ]
}
```

---

## 📋 Phase 3: ML Features (Model Size Aware)

### Face Detection (Required for reframe/blur)
- [ ] Research MediaPipe vs tract+ONNX
- [ ] Add face detection dependency (~10MB)
- [ ] Implement face detection in video frames

### Auto-Reframe (Horizontal → Vertical)
- [ ] Crop 16:9 to 9:16 following speaker's face
- [ ] Smooth camera movement between faces
- [ ] Add `--reframe` flag

### Background Blur
- [ ] Person segmentation (MODNet ~25MB)
- [ ] Blur background while keeping speaker sharp
- [ ] Add `--blur-background` flag

---

## ❌ Not Doing (Too Heavy)

### Eye Contact Correction
- Requires ~100MB+ model
- Complex pixel manipulation
- Not worth the cost

### Large Language Models
- GB-sized models
- Out of scope for this tool

### Thumbnail Generation
- Not our arena
- We provide the guide, user creates the thumbnail

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

## Competitor Research

| Feature | Descript | Gling | Adobe Podcast | Auphonic | Us |
|---------|----------|-------|---------------|----------|-----|
| Silence removal | ✅ | ✅ | ✅ | ✅ | ✅ |
| Filler removal | ✅ | ✅ | ✅ | ✅ | ✅ |
| Audio enhancement | ✅ | ❌ | ✅ | ✅ | ✅ |
| Noise reduction | ✅ | ❌ | ✅ | ✅ | ✅ |
| Video stabilization | ✅ | ❌ | ❌ | ❌ | TODO |
| Auto-color correction | ✅ | ❌ | ❌ | ❌ | TODO |
| Thumbnail guide | ❌ | ✅ | ❌ | ❌ | TODO |
| Auto-reframe | ✅ | ✅ | ❌ | ❌ | LATER |
| Background blur | ✅ | ❌ | ❌ | ❌ | LATER |
| Eye contact correction | ✅ | ✅ | ❌ | ❌ | NO |

---

## Model Size Reference

| Feature | Model | Size | Verdict |
|---------|-------|------|---------|
| Whisper STT | Whisper Tiny | ~75MB | ✅ Already using |
| Face detection | MediaPipe | ~10MB | ✅ OK to add |
| Person segmentation | MODNet | ~25MB | ✅ OK to add |
| Background blur | Uses segmentation | ~25MB | ✅ OK to add |
| Auto-reframe | Uses face detection | ~10MB | ✅ OK to add |
| Eye contact | Custom model | ~100MB+ | ⚠️ Too heavy |
| LLM for titles | Various | GBs | ❌ No |

---

## Configuration

Save as `ai-vid-editor.toml` in project root or `~/.config/ai-vid-editor/config.toml`:

```toml
[paths]
input_dir = "watch"
output_dir = "output"
music_dir = "music"

[silence]
threshold_db = -30.0
min_duration = 0.5
padding = 0.1
mode = "cut"
speedup_factor = 4.0

[filler_words]
enabled = false
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
thumbnail_guide = false

[watch]
enabled = false
interval = 5
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
  --music-dir ./music \
  --stabilize \
  --color-correct

# Batch processing
ai-vid-editor -I ./raw_videos -O ./edited --preset youtube

# Watch mode (daemon)
ai-vid-editor --watch ./incoming -O ./processed --preset youtube

# Preview
ai-vid-editor -i input.mp4 --dry-run

# Generate thumbnail guide
ai-vid-editor -i input.mp4 -o output.mp4 --export-thumbnail-guide