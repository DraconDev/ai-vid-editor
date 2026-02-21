use crate::analyzer::Segment;
use std::path::Path;
use anyhow::{Result, Context};
use std::process::Command;

pub fn calculate_keep_segments(silence_segments: &[Segment], total_duration: f32) -> Vec<Segment> {
    let mut keep = Vec::new();
    let mut current_pos = 0.0;

    for silence in silence_segments {
        if silence.start > current_pos {
            keep.push(Segment {
                start: current_pos,
                end: silence.start,
            });
        }
        current_pos = silence.end;
    }

    if current_pos < total_duration {
        keep.push(Segment {
            start: current_pos,
            end: total_duration,
        });
    }

    keep
}

pub trait VideoEditor {
    fn trim_video(&self, input: &Path, output: &Path, keep_segments: &[Segment]) -> Result<()>;
}

pub struct FfmpegEditor;

impl VideoEditor for FfmpegEditor {
    fn trim_video(&self, input: &Path, output: &Path, keep_segments: &[Segment]) -> Result<()> {
        if keep_segments.is_empty() {
            anyhow::bail!("No segments to keep");
        }

        let (v_filter, a_filter) = generate_trim_filters(keep_segments);

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
}

fn generate_trim_filters(segments: &[Segment]) -> (String, String) {
    let mut v_filter = String::new();
    let mut a_filter = String::new();
    let mut v_concat = String::new();
    let mut a_concat = String::new();

    for (i, seg) in segments.iter().enumerate() {
        v_filter.push_str(&format!(
            "[0:v]trim=start={}:end={},setpts=PTS-STARTPTS[v{}];",
            seg.start, seg.end, i
        ));
        a_filter.push_str(&format!(
            "[0:a]atrim=start={}:end={},asetpts=PTS-STARTPTS[a{}];",
            seg.start, seg.end, i
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_keep_segments() {
        let silences = vec![
            Segment { start: 1.0, end: 2.0 },
            Segment { start: 4.0, end: 5.0 },
        ];
        let duration = 10.0;
        let keeps = calculate_keep_segments(&silences, duration);

        assert_eq!(keeps.len(), 3);
        assert_eq!(keeps[0], Segment { start: 0.0, end: 1.0 });
        assert_eq!(keeps[1], Segment { start: 2.0, end: 4.0 });
        assert_eq!(keeps[2], Segment { start: 5.0, end: 10.0 });
    }

    #[test]
    fn test_generate_trim_filters() {
        let segments = vec![
            Segment { start: 0.0, end: 1.0 },
            Segment { start: 2.0, end: 4.0 },
        ];
        let (v, a) = generate_trim_filters(&segments);
        assert!(v.contains("trim=start=0:end=1"));
        assert!(v.contains("trim=start=2:end=4"));
        assert!(v.contains("concat=n=2:v=1:a=0[outv]"));
        assert!(a.contains("atrim=start=0:end=1"));
        assert!(a.contains("atrim=start=2:end=4"));
        assert!(a.contains("concat=n=2:v=0:a=1[outa]"));
    }
}
