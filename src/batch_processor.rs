use std::path::{PathBuf, Path};
use anyhow::{Result, Context};
use std::fs;

use crate::analyzer::VideoAnalyzer;
use crate::editor::VideoEditor;
use crate::editor::calculate_keep_segments;
use crate::utils::find_video_files;
use crate::config::Config;

// Trait for getting video duration
pub trait DurationGetter {
    fn get_duration(&self, path: &Path) -> Result<f32>;
}

// Concrete implementation using ffprobe
pub struct FfmpegDurationGetter;

impl DurationGetter for FfmpegDurationGetter {
    fn get_duration(&self, path: &Path) -> Result<f32> {
        let output = std::process::Command::new("ffprobe")
            .args([
                "-v", "error",
                "-show_entries", "format=duration",
                "-of", "default=noprint_wrappers=1:nokey=1",
                path.to_str().context("invalid path")?,
            ])
            .output()
            .context("failed to execute ffprobe")?;

        let val_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        val_str.parse::<f32>().context("failed to parse duration")
    }
}


pub fn process_single_file<A, E, D>(
    input_file: PathBuf, 
    output_file: PathBuf, 
    config: &Config,
    analyzer: &A, 
    editor: &E, 
    duration_getter: &D
) -> Result<()>
where
    A: VideoAnalyzer,
    E: VideoEditor,
    D: DurationGetter,
{
    println!("Analyzing video: {:?}", input_file);
    println!("Silence mode: {:?}", config.silence.mode);
    
    let silences = analyzer.detect_silence(
        &input_file, 
        config.silence.threshold_db, 
        config.silence.min_duration
    ).context("Failed to detect silence")?;

    println!("Detected {} silent segments.", silences.len());

    let video_duration = duration_getter.get_duration(&input_file)?; 
    println!("Total duration: {}s", video_duration);

    let processed_segments = calculate_keep_segments(
        &silences, 
        video_duration, 
        config.silence.padding,
        config.silence.mode,
        config.silence.speedup_factor,
        config.silence.min_silence_for_speedup,
    );
    println!("Segments to process: {}", processed_segments.len());

    editor.trim_video(&input_file, &output_file, &processed_segments)
        .context("Failed to trim video")?;

    println!("Successfully saved trimmed video to: {:?}", output_file);
    Ok(())
}

pub fn process_batch_dir<A, E, D>(
    input_dir: PathBuf, 
    output_dir: PathBuf, 
    config: &Config,
    analyzer: &A, 
    editor: &E, 
    duration_getter: &D
) -> Result<()>
where
    A: VideoAnalyzer,
    E: VideoEditor,
    D: DurationGetter,
{
    println!("Processing directory: {:?}", input_dir);
    println!("Output directory: {:?}", output_dir);
    println!("Silence mode: {:?}", config.silence.mode);

    fs::create_dir_all(&output_dir)
        .context(format!("Failed to create output directory {:?}", output_dir))?;

    let video_files = find_video_files(&input_dir)?;

    if video_files.is_empty() {
        println!("No supported video files found in {:?}", input_dir);
        return Ok(());
    }

    let total_files = video_files.len();
    let mut successful_files = 0;
    let mut failed_files = 0;

    for (index, input_file) in video_files.iter().enumerate() {
        let file_name = input_file.file_name()
            .context(format!("Could not get file name for {:?}", input_file))?;
        let output_file = output_dir.join(file_name);

        println!("\n--- Processing file {}/{} : {:?} ---", index + 1, total_files, input_file);
        match process_single_file(input_file.clone(), output_file.clone(), config, analyzer, editor, duration_getter) {
            Ok(_) => {
                println!("Successfully processed {:?}", input_file);
                successful_files += 1;
            },
            Err(e) => {
                eprintln!("Error processing {:?}: {}", input_file, e);
                failed_files += 1;
            },
        }
    }

    println!("\n--- Batch Processing Summary ---");
    println!("Total files processed: {}", total_files);
    println!("Successful: {}", successful_files);
    println!("Failed: {}", failed_files);

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    // Mock implementations for testing without actual ffmpeg calls
    struct MockFfmpegAnalyzer;
    impl VideoAnalyzer for MockFfmpegAnalyzer {
        fn detect_silence(&self, _path: &Path, _threshold_db: f32, _duration_s: f32) -> Result<Vec<crate::analyzer::Segment>> {
            Ok(vec![]) // Simulate no silences detected for simplicity
        }
    }

    struct MockFfmpegEditor;
    impl VideoEditor for MockFfmpegEditor {
        fn trim_video(&self, _input: &Path, output: &Path, _segments: &[crate::analyzer::ProcessedSegment]) -> Result<()> {
            // Simulate successful trimming by creating an empty output file
            fs::File::create(output)?;
            Ok(())
        }

        fn mix_with_music(&self, _input: &Path, _music: &Path, _output: &Path, _transcript: &[crate::stt_analyzer::TranscriptSegment]) -> Result<()> {
            Ok(())
        }

        fn enhance_audio(&self, _input: &Path, _output: &Path) -> Result<()> {
            Ok(())
        }
    }

    // Mock DurationGetter for testing purposes
    struct MockDurationGetter;
    impl DurationGetter for MockDurationGetter {
        fn get_duration(&self, _path: &Path) -> Result<f32> {
            Ok(60.0) // Return a dummy duration
        }
    }

    #[test]
    fn test_batch_processing_integration() -> Result<()> {
        let input_dir = tempdir()?;
        let output_dir = tempdir()?;

        // Create dummy video files
        fs::File::create(input_dir.path().join("video1.mp4"))?;
        fs::File::create(input_dir.path().join("video2.mov"))?;
        fs::File::create(input_dir.path().join("document.txt"))?; // Should be ignored

        // Use mock implementations
        let mock_analyzer = MockFfmpegAnalyzer;
        let mock_editor = MockFfmpegEditor;
        let mock_duration_getter = MockDurationGetter;
        
        // Use default config
        let config = Config::default();

        let result = process_batch_dir(
            input_dir.path().to_path_buf(),
            output_dir.path().to_path_buf(),
            &config,
            &mock_analyzer,
            &mock_editor,
            &mock_duration_getter,
        );

        assert!(result.is_ok());

        // Check if output files were created
        let output_files: Vec<_> = fs::read_dir(output_dir.path())?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();

        assert_eq!(output_files.len(), 2);
        assert!(output_files.iter().any(|p| p.ends_with("video1.mp4")));
        assert!(output_files.iter().any(|p| p.ends_with("video2.mov")));

        Ok(())
    }
}
