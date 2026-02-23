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

use crate::batch_processor::{process_single_file_with_intro_outro, process_batch_dir, FfmpegDurationGetter};
use crate::analyzer::FfmpegAnalyzer;
use crate::editor::FfmpegEditor;
use crate::config::{Config, Preset};

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "AI-powered video editor for automatic silence removal and audio enhancement",
    long_about = "AI Video Editor - Automatically remove silences, enhance audio, and add intros/outros.\n\n\
        EXAMPLES:\n\
          ai-vid-editor -i input.mp4 -o output.mp4                    # Basic silence removal\n\
          ai-vid-editor -i input.mp4 -o output.mp4 --preset youtube   # YouTube preset\n\
          ai-vid-editor -i input.mp4 -o output.mp4 --enhance --music bg.mp3  # With music\n\
          ai-vid-editor -I ./raw -O ./edited --preset youtube         # Batch process\n\
          ai-vid-editor --watch ./incoming -O ./processed             # Watch mode\n\n\
        For more info: https://github.com/DraconDev/ai-vid-editor"
)]
pub struct Cli {
    /// Input video file to process
    #[arg(group = "input_group", short, long, value_name = "FILE")]
    pub input_file: Option<PathBuf>,

    /// Input directory for batch processing (processes all videos in folder)
    #[arg(group = "input_group", short = 'I', long, value_name = "DIRECTORY")]
    pub input_dir: Option<PathBuf>,

    /// Output video file path
    #[arg(group = "output_group", short, long, value_name = "FILE")]
    pub output_file: Option<PathBuf>,

    /// Output directory for batch processing
    #[arg(group = "output_group", short = 'O', long, value_name = "DIRECTORY")]
    pub output_dir: Option<PathBuf>,

    /// Use a preset profile (youtube, shorts, podcast, minimal)
    /// 
    /// Presets configure optimal settings for common use cases:
    ///   youtube  - Cut silences, enhance audio, export chapters + FCPXML
    ///   shorts   - Speedup silences (3x), enhance audio, tight padding
    ///   podcast  - Cut silences, enhance audio (-16 LUFS), export SRT
    ///   minimal  - Just silence detection, no enhancement
    #[arg(short = 'P', long, value_name = "PRESET")]
    pub preset: Option<String>,

    /// Path to TOML configuration file
    /// 
    /// Config files can be placed at:
    ///   - ./ai-vid-editor.toml (project-local)
    ///   - ~/.config/ai-vid-editor/config.toml (user-global)
    #[arg(short = 'c', long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Silence detection threshold in decibels (e.g., -30.0)
    /// 
    /// Lower values = more sensitive (detects quieter silences)
    /// Common values: -25 (aggressive) to -40 (conservative)
    #[arg(short, long, allow_hyphen_values = true)]
    pub threshold: Option<f32>,

    /// Minimum silence duration to detect in seconds
    /// 
    /// Silences shorter than this are ignored
    #[arg(short, long)]
    pub duration: Option<f32>,

    /// Padding around cuts in seconds
    /// 
    /// Adds a small buffer before and after cuts for natural transitions
    #[arg(short, long)]
    pub padding: Option<f32>,

    /// Speed up silences instead of cutting them
    /// 
    /// Silences are played at 4x speed (configurable via config) instead of being removed
    #[arg(short = 's', long)]
    pub speedup: bool,

    /// Enable audio enhancement (loudness normalization + EQ)
    /// 
    /// Normalizes to -14 LUFS (YouTube standard) and applies speech EQ
    #[arg(short = 'E', long)]
    pub enhance: bool,

    /// Enable audio noise reduction
    /// 
    /// Removes background noise (fans, AC, hiss) using ffmpeg afftdn filter
    #[arg(long)]
    pub noise_reduction: bool,

    /// Background music file to mix with video audio
    /// 
    /// Music will be auto-ducked (lowered) during speech segments
    #[arg(short = 'm', long, value_name = "FILE")]
    pub music: Option<PathBuf>,

    /// Directory containing music files (picks a random track)
    /// 
    /// Supports: mp3, wav, m4a, aac, ogg, flac
    #[arg(long, value_name = "DIRECTORY")]
    pub music_dir: Option<PathBuf>,

    /// Video file to prepend as intro
    /// 
    /// The intro video will be concatenated before the processed content
    #[arg(long, value_name = "FILE")]
    pub intro: Option<PathBuf>,

    /// Video file to append as outro
    /// 
    /// The outro video will be concatenated after the processed content
    #[arg(long, value_name = "FILE")]
    pub outro: Option<PathBuf>,

    /// Generate SRT subtitle file (requires STT - placeholder)
    #[arg(long)]
    pub export_srt: bool,

    /// Generate YouTube chapters file
    #[arg(long)]
    pub export_chapters: bool,

    /// Generate FCPXML for DaVinci Resolve / Premiere Pro
    #[arg(long)]
    pub export_fcpxml: bool,

    /// Generate EDL (Edit Decision List)
    #[arg(long)]
    pub export_edl: bool,

    /// Remove filler words (um, uh, like, etc.) using Whisper STT
    /// 
    /// Requires downloading Whisper model from HuggingFace on first use
    #[arg(long)]
    pub remove_fillers: bool,

    /// Dry run: analyze and show what would be done without processing
    /// 
    /// Shows: input duration, silent segments, estimated output duration, time saved
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Output results as JSON (useful for scripting and CI/CD)
    #[arg(short = 'j', long)]
    pub json: bool,

    /// Generate a sample configuration file
    /// 
    /// Outputs a default TOML config that can be saved to a file
    #[arg(long)]
    pub generate_config: bool,

    /// Watch a directory for new videos and process them automatically
    /// 
    /// Runs continuously, processing new videos as they appear.
    /// Existing files are skipped (not reprocessed).
    #[arg(short = 'w', long, value_name = "DIRECTORY")]
    pub watch: Option<PathBuf>,

    /// Polling interval for watch mode in seconds
    #[arg(long, default_value = "5")]
    pub watch_interval: u64,

    /// Project directory containing config.toml and subfolders (watch/, output/, music/)
    /// 
    /// Auto-loads config.toml and sets up paths for the project
    #[arg(long, value_name = "DIRECTORY")]
    pub project: Option<PathBuf>,

    /// Join multiple input files into one output video
    /// 
    /// Files are concatenated in the order specified
    #[arg(long)]
    pub join: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // If no input specified and not a special command, show help
    if cli.input_file.is_none() 
        && cli.input_dir.is_none() 
        && cli.watch.is_none()
        && !cli.generate_config
        && !cli.dry_run
    {
        // Show help and exit
        use clap::CommandFactory;
        Cli::command().print_help()?;
        println!();
        return Ok(());
    }

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

    // Apply paths from config if not specified on CLI
    let input_file = cli.input_file.clone().or(config.paths.input.clone());
    let input_dir = cli.input_dir.clone().or(config.paths.input_dir.clone());
    let output_file = cli.output_file.clone().or(config.paths.output.clone());
    let output_dir = cli.output_dir.clone().or(config.paths.output_dir.clone());
    let intro = cli.intro.clone().or(config.paths.intro.clone());
    let outro = cli.outro.clone().or(config.paths.outro.clone());
    
    // Music: CLI takes precedence, then config
    let music = cli.music.clone().or(config.paths.music.clone());
    let music_dir = cli.music_dir.clone().or(config.paths.music_dir.clone());

    // Apply CLI overrides for processing settings
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
    if cli.noise_reduction {
        config.audio.noise_reduction = true;
    }
    
    // Handle music selection: --music takes precedence over --music-dir
    if let Some(ref music_path) = cli.music {
        config.audio.music_file = Some(music_path.clone());
    } else if let Some(ref music_dir) = cli.music_dir {
        // Pick a random music file from the directory
        if let Some(random_music) = pick_random_music_file(music_dir)? {
            if !cli.json {
                println!("Selected random music: {:?}", random_music);
            }
            config.audio.music_file = Some(random_music);
        }
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
    if cli.remove_fillers {
        config.filler_words.enabled = true;
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

    // Handle watch mode
    if let Some(watch_dir) = &cli.watch {
        let output_dir = cli.output_dir.clone()
            .ok_or_else(|| anyhow::anyhow!("Output directory (--output-dir) required for watch mode"))?;
        return run_watch_mode(watch_dir, &output_dir, &config, &cli);
    }

    let analyzer = FfmpegAnalyzer;
    let editor = FfmpegEditor;
    let duration_getter = FfmpegDurationGetter;

    if let Some(input_file) = cli.input_file {
        // Single file processing logic
        let output_file = cli.output_file.ok_or_else(|| anyhow::anyhow!("Output file must be specified for single file processing"))?;
        process_single_file_with_intro_outro(
            input_file, 
            output_file, 
            &config, 
            &analyzer, 
            &editor, 
            &duration_getter,
            cli.intro.clone(),
            cli.outro.clone()
        )?;
    } else if let Some(input_dir) = cli.input_dir {
        // Batch processing logic
        let output_dir = cli.output_dir.ok_or_else(|| anyhow::anyhow!("Output directory must be specified for batch processing"))?;
        process_batch_dir(input_dir, output_dir, &config, &analyzer, &editor, &duration_getter)?;
    } else {
        anyhow::bail!("Either an input file or an input directory must be specified.");
    }

    Ok(())
}

/// Run in watch mode - monitor a directory and process new videos
fn run_watch_mode(watch_dir: &PathBuf, output_dir: &PathBuf, config: &Config, cli: &Cli) -> Result<()> {
    use std::collections::HashSet;
    use std::time::Duration;
    
    println!("=== WATCH MODE ===");
    println!("Watching: {:?}", watch_dir);
    println!("Output to: {:?}", output_dir);
    println!("Polling interval: {}s", cli.watch_interval);
    println!("Press Ctrl+C to stop\n");
    
    // Create output directory if it doesn't exist
    std::fs::create_dir_all(output_dir)?;
    
    // Track processed files
    let mut processed: HashSet<PathBuf> = HashSet::new();
    
    // Initial scan - process existing files
    let video_extensions = ["mp4", "mov", "avi", "mkv", "webm"];
    for entry in std::fs::read_dir(watch_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if video_extensions.contains(&ext.to_lowercase().as_str()) {
                processed.insert(path.clone());
            }
        }
    }
    
    println!("Found {} existing files (will not reprocess)", processed.len());
    
    let analyzer = FfmpegAnalyzer;
    let editor = FfmpegEditor;
    let duration_getter = FfmpegDurationGetter;
    
    loop {
        std::thread::sleep(Duration::from_secs(cli.watch_interval));
        
        // Check for new files
        if let Ok(entries) = std::fs::read_dir(watch_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if video_extensions.contains(&ext.to_lowercase().as_str()) && !processed.contains(&path) {
                        println!("\n[NEW FILE] {:?}", path);
                        
                        let file_name = path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "output.mp4".to_string());
                        let output_path = output_dir.join(&file_name);
                        
                        // Process with intro/outro if specified
                        match process_single_file_with_intro_outro(
                            path.clone(),
                            output_path.clone(),
                            config,
                            &analyzer,
                            &editor,
                            &duration_getter,
                            cli.intro.clone(),
                            cli.outro.clone()
                        ) {
                            Ok(_) => {
                                println!("[DONE] Processed: {:?}", path);
                                processed.insert(path);
                            }
                            Err(e) => {
                                eprintln!("[ERROR] Failed to process {:?}: {}", path, e);
                                // Still mark as processed to avoid retrying
                                processed.insert(path);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Pick a random music file from a directory
fn pick_random_music_file(music_dir: &PathBuf) -> Result<Option<PathBuf>> {
    use std::fs;
    use rand::prelude::*;
    
    if !music_dir.exists() {
        anyhow::bail!("Music directory does not exist: {:?}", music_dir);
    }
    
    let music_extensions = ["mp3", "wav", "m4a", "aac", "ogg", "flac"];
    
    let music_files: Vec<PathBuf> = fs::read_dir(music_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| music_extensions.contains(&ext.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .collect();
    
    if music_files.is_empty() {
        eprintln!("Warning: No music files found in {:?}", music_dir);
        return Ok(None);
    }
    
    let mut rng = rand::rng();
    Ok(music_files.choose(&mut rng).cloned())
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
