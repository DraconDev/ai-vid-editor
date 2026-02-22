# Track Plan: whisper_smart_edit_20260222

## Phase 1: High-Quality STT Engine Integration [checkpoint: ac74535]
- [x] Task: Research and select the most optimized Rust bindings for Whisper (e.g. `whisper-rs`) (0e6c383)
    - *Note: Switched to candle-whisper for pure-Rust implementation.*
- [x] Task: Implement `SttAnalyzer` module to generate timestamped transcripts (9a15947)
    - [x] Write Failing Tests (Red Phase)
    - [x] Implement to Pass Tests (Green Phase)
    - [x] Verify Coverage
- [x] Task: Conductor - User Manual Verification 'Phase 1: High-Quality STT Engine Integration' (Protocol in workflow.md) (ac74535)

## Phase 2: Intelligent Editing (Semantic Cuts)
- [x] Task: Implement filler word detection and automatic segment removal (um/uh) (587cd69)
    - [x] Write Failing Tests (Red Phase)
    - [x] Implement to Pass Tests (Green Phase)
    - [x] Verify Coverage
- [x] Task: Implement "Smart Speed-up" option for nonsense/low-confidence segments (69537d4)
    - [x] Write Failing Tests (Red Phase)
    - [x] Implement to Pass Tests (Green Phase)
    - [x] Verify Coverage
- [~] Task: Add configurable padding logic to all cuts
    - [ ] Write Failing Tests (Red Phase)
    - [ ] Implement to Pass Tests (Green Phase)
    - [ ] Verify Coverage
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Intelligent Editing (Semantic Cuts)' (Protocol in workflow.md)

## Phase 3: Smart Audio Mixing & Normalization
- [ ] Task: Implement STT-driven music ducking (leveraging transcript timestamps)
    - [ ] Write Failing Tests (Red Phase)
    - [ ] Implement to Pass Tests (Green Phase)
    - [ ] Verify Coverage
- [ ] Task: Integrate FFmpeg `loudnorm` and basic EQ for voice enhancement
    - [ ] Write Failing Tests (Red Phase)
    - [ ] Implement to Pass Tests (Green Phase)
    - [ ] Verify Coverage
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Smart Audio Mixing & Normalization' (Protocol in workflow.md)

## Phase 4: Professional Exports (XML/EDL/Chapters)
- [ ] Task: Implement DaVinci Resolve / Premiere compatible XML export
    - [ ] Write Failing Tests (Red Phase)
    - [ ] Implement to Pass Tests (Green Phase)
    - [ ] Verify Coverage
- [ ] Task: Implement basic EDL and Subtitle (SRT/VTT) export
    - [ ] Write Failing Tests (Red Phase)
    - [ ] Implement to Pass Tests (Green Phase)
    - [ ] Verify Coverage
- [ ] Task: Implement YouTube-ready Chapter Marker generation
    - [ ] Write Failing Tests (Red Phase)
    - [ ] Implement to Pass Tests (Green Phase)
    - [ ] Verify Coverage
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Professional Exports (XML/EDL/Chapters)' (Protocol in workflow.md)
