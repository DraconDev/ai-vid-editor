use clap::Parser;
use std::path::PathBuf;
use anyhow::{Result, Context};

pub mod analyzer;
pub mod editor;
pub mod utils;
pub mod batch_processor;

use crate::batch_processor::{process_single_file, process_batch_dir, FfmpegDurationGetter};
use crate::analyzer::FfmpegAnalyzer;
use crate::editor::FfmpegEditor;

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

    let analyzer = FfmpegAnalyzer;
    let editor = FfmpegEditor;
    let duration_getter = FfmpegDurationGetter;

    if let Some(input_file) = cli.input_file {
        // Single file processing logic
        let output_file = cli.output_file.ok_or_else(|| anyhow::anyhow!("Output file must be specified for single file processing"))?;
        process_single_file(input_file, output_file, cli.threshold, cli.duration, &analyzer, &editor, &duration_getter)?;
    } else if let Some(input_dir) = cli.input_dir {
        // Batch processing logic
        let output_dir = cli.output_dir.ok_or_else(|| anyhow::anyhow!("Output directory must be specified for batch processing"))?;
        process_batch_dir(input_dir, output_dir, cli.threshold, cli.duration, &analyzer, &editor, &duration_getter)?;
    } else {
        anyhow::bail!("Either an input file or an input directory must be specified.");
    }

    Ok(())
}
