#![allow(clippy::too_many_arguments)]
#![allow(clippy::should_implement_trait)]

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

pub mod analyzer;
pub mod batch_processor;
pub mod config;
pub mod editor;
pub mod exporter;
pub mod ml;
pub mod stt_analyzer;
pub mod utils;

use crate::analyzer::FfmpegAnalyzer;
use crate::batch_processor::{
    FfmpegDurationGetter, process_batch_dir, process_single_file_with_intro_outro,
};

fn timestamp() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
use crate::config::{Config, Preset};
use crate::editor::FfmpegEditor;

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

    /// Send desktop notifications on processing events
    #[arg(long)]
    pub notify: bool,

    /// Launch graphical interface
    #[arg(long)]
    pub gui: bool,

    /// Verbose output (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Quiet mode (suppress non-error output)
    #[arg(short, long, conflicts_with = "verbose")]
    pub quiet: bool,
}

#[cfg(feature = "notify-rust")]
fn send_notification(summary: &str, body: &str) {
    let _ = notify_rust::Notification::new()
        .summary(summary)
        .body(body)
        .show();
}

#[cfg(not(feature = "notify-rust"))]
fn send_notification(_summary: &str, _body: &str) {}

fn notify_processing(input: &std::path::Path) {
    send_notification("Processing Started", &format!("{}", input.display()));
}

fn notify_complete(input: &std::path::Path, output: &std::path::Path) {
    send_notification(
        "Processing Complete",
        &format!("{}\n→ {}", input.display(), output.display()),
    );
}

fn notify_error(input: &std::path::Path, error: &str) {
    send_notification(
        "Processing Error",
        &format!("{}\nError: {}", input.display(), error),
    );
}

fn init_logging(verbose: u8, quiet: bool) {
    let filter = if quiet {
        "error"
    } else {
        match verbose {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        }
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::new(filter)
                .add_directive("candle=warn".parse().unwrap())
                .add_directive("tract=warn".parse().unwrap()),
        )
        .with_target(false)
        .with_file(verbose >= 2)
        .with_line_number(verbose >= 2)
        .init();
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging (skip for GUI)
    if !cli.gui {
        init_logging(cli.verbose, cli.quiet);
    }

    // Handle --gui flag
    if cli.gui {
        #[cfg(feature = "gui")]
        {
            return run_gui();
        }
        #[cfg(not(feature = "gui"))]
        {
            anyhow::bail!("GUI not compiled in. Build with --features gui to enable.");
        }
    }

    // Pre-load config to check if watch_folders are configured
    // This must happen before the TTY/GUI check
    let preloaded_config = {
        let mut cfg = Config::default();
        if let Some(ref config_path) = cli.config {
            if config_path.exists() {
                if let Ok(file_config) = Config::from_file(config_path) {
                    cfg = cfg.merge(file_config);
                }
            }
        } else if let Some(default_path) = Config::default_config_path() {
            if default_path.exists() {
                if let Ok(file_config) = Config::from_file(&default_path) {
                    cfg = cfg.merge(file_config);
                }
            }
        }
        cfg
    };

    let has_watch_folders = preloaded_config
        .paths
        .watch_folders
        .iter()
        .any(|f| f.enabled);

    // If no input specified and not a special command, launch GUI or show help
    if cli.input_file.is_none()
        && cli.input_dir.is_none()
        && cli.watch.is_none()
        && !cli.generate_config
        && !cli.dry_run
        && !has_watch_folders
    {
        // Check if running from terminal (TTY) or launched from desktop
        let is_tty = unsafe { libc::isatty(libc::STDOUT_FILENO) != 0 };

        #[cfg(feature = "gui")]
        if !is_tty {
            return run_gui();
        }

        // Show help and exit
        use clap::CommandFactory;
        Cli::command().print_help()?;
        println!();

        #[cfg(feature = "gui")]
        println!("Run with --gui to launch the graphical interface.");

        return Ok(());
    }

    // Handle --generate-config
    if cli.generate_config {
        let config_content = Config::generate_default_toml()?;
        println!("{}", config_content);
        println!("\n# Save this to 'ai-vid-editor.toml' or '~/.config/ai-vid-editor/config.toml'");
        return Ok(());
    }

    // Apply config: use preloaded config if no CLI preset was specified
    let mut config = if let Some(ref preset_str) = cli.preset {
        let preset = Preset::from_str(preset_str).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown preset: {}. Valid presets: youtube, shorts, podcast, minimal",
                preset_str
            )
        })?;
        if !cli.json {
            println!("Using preset: {}", preset.as_str());
        }
        let mut c = preset.to_config();
        // Merge preloaded config over preset (so watch_folders etc. are preserved)
        c = c.merge(preloaded_config);
        c
    } else {
        preloaded_config
    };

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
        // Only try to pick music if the directory exists
        if music_dir_path.exists()
            && let Some(random_music) = pick_random_music_file(music_dir_path)?
        {
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
    if cli.reframe {
        config.video.reframe = true;
    }
    if cli.blur_background {
        config.video.blur_background = true;
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
        println!(
            "  Export: SRT={} Chapters={} FCPXML={} EDL={}",
            config.export.subtitles,
            config.export.chapters,
            config.export.fcpxml,
            config.export.edl
        );
    }

    // Handle dry-run mode
    if cli.dry_run {
        return handle_dry_run(&cli, &config);
    }

    // Handle watch mode (from config or CLI)
    let has_watch_folders = config
        .paths
        .watch_folders
        .iter()
        .any(|f| f.enabled);
    let watch_enabled = config.watch.enabled || cli.watch.is_some() || has_watch_folders;
    eprintln!("[DEBUG] watch.enabled={}, has_watch_folders={}, watch_enabled={}, watch_folders={}", 
        config.watch.enabled, has_watch_folders, watch_enabled, config.paths.watch_folders.len());
    let watch_dir = cli.watch.clone().or(input_dir.clone());

    if watch_enabled {
        // If CLI --watch flag was used, watch that single directory
        if cli.watch.is_some() {
            let watch_path = watch_dir.clone().ok_or_else(|| {
                anyhow::anyhow!("Watch directory required (use --watch <dir>)")
            })?;
            let out_dir = output_dir
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Output directory required for watch mode"))?;
            return run_watch_mode(&watch_path, &out_dir, &config, &intro, &outro, cli.notify);
        }

        // Use watch_folders from config if available and enabled
        if has_watch_folders {
            return run_multi_watch_mode(&config, &cli);
        }

        // Fallback: single watch dir from input_dir
        let watch_path = watch_dir.clone().ok_or_else(|| {
            anyhow::anyhow!("Watch directory required (set input_dir in config or use --watch)")
        })?;
        let out_dir = output_dir
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Output directory required for watch mode"))?;
        return run_watch_mode(&watch_path, &out_dir, &config, &intro, &outro, cli.notify);
    }

    let analyzer = FfmpegAnalyzer;
    let editor = FfmpegEditor;
    let duration_getter = FfmpegDurationGetter;

    if let Some(input) = input_file {
        // Single file processing logic
        let out = output_file.ok_or_else(|| anyhow::anyhow!("Output file must be specified"))?;

        if cli.notify {
            notify_processing(&input);
        }

        let result = process_single_file_with_intro_outro(
            input.clone(),
            out.clone(),
            &config,
            &analyzer,
            &editor,
            &duration_getter,
            intro,
            outro,
        );

        match &result {
            Ok(_) => {
                if cli.notify {
                    notify_complete(&input, &out);
                }
            }
            Err(e) => {
                if cli.notify {
                    notify_error(&input, &e.to_string());
                }
            }
        }

        result?;
    } else if let Some(in_dir) = input_dir {
        // Batch processing logic
        let out_dir =
            output_dir.ok_or_else(|| anyhow::anyhow!("Output directory must be specified"))?;
        process_batch_dir(
            in_dir,
            out_dir,
            &config,
            &analyzer,
            &editor,
            &duration_getter,
        )?;
    } else {
        anyhow::bail!(
            "Either an input file or an input directory must be specified (set in config or CLI)."
        );
    }

    Ok(())
}

/// Run in watch mode - monitor a directory and process new videos
fn run_watch_mode(
    watch_dir: &PathBuf,
    output_dir: &PathBuf,
    config: &Config,
    intro: &Option<PathBuf>,
    outro: &Option<PathBuf>,
    notify: bool,
) -> Result<()> {
    use std::collections::HashSet;
    use std::time::Duration;

    println!("=== WATCH MODE ===");
    println!("Watching: {}", watch_dir.display());
    println!("Output to: {}", output_dir.display());
    println!("Polling every {}s", config.watch.interval);
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

        if let Some(ext) = path.extension().and_then(|e| e.to_str())
            && video_extensions.contains(&ext.to_lowercase().as_str())
        {
            let name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
            println!("  Skipping existing: {}", name);
            processed.insert(path);
        }
    }

    if processed.is_empty() {
        println!("  No existing videos found. Drop a video in {:?} to start processing.", watch_dir);
    } else {
        println!("  {} existing file(s) skipped (already present on startup).", processed.len());
    }

    let analyzer = FfmpegAnalyzer;
    let editor = FfmpegEditor;
    let duration_getter = FfmpegDurationGetter;

    let mut heartbeat = 0u32;

    loop {
        std::thread::sleep(Duration::from_secs(config.watch.interval));
        heartbeat += 1;

        // Print a heartbeat every ~30s so user knows we're still watching
        if heartbeat % 6 == 0 {
            println!("[{}] Watching {:?} for new files...", timestamp(), watch_dir);
        }

        // Check for new files
        if let Ok(entries) = std::fs::read_dir(watch_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if let Some(ext) = path.extension().and_then(|e| e.to_str())
                    && video_extensions.contains(&ext.to_lowercase().as_str())
                    && !processed.contains(&path)
                {
                    let now = timestamp();
                    println!("\n[{}] [NEW FILE] {:?}", now, path);

                    let file_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "output.mp4".to_string());
                    let output_path = output_dir.join(&file_name);

                    println!("[{}] [START] Processing {}...", now, file_name);

                    if notify {
                        notify_processing(&path);
                    }

                    let start_time = std::time::Instant::now();

                    // Process with intro/outro, showing progress
                    let file_name_for_progress = file_name.clone();
                    let result = crate::batch_processor::process_single_file_with_intro_outro_progress(
                        path.clone(),
                        output_path.clone(),
                        config,
                        &analyzer,
                        &editor,
                        &duration_getter,
                        intro.clone(),
                        outro.clone(),
                        move |p| {
                            let now = timestamp();
                            println!(
                                "[{}] [{:.0}%] {} - {}",
                                now,
                                p.fraction * 100.0,
                                file_name_for_progress,
                                p.stage
                            );
                        },
                    );

                    match &result {
                        Ok(_) => {
                            let elapsed = start_time.elapsed().as_secs_f32();
                            println!(
                                "[{}] [DONE] {} -> {} ({:.1}s)",
                                timestamp(),
                                file_name,
                                output_path.display(),
                                elapsed
                            );
                            if notify {
                                notify_complete(&path, &output_path);
                            }
                            processed.insert(path);
                        }
                        Err(e) => {
                            let elapsed = start_time.elapsed().as_secs_f32();
                            eprintln!(
                                "[{}] [ERROR] {} failed after {:.1}s: {}",
                                timestamp(),
                                file_name,
                                elapsed,
                                e
                            );
                            if notify {
                                notify_error(&path, &e.to_string());
                            }
                            // Still mark as processed to avoid retrying
                            processed.insert(path);
                        }
                    }
                }
            }
        }
    }
}

/// Run watch mode for multiple folders from config
fn run_multi_watch_mode(config: &Config, cli: &Cli) -> Result<()> {
    use std::collections::HashSet;
    use std::time::Duration;

    let enabled_folders: Vec<&crate::config::WatchFolder> = config
        .paths
        .watch_folders
        .iter()
        .filter(|f| f.enabled)
        .collect();

    if enabled_folders.is_empty() {
        anyhow::bail!("No enabled watch folders in config");
    }

    println!("=== MULTI-FOLDER WATCH MODE ===");
    println!("Config: ~/.config/ai-vid-editor/config.toml");
    println!("Watching {} folder(s):", enabled_folders.len());
    for folder in &enabled_folders {
        println!("  {} -> {} [{}]", folder.input.display(), folder.output.display(), folder.preset);
    }
    println!("Polling every {}s", config.watch.interval);
    println!("Press Ctrl+C to stop\n");

    let analyzer = FfmpegAnalyzer;
    let editor = FfmpegEditor;
    let duration_getter = FfmpegDurationGetter;
    let video_extensions = ["mp4", "mov", "avi", "mkv", "webm"];

    // Track processed files per folder
    let mut processed_sets: Vec<HashSet<PathBuf>> = Vec::new();
    for folder in &enabled_folders {
        let mut processed: HashSet<PathBuf> = HashSet::new();

        // Create output directory
        std::fs::create_dir_all(&folder.output)?;

        // Initial scan - mark existing files as already processed
        if let Ok(entries) = std::fs::read_dir(&folder.input) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str())
                    && video_extensions.contains(&ext.to_lowercase().as_str())
                {
                    let name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
                    println!("  Skipping existing: {}", name);
                    processed.insert(path);
                }
            }
        }

        if processed.is_empty() {
            println!("  {:?}: No videos found. Drop a video here to start processing.", folder.input);
        } else {
            println!("  {:?}: {} existing file(s) skipped.", folder.input, processed.len());
        }
        processed_sets.push(processed);
    }

    let mut heartbeat = 0u32;

    loop {
        std::thread::sleep(Duration::from_secs(config.watch.interval));
        heartbeat += 1;

        // Print a heartbeat every ~30s so user knows we're still watching
        if heartbeat % 6 == 0 {
            println!("[{}] Watching {} folder(s) for new files...", timestamp(), enabled_folders.len());
        }

        for (idx, folder) in enabled_folders.iter().enumerate() {
            if let Ok(entries) = std::fs::read_dir(&folder.input) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if let Some(ext) = path.extension().and_then(|e| e.to_str())
                        && video_extensions.contains(&ext.to_lowercase().as_str())
                        && !processed_sets[idx].contains(&path)
                    {
                        let now = timestamp();
                        println!("\n[{}] [NEW FILE] {:?}", now, path);

                        // Build config for this folder's preset
                        let folder_config = if let Some(preset) =
                            crate::config::Preset::from_str(&folder.preset)
                        {
                            let mut c = preset.to_config();
                            // Apply folder-level settings overrides
                            if let Some(enhance) = folder.settings.enhance_audio {
                                c.audio.enhance = enhance;
                            }
                            if let Some(threshold) = folder.settings.silence_threshold_db {
                                c.silence.threshold_db = threshold;
                            }
                            if let Some(lufs) = folder.settings.target_lufs {
                                c.audio.target_lufs = lufs;
                            }
                            if let Some(stabilize) = folder.settings.stabilize {
                                c.video.stabilize = stabilize;
                            }
                            if let Some(color_correct) = folder.settings.color_correct {
                                c.video.color_correct = color_correct;
                            }
                            if let Some(reframe) = folder.settings.reframe {
                                c.video.reframe = reframe;
                            }
                            if let Some(blur) = folder.settings.blur_background {
                                c.video.blur_background = blur;
                            }
                            c
                        } else {
                            eprintln!(
                                "Warning: Unknown preset '{}', using default config",
                                folder.preset
                            );
                            config.clone()
                        };

                        let file_name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "output.mp4".to_string());
                        let output_path = folder.output.join(&file_name);

                        println!("[{}] [START] Processing {}...", now, file_name);

                        if cli.notify {
                            notify_processing(&path);
                        }

                        let start_time = std::time::Instant::now();
                        let progress_name = file_name.clone();
                        let result = crate::batch_processor::process_single_file_with_intro_outro_progress(
                            path.clone(),
                            output_path.clone(),
                            &folder_config,
                            &analyzer,
                            &editor,
                            &duration_getter,
                            None,
                            None,
                            move |p| {
                                let now = timestamp();
                                println!(
                                    "[{}] [{:.0}%] {} - {}",
                                    now,
                                    p.fraction * 100.0,
                                    progress_name,
                                    p.stage
                                );
                            },
                        );

                        match &result {
                            Ok(_) => {
                                let elapsed = start_time.elapsed().as_secs_f32();
                                println!(
                                    "[{}] [DONE] {} -> {} ({:.1}s)",
                                    timestamp(),
                                    file_name,
                                    output_path.display(),
                                    elapsed
                                );
                                if cli.notify {
                                    notify_complete(&path, &output_path);
                                }
                                processed_sets[idx].insert(path);
                            }
                            Err(e) => {
                                let elapsed = start_time.elapsed().as_secs_f32();
                                eprintln!(
                                    "[{}] [ERROR] {} failed after {:.1}s: {}",
                                    timestamp(),
                                    file_name,
                                    elapsed,
                                    e
                                );
                                if cli.notify {
                                    notify_error(&path, &e.to_string());
                                }
                                processed_sets[idx].insert(path);
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
    use rand::prelude::*;
    use std::fs;

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

    let input_path = cli
        .input_file
        .as_ref()
        .or(cli.input_dir.as_ref())
        .ok_or_else(|| anyhow::anyhow!("Input file or directory required"))?;

    let silences = analyzer.detect_silence(
        input_path,
        config.silence.threshold_db,
        config.silence.min_duration,
    )?;
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
        println!(
            "Input duration: {:.1}s ({:.1} min)",
            video_duration,
            video_duration / 60.0
        );
        println!("Silent segments detected: {}", silences.len());
        println!(
            "Total silence: {:.1}s ({:.1} min)",
            total_silence,
            total_silence / 60.0
        );
        println!("\nWould produce:");
        println!(
            "  Output duration: {:.1}s ({:.1} min)",
            output_duration,
            output_duration / 60.0
        );
        println!(
            "  Time saved: {:.1}s ({:.1} min)",
            video_duration - output_duration,
            (video_duration - output_duration) / 60.0
        );
        println!("\nOperations:");
        println!("  - Silence mode: {:?}", config.silence.mode);
        if config.audio.enhance {
            println!(
                "  - Audio enhancement: enabled (target {} LUFS)",
                config.audio.target_lufs
            );
        }
        if config.audio.music_file.is_some() {
            println!("  - Background music: {:?}", config.audio.music_file);
        }
        if config.export.subtitles {
            println!("  - Export SRT subtitles");
        }
        if config.export.chapters {
            println!("  - Export YouTube chapters");
        }
        if config.export.fcpxml {
            println!("  - Export FCPXML");
        }
        if config.export.edl {
            println!("  - Export EDL");
        }
    }

    Ok(())
}

#[cfg(feature = "gui")]
fn run_gui() -> Result<()> {
    use eframe::egui;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1150.0, 820.0])
            .with_min_inner_size([1000.0, 700.0])
            .with_title("AI Video Editor")
            .with_app_id("ai-vid-editor"),
        ..Default::default()
    };

    eframe::run_native(
        "AI Video Editor",
        options,
        Box::new(|cc| {
            configure_dark_theme(&cc.egui_ctx);
            Ok(Box::new(gui::App::new()))
        }),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {}", e))
}

#[cfg(feature = "gui")]
fn configure_dark_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.visuals = egui::Visuals::dark();

    style.visuals.panel_fill = egui::Color32::from_rgb(10, 10, 10);
    style.visuals.window_fill = egui::Color32::from_rgb(20, 20, 20);
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(5, 5, 5);

    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 30, 30);
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(35, 35, 35);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(45, 45, 45);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(50, 50, 50);

    style.visuals.selection.bg_fill = egui::Color32::from_rgb(230, 57, 70);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 107, 107));

    style.visuals.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(150, 150, 150));
    style.visuals.widgets.inactive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 200));
    style.visuals.widgets.hovered.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(245, 245, 245));
    style.visuals.widgets.active.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 255, 255));

    style.visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(42, 42, 42));

    ctx.set_style(style);
}

#[cfg(feature = "gui")]
mod gui;
