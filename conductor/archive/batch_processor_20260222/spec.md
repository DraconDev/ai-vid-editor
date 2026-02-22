# Track Spec: batch_processor_20260222
**Description:** Implement robust batch processing engine for automated video trimming.

## Context
Extend the AI Video Editor (CLI) to support processing multiple video files in a single execution. This enables users to automate silence trimming across entire directories of content.

## Requirements
- **Input Directory:** Support specifying a directory path as input.
- **File Discovery:** Automatically find all supported video files (MP4, MOV, AVI) within the input directory.
- **Output Directory:** Support specifying an output directory where processed videos will be saved.
- **Silence Trimming Integration:** Apply the existing silence detection and trimming logic to each discovered file.
- **Progress Reporting:** Provide clear feedback on which file is being processed and the overall batch progress.
- **Error Handling:** Gracefully handle individual file failures without halting the entire batch.

## Success Criteria
- [ ] The CLI correctly identifies all video files in a directory.
- [ ] Each file is processed using the silence trimmer logic.
- [ ] Processed files are saved to the correct output directory.
- [ ] The tool provides a summary of the batch operation (e.g., "X files processed, Y skipped").
- [ ] Code coverage for the batch logic is >80%.
