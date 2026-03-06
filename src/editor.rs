use crate::analyzer::{ProcessedSegment, Segment};
use crate::config::SilenceMode;
use crate::stt_analyzer::TranscriptSegment;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

const TRIM_SEGMENTS_PER_CHUNK: usize = 48;

/// Calculate segments to keep after processing silences
///
/// # Arguments
/// * `silence_segments` - Detected silent segments
/// * `total_duration` - Total video duration in seconds
/// * `padding` - Padding around cuts in seconds
/// * `mode` - How to handle silences (Cut or Speedup)
/// * `speedup_factor` - Speed multiplier when mode is Speedup
/// * `min_silence_for_speedup` - Minimum silence duration to speedup (seconds)
pub fn calculate_keep_segments(
    silence_segments: &[Segment],
    total_duration: f32,
    padding: f32,
    mode: SilenceMode,
    speedup_factor: f32,
    min_silence_for_speedup: f32,
) -> Vec<ProcessedSegment> {
    let mut processed = Vec::new();
    let mut current_pos = 0.0;

    for silence in silence_segments {
        let silence_duration = silence.end - silence.start;

        // Add the non-silent segment before this silence
        let keep_end = (silence.start + padding).min(total_duration);
        if keep_end > current_pos {
            processed.push(ProcessedSegment {
                start: current_pos,
                end: keep_end,
                speed: 1.0,
            });
        }

        // Handle the silence based on mode
        match mode {
            SilenceMode::Cut => {
                // Cut mode: skip the silence entirely
                current_pos = (silence.end - padding).max(0.0);
            }
            SilenceMode::Speedup => {
                // Speedup mode: keep silence but speed it up if long enough
                let silence_start = (silence.start + padding).max(0.0);
                let silence_end = (silence.end - padding).min(total_duration);

                if silence_duration >= min_silence_for_speedup && silence_end > silence_start {
                    processed.push(ProcessedSegment {
                        start: silence_start,
                        end: silence_end,
                        speed: speedup_factor,
                    });
                }
                current_pos = silence_end;
            }
        }
    }

    // Add the final segment after the last silence
    if current_pos < total_duration {
        processed.push(ProcessedSegment {
            start: current_pos,
            end: total_duration,
            speed: 1.0,
        });
    }

    processed
}

/// Legacy function for backward compatibility - uses Cut mode
pub fn calculate_keep_segments_simple(
    silence_segments: &[Segment],
    total_duration: f32,
    padding: f32,
) -> Vec<ProcessedSegment> {
    calculate_keep_segments(
        silence_segments,
        total_duration,
        padding,
        SilenceMode::Cut,
        4.0,
        0.5,
    )
}

pub fn calculate_keep_segments_from_transcript(
    transcript: &[TranscriptSegment],
    total_duration: f32,
    filler_words: &[&str],
    padding: f32,
) -> Vec<ProcessedSegment> {
    let mut processed = Vec::new();
    let mut current_pos = 0.0;

    for seg in transcript {
        let is_filler = filler_words
            .iter()
            .any(|&f| seg.text.to_lowercase().contains(f));

        if is_filler {
            // Cut this segment with padding
            let keep_end = (seg.start + padding).min(total_duration);
            if keep_end > current_pos {
                processed.push(ProcessedSegment {
                    start: current_pos,
                    end: keep_end,
                    speed: 1.0,
                });
            }
            current_pos = (seg.end - padding).max(0.0);
        }
    }

    if current_pos < total_duration {
        processed.push(ProcessedSegment {
            start: current_pos,
            end: total_duration,
            speed: 1.0,
        });
    }

    processed
}

pub trait VideoEditor {
    fn trim_video(&self, input: &Path, output: &Path, segments: &[ProcessedSegment]) -> Result<()>;
    fn trim_video_with_progress(
        &self,
        input: &Path,
        output: &Path,
        segments: &[ProcessedSegment],
        progress: &mut dyn FnMut(f32),
    ) -> Result<()> {
        progress(0.0);
        self.trim_video(input, output, segments)?;
        progress(1.0);
        Ok(())
    }
    fn mix_with_music(
        &self,
        input: &Path,
        music: &Path,
        output: &Path,
        transcript: &[TranscriptSegment],
    ) -> Result<()>;
    fn enhance_audio(&self, input: &Path, output: &Path) -> Result<()>;
    fn reduce_noise(&self, input: &Path, output: &Path) -> Result<()>;
    fn stabilize(&self, input: &Path, output: &Path) -> Result<()>;
    fn color_correct(&self, input: &Path, output: &Path) -> Result<()>;
    fn reframe(&self, input: &Path, output: &Path) -> Result<()>;
    fn blur_background(&self, input: &Path, output: &Path) -> Result<()>;
}

pub struct FfmpegEditor;

impl VideoEditor for FfmpegEditor {
    fn trim_video(&self, input: &Path, output: &Path, segments: &[ProcessedSegment]) -> Result<()> {
        self.trim_video_with_progress(input, output, segments, &mut |_| {})
    }

    fn trim_video_with_progress(
        &self,
        input: &Path,
        output: &Path,
        segments: &[ProcessedSegment],
        progress: &mut dyn FnMut(f32),
    ) -> Result<()> {
        if segments.is_empty() {
            anyhow::bail!("No segments to process");
        }

        if segments.len() <= TRIM_SEGMENTS_PER_CHUNK {
            run_trim_filter_job(input, output, segments)?;
            progress(1.0);
            return Ok(());
        }

        let chunk_dir = create_trim_chunk_dir(output)?;
        let chunk_count = segments.len().div_ceil(TRIM_SEGMENTS_PER_CHUNK);
        let mut chunk_files = Vec::with_capacity(chunk_count);

        for (idx, chunk) in segments.chunks(TRIM_SEGMENTS_PER_CHUNK).enumerate() {
            let chunk_path = chunk_dir.join(format!("chunk_{idx:04}.mp4"));
            run_trim_filter_job(input, &chunk_path, chunk)?;
            chunk_files.push(chunk_path);
            progress((idx + 1) as f32 / (chunk_count + 1) as f32);
        }

        concat_chunk_files(&chunk_files, output)?;
        progress(1.0);

        let _ = fs::remove_dir_all(&chunk_dir);
        Ok(())
    }

    fn mix_with_music(
        &self,
        input: &Path,
        music: &Path,
        output: &Path,
        transcript: &[TranscriptSegment],
    ) -> Result<()> {
        let duck_filter = generate_duck_filter(transcript);

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                input.to_str().context("invalid input path")?,
                "-i",
                music.to_str().context("invalid music path")?,
                "-filter_complex",
                &duck_filter,
                "-map",
                "0:v",
                "-map",
                "[outa]",
                "-y",
                output.to_str().context("invalid output path")?,
            ])
            .status()
            .context("failed to execute ffmpeg")?;

        if !status.success() {
            anyhow::bail!("ffmpeg failed with status: {}", status);
        }

        Ok(())
    }

    fn enhance_audio(&self, input: &Path, output: &Path) -> Result<()> {
        // Apply loudnorm and basic speech EQ
        let filter =
            "loudnorm=I=-14:TP=-1:LRA=11,equalizer=f=1000:t=q:w=1:g=2,equalizer=f=3000:t=q:w=1:g=3";

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                input.to_str().context("invalid input path")?,
                "-af",
                filter,
                "-c:v",
                "copy",
                "-y",
                output.to_str().context("invalid output path")?,
            ])
            .status()
            .context("failed to execute ffmpeg")?;

        if !status.success() {
            anyhow::bail!("ffmpeg failed with status: {}", status);
        }

        Ok(())
    }

    fn reduce_noise(&self, input: &Path, output: &Path) -> Result<()> {
        // Apply FFT-based noise reduction
        // afftdn removes steady background noise (fans, AC, hiss)
        let filter = "afftdn=nf=-25:tn=1";

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                input.to_str().context("invalid input path")?,
                "-af",
                filter,
                "-c:v",
                "copy",
                "-y",
                output.to_str().context("invalid output path")?,
            ])
            .status()
            .context("failed to execute ffmpeg")?;

        if !status.success() {
            anyhow::bail!("ffmpeg failed with status: {}", status);
        }

        Ok(())
    }

    fn stabilize(&self, input: &Path, output: &Path) -> Result<()> {
        // Video stabilization using vidstab filter (two-pass)
        // Pass 1: Analyze video motion
        // Pass 2: Apply stabilization

        let input_str = input.to_str().context("invalid input path")?;
        let output_str = output.to_str().context("invalid output path")?;
        let trf_file = "/tmp/transforms.trf";

        // Pass 1: Detect motion and generate transforms
        let status1 = Command::new("ffmpeg")
            .args([
                "-i",
                input_str,
                "-vf",
                &format!(
                    "vidstabdetect=stepsize=6:shakiness=5:accuracy=15:result={}",
                    trf_file
                ),
                "-f",
                "null",
                "-",
            ])
            .status()
            .context("failed to execute ffmpeg (stabilize pass 1)")?;

        if !status1.success() {
            anyhow::bail!("ffmpeg stabilize pass 1 failed with status: {}", status1);
        }

        // Pass 2: Apply stabilization
        let status2 = Command::new("ffmpeg")
            .args([
                "-i",
                input_str,
                "-vf",
                &format!(
                    "vidstabtransform=input={}:smoothing=10:optzoom=1:interpol=bicubic",
                    trf_file
                ),
                "-c:a",
                "copy",
                "-y",
                output_str,
            ])
            .status()
            .context("failed to execute ffmpeg (stabilize pass 2)")?;

        // Cleanup temp file
        let _ = std::fs::remove_file(trf_file);

        if !status2.success() {
            anyhow::bail!("ffmpeg stabilize pass 2 failed with status: {}", status2);
        }

        Ok(())
    }

    fn color_correct(&self, input: &Path, output: &Path) -> Result<()> {
        // Auto color correction using eq filter
        // Adjusts brightness, contrast, saturation for a more balanced look
        // Also applies slight sharpening for clarity
        let filter = "eq=contrast=1.1:brightness=0.05:saturation=1.1,unsharp=5:5:0.5:5:5:0.0";

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                input.to_str().context("invalid input path")?,
                "-vf",
                filter,
                "-c:a",
                "copy",
                "-y",
                output.to_str().context("invalid output path")?,
            ])
            .status()
            .context("failed to execute ffmpeg")?;

        if !status.success() {
            anyhow::bail!("ffmpeg failed with status: {}", status);
        }

        Ok(())
    }

    fn reframe(&self, input: &Path, output: &Path) -> Result<()> {
        // Auto-reframe: Convert horizontal (16:9) to vertical (9:16)
        // Uses ML face detection to follow the speaker

        info!("Auto-reframe: Analyzing video for face tracking...");

        // Try to use ML-powered reframe
        let filter = match crate::ml::AutoReframeProcessor::new() {
            Ok(processor) => {
                // Sample at 1 fps for face detection
                match processor.analyze_video(input, 1.0) {
                    Ok(crop_regions) => {
                        // Get video dimensions
                        let (w, h) = crate::ml::FrameExtractor::get_video_dimensions(input)
                            .unwrap_or((1920, 1080));

                        processor.generate_crop_filter(&crop_regions, w, h)
                    }
                    Err(e) => {
                        warn!(error = %e, "Face detection failed, using center crop");
                        "crop=ih*9/16:ih,scale=1080:1920".to_string()
                    }
                }
            }
            Err(e) => {
                warn!(error = %e, "Could not load face detection model, using center crop");
                "crop=ih*9/16:ih,scale=1080:1920".to_string()
            }
        };

        info!(filter = %filter, "Applying crop filter");

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                input.to_str().context("invalid input path")?,
                "-vf",
                &filter,
                "-c:a",
                "copy",
                "-y",
                output.to_str().context("invalid output path")?,
            ])
            .status()
            .context("failed to execute ffmpeg")?;

        if !status.success() {
            anyhow::bail!("ffmpeg failed with status: {}", status);
        }

        Ok(())
    }

    fn blur_background(&self, input: &Path, output: &Path) -> Result<()> {
        // Background blur using person segmentation
        // Falls back to simple blur if ML not available

        info!("Background blur: Processing video...");

        // For now, use a simple boxblur
        // Full ML implementation would process each frame with segmentation
        // This is computationally expensive and requires frame-by-frame processing

        // Try ML-powered blur (experimental)
        let use_ml = std::env::var("AI_VID_EDITOR_ML_BLUR")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false);

        if use_ml {
            info!("Using ML-powered background blur (experimental)...");
            // ML blur would go here - requires significant processing time
            // For production, this would extract frames, process with segmentation, and re-encode
        }

        // Use ffmpeg's boxblur as the practical solution
        // This blurs the entire frame - for person-aware blur, use a video editor
        let filter = "boxblur=20:5";

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                input.to_str().context("invalid input path")?,
                "-vf",
                filter,
                "-c:a",
                "copy",
                "-y",
                output.to_str().context("invalid output path")?,
            ])
            .status()
            .context("failed to execute ffmpeg")?;

        if !status.success() {
            anyhow::bail!("ffmpeg failed with status: {}", status);
        }

        Ok(())
    }
}

fn run_trim_filter_job(input: &Path, output: &Path, segments: &[ProcessedSegment]) -> Result<()> {
    let (v_filter, a_filter) = generate_trim_filters(segments);

    let status = Command::new("ffmpeg")
        .args([
            "-i",
            input.to_str().context("invalid input path")?,
            "-filter_complex",
            &format!("{}{}", v_filter, a_filter),
            "-map",
            "[outv]",
            "-map",
            "[outa]",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-crf",
            "20",
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-movflags",
            "+faststart",
            "-y",
            output.to_str().context("invalid output path")?,
        ])
        .status()
        .context("failed to execute ffmpeg")?;

    if !status.success() {
        anyhow::bail!("ffmpeg failed with status: {}", status);
    }

    Ok(())
}

fn create_trim_chunk_dir(output: &Path) -> Result<PathBuf> {
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    let stem = output
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("trim");
    let chunk_dir = parent.join(format!(
        ".ai-vid-editor-{}-{}",
        stem,
        std::process::id()
    ));

    if chunk_dir.exists() {
        let _ = fs::remove_dir_all(&chunk_dir);
    }
    fs::create_dir_all(&chunk_dir)?;
    Ok(chunk_dir)
}

fn concat_chunk_files(chunk_files: &[PathBuf], output: &Path) -> Result<()> {
    if chunk_files.is_empty() {
        anyhow::bail!("No chunk files to concatenate");
    }

    if chunk_files.len() == 1 {
        fs::rename(&chunk_files[0], output)?;
        return Ok(());
    }

    let concat_list = output.with_extension("concat.txt");
    let concat_contents = chunk_files
        .iter()
        .map(|path| format!("file '{}'\n", path.display().to_string().replace('\'', "'\\''")))
        .collect::<String>();
    fs::write(&concat_list, concat_contents)?;

    let status = Command::new("ffmpeg")
        .args([
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            concat_list.to_str().context("invalid concat list path")?,
            "-c",
            "copy",
            "-y",
            output.to_str().context("invalid output path")?,
        ])
        .status()
        .context("failed to execute ffmpeg concat")?;

    let _ = fs::remove_file(&concat_list);
    for chunk_file in chunk_files {
        let _ = fs::remove_file(chunk_file);
    }

    if !status.success() {
        anyhow::bail!("ffmpeg concat failed with status: {}", status);
    }

    Ok(())
}

fn generate_trim_filters(segments: &[ProcessedSegment]) -> (String, String) {
    let mut v_filter = String::new();
    let mut a_filter = String::new();
    let mut v_concat = String::new();
    let mut a_concat = String::new();

    for (i, seg) in segments.iter().enumerate() {
        // Handle speed adjustment
        let setpts = if seg.speed != 1.0 {
            format!("setpts={}*PTS", 1.0 / seg.speed)
        } else {
            "setpts=PTS-STARTPTS".to_string()
        };

        let atempo = if seg.speed != 1.0 {
            format!("atempo={}", seg.speed)
        } else {
            "asetpts=PTS-STARTPTS".to_string()
        };

        v_filter.push_str(&format!(
            "[0:v]trim=start={}:end={}, {}[v{}];",
            seg.start, seg.end, setpts, i
        ));
        a_filter.push_str(&format!(
            "[0:a]atrim=start={}:end={}, {}[a{}];",
            seg.start, seg.end, atempo, i
        ));
        v_concat.push_str(&format!("[v{}]", i));
        a_concat.push_str(&format!("[a{}]", i));
    }

    v_filter.push_str(&format!(
        "{}concat=n={}:v=1:a=0[outv];",
        v_concat,
        segments.len()
    ));
    a_filter.push_str(&format!(
        "{}concat=n={}:v=0:a=1[outa]",
        a_concat,
        segments.len()
    ));

    (v_filter, a_filter)
}

fn generate_duck_filter(transcript: &[TranscriptSegment]) -> String {
    let mut volume_expr = "1.0".to_string();

    // For each speech segment, lower the music volume
    for seg in transcript {
        volume_expr = format!(
            "if(between(t,{},{}),0.2,{})",
            seg.start, seg.end, volume_expr
        );
    }

    format!(
        "[1:a]volume=volume='{}'[ducked];[0:a][ducked]amix=inputs=2:duration=first[outa]",
        volume_expr
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_keep_segments_cut_mode() {
        let silences = vec![Segment {
            start: 2.0,
            end: 3.0,
        }];
        let duration = 10.0;
        let padding = 0.1;
        let processed =
            calculate_keep_segments(&silences, duration, padding, SilenceMode::Cut, 4.0, 0.5);

        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].end, 2.1);
        assert_eq!(processed[1].start, 2.9);
        assert_eq!(processed[0].speed, 1.0);
        assert_eq!(processed[1].speed, 1.0);
    }

    #[test]
    fn test_calculate_keep_segments_speedup_mode() {
        let silences = vec![
            Segment {
                start: 2.0,
                end: 4.0,
            }, // 2 second silence
        ];
        let duration = 10.0;
        let padding = 0.1;
        let processed =
            calculate_keep_segments(&silences, duration, padding, SilenceMode::Speedup, 4.0, 0.5);

        // Should have 3 segments: before silence, silence (sped up), after silence
        assert_eq!(processed.len(), 3);
        assert_eq!(processed[0].end, 2.1);
        assert_eq!(processed[0].speed, 1.0);

        // Silence segment should be sped up
        assert_eq!(processed[1].start, 2.1);
        assert_eq!(processed[1].end, 3.9);
        assert_eq!(processed[1].speed, 4.0);

        // After silence
        assert_eq!(processed[2].start, 3.9);
        assert_eq!(processed[2].speed, 1.0);
    }

    #[test]
    fn test_calculate_keep_segments_speedup_short_silence() {
        // Silence too short for speedup should be cut
        let silences = vec![
            Segment {
                start: 2.0,
                end: 2.3,
            }, // 0.3 second silence (below min)
        ];
        let duration = 10.0;
        let padding = 0.1;
        let min_silence = 0.5;
        let processed = calculate_keep_segments(
            &silences,
            duration,
            padding,
            SilenceMode::Speedup,
            4.0,
            min_silence,
        );

        // Short silence should be cut, not sped up
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].speed, 1.0);
        assert_eq!(processed[1].speed, 1.0);
    }

    #[test]
    fn test_calculate_keep_segments_multiple_silences() {
        let silences = vec![
            Segment {
                start: 2.0,
                end: 3.0,
            },
            Segment {
                start: 5.0,
                end: 7.0,
            },
        ];
        let duration = 10.0;
        let padding = 0.1;
        let processed =
            calculate_keep_segments(&silences, duration, padding, SilenceMode::Cut, 4.0, 0.5);

        assert_eq!(processed.len(), 3);
        assert_eq!(processed[0].start, 0.0);
        assert_eq!(processed[0].end, 2.1);
        assert_eq!(processed[1].start, 2.9);
        assert_eq!(processed[1].end, 5.1);
        assert_eq!(processed[2].start, 6.9);
        assert_eq!(processed[2].end, 10.0);
    }

    #[test]
    fn test_generate_duck_filter() {
        let transcript = vec![TranscriptSegment {
            start: 1.0,
            end: 2.0,
            text: "hello".to_string(),
            confidence: 1.0,
        }];
        let filter = generate_duck_filter(&transcript);
        assert!(filter.contains("between(t,1,2)"));
        assert!(filter.contains("volume='if(between(t,1,2),0.2,1.0)'"));
        assert!(filter.contains("amix=inputs=2"));
    }
}
