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
    #[arg(group = "input_group", short, long, value_name = "FILE")]
    pub input_file: Option<PathBuf>,

    #[arg(group = "input_group", short = 'I', long, value_name = "DIRECTORY")]
    pub input_dir: Option<PathBuf>,

    #[arg(group = "output_group", short, long, value_name = "FILE")]
    pub output_file: Option<PathBuf>,

    #[arg(group = "output_group", short = 'O', long, value_name = "DIRECTORY")]
    pub output_dir: Option<PathBuf>,

    /// Silence threshold in decibels (e.g., -30.0)
    #[arg(short, long, default_value_t = -30.0, allow_hyphen_values = true)]
    pub threshold: f32,

    /// Minimum silence duration in seconds
    #[arg(short, long, default_value_t = 0.5)]
    pub duration: f32,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(input_file) = cli.input_file {
        // Single file processing logic
        let output_file = cli.output_file.context("Output file must be specified for single file processing")?;
        process_single_file(input_file, output_file, cli.threshold, cli.duration)?;
    } else if let Some(input_dir) = cli.input_dir {
        // Batch processing logic
        let output_dir = cli.output_dir.context("Output directory must be specified for batch processing")?;
        process_batch_dir(input_dir, output_dir, cli.threshold, cli.duration)?;
    } else {
        anyhow::bail!("Either an input file or an input directory must be specified.");
    }

    Ok(())
}

fn process_single_file(input_file: PathBuf, output_file: PathBuf, threshold: f32, duration: f32) -> Result<()> {
    println!("Analyzing video: {:?}", input_file);
    let analyzer = FfmpegAnalyzer;
    let silences = analyzer.detect_silence(&input_file, threshold, duration)
        .context("Failed to detect silence")?;

    println!("Detected {} silent segments.", silences.len());

    let duration = get_video_duration(&input_file)?;
    println!("Total duration: {}s", duration);

    let keep_segments = calculate_keep_segments(&silences, duration);
    println!("Segments to keep: {}", keep_segments.len());

    let editor = FfmpegEditor;
    editor.trim_video(&input_file, &output_file, &keep_segments)
        .context("Failed to trim video")?;

    println!("Successfully saved trimmed video to: {:?}", output_file);
    Ok(())
}

fn process_batch_dir(input_dir: PathBuf, output_dir: PathBuf, threshold: f32, duration: f32) -> Result<()> {
    println!("Processing directory: {:?}", input_dir);
    println!("Output directory: {:?}", output_dir);
    // TODO: Implement file discovery and batch processing loop
    anyhow::bail!("Batch processing not yet implemented.");
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
