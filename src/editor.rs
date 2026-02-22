use crate::analyzer::{Segment, ProcessedSegment};
use crate::stt_analyzer::TranscriptSegment;
use std::path::Path;
use anyhow::{Result, Context};
use std::process::Command;

pub fn calculate_keep_segments(silence_segments: &[Segment], total_duration: f32, padding: f32) -> Vec<ProcessedSegment> {
    let mut processed = Vec::new();
    let mut current_pos = 0.0;

    for silence in silence_segments {
        let keep_end = (silence.start + padding).min(total_duration);
        if keep_end > current_pos {
            processed.push(ProcessedSegment {
                start: current_pos,
                end: keep_end,
                speed: 1.0,
            });
        }
        current_pos = (silence.end - padding).max(0.0);
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

pub fn calculate_keep_segments_from_transcript(
    transcript: &[TranscriptSegment],
    total_duration: f32,
    filler_words: &[&str],
    padding: f32,
) -> Vec<ProcessedSegment> {
    let mut processed = Vec::new();
    let mut current_pos = 0.0;

    for seg in transcript {
        let is_filler = filler_words.iter().any(|&f| seg.text.to_lowercase().contains(f));
        
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
    fn mix_with_music(&self, input: &Path, music: &Path, output: &Path, transcript: &[TranscriptSegment]) -> Result<()>;
}

pub struct FfmpegEditor;

impl VideoEditor for FfmpegEditor {
    fn trim_video(&self, input: &Path, output: &Path, segments: &[ProcessedSegment]) -> Result<()> {
        if segments.is_empty() {
            anyhow::bail!("No segments to process");
        }

        let (v_filter, a_filter) = generate_trim_filters(segments);

        let status = Command::new("ffmpeg")
            .args([
                "-i", input.to_str().context("invalid input path")?,
                "-filter_complex", &format!("{}{}", v_filter, a_filter),
                "-map", "[outv]",
                "-map", "[outa]",
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

    fn mix_with_music(&self, input: &Path, music: &Path, output: &Path, transcript: &[TranscriptSegment]) -> Result<()> {
        let duck_filter = generate_duck_filter(transcript);
        
        let status = Command::new("ffmpeg")
            .args([
                "-i", input.to_str().context("invalid input path")?,
                "-i", music.to_str().context("invalid music path")?,
                "-filter_complex", &duck_filter,
                "-map", "0:v",
                "-map", "[outa]",
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
        // Simple ducking: 0.2 volume during speech, 1.0 otherwise
        // Use if(between(t, start, end), 0.2, 1.0) logic
        volume_expr = format!("if(between(t,{},{}),0.2,{})", seg.start, seg.end, volume_expr);
    }

    format!("[1:a]volume=volume='{}'[ducked];[0:a][ducked]amix=inputs=2:duration=first[outa]", volume_expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_keep_segments_with_padding() {
        let silences = vec![
            Segment { start: 2.0, end: 3.0 },
        ];
        let duration = 10.0;
        let padding = 0.1;
        let processed = calculate_keep_segments(&silences, duration, padding);

        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].end, 2.1);
        assert_eq!(processed[1].start, 2.9);
    }

    #[test]
    fn test_generate_duck_filter() {
        let transcript = vec![
            TranscriptSegment { start: 1.0, end: 2.0, text: "hello".to_string(), confidence: 1.0 },
        ];
        let filter = generate_duck_filter(&transcript);
        assert!(filter.contains("between(t,1,2)"));
        assert!(filter.contains("volume='if(between(t,1,2),0.2,1.0)'"));
        assert!(filter.contains("amix=inputs=2"));
    }
}
