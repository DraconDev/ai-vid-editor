use clap::Parser;
use std::path::PathBuf;
use anyhow::{Result, Context};

pub mod analyzer;
pub mod editor;

use crate::analyzer::{VideoAnalyzer, FfmpegAnalyzer};
use crate::editor::{VideoEditor, FfmpegEditor, calculate_keep_segments};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Input video file
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,

    /// Output video file
    #[arg(short, long, value_name = "FILE")]
    pub output: PathBuf,

    /// Silence threshold in decibels (e.g., -30.0)
    #[arg(short, long, default_value_t = -30.0, allow_hyphen_values = true)]
    pub threshold: f32,

    /// Minimum silence duration in seconds
    #[arg(short, long, default_value_t = 0.5)]
    pub duration: f32,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("Analyzing video: {:?}", cli.input);
    let analyzer = FfmpegAnalyzer;
    let silences = analyzer.detect_silence(&cli.input, cli.threshold, cli.duration)
        .context("Failed to detect silence")?;

    println!("Detected {} silent segments.", silences.len());

    // For now, we need to know the total duration to calculate keep segments.
    // We can get this from ffmpeg as well.
    let duration = get_video_duration(&cli.input)?;
    println!("Total duration: {}s", duration);

    let keep_segments = calculate_keep_segments(&silences, duration);
    println!("Segments to keep: {}", keep_segments.len());

    let editor = FfmpegEditor;
    editor.trim_video(&cli.input, &cli.output, &keep_segments)
        .context("Failed to trim video")?;

    println!("Successfully saved trimmed video to: {:?}", cli.output);

    Ok(())
}

fn get_video_duration(path: &std::path::Path) -> Result<f32> {
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
