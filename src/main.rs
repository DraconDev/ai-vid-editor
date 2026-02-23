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
use crate::config::{Config, Preset};

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

    /// Use a preset profile: youtube, shorts, podcast, minimal
    #[arg(short = 'P', long, value_name = "PRESET")]
    pub preset: Option<String>,

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

    /// Dry run: analyze and show what would be done without processing
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Output results as JSON (useful for scripting)
    #[arg(short = 'j', long)]
    pub json: bool,

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

    // Start with preset or default config
    let mut config = if let Some(ref preset_str) = cli.preset {
        let preset = Preset::from_str(preset_str)
            .ok_or_else(|| anyhow::anyhow!("Unknown preset: {}. Valid presets: youtube, shorts, podcast, minimal", preset_str))?;
        if !cli.json {
            println!("Using preset: {}", preset.as_str());
        }
        preset.to_config()
    } else {
        Config::default()
    };

    // Apply config file if specified
    if let Some(ref config_path) = cli.config {
        if config_path.exists() {
            let file_config = Config::from_file(config_path)?;
            config = config.merge(file_config);
        }
    }

    // Apply CLI overrides
    if let Some(threshold) = cli.threshold {
        config.silence.threshold_db = threshold;
    }
    if let Some(duration) = cli.duration {
        config.silence.min_duration = duration;
    }
    if let Some(padding) = cli.padding {
        config.silence.padding = padding;
    }
    if cli.speedup {
        config.silence.mode = crate::config::SilenceMode::Speedup;
    }
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

    // Print config (unless JSON mode)
    if !cli.json {
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
    }

    // Handle dry-run mode
    if cli.dry_run {
        return handle_dry_run(&cli, &config);
    }

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

/// Handle dry-run mode: analyze and show what would be done
fn handle_dry_run(cli: &Cli, config: &Config) -> Result<()> {
    use crate::analyzer::VideoAnalyzer;
    use crate::batch_processor::DurationGetter;
    use serde_json::json;

    let analyzer = FfmpegAnalyzer;
    let duration_getter = FfmpegDurationGetter;

    let input_path = cli.input_file.as_ref()
        .or(cli.input_dir.as_ref())
        .ok_or_else(|| anyhow::anyhow!("Input file or directory required"))?;

    let silences = analyzer.detect_silence(input_path, config.silence.threshold_db, config.silence.min_duration)?;
    let video_duration = duration_getter.get_duration(input_path)?;

    // Calculate total silence duration
    let total_silence: f32 = silences.iter().map(|s| s.end - s.start).sum();
    let output_duration = match config.silence.mode {
        crate::config::SilenceMode::Cut => video_duration - total_silence,
        crate::config::SilenceMode::Speedup => {
            // Approximate: silences are sped up
            video_duration - total_silence + (total_silence / config.silence.speedup_factor)
        }
    };

    if cli.json {
        let result = json!({
            "input": input_path.to_string_lossy(),
            "input_duration_sec": video_duration,
            "silence_segments": silences.len(),
            "total_silence_sec": total_silence,
            "output_duration_sec": output_duration,
            "time_saved_sec": video_duration - output_duration,
            "config": {
                "silence_mode": format!("{:?}", config.silence.mode),
                "threshold_db": config.silence.threshold_db,
                "padding_sec": config.silence.padding,
                "enhance_audio": config.audio.enhance,
            }
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("\n=== DRY RUN ANALYSIS ===");
        println!("Input: {:?}", input_path);
        println!("Input duration: {:.1}s ({:.1} min)", video_duration, video_duration / 60.0);
        println!("Silent segments detected: {}", silences.len());
        println!("Total silence: {:.1}s ({:.1} min)", total_silence, total_silence / 60.0);
        println!("\nWould produce:");
        println!("  Output duration: {:.1}s ({:.1} min)", output_duration, output_duration / 60.0);
        println!("  Time saved: {:.1}s ({:.1} min)", video_duration - output_duration, (video_duration - output_duration) / 60.0);
        println!("\nOperations:");
        println!("  - Silence mode: {:?}", config.silence.mode);
        if config.audio.enhance {
            println!("  - Audio enhancement: enabled (target {} LUFS)", config.audio.target_lufs);
        }
        if config.audio.music_file.is_some() {
            println!("  - Background music: {:?}", config.audio.music_file);
        }
        if config.export.subtitles { println!("  - Export SRT subtitles"); }
        if config.export.chapters { println!("  - Export YouTube chapters"); }
        if config.export.fcpxml { println!("  - Export FCPXML"); }
        if config.export.edl { println!("  - Export EDL"); }
    }

    Ok(())
}
