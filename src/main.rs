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
pub mod ml;

use crate::batch_processor::{process_single_file_with_intro_outro, process_batch_dir, FfmpegDurationGetter};
use crate::analyzer::FfmpegAnalyzer;
use crate::editor::FfmpegEditor;
use crate::config::{Config, Preset};

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "AI video editor - configure via config.toml, CLI for quick overrides",
    long_about = "AI Video Editor\n\n\
        CONFIG-FIRST: Create ai-vid-editor.toml with all your settings.\n\
        CLI flags are optional overrides.\n\n\
        QUICK START:\n\
          ai-vid-editor --generate-config > config.toml  # Create config\n\
          ai-vid-editor --config config.toml             # Run with config\n\
          ai-vid-editor --project ./my-project           # Project mode\n\n\
        See ai-vid-editor.example.toml for full documentation."
)]
pub struct Cli {
    /// Input video file
    #[arg(group = "input_group", short, long)]
    pub input_file: Option<PathBuf>,

    /// Input directory (batch mode)
    #[arg(group = "input_group", short = 'I', long)]
    pub input_dir: Option<PathBuf>,

    /// Output video file
    #[arg(group = "output_group", short, long)]
    pub output_file: Option<PathBuf>,

    /// Output directory (batch mode)
    #[arg(group = "output_group", short = 'O', long)]
    pub output_dir: Option<PathBuf>,

    /// Config file path (default: ./ai-vid-editor.toml)
    #[arg(short = 'c', long)]
    pub config: Option<PathBuf>,

    /// Project directory (loads config.toml from here)
    #[arg(long)]
    pub project: Option<PathBuf>,

    /// Preset: youtube, shorts, podcast, minimal
    #[arg(short = 'P', long)]
    pub preset: Option<String>,

    /// Watch mode - monitor input_dir for new videos
    #[arg(short = 'w', long)]
    pub watch: Option<PathBuf>,

    /// Dry run - preview without processing
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// JSON output
    #[arg(short = 'j', long)]
    pub json: bool,

    /// Generate sample config
    #[arg(long)]
    pub generate_config: bool,

    // === Advanced overrides (use config file instead) ===
    
    #[arg(long, hide = true)]
    pub threshold: Option<f32>,
    
    #[arg(long, hide = true)]
    pub duration: Option<f32>,
    
    #[arg(short = 'p', long, hide = true)]
    pub padding: Option<f32>,
    
    #[arg(short = 's', long, hide = true)]
    pub speedup: bool,
    
    #[arg(short = 'E', long, hide = true)]
    pub enhance: bool,
    
    #[arg(long, hide = true)]
    pub noise_reduction: bool,
    
    #[arg(short = 'm', long, hide = true)]
    pub music: Option<PathBuf>,
    
    #[arg(long, hide = true)]
    pub music_dir: Option<PathBuf>,
    
    #[arg(long, hide = true)]
    pub intro: Option<PathBuf>,
    
    #[arg(long, hide = true)]
    pub outro: Option<PathBuf>,
    
    #[arg(long, hide = true)]
    pub export_srt: bool,
    
    #[arg(long, hide = true)]
    pub export_chapters: bool,
    
    #[arg(long, hide = true)]
    pub export_fcpxml: bool,
    
    #[arg(long, hide = true)]
    pub export_edl: bool,
    
    #[arg(long, hide = true)]
    pub remove_fillers: bool,
    
    #[arg(long, hide = true, default_value = "5")]
    pub watch_interval: u64,
    
    #[arg(long, hide = true)]
    pub join: bool,
    
    /// Enable video stabilization (removes camera shake)
    #[arg(long)]
    pub stabilize: bool,
    
    /// Enable auto color correction
    #[arg(long)]
    pub color_correct: bool,
    
    /// Auto-reframe horizontal video to vertical (9:16) following speaker's face
    #[arg(long)]
    pub reframe: bool,
    
    /// Blur background while keeping speaker sharp
    #[arg(long)]
    pub blur_background: bool,
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
    if let Some(ref music_path) = music {
        config.audio.music_file = Some(music_path.clone());
    } else if let Some(ref music_dir_path) = music_dir {
        // Pick a random music file from the directory
        if let Some(random_music) = pick_random_music_file(music_dir_path)? {
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
    
    // Video processing overrides
    if cli.stabilize {
        config.video.stabilize = true;
    }
    if cli.color_correct {
        config.video.color_correct = true;
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

    // Handle watch mode (from config or CLI)
    let watch_enabled = config.watch.enabled || cli.watch.is_some();
    let watch_dir = cli.watch.clone().or(input_dir.clone());
    
    if watch_enabled {
        let watch_path = watch_dir.clone()
            .ok_or_else(|| anyhow::anyhow!("Watch directory required (set input_dir in config or use --watch)"))?;
        let out_dir = output_dir.clone()
            .ok_or_else(|| anyhow::anyhow!("Output directory required for watch mode"))?;
        return run_watch_mode(&watch_path, &out_dir, &config, &intro, &outro);
    }

    let analyzer = FfmpegAnalyzer;
    let editor = FfmpegEditor;
    let duration_getter = FfmpegDurationGetter;

    if let Some(input) = input_file {
        // Single file processing logic
        let out = output_file.ok_or_else(|| anyhow::anyhow!("Output file must be specified"))?;
        process_single_file_with_intro_outro(
            input, 
            out, 
            &config, 
            &analyzer, 
            &editor, 
            &duration_getter,
            intro,
            outro
        )?;
    } else if let Some(in_dir) = input_dir {
        // Batch processing logic
        let out_dir = output_dir.ok_or_else(|| anyhow::anyhow!("Output directory must be specified"))?;
        process_batch_dir(in_dir, out_dir, &config, &analyzer, &editor, &duration_getter)?;
    } else {
        anyhow::bail!("Either an input file or an input directory must be specified (set in config or CLI).");
    }

    Ok(())
}

/// Run in watch mode - monitor a directory and process new videos
fn run_watch_mode(watch_dir: &PathBuf, output_dir: &PathBuf, config: &Config, intro: &Option<PathBuf>, outro: &Option<PathBuf>) -> Result<()> {
    use std::collections::HashSet;
    use std::time::Duration;
    
    println!("=== WATCH MODE ===");
    println!("Watching: {:?}", watch_dir);
    println!("Output to: {:?}", output_dir);
    println!("Polling interval: {}s", config.watch.interval);
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
        std::thread::sleep(Duration::from_secs(config.watch.interval));
        
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
                            intro.clone(),
                            outro.clone()
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
