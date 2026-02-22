# AI Video Editor - Feature Status

## ✅ Implemented

### Silence Detection & Trimming
- [x] Silence detection via ffmpeg `silencedetect`
- [x] Automatic trimming of silent segments
- [x] Configurable threshold (dB), min duration, and padding
- [x] **NEW: Speedup mode** - speed through silences instead of cutting
- [x] **NEW: TOML configuration file support**

### Video Processing
- [x] Single file processing
- [x] Batch directory processing
- [x] Support for MP4, MOV, AVI formats

### Audio
- [x] Audio enhancement (loudnorm + EQ) - `enhance_audio()` implemented
- [x] Loudness normalization targeting -14 LUFS (YouTube standard)
- [x] Music ducking filter generation - `mix_with_music()` implemented

### Export
- [x] FCPXML export for DaVinci Resolve / Premiere Pro
- [x] EDL (Edit Decision List) export
- [x] SRT subtitle export
- [x] YouTube chapters export

---

## 🔧 Partially Implemented (Needs Wiring)

### STT / Whisper Integration
- [x] Model loading from HuggingFace Hub
- [x] Audio extraction via ffmpeg
- [ ] **TODO: Mel spectrogram calculation** (placeholder exists)
- [ ] **TODO: Decode loop (greedy/beam search)** (returns dummy text)

### Filler Word Removal
- [x] Config structure with filler words list
- [x] `calculate_keep_segments_from_transcript()` implemented
- [ ] **TODO: Wire to CLI** (needs STT to work first)

### Audio Mixing
- [x] `mix_with_music()` function implemented
- [x] Duck filter generation based on transcript
- [ ] **TODO: Wire to CLI** (needs `--music` flag)

### Audio Enhancement
- [x] `enhance_audio()` function implemented
- [ ] **TODO: Wire to CLI** (needs `--enhance` flag)

---

## ❌ Not Yet Implemented

### CLI Flags Needed
- [ ] `--enhance` - Enable audio enhancement
- [ ] `--music <file>` - Add background music with auto-ducking
- [ ] `--remove-fillers` - Enable filler word removal (needs STT)
- [ ] `--export-srt`, `--export-fcpxml`, `--export-chapters` flags

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

1. **Wire audio enhancement to CLI** - Add `--enhance` flag
2. **Wire music mixing to CLI** - Add `--music <file>` flag
3. **Complete Whisper STT** - Implement mel spectrogram + decode loop
4. **Wire filler word removal** - Add `--remove-fillers` flag
5. **Wire export options** - Add export flags to CLI