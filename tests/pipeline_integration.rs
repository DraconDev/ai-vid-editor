mod common;

use ai_vid_editor::FfmpegAnalyzer;
use ai_vid_editor::FfmpegEditor;
use ai_vid_editor::analyzer::VideoAnalyzer;
use ai_vid_editor::editor::VideoEditor;
use common::*;

fn check_ffmpeg() {
    if !has_ffmpeg() || !has_ffprobe() {
        eprintln!("Skipping test: ffmpeg/ffprobe not available");
        return;
    }
}

#[test]
fn test_silence_detection() {
    check_ffmpeg();

    let analyzer = FfmpegAnalyzer;
    let video_path = test_video_path();

    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let result = analyzer.detect_silence(&video_path, -30.0, 0.5);
    assert!(result.is_ok(), "Silence detection should succeed");

    let silences = result.unwrap();
    println!("Detected {} silent segments", silences.len());
}

#[test]
fn test_silence_detection_threshold() {
    check_ffmpeg();

    let analyzer = FfmpegAnalyzer;
    let video_path = test_video_path();

    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    // Higher threshold should detect more silence
    let silences_high = analyzer.detect_silence(&video_path, -20.0, 0.5).unwrap();
    let silences_low = analyzer.detect_silence(&video_path, -50.0, 0.5).unwrap();

    println!(
        "Silences at -20dB: {}, at -50dB: {}",
        silences_high.len(),
        silences_low.len()
    );
    assert!(
        silences_high.len() >= silences_low.len(),
        "Higher threshold should detect equal or more silence"
    );
}

#[test]
fn test_audio_enhancement() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("enhanced.mp4");

    let editor = FfmpegEditor;
    let result = editor.enhance_audio(&video_path, &output_path, -14.0);

    assert!(result.is_ok(), "Audio enhancement should succeed");
    assert!(output_path.exists(), "Output file should exist");
}

#[test]
fn test_video_stabilization() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("stabilized.mp4");

    let editor = FfmpegEditor;
    let result = editor.stabilize(&video_path, &output_path);

    assert!(result.is_ok(), "Video stabilization should succeed");
    assert!(output_path.exists(), "Output file should exist");
}

#[test]
fn test_color_correction() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("corrected.mp4");

    let editor = FfmpegEditor;
    let result = editor.color_correct(&video_path, &output_path);

    assert!(result.is_ok(), "Color correction should succeed");
    assert!(output_path.exists(), "Output file should exist");
}

#[test]
fn test_auto_reframe() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("reframed.mp4");

    let editor = FfmpegEditor;
    let result = editor.reframe(&video_path, &output_path);

    // Note: This will use center crop if ML models fail to load
    assert!(
        result.is_ok(),
        "Auto-reframe should succeed (with or without ML)"
    );
    assert!(output_path.exists(), "Output file should exist");
}
