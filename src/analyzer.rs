use anyhow::Result;
use std::path::Path;

#[derive(Debug, PartialEq, Clone)]
pub struct Segment {
    pub start: f32,
    pub end: f32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ProcessedSegment {
    pub start: f32,
    pub end: f32,
    pub speed: f32,
}

pub trait VideoAnalyzer {
    fn detect_silence(
        &self,
        path: &Path,
        threshold_db: f32,
        duration_s: f32,
    ) -> Result<Vec<Segment>>;
}

pub struct FfmpegAnalyzer;

impl VideoAnalyzer for FfmpegAnalyzer {
    fn detect_silence(
        &self,
        path: &Path,
        threshold_db: f32,
        duration_s: f32,
    ) -> Result<Vec<Segment>> {
        let output = std::process::Command::new("ffmpeg")
            .args([
                "-i",
                path.to_str().context("invalid path")?,
                "-af",
                &format!("silencedetect=noise={}dB:d={}", threshold_db, duration_s),
                "-f",
                "null",
                "-",
            ])
            .output()
            .context("failed to execute ffmpeg")?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(parse_ffmpeg_silence(&stderr))
    }
}

fn parse_ffmpeg_silence(output: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut current_start: Option<f32> = None;

    for line in output.lines() {
        if line.contains("silence_start:") {
            if let Some(pos) = line.find("silence_start:") {
                let val_str = &line[pos + "silence_start:".len()..].trim();
                if let Ok(start) = val_str.parse::<f32>() {
                    current_start = Some(start);
                }
            }
        } else if line.contains("silence_end:")
            && let Some(start) = current_start.take()
            && let Some(pos) = line.find("silence_end:")
        {
            let part = &line[pos + "silence_end:".len()..];
            if let Some(pipe_pos) = part.find('|') {
                let val_str = &part[..pipe_pos].trim();
                if let Ok(end) = val_str.parse::<f32>() {
                    segments.push(Segment { start, end });
                }
            } else {
                let val_str = part.trim();
                if let Ok(end) = val_str.parse::<f32>() {
                    segments.push(Segment { start, end });
                }
            }
        }
    }
    segments
}

use anyhow::Context;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_silence_output() {
        let output = r#"[silencedetect @ 0x559e1c2c4840] silence_start: 1.234
[silencedetect @ 0x559e1c2c4840] silence_end: 4.567 | silence_duration: 3.333
[silencedetect @ 0x559e1c2c4840] silence_start: 10.0
[silencedetect @ 0x559e1c2c4840] silence_end: 12.5 | silence_duration: 2.5"#;

        let segments = parse_ffmpeg_silence(output);
        assert_eq!(segments.len(), 2);
        assert_eq!(
            segments[0],
            Segment {
                start: 1.234,
                end: 4.567
            }
        );
        assert_eq!(
            segments[1],
            Segment {
                start: 10.0,
                end: 12.5
            }
        );
    }
}
