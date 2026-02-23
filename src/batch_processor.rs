use std::path::{PathBuf, Path};
use anyhow::{Result, Context};
use std::fs;

use crate::analyzer::VideoAnalyzer;
use crate::editor::VideoEditor;
use crate::editor::calculate_keep_segments;
use crate::utils::find_video_files;
use crate::config::Config;
use crate::exporter;
use crate::analyzer::ProcessedSegment;

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


/// Concatenate intro/outro videos using ffmpeg
fn concatenate_videos(intro: Option<&Path>, main: &Path, outro: Option<&Path>, output: &Path) -> Result<()> {
    let has_intro = intro.is_some();
    let has_outro = outro.is_some();
    
    if !has_intro && !has_outro {
        // No intro/outro, just copy
        fs::copy(main, output)?;
        return Ok(());
    }
    
    // Build ffmpeg concat filter
    let mut inputs: Vec<String> = vec![];
    let mut concat_inputs = String::new();
    let mut input_idx = 0;
    
    if let Some(intro_path) = intro {
        inputs.push(format!("-i {}", intro_path.to_str().context("invalid intro path")?));
        concat_inputs.push_str(&format!("[{}:v][{}:a]", input_idx, input_idx));
        input_idx += 1;
    }
    
    inputs.push(format!("-i {}", main.to_str().context("invalid main path")?));
    concat_inputs.push_str(&format!("[{}:v][{}:a]", input_idx, input_idx));
    input_idx += 1;
    
    if let Some(outro_path) = outro {
        inputs.push(format!("-i {}", outro_path.to_str().context("invalid outro path")?));
        concat_inputs.push_str(&format!("[{}:v][{}:a]", input_idx, input_idx));
    }
    
    let n = inputs.len();
    let filter = format!("{}concat=n={}:v=1:a=1[outv][outa]", concat_inputs, n);
    
    let mut args = vec![];
    for input in &inputs {
        args.push(input.clone());
    }
    args.push("-filter_complex".to_string());
    args.push(filter);
    args.push("-map".to_string());
    args.push("[outv]".to_string());
    args.push("-map".to_string());
    args.push("[outa]".to_string());
    args.push("-y".to_string());
    args.push(output.to_str().context("invalid output path")?.to_string());
    
    let status = std::process::Command::new("ffmpeg")
        .args(&args)
        .status()
        .context("failed to execute ffmpeg for concat")?;
    
    if !status.success() {
        anyhow::bail!("ffmpeg concat failed with status: {}", status);
    }
    
    Ok(())
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
    process_single_file_with_intro_outro(input_file, output_file, config, analyzer, editor, duration_getter, None, None)
}

pub fn process_single_file_with_intro_outro<A, E, D>(
    input_file: PathBuf, 
    output_file: PathBuf, 
    config: &Config,
    analyzer: &A, 
    editor: &E, 
    duration_getter: &D,
    intro: Option<PathBuf>,
    outro: Option<PathBuf>,
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

    // Step 1: Trim video (silence removal/speedup)
    let trimmed_file = if config.audio.enhance || config.audio.music_file.is_some() || intro.is_some() || outro.is_some() {
        output_file.with_extension("trimmed.mp4")
    } else {
        output_file.clone()
    };

    editor.trim_video(&input_file, &trimmed_file, &processed_segments)
        .context("Failed to trim video")?;
    println!("Trimmed video saved to: {:?}", trimmed_file);

    // Step 2: Audio enhancement (optional)
    let enhanced_file = if config.audio.enhance {
        let enhanced = output_file.with_extension("enhanced.mp4");
        println!("Enhancing audio...");
        editor.enhance_audio(&trimmed_file, &enhanced)
            .context("Failed to enhance audio")?;
        
        if trimmed_file != output_file {
            let _ = fs::remove_file(&trimmed_file);
        }
        enhanced
    } else {
        trimmed_file
    };

    // Step 3: Music mixing (optional)
    let with_music_file = if let Some(ref music_path) = config.audio.music_file {
        let with_music = output_file.with_extension("music.mp4");
        println!("Mixing with background music: {:?}", music_path);
        
        let empty_transcript = vec![];
        editor.mix_with_music(&enhanced_file, music_path, &with_music, &empty_transcript)
            .context("Failed to mix music")?;
        
        if enhanced_file != output_file {
            let _ = fs::remove_file(&enhanced_file);
        }
        with_music
    } else {
        enhanced_file
    };

    // Step 4: Intro/Outro concatenation (optional)
    let final_file = if intro.is_some() || outro.is_some() {
        println!("Adding intro/outro...");
        concatenate_videos(
            intro.as_deref(),
            &with_music_file,
            outro.as_deref(),
            &output_file
        )?;
        
        if with_music_file != output_file {
            let _ = fs::remove_file(&with_music_file);
        }
        output_file.clone()
    } else {
        if with_music_file != output_file {
            fs::rename(&with_music_file, &output_file)?;
        }
        output_file.clone()
    };

    // Step 5: Export additional files
    export_additional_files(&input_file, &final_file, &processed_segments, config)?;

    println!("Successfully saved final video to: {:?}", final_file);
    Ok(())
}

/// Export additional files (SRT, chapters, FCPXML, EDL) based on config
fn export_additional_files(
    input_file: &Path,
    output_file: &Path,
    segments: &[ProcessedSegment],
    config: &Config,
) -> Result<()> {
    let base_path = output_file.with_extension("");

    if config.export.subtitles {
        let srt_path = format!("{}.srt", base_path.display());
        println!("Exporting SRT subtitles to: {}", srt_path);
        // TODO: Need actual transcript for subtitles
        // For now, create placeholder
        fs::write(&srt_path, "# Subtitles will be generated when STT is complete\n")?;
    }

    if config.export.chapters {
        let chapters_path = format!("{}.chapters.txt", base_path.display());
        println!("Exporting YouTube chapters to: {}", chapters_path);
        // TODO: Need actual transcript for chapters
        fs::write(&chapters_path, "00:00 Intro\n")?;
    }

    if config.export.fcpxml {
        let fcpxml_path = format!("{}.fcpxml", base_path.display());
        println!("Exporting FCPXML to: {}", fcpxml_path);
        exporter::export_fcpxml(segments, input_file, Path::new(&fcpxml_path))?;
    }

    if config.export.edl {
        let edl_path = format!("{}.edl", base_path.display());
        println!("Exporting EDL to: {}", edl_path);
        exporter::export_edl(segments, input_file, Path::new(&edl_path))?;
    }

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
        fn reframe(&self, _input: &Path, _output: &Path) -> Result<()> {
            Ok(())
        }
        
        fn blur_background(&self, _input: &Path, _output: &Path) -> Result<()> {
            Ok(())
        }
        
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

        fn reduce_noise(&self, _input: &Path, _output: &Path) -> Result<()> {
            Ok(())
        }

        fn stabilize(&self, _input: &Path, _output: &Path) -> Result<()> {
            Ok(())
        }

        fn color_correct(&self, _input: &Path, _output: &Path) -> Result<()> {
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
        
        // Use config with audio enhancement disabled (mock doesn't create files)
        let mut config = Config::default();
        config.audio.enhance = false;

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
