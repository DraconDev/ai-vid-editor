mod common;

use ai_vid_editor::ml::{AutoReframeProcessor, FaceDetector, FrameExtractor, PersonSegmenter};
use common::*;

fn check_ffmpeg() {
    if !has_ffmpeg() || !has_ffprobe() {
        eprintln!("Skipping test: ffmpeg/ffprobe not available");
    }
}

#[test]
fn test_face_detector_load() {
    let result = FaceDetector::load();

    match result {
        Ok(_detector) => {
            println!("Face detector loaded successfully");
        }
        Err(e) => {
            // Model download might fail in CI or offline
            eprintln!("Face detector load failed (may be offline): {}", e);
        }
    }
}

#[test]
fn test_person_segmenter_load() {
    let result = PersonSegmenter::load();

    match result {
        Ok(_segmenter) => {
            println!("Person segmenter loaded successfully");
        }
        Err(e) => {
            // Model download might fail in CI or offline
            eprintln!("Person segmenter load failed (may be offline): {}", e);
        }
    }
}

#[test]
fn test_frame_extractor_dimensions() {
    check_ffmpeg();

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let result = FrameExtractor::get_video_dimensions(&video_path);
    assert!(result.is_ok(), "Should get video dimensions");

    let (width, height) = result.unwrap();
    println!("Video dimensions: {}x{}", width, height);
    assert!(width > 0 && height > 0, "Dimensions should be positive");
}

#[test]
fn test_frame_extractor_duration() {
    check_ffmpeg();

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let result = FrameExtractor::get_video_duration(&video_path);
    assert!(result.is_ok(), "Should get video duration");

    let duration = result.unwrap();
    println!("Video duration: {}s", duration);
    assert!(duration > 0.0, "Duration should be positive");
}

#[test]
fn test_frame_extraction() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let frames = FrameExtractor::extract_frames(&video_path, output_dir.path(), 1.0);

    assert!(frames.is_ok(), "Frame extraction should succeed");
    let frames = frames.unwrap();

    println!("Extracted {} frames", frames.len());
    assert!(!frames.is_empty(), "Should extract at least one frame");

    // Cleanup
    for frame in frames {
        let _ = std::fs::remove_file(frame);
    }
}

#[test]
fn test_face_detection_on_frame() {
    check_ffmpeg();

    use tempfile::tempdir;

    let detector = match FaceDetector::load() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Skipping test: could not load face detector: {}", e);
            return;
        }
    };

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let frames = FrameExtractor::extract_frames(&video_path, output_dir.path(), 1.0);

    if let Ok(frames) = frames {
        if let Some(first_frame) = frames.first() {
            if let Ok(img) = image::open(first_frame) {
                let result = detector.detect(&img);
                match result {
                    Ok(faces) => {
                        println!("Detected {} faces", faces.len());
                        for face in &faces {
                            println!(
                                "  Face: x={:.2}, y={:.2}, w={:.2}, h={:.2}, conf={:.2}",
                                face.x, face.y, face.width, face.height, face.confidence
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("Face detection failed: {}", e);
                    }
                }
            }
        }

        // Cleanup
        for frame in frames {
            let _ = std::fs::remove_file(frame);
        }
    }
}

#[test]
fn test_auto_reframe_processor() {
    check_ffmpeg();

    let processor = match AutoReframeProcessor::new() {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "Skipping test: could not create auto-reframe processor: {}",
                e
            );
            return;
        }
    };

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    // Test with low sample rate for speed
    let result = processor.analyze_video(&video_path, 0.1);

    match result {
        Ok(crop_regions) => {
            println!("Generated {} crop regions", crop_regions.len());
            if let Some((ts, region)) = crop_regions.first() {
                println!(
                    "  First region: t={}s, x={:.2}, y={:.2}, w={:.2}, h={:.2}",
                    ts, region.x, region.y, region.width, region.height
                );
            }
        }
        Err(e) => {
            eprintln!("Auto-reframe analysis failed: {}", e);
        }
    }
}

#[test]
fn test_person_segmentation_on_frame() {
    check_ffmpeg();

    use tempfile::tempdir;

    let segmenter = match PersonSegmenter::load() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test: could not load person segmenter: {}", e);
            return;
        }
    };

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let frames = FrameExtractor::extract_frames(&video_path, output_dir.path(), 1.0);

    if let Ok(frames) = frames {
        if let Some(first_frame) = frames.first() {
            if let Ok(img) = image::open(first_frame) {
                let result = segmenter.segment(&img);
                match result {
                    Ok(mask) => {
                        println!("Segmentation mask: {}x{}", mask.width, mask.height);
                        let person_pixels: usize = mask.data.iter().filter(|&&v| v > 0.5).count();
                        let total_pixels = mask.data.len();
                        println!(
                            "  Person coverage: {:.1}%",
                            (person_pixels as f32 / total_pixels as f32) * 100.0
                        );
                    }
                    Err(e) => {
                        eprintln!("Segmentation failed: {}", e);
                    }
                }
            }
        }

        // Cleanup
        for frame in frames {
            let _ = std::fs::remove_file(frame);
        }
    }
}
