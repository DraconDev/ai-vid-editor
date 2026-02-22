use clap::Parser;
use std::path::PathBuf;
use anyhow::Result;

pub mod analyzer;
pub mod editor;
pub mod utils;
pub mod batch_processor;
pub mod stt_analyzer;
pub mod exporter;
pub mod config;

use crate::batch_processor::{process_single_file, process_batch_dir, FfmpegDurationGetter};
use crate::analyzer::FfmpegAnalyzer;
use crate::editor::FfmpegEditor;
use crate::config::Config;

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

    /// Path to config file (TOML)
    #[arg(short = 'c', long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Silence threshold in decibels (e.g., -30.0)
    #[arg(short, long, allow_hyphen_values = true)]
    pub threshold: Option<f32>,

    /// Minimum silence duration in seconds
    #[arg(short, long)]
    pub duration: Option<f32>,

    /// Padding in seconds to add around cuts
    #[arg(short, long)]
    pub padding: Option<f32>,

    /// Speed up silences instead of cutting them
    #[arg(short = 's', long)]
    pub speedup: bool,

    /// Enable audio enhancement (loudness normalization + EQ)
    #[arg(short = 'E', long)]
    pub enhance: bool,

    /// Path to background music file (enables auto-ducking)
    #[arg(short = 'm', long, value_name = "FILE")]
    pub music: Option<PathBuf>,

    /// Generate SRT subtitles
    #[arg(long)]
    pub export_srt: bool,

    /// Generate YouTube chapters
    #[arg(long)]
    pub export_chapters: bool,

    /// Generate FCPXML for DaVinci Resolve/Premiere Pro
    #[arg(long)]
    pub export_fcpxml: bool,

    /// Generate EDL (Edit Decision List)
    #[arg(long)]
    pub export_edl: bool,

    /// Generate a default config file
    #[arg(long)]
    pub generate_config: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle --generate-config
    if cli.generate_config {
        let config_content = Config::generate_default_toml()?;
        println!("{}", config_content);
        println!("\n# Save this to 'ai-vid-editor.toml' or '~/.config/ai-vid-editor/config.toml'");
        return Ok(());
    }

    // Load config with precedence: CLI > project config > global config > defaults
    let mut config = Config::load_with_precedence(
        cli.config.as_deref(),
        cli.threshold,
        cli.duration,
        cli.padding,
        cli.speedup,
    )?;

    // Apply CLI overrides for new flags
    if cli.enhance {
        config.audio.enhance = true;
    }
    if let Some(ref music_path) = cli.music {
        config.audio.music_file = Some(music_path.clone());
    }
    if cli.export_srt {
        config.export.subtitles = true;
    }
    if cli.export_chapters {
        config.export.chapters = true;
    }
    if cli.export_fcpxml {
        config.export.fcpxml = true;
    }
    if cli.export_edl {
        config.export.edl = true;
    }

    println!("Loaded configuration:");
    println!("  Silence threshold: {} dB", config.silence.threshold_db);
    println!("  Silence mode: {:?}", config.silence.mode);
    println!("  Padding: {}s", config.silence.padding);
    println!("  Audio enhance: {}", config.audio.enhance);
    if let Some(ref music) = config.audio.music_file {
        println!("  Background music: {:?}", music);
    }
    println!("  Export: SRT={} Chapters={} FCPXML={} EDL={}", 
        config.export.subtitles, config.export.chapters, 
        config.export.fcpxml, config.export.edl);

    let analyzer = FfmpegAnalyzer;
    let editor = FfmpegEditor;
    let duration_getter = FfmpegDurationGetter;

    if let Some(input_file) = cli.input_file {
        // Single file processing logic
        let output_file = cli.output_file.ok_or_else(|| anyhow::anyhow!("Output file must be specified for single file processing"))?;
        process_single_file(input_file, output_file, &config, &analyzer, &editor, &duration_getter)?;
    } else if let Some(input_dir) = cli.input_dir {
        // Batch processing logic
        let output_dir = cli.output_dir.ok_or_else(|| anyhow::anyhow!("Output directory must be specified for batch processing"))?;
        process_batch_dir(input_dir, output_dir, &config, &analyzer, &editor, &duration_getter)?;
    } else {
        anyhow::bail!("Either an input file or an input directory must be specified.");
    }

    Ok(())
}
