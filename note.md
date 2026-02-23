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
- [x] **Video stabilization** - `--stabilize` (ffmpeg vidstab, two-pass)
- [x] **Auto color correction** - `--color-correct` (contrast, brightness, saturation, sharpening)

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

## 📋 Phase 3: ML Features (Future)

### Cost Analysis

| Feature | Model | Disk Size | Memory (when used) | Quality |
|---------|-------|-----------|-------------------|---------|
| Face detection | MediaPipe | ~10MB | ~100MB | ⭐⭐⭐⭐⭐ |
| Auto-reframe | Uses face detection | ~10MB | ~100MB | ⭐⭐⭐⭐ |
| Background blur | MODNet/U²-Net | ~25MB | ~200MB | ⭐⭐⭐⭐ |

**Key points:**
- Models are **lazy loaded** - memory only used when feature is enabled
- If user doesn't use `--reframe` or `--blur-background`, no extra memory
- Total disk impact: ~35MB for all ML models
- Uses `tract` (pure Rust ONNX runtime) - no external dependencies

### Auto-Reframe (Horizontal → Vertical)
- [x] Add face detection dependency (`tract-onnx`)
- [x] Add `--reframe` CLI flag
- [x] Add config option `video.reframe`
- [ ] Download MediaPipe face model from HuggingFace
- [ ] Implement frame extraction and face tracking
- [ ] Crop 16:9 to 9:16 following speaker's face
- [ ] Smooth camera movement between faces

### Background Blur
- [x] Add person segmentation dependency (`tract-onnx`)
- [x] Add `--blur-background` CLI flag
- [x] Add config option `video.blur_background`
- [ ] Download MODNet model from HuggingFace
- [ ] Implement frame extraction and segmentation
- [ ] Blur background while keeping speaker sharp

---

## ❌ Not Doing

### Eye Contact Correction
- Requires ~100MB+ model
- Complex pixel manipulation
- Not worth the cost

### Large Language Models
- GB-sized models
- Out of scope for this tool
- Note: Whisper is NOT an LLM (it's speech-to-text, ~75MB)

### Thumbnail Generation
- Not our arena
- We provide transcript via SRT, user creates thumbnail

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
| Video stabilization | ✅ | ❌ | ❌ | ❌ | ✅ |
| Auto-color correction | ✅ | ❌ | ❌ | ❌ | ✅ |
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