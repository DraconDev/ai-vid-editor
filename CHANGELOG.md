# Changelog

## 0.1.466

### Watch Mode: CLI + GUI Parity
- CLI now auto-loads `~/.config/ai-vid-editor/config.toml` — no `--config` flag needed
- CLI reads `[[paths.watch_folders]]` from config, same as GUI
- Running `ai-vid-editor` with no args starts multi-folder watch mode if configured
- Per-folder presets and settings overrides work from CLI

### Progress Feedback
- Timestamped output during processing: `[NEW FILE]`, `[START]`, `[XX%]` stage updates, `[DONE]` with elapsed time
- Heartbeat message every ~30s so users know the watcher is alive
- Initial scan lists existing files that are skipped

### Audio Enhancement
- Replaced single-pass loudnorm with **two-pass** (measures audio first, then applies correction with `linear=true`)
  - Preserves natural speech dynamics instead of crushing them
- Removed harsh EQ peaks (1kHz +2dB, 3kHz +3dB narrow bandwidth)
  - Replaced with gentle wide-band +1.5dB at 1.5kHz for natural presence
- Added high-pass filter at 80Hz (removes low rumble)
- Added low-pass filter at 12kHz (removes sibilant hiss)

### Config Merge Fix
- `Config::merge()` now properly merges `paths.*`, `watch_folders`, `watch`, and `video` fields
  - Previously these were silently ignored, causing defaults to persist over config values

## 0.1.424 (Previous Release)

See `release/0.1.424/docs/` for prior release documentation.
