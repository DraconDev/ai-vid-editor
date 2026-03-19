use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// How to handle detected silences
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SilenceMode {
    /// Cut out silences completely (default)
    #[default]
    Cut,
    /// Speed up silences instead of cutting
    Speedup,
}

/// Preset profiles for common use cases
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    /// YouTube long-form: silence cut + audio enhance + chapters
    Youtube,
    /// YouTube Shorts/TikTok: speedup mode + audio enhance
    Shorts,
    /// Podcast: silence cut + audio enhance + SRT subtitles
    Podcast,
    /// Minimal: just silence detection, no enhancement
    Minimal,
}

impl Preset {
    /// Apply preset to create a config
    pub fn to_config(&self) -> Config {
        let mut config = Config::default();

        match self {
            Preset::Youtube => {
                // Long-form YouTube: cut silences, enhance audio, generate chapters
                config.silence.mode = SilenceMode::Cut;
                config.silence.padding = 0.15; // Slightly more padding for natural flow
                config.audio.enhance = true;
                config.export.chapters = true;
                config.export.fcpxml = true;
            }
            Preset::Shorts => {
                // Short-form: speedup silences, enhance audio, extract clips
                config.silence.mode = SilenceMode::Speedup;
                config.silence.speedup_factor = 3.0;
                config.silence.padding = 0.05; // Tighter for fast-paced content
                config.audio.enhance = true;
                config.export.clips = true;
            }
            Preset::Podcast => {
                // Podcast: cut silences, enhance audio, generate subtitles
                config.silence.mode = SilenceMode::Cut;
                config.silence.padding = 0.2; // More padding for conversational flow
                config.audio.enhance = true;
                config.audio.target_lufs = -16.0; // Podcast standard
                config.export.subtitles = true;
                config.export.captions = true;
            }
            Preset::Minimal => {
                // Just silence detection, nothing else
                config.silence.mode = SilenceMode::Cut;
                config.audio.enhance = false;
            }
        }

        config
    }

    /// Get preset name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Preset::Youtube => "youtube",
            Preset::Shorts => "shorts",
            Preset::Podcast => "podcast",
            Preset::Minimal => "minimal",
        }
    }

    /// Parse preset from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "youtube" => Some(Preset::Youtube),
            "shorts" | "tiktok" | "reels" => Some(Preset::Shorts),
            "podcast" => Some(Preset::Podcast),
            "minimal" => Some(Preset::Minimal),
            _ => None,
        }
    }
}

/// Configuration for silence detection and handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SilenceConfig {
    /// Silence detection threshold in dB (e.g., -30.0)
    #[serde(default = "default_threshold_db")]
    pub threshold_db: f32,

    /// Minimum silence duration to detect (seconds)
    #[serde(default = "default_min_duration")]
    pub min_duration: f32,

    /// Padding around cuts (seconds)
    #[serde(default = "default_padding")]
    pub padding: f32,

    /// How to handle silences: "cut" or "speedup"
    #[serde(default)]
    pub mode: SilenceMode,

    /// Speed multiplier when mode = "speedup"
    #[serde(default = "default_speedup_factor")]
    pub speedup_factor: f32,

    /// Only speedup silences longer than this (seconds)
    #[serde(default = "default_min_silence_for_speedup")]
    pub min_silence_for_speedup: f32,
}

fn default_threshold_db() -> f32 {
    -30.0
}
fn default_min_duration() -> f32 {
    0.5
}
fn default_padding() -> f32 {
    0.1
}
fn default_speedup_factor() -> f32 {
    4.0
}
fn default_min_silence_for_speedup() -> f32 {
    0.5
}

impl Default for SilenceConfig {
    fn default() -> Self {
        Self {
            threshold_db: default_threshold_db(),
            min_duration: default_min_duration(),
            padding: default_padding(),
            mode: SilenceMode::Cut,
            speedup_factor: default_speedup_factor(),
            min_silence_for_speedup: default_min_silence_for_speedup(),
        }
    }
}

/// Configuration for filler word removal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillerWordsConfig {
    /// Enable filler word removal (requires STT)
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Words to remove
    #[serde(default = "default_filler_words")]
    pub words: Vec<String>,

    /// Padding around filler cuts (seconds)
    #[serde(default = "default_filler_padding")]
    pub padding: f32,
}

fn default_true() -> bool {
    true
}
fn default_filler_words() -> Vec<String> {
    vec!["um".into(), "uh".into(), "ah".into(), "er".into()]
}
fn default_filler_padding() -> f32 {
    0.05
}

impl Default for FillerWordsConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            words: default_filler_words(),
            padding: default_filler_padding(),
        }
    }
}

/// Configuration for audio processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Enable audio enhancement
    #[serde(default = "default_true")]
    pub enhance: bool,

    /// Enable noise reduction
    #[serde(default)]
    pub noise_reduction: bool,

    /// Target loudness (LUFS) - YouTube standard is -14
    #[serde(default = "default_target_lufs")]
    pub target_lufs: f32,

    /// Path to background music file
    #[serde(default)]
    pub music_file: Option<PathBuf>,

    /// Volume reduction during speech (0.0-1.0)
    #[serde(default = "default_duck_volume")]
    pub duck_volume: f32,
}

fn default_target_lufs() -> f32 {
    -14.0
}
fn default_duck_volume() -> f32 {
    0.2
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            enhance: default_true(),
            noise_reduction: false,
            target_lufs: default_target_lufs(),
            music_file: None,
            duck_volume: default_duck_volume(),
        }
    }
}

/// Configuration for export options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportConfig {
    /// Generate SRT subtitles (raw text with timestamps)
    #[serde(default)]
    pub subtitles: bool,

    /// Burn styled subtitles into the video
    #[serde(default)]
    pub captions: bool,

    /// Generate YouTube chapters
    #[serde(default)]
    pub chapters: bool,

    /// Extract highlight clips for Shorts/Reels
    #[serde(default)]
    pub clips: bool,

    /// Number of clips to extract
    #[serde(default = "default_clip_count")]
    pub clip_count: u32,

    /// Minimum clip duration in seconds
    #[serde(default = "default_clip_min_duration")]
    pub clip_min_duration: f32,

    /// Maximum clip duration in seconds
    #[serde(default = "default_clip_max_duration")]
    pub clip_max_duration: f32,

    /// Generate FCPXML for DaVinci/Premiere
    #[serde(default)]
    pub fcpxml: bool,

    /// Generate EDL
    #[serde(default)]
    pub edl: bool,
}

fn default_clip_count() -> u32 {
    3
}
fn default_clip_min_duration() -> f32 {
    15.0
}
fn default_clip_max_duration() -> f32 {
    60.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FolderSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_audio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_silence: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silence_threshold_db: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_lufs: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stabilize: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_correct: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reframe: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blur_background: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitles: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapters: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clips: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchFolder {
    pub input: PathBuf,
    pub output: PathBuf,
    #[serde(default = "default_preset")]
    pub preset: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "FolderSettings::is_default")]
    pub settings: FolderSettings,
}

impl FolderSettings {
    fn is_default(&self) -> bool {
        self.enhance_audio.is_none()
            && self.remove_silence.is_none()
            && self.silence_threshold_db.is_none()
            && self.target_lufs.is_none()
            && self.stabilize.is_none()
            && self.color_correct.is_none()
            && self.reframe.is_none()
            && self.blur_background.is_none()
    }
}

fn default_preset() -> String {
    "youtube".to_string()
}

impl Default for WatchFolder {
    fn default() -> Self {
        Self {
            input: PathBuf::from("videos"),
            output: PathBuf::from("videos/output"),
            preset: default_preset(),
            enabled: true,
            settings: FolderSettings::default(),
        }
    }
}

/// Configuration for paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    /// Input video file (single file mode)
    #[serde(default)]
    pub input: Option<PathBuf>,

    /// Input directory (batch mode)
    #[serde(default)]
    pub input_dir: Option<PathBuf>,

    /// Output video file (single file mode)
    #[serde(default)]
    pub output: Option<PathBuf>,

    /// Output directory (batch mode)
    #[serde(default)]
    pub output_dir: Option<PathBuf>,

    /// Background music file
    #[serde(default)]
    pub music: Option<PathBuf>,

    /// Background music directory
    #[serde(default)]
    pub music_dir: Option<PathBuf>,

    /// Intro video
    #[serde(default)]
    pub intro: Option<PathBuf>,

    /// Outro video
    #[serde(default)]
    pub outro: Option<PathBuf>,

    /// Watch folders for GUI mode
    #[serde(default = "default_watch_folders")]
    pub watch_folders: Vec<WatchFolder>,
}

fn default_watch_folders() -> Vec<WatchFolder> {
    vec![WatchFolder::default()]
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            input: None,
            input_dir: Some(PathBuf::from("watch")),
            output: None,
            output_dir: Some(PathBuf::from("output")),
            music: None,
            music_dir: Some(PathBuf::from("music")),
            intro: None,
            outro: None,
            watch_folders: default_watch_folders(),
        }
    }
}

/// Configuration for watch mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    /// Enable watch mode
    #[serde(default)]
    pub enabled: bool,

    /// Polling interval in seconds
    #[serde(default = "default_watch_interval")]
    pub interval: u64,
}

fn default_watch_interval() -> u64 {
    5
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval: default_watch_interval(),
        }
    }
}

/// Configuration for video processing
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VideoConfig {
    /// Enable video stabilization (vidstab filter)
    #[serde(default)]
    pub stabilize: bool,

    /// Enable auto color correction
    #[serde(default)]
    pub color_correct: bool,

    /// Enable auto-reframe (horizontal to vertical, follows face)
    #[serde(default)]
    pub reframe: bool,

    /// Enable background blur (person segmentation)
    #[serde(default)]
    pub blur_background: bool,
}

/// Join mode for combining processed videos
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum JoinMode {
    /// Don't join videos
    #[default]
    Off,
    /// Join videos by date (newest first)
    ByDate,
    /// Join videos alphabetically by name
    ByName,
    /// Join after N videos processed
    AfterCount,
}

/// Configuration for processing options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    /// How to join processed videos
    #[serde(default)]
    pub join_mode: JoinMode,

    /// Number of videos after which to join (when join_mode = AfterCount)
    #[serde(default = "default_join_after_count")]
    pub join_after_count: u32,

    /// Output filename pattern for joined videos
    /// Supports: {date}, {time}, {count}
    #[serde(default = "default_join_pattern")]
    pub join_output_pattern: String,
}

fn default_join_after_count() -> u32 {
    5
}
fn default_join_pattern() -> String {
    "joined_{date}.mp4".to_string()
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            join_mode: JoinMode::Off,
            join_after_count: default_join_after_count(),
            join_output_pattern: default_join_pattern(),
        }
    }
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Paths for input/output/music/intro/outro
    #[serde(default)]
    pub paths: PathsConfig,

    /// Silence detection and handling
    #[serde(default)]
    pub silence: SilenceConfig,

    /// Filler word removal
    #[serde(default)]
    pub filler_words: FillerWordsConfig,

    /// Audio processing
    #[serde(default)]
    pub audio: AudioConfig,

    /// Video processing
    #[serde(default)]
    pub video: VideoConfig,

    /// Processing options (join mode, etc.)
    #[serde(default)]
    pub processing: ProcessingConfig,

    /// Export options
    #[serde(default)]
    pub export: ExportConfig,

    /// Watch mode settings
    #[serde(default)]
    pub watch: WatchConfig,
}

impl Config {
    /// Load configuration from a file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;

        Ok(config)
    }

    /// Save configuration to a file
    pub fn to_file(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        Ok(())
    }

    /// Get the default config file path in user's config directory
    pub fn default_config_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("com", "ai-vid-editor", "ai-vid-editor")
            .map(|dirs| dirs.config_dir().join("config.toml"))
    }

    /// Get the project-local config path
    pub fn project_config_path() -> PathBuf {
        PathBuf::from("ai-vid-editor.toml")
    }

    /// Load configuration with precedence: CLI > project > global > defaults
    pub fn load_with_precedence(
        cli_config_path: Option<&Path>,
        cli_threshold: Option<f32>,
        cli_duration: Option<f32>,
        cli_padding: Option<f32>,
        cli_speedup: bool,
    ) -> Result<Self> {
        let mut config = Config::default();

        // Try to load global config first
        if let Some(global_path) = Self::default_config_path()
            && global_path.exists()
        {
            config = Self::from_file(&global_path)?;
        }

        // Then try project config (overrides global)
        let project_path = Self::project_config_path();
        if project_path.exists() {
            let project_config = Self::from_file(&project_path)?;
            config = config.merge(project_config);
        }

        // Then try explicitly specified config (overrides project)
        if let Some(path) = cli_config_path
            && path.exists()
        {
            let file_config = Self::from_file(path)?;
            config = config.merge(file_config);
        }

        // Finally, apply CLI overrides (highest precedence)
        if let Some(threshold) = cli_threshold {
            config.silence.threshold_db = threshold;
        }
        if let Some(duration) = cli_duration {
            config.silence.min_duration = duration;
        }
        if let Some(padding) = cli_padding {
            config.silence.padding = padding;
        }
        if cli_speedup {
            config.silence.mode = SilenceMode::Speedup;
        }

        Ok(config)
    }

    /// Merge another config into this one (other takes precedence)
    pub fn merge(mut self, other: Self) -> Self {
        // Silence config
        if other.silence.threshold_db != default_threshold_db() {
            self.silence.threshold_db = other.silence.threshold_db;
        }
        if other.silence.min_duration != default_min_duration() {
            self.silence.min_duration = other.silence.min_duration;
        }
        if other.silence.padding != default_padding() {
            self.silence.padding = other.silence.padding;
        }
        if other.silence.mode != SilenceMode::Cut {
            self.silence.mode = other.silence.mode;
        }
        if other.silence.speedup_factor != default_speedup_factor() {
            self.silence.speedup_factor = other.silence.speedup_factor;
        }
        if other.silence.min_silence_for_speedup != default_min_silence_for_speedup() {
            self.silence.min_silence_for_speedup = other.silence.min_silence_for_speedup;
        }

        // Filler words config
        if !other.filler_words.enabled {
            self.filler_words.enabled = other.filler_words.enabled;
        }
        if !other.filler_words.words.is_empty() {
            self.filler_words.words = other.filler_words.words;
        }
        if other.filler_words.padding != default_filler_padding() {
            self.filler_words.padding = other.filler_words.padding;
        }

        // Audio config
        if !other.audio.enhance {
            self.audio.enhance = other.audio.enhance;
        }
        if other.audio.target_lufs != default_target_lufs() {
            self.audio.target_lufs = other.audio.target_lufs;
        }
        if other.audio.music_file.is_some() {
            self.audio.music_file = other.audio.music_file;
        }
        if other.audio.duck_volume != default_duck_volume() {
            self.audio.duck_volume = other.audio.duck_volume;
        }

        // Export config
        if other.export.subtitles {
            self.export.subtitles = true;
        }
        if other.export.chapters {
            self.export.chapters = true;
        }
        if other.export.fcpxml {
            self.export.fcpxml = true;
        }
        if other.export.edl {
            self.export.edl = true;
        }

        // Paths config
        if other.paths.input.is_some() {
            self.paths.input = other.paths.input;
        }
        if other.paths.input_dir.is_some() {
            self.paths.input_dir = other.paths.input_dir;
        }
        if other.paths.output.is_some() {
            self.paths.output = other.paths.output;
        }
        if other.paths.output_dir.is_some() {
            self.paths.output_dir = other.paths.output_dir;
        }
        if other.paths.music.is_some() {
            self.paths.music = other.paths.music;
        }
        if other.paths.music_dir.is_some() {
            self.paths.music_dir = other.paths.music_dir;
        }
        if other.paths.intro.is_some() {
            self.paths.intro = other.paths.intro;
        }
        if other.paths.outro.is_some() {
            self.paths.outro = other.paths.outro;
        }
        if !other.paths.watch_folders.is_empty() {
            self.paths.watch_folders = other.paths.watch_folders;
        }

        // Watch config
        if other.watch.enabled {
            self.watch.enabled = true;
        }
        if other.watch.interval != default_watch_interval() {
            self.watch.interval = other.watch.interval;
        }

        // Video config
        if other.video.stabilize {
            self.video.stabilize = true;
        }
        if other.video.color_correct {
            self.video.color_correct = true;
        }
        if other.video.reframe {
            self.video.reframe = true;
        }
        if other.video.blur_background {
            self.video.blur_background = true;
        }

        self
    }

    /// Generate a default config file content
    pub fn generate_default_toml() -> Result<String> {
        let config = Config::default();
        let toml = toml::to_string_pretty(&config).context("Failed to serialize default config")?;

        // Fix floating point precision artifacts (e.g., 0.10000000149011612 -> 0.1)
        // This happens because f32 values get serialized as f64
        fn fix_floats(s: &str) -> String {
            let mut result = String::new();
            for line in s.lines() {
                if line.contains('=') && line.chars().any(|c| c == '.') {
                    let parts: Vec<&str> = line.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim_end();
                        let value = parts[1].trim();
                        if let Ok(float_val) = value.parse::<f64>() {
                            let rounded = (float_val * 100.0).round() / 100.0;
                            if rounded == rounded.trunc() {
                                result.push_str(&format!("{} = {}\n", key, rounded as i64));
                            } else {
                                result.push_str(&format!("{} = {}\n", key, rounded));
                            }
                            continue;
                        }
                    }
                }
                result.push_str(line);
                result.push('\n');
            }
            result
        }

        Ok(fix_floats(&toml))
    }

    /// Load a preset from a TOML file in the presets directory
    pub fn from_preset_file(preset_name: &str) -> Result<Self> {
        let preset_path = PathBuf::from("presets").join(format!("{}.toml", preset_name));
        if preset_path.exists() {
            Self::from_file(&preset_path)
        } else {
            anyhow::bail!("Preset file not found: {:?}", preset_path)
        }
    }

    /// Get list of available preset names from presets directory
    pub fn available_presets() -> Vec<String> {
        let presets_dir = PathBuf::from("presets");
        if !presets_dir.exists() {
            return vec![
                "youtube".to_string(),
                "shorts".to_string(),
                "podcast".to_string(),
                "minimal".to_string(),
            ];
        }

        let mut presets = Vec::new();
        if let Ok(entries) = fs::read_dir(&presets_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "toml").unwrap_or(false)
                    && let Some(stem) = path.file_stem()
                {
                    presets.push(stem.to_string_lossy().to_string());
                }
            }
        }
        presets.sort();

        if presets.is_empty() {
            vec![
                "youtube".to_string(),
                "shorts".to_string(),
                "podcast".to_string(),
                "minimal".to_string(),
            ]
        } else {
            presets
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.silence.threshold_db, -30.0);
        assert_eq!(config.silence.min_duration, 0.5);
        assert_eq!(config.silence.padding, 0.1);
        assert_eq!(config.silence.mode, SilenceMode::Cut);
        assert_eq!(config.silence.speedup_factor, 4.0);
    }

    #[test]
    fn test_config_serialize_deserialize() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.silence.threshold_db, config.silence.threshold_db);
    }

    #[test]
    fn test_config_from_file() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("test_config.toml");

        let content = r#"
[silence]
threshold_db = -35.0
mode = "speedup"
speedup_factor = 2.0

[audio]
enhance = false
"#;
        fs::write(&config_path, content).unwrap();

        let config = Config::from_file(&config_path).unwrap();
        assert_eq!(config.silence.threshold_db, -35.0);
        assert_eq!(config.silence.mode, SilenceMode::Speedup);
        assert_eq!(config.silence.speedup_factor, 2.0);
        assert!(!config.audio.enhance);
    }

    #[test]
    fn test_config_to_file() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("output_config.toml");

        let mut config = Config::default();
        config.silence.threshold_db = -40.0;
        config.silence.mode = SilenceMode::Speedup;

        config.to_file(&config_path).unwrap();

        let loaded = Config::from_file(&config_path).unwrap();
        assert_eq!(loaded.silence.threshold_db, -40.0);
        assert_eq!(loaded.silence.mode, SilenceMode::Speedup);
    }

    #[test]
    fn test_merge_configs() {
        let base = Config::default();

        let mut override_config = Config::default();
        override_config.silence.threshold_db = -40.0;
        override_config.silence.mode = SilenceMode::Speedup;
        override_config.export.subtitles = true;

        let merged = base.merge(override_config);
        assert_eq!(merged.silence.threshold_db, -40.0);
        assert_eq!(merged.silence.mode, SilenceMode::Speedup);
        assert!(merged.export.subtitles);
    }

    #[test]
    fn test_cli_overrides() {
        let config = Config::load_with_precedence(
            None,
            Some(-50.0), // cli_threshold
            Some(1.0),   // cli_duration
            Some(0.2),   // cli_padding
            true,        // cli_speedup
        )
        .unwrap();

        assert_eq!(config.silence.threshold_db, -50.0);
        assert_eq!(config.silence.min_duration, 1.0);
        assert_eq!(config.silence.padding, 0.2);
        assert_eq!(config.silence.mode, SilenceMode::Speedup);
    }

    #[test]
    fn test_silence_mode_serde() {
        // Test serialization through a config struct
        let mut config = Config::default();
        config.silence.mode = SilenceMode::Speedup;

        let serialized = toml::to_string_pretty(&config).unwrap();
        assert!(serialized.contains("mode = \"speedup\""));

        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.silence.mode, SilenceMode::Speedup);

        // Test cut mode
        config.silence.mode = SilenceMode::Cut;
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.silence.mode, SilenceMode::Cut);
    }

    #[test]
    fn test_preset_youtube() {
        let config = Preset::Youtube.to_config();
        assert_eq!(config.silence.mode, SilenceMode::Cut);
        assert_eq!(config.silence.padding, 0.15);
        assert!(config.audio.enhance);
        assert!(config.export.chapters);
        assert!(config.export.fcpxml);
    }

    #[test]
    fn test_preset_shorts() {
        let config = Preset::Shorts.to_config();
        assert_eq!(config.silence.mode, SilenceMode::Speedup);
        assert_eq!(config.silence.speedup_factor, 3.0);
        assert_eq!(config.silence.padding, 0.05);
        assert!(config.audio.enhance);
    }

    #[test]
    fn test_preset_podcast() {
        let config = Preset::Podcast.to_config();
        assert_eq!(config.silence.mode, SilenceMode::Cut);
        assert_eq!(config.silence.padding, 0.2);
        assert!(config.audio.enhance);
        assert_eq!(config.audio.target_lufs, -16.0);
        assert!(config.export.subtitles);
    }

    #[test]
    fn test_preset_minimal() {
        let config = Preset::Minimal.to_config();
        assert_eq!(config.silence.mode, SilenceMode::Cut);
        assert!(!config.audio.enhance);
    }

    #[test]
    fn test_preset_from_str() {
        assert_eq!(Preset::from_str("youtube"), Some(Preset::Youtube));
        assert_eq!(Preset::from_str("SHORTS"), Some(Preset::Shorts));
        assert_eq!(Preset::from_str("tiktok"), Some(Preset::Shorts));
        assert_eq!(Preset::from_str("reels"), Some(Preset::Shorts));
        assert_eq!(Preset::from_str("podcast"), Some(Preset::Podcast));
        assert_eq!(Preset::from_str("minimal"), Some(Preset::Minimal));
        assert_eq!(Preset::from_str("invalid"), None);
    }
}
