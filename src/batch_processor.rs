use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::analyzer::ProcessedSegment;
use crate::analyzer::VideoAnalyzer;
use crate::config::Config;
use crate::editor::calculate_keep_segments;
use crate::editor::VideoEditor;
use crate::exporter;
use crate::stt_analyzer::{CandleSttAnalyzer, TranscriptSegment, VideoSttAnalyzer};
use crate::utils::find_video_files;

#[derive(Debug, Clone)]
pub struct ProcessingProgress {
    pub fraction: f32,
    pub stage: String,
}

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
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                path.to_str().context("invalid path")?,
            ])
            .output()
            .context("failed to execute ffprobe")?;

        let val_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        val_str.parse::<f32>().context("failed to parse duration")
    }
}

/// Concatenate intro/outro videos using ffmpeg
fn concatenate_videos(
    intro: Option<&Path>,
    main: &Path,
    outro: Option<&Path>,
    output: &Path,
) -> Result<()> {
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
        inputs.push(format!(
            "-i {}",
            intro_path.to_str().context("invalid intro path")?
        ));
        concat_inputs.push_str(&format!("[{}:v][{}:a]", input_idx, input_idx));
        input_idx += 1;
    }

    inputs.push(format!(
        "-i {}",
        main.to_str().context("invalid main path")?
    ));
    concat_inputs.push_str(&format!("[{}:v][{}:a]", input_idx, input_idx));
    input_idx += 1;

    if let Some(outro_path) = outro {
        inputs.push(format!(
            "-i {}",
            outro_path.to_str().context("invalid outro path")?
        ));
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
    duration_getter: &D,
) -> Result<()>
where
    A: VideoAnalyzer,
    E: VideoEditor,
    D: DurationGetter,
{
    process_single_file_with_intro_outro(
        input_file,
        output_file,
        config,
        analyzer,
        editor,
        duration_getter,
        None,
        None,
    )
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
    process_single_file_with_intro_outro_progress(
        input_file,
        output_file,
        config,
        analyzer,
        editor,
        duration_getter,
        intro,
        outro,
        |_| {},
    )
}

pub fn process_single_file_with_intro_outro_progress<A, E, D, F>(
    input_file: PathBuf,
    output_file: PathBuf,
    config: &Config,
    analyzer: &A,
    editor: &E,
    duration_getter: &D,
    intro: Option<PathBuf>,
    outro: Option<PathBuf>,
    mut progress: F,
) -> Result<()>
where
    A: VideoAnalyzer,
    E: VideoEditor,
    D: DurationGetter,
    F: FnMut(ProcessingProgress),
{
    report_progress(&mut progress, 0.02, "Analyzing silence");
    info!(file = ?input_file, "Analyzing video");
    debug!(mode = ?config.silence.mode, "Silence mode");

    let silences = analyzer
        .detect_silence(
            &input_file,
            config.silence.threshold_db,
            config.silence.min_duration,
        )
        .context("Failed to detect silence")?;

    info!(count = silences.len(), "Detected silent segments");

    report_progress(&mut progress, 0.1, "Planning edits");
    let video_duration = duration_getter.get_duration(&input_file)?;
    debug!(duration = video_duration, "Video duration");

    let processed_segments = calculate_keep_segments(
        &silences,
        video_duration,
        config.silence.padding,
        config.silence.mode,
        config.silence.speedup_factor,
        config.silence.min_silence_for_speedup,
    );
    debug!(count = processed_segments.len(), "Segments to process");

    let trimmed_file = if config.audio.enhance
        || config.audio.music_file.is_some()
        || intro.is_some()
        || outro.is_some()
    {
        output_file.with_extension("trimmed.mp4")
    } else {
        output_file.clone()
    };

    report_progress(&mut progress, 0.15, "Trimming video");
    editor
        .trim_video_with_progress(
            &input_file,
            &trimmed_file,
            &processed_segments,
            &mut |value| {
                let percent = 0.15 + (value * 0.6);
                report_progress(
                    &mut progress,
                    percent,
                    format!("Trimming video ({:.0}%)", value * 100.0),
                );
            },
        )
        .context("Failed to trim video")?;
    debug!(file = ?trimmed_file, "Trimmed video saved");

    let enhanced_file = if config.audio.enhance {
        let enhanced = output_file.with_extension("enhanced.mp4");
        report_progress(&mut progress, 0.78, "Enhancing audio");
        info!("Enhancing audio");
        editor
            .enhance_audio(&trimmed_file, &enhanced, config.audio.target_lufs)
            .context("Failed to enhance audio")?;

        if trimmed_file != output_file {
            let _ = fs::remove_file(&trimmed_file);
        }
        enhanced
    } else {
        trimmed_file
    };

    let with_music_file = if let Some(ref music_path) = config.audio.music_file {
        let with_music = output_file.with_extension("music.mp4");
        report_progress(&mut progress, 0.84, "Mixing background music");
        info!(music = ?music_path, "Mixing background music");

        let empty_transcript = vec![];
        editor
            .mix_with_music(&enhanced_file, music_path, &with_music, &empty_transcript)
            .context("Failed to mix music")?;

        if enhanced_file != output_file {
            let _ = fs::remove_file(&enhanced_file);
        }
        with_music
    } else {
        enhanced_file
    };

    let concat_file = if intro.is_some() || outro.is_some() {
        report_progress(&mut progress, 0.88, "Adding intro/outro");
        info!("Adding intro/outro");
        concatenate_videos(
            intro.as_deref(),
            &with_music_file,
            outro.as_deref(),
            &output_file,
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

    let mut current_file = concat_file;

    if config.video.stabilize {
        let stabilized = output_file.with_extension("stabilized.mp4");
        report_progress(&mut progress, 0.9, "Stabilizing video");
        info!("Stabilizing video");
        editor.stabilize(&current_file, &stabilized)?;
        if current_file != output_file {
            let _ = fs::remove_file(&current_file);
        }
        current_file = stabilized;
    }

    if config.video.color_correct {
        let corrected = output_file.with_extension("corrected.mp4");
        report_progress(&mut progress, 0.93, "Color correcting");
        info!("Color correcting");
        editor.color_correct(&current_file, &corrected)?;
        if current_file != output_file {
            let _ = fs::remove_file(&current_file);
        }
        current_file = corrected;
    }

    if config.video.reframe {
        let reframed = output_file.with_extension("reframed.mp4");
        report_progress(&mut progress, 0.95, "Auto-reframing");
        info!("Auto-reframing to vertical (9:16)");
        editor.reframe(&current_file, &reframed)?;
        if current_file != output_file {
            let _ = fs::remove_file(&current_file);
        }
        current_file = reframed;
    }

    if config.video.blur_background {
        let blurred = output_file.with_extension("blurred.mp4");
        report_progress(&mut progress, 0.97, "Blurring background");
        info!("Blurring background");
        editor.blur_background(&current_file, &blurred)?;
        if current_file != output_file {
            let _ = fs::remove_file(&current_file);
        }
        current_file = blurred;
    }

    let final_file = output_file.clone();
    if current_file != final_file {
        fs::rename(&current_file, &final_file)?;
    }

    report_progress(&mut progress, 0.99, "Writing exports");
    export_additional_files(&input_file, &final_file, &processed_segments, config)?;

    report_progress(&mut progress, 1.0, "Done");
    info!(file = ?final_file, "Successfully saved video");
    Ok(())
}

fn report_progress<F>(progress: &mut F, fraction: f32, stage: impl Into<String>)
where
    F: FnMut(ProcessingProgress),
{
    progress(ProcessingProgress {
        fraction: fraction.clamp(0.0, 1.0),
        stage: stage.into(),
    });
}

/// Export additional files (SRT, chapters, FCPXML, EDL, clips) based on config
fn export_additional_files(
    input_file: &Path,
    output_file: &Path,
    segments: &[ProcessedSegment],
    config: &Config,
) -> Result<()> {
    let base_path = output_file.with_extension("");

    // Run transcription if we need transcript for any export
    let transcript = if config.export.subtitles
        || config.export.chapters
        || config.export.captions
        || config.export.clips
    {
        match CandleSttAnalyzer.transcribe(output_file) {
            Ok(t) => {
                info!(segments = t.len(), "Transcription complete");
                Some(t)
            }
            Err(e) => {
                warn!(error = %e, "Transcription failed");
                None
            }
        }
    } else {
        None
    };

    if config.export.subtitles {
        let srt_path = format!("{}.srt", base_path.display());
        debug!(path = %srt_path, "Exporting SRT subtitles");
        if let Some(ref t) = transcript {
            exporter::export_srt(t, Path::new(&srt_path))?;
        } else {
            fs::write(&srt_path, "# Transcription failed\n")?;
        }
    }

    if config.export.chapters {
        let chapters_path = format!("{}.chapters.txt", base_path.display());
        debug!(path = %chapters_path, "Exporting YouTube chapters");
        if let Some(ref t) = transcript {
            exporter::export_youtube_chapters(t, Path::new(&chapters_path))?;
        } else {
            fs::write(&chapters_path, "00:00 Intro\n")?;
        }
    }

    if config.export.captions {
        let ass_path = PathBuf::from(format!("{}.ass", base_path.display()));
        debug!(path = %ass_path.display(), "Generating styled captions");
        // Generate styled ASS captions from transcript
        if let Some(ref t) = transcript {
            if let Err(e) = generate_styled_captions(t, &ass_path) {
                warn!(error = %e, "Failed to generate styled captions");
            } else {
                // Burn captions into video
                info!("Burning captions into video");
                let captioned_path = output_file.with_extension("captioned.mp4");
                burn_subtitles_into_video(output_file, &ass_path, &captioned_path)?;
            }
        }
    }

    if config.export.clips {
        if let Some(ref t) = transcript {
            let clips_output_dir = base_path.parent().unwrap_or_else(|| Path::new("."));
            let clip_pattern = format!(
                "{}_clip",
                base_path.file_stem().unwrap_or_default().to_string_lossy()
            );
            match extract_highlight_clips(
                output_file,
                t,
                config.export.clip_count,
                config.export.clip_min_duration,
                config.export.clip_max_duration,
                clips_output_dir,
                &clip_pattern,
            ) {
                Ok(clip_paths) => {
                    info!(count = clip_paths.len(), "Extracted highlight clips");
                }
                Err(e) => {
                    warn!(error = %e, "Failed to extract highlight clips");
                }
            }
        }
    }

    if config.export.fcpxml {
        let fcpxml_path = format!("{}.fcpxml", base_path.display());
        debug!(path = %fcpxml_path, "Exporting FCPXML");
        exporter::export_fcpxml(segments, input_file, Path::new(&fcpxml_path))?;
    }

    if config.export.edl {
        let edl_path = format!("{}.edl", base_path.display());
        debug!(path = %edl_path, "Exporting EDL");
        exporter::export_edl(segments, input_file, Path::new(&edl_path))?;
    }

    Ok(())
}

/// Generate styled ASS subtitle file from transcript
fn generate_styled_captions(transcript: &[TranscriptSegment], output_path: &Path) -> Result<()> {
    let mut ass = String::new();
    ass.push_str("[Script Info]\n");
    ass.push_str("Title: Generated Captions\n");
    ass.push_str("ScriptType: v4.00+\n");
    ass.push_str("Collisions: Normal\n");
    ass.push_str("PlayDepth: 0\n\n");

    ass.push_str("[V4+ Styles]\n");
    ass.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");
    ass.push_str("Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,-1,0,0,0,100,100,0,0,1,2,2,2,10,10,30,1\n\n");

    ass.push_str("[Events]\n");
    ass.push_str(
        "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
    );

    for seg in transcript {
        let text = seg.text.trim();
        if text.is_empty() || text == "[No speech detected]" {
            continue;
        }
        let start = format_ass_time(seg.start);
        let end = format_ass_time(seg.end);
        // Escape text for ASS format
        let escaped = text
            .replace('\\', "\\N")
            .replace('\n', "\\N")
            .replace('\r', "");
        ass.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
            start, end, escaped
        ));
    }

    fs::write(output_path, ass)?;
    Ok(())
}

fn format_ass_time(seconds: f32) -> String {
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;
    let centisecs = ((seconds % 1.0) * 100.0) as u32;
    format!("{}:{:02}:{:02}.{:02}", hours, minutes, secs, centisecs)
}

/// Burn subtitle file into video using FFmpeg
fn burn_subtitles_into_video(
    video_path: &Path,
    subtitle_path: &Path,
    output_path: &Path,
) -> Result<()> {
    let status = std::process::Command::new("ffmpeg")
        .args([
            "-i",
            video_path.to_str().context("invalid video path")?,
            "-vf",
            &format!(
                "subtitles='{}'",
                subtitle_path.to_str().context("invalid subtitle path")?
            ),
            "-c:a",
            "copy",
            "-y",
            output_path.to_str().context("invalid output path")?,
        ])
        .status()
        .context("failed to burn subtitles")?;

    if !status.success() {
        anyhow::bail!("ffmpeg subtitle burn failed with status: {}", status);
    }

    // Replace original with captioned version
    if output_path.exists() {
        fs::rename(output_path, video_path)?;
    }
    Ok(())
}

/// Extract highlight clips based on audio energy peaks in transcript
fn extract_highlight_clips(
    video_path: &Path,
    transcript: &[TranscriptSegment],
    clip_count: u32,
    min_duration: f32,
    max_duration: f32,
    output_dir: &Path,
    clip_pattern: &str,
) -> Result<Vec<PathBuf>> {
    use std::process::Command as Proc;

    // Analyze audio energy per transcript segment using ffprobe
    let mut segment_energy: Vec<(f32, f32, f32)> = Vec::new(); // (start, end, energy)

    for seg in transcript {
        let text = seg.text.trim();
        if text.is_empty() || text == "[No speech detected]" {
            continue;
        }

        // Estimate energy from segment duration and word count (proxy for speech energy)
        // Longer segments with more words = more content = higher energy
        let duration = seg.end - seg.start;
        let word_count = text.split_whitespace().count() as f32;
        let energy = word_count / duration.max(1.0); // words per second

        segment_energy.push((seg.start, seg.end, energy));
    }

    if segment_energy.is_empty() {
        return Ok(vec![]);
    }

    // Find peaks: sort by energy and take top N segments
    segment_energy.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    let mut clip_times: Vec<(f32, f32)> = Vec::new();
    for &(start, end, _) in segment_energy.iter().take(clip_count as usize) {
        let duration = end - start;
        // Expand segment to reasonable clip duration
        let clip_start = (start - 2.0).max(0.0);
        let clip_end = (end + 2.0).min(
            segment_energy
                .iter()
                .map(|(_, e, _)| *e)
                .fold(0.0f32, f32::max),
        );
        let clip_duration = clip_end - clip_start;

        if clip_duration >= min_duration && clip_duration <= max_duration {
            clip_times.push((clip_start, clip_end));
        }
    }

    // Extract clips using FFmpeg
    let mut clip_paths = Vec::new();
    for (i, (clip_start, clip_end)) in clip_times.iter().enumerate() {
        let clip_path = output_dir.join(format!("{}_{}.mp4", clip_pattern, i + 1));

        let duration = clip_end - clip_start;
        let status = Proc::new("ffmpeg")
            .args([
                "-i",
                video_path.to_str().context("invalid path")?,
                "-ss",
                &format!("{}", clip_start),
                "-t",
                &format!("{}", duration),
                "-c",
                "copy",
                "-y",
                clip_path.to_str().context("invalid output path")?,
            ])
            .status()
            .context("failed to extract clip")?;

        if status.success() && clip_path.exists() {
            clip_paths.push(clip_path);
        }
    }

    Ok(clip_paths)
}

pub fn process_batch_dir<A, E, D>(
    input_dir: PathBuf,
    output_dir: PathBuf,
    config: &Config,
    analyzer: &A,
    editor: &E,
    duration_getter: &D,
) -> Result<()>
where
    A: VideoAnalyzer,
    E: VideoEditor,
    D: DurationGetter,
{
    info!(dir = ?input_dir, "Processing directory");
    debug!(output = ?output_dir, mode = ?config.silence.mode, "Batch config");

    fs::create_dir_all(&output_dir).context(format!(
        "Failed to create output directory {:?}",
        output_dir
    ))?;

    let video_files = find_video_files(&input_dir)?;

    if video_files.is_empty() {
        warn!(dir = ?input_dir, "No supported video files found");
        return Ok(());
    }

    let total_files = video_files.len();
    let mut successful_files = 0;
    let mut failed_files = 0;

    for (index, input_file) in video_files.iter().enumerate() {
        let file_name = input_file
            .file_name()
            .context(format!("Could not get file name for {:?}", input_file))?;
        let output_file = output_dir.join(file_name);

        info!(current = index + 1, total = total_files, file = ?input_file, "Processing file");
        match process_single_file(
            input_file.clone(),
            output_file.clone(),
            config,
            analyzer,
            editor,
            duration_getter,
        ) {
            Ok(_) => {
                info!(file = ?input_file, "Successfully processed");
                successful_files += 1;
            }
            Err(e) => {
                warn!(file = ?input_file, error = %e, "Failed to process");
                failed_files += 1;
            }
        }
    }

    info!(
        total = total_files,
        successful = successful_files,
        failed = failed_files,
        "Batch processing complete"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // Mock implementations for testing without actual ffmpeg calls
    struct MockFfmpegAnalyzer;
    impl VideoAnalyzer for MockFfmpegAnalyzer {
        fn detect_silence(
            &self,
            _path: &Path,
            _threshold_db: f32,
            _duration_s: f32,
        ) -> Result<Vec<crate::analyzer::Segment>> {
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

        fn trim_video(
            &self,
            _input: &Path,
            output: &Path,
            _segments: &[crate::analyzer::ProcessedSegment],
        ) -> Result<()> {
            // Simulate successful trimming by creating an empty output file
            fs::File::create(output)?;
            Ok(())
        }

        fn mix_with_music(
            &self,
            _input: &Path,
            _music: &Path,
            _output: &Path,
            _transcript: &[crate::stt_analyzer::TranscriptSegment],
        ) -> Result<()> {
            Ok(())
        }

        fn enhance_audio(&self, _input: &Path, _output: &Path, _target_lufs: f32) -> Result<()> {
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
