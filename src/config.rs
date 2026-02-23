use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

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
                // Short-form: speedup silences, enhance audio, tighter cuts
                config.silence.mode = SilenceMode::Speedup;
                config.silence.speedup_factor = 3.0;
                config.silence.padding = 0.05; // Tighter for fast-paced content
                config.audio.enhance = true;
            }
            Preset::Podcast => {
                // Podcast: cut silences, enhance audio, generate subtitles
                config.silence.mode = SilenceMode::Cut;
                config.silence.padding = 0.2; // More padding for conversational flow
                config.audio.enhance = true;
                config.audio.target_lufs = -16.0; // Podcast standard
                config.export.subtitles = true;
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

fn default_threshold_db() -> f32 { -30.0 }
fn default_min_duration() -> f32 { 0.5 }
fn default_padding() -> f32 { 0.1 }
fn default_speedup_factor() -> f32 { 4.0 }
fn default_min_silence_for_speedup() -> f32 { 0.5 }

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

fn default_true() -> bool { true }
fn default_filler_words() -> Vec<String> { 
    vec!["um".into(), "uh".into(), "ah".into(), "er".into()] 
}
fn default_filler_padding() -> f32 { 0.05 }

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

fn default_target_lufs() -> f32 { -14.0 }
fn default_duck_volume() -> f32 { 0.2 }

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Generate SRT subtitles
    #[serde(default)]
    pub subtitles: bool,
    
    /// Generate YouTube chapters
    #[serde(default)]
    pub chapters: bool,
    
    /// Generate FCPXML for DaVinci/Premiere
    #[serde(default)]
    pub fcpxml: bool,
    
    /// Generate EDL
    #[serde(default)]
    pub edl: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            subtitles: false,
            chapters: false,
            fcpxml: false,
            edl: false,
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

fn default_watch_interval() -> u64 { 5 }

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval: default_watch_interval(),
        }
    }
}

/// Configuration for video processing
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            stabilize: false,
            color_correct: false,
            reframe: false,
            blur_background: false,
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
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
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
        if let Some(global_path) = Self::default_config_path() {
            if global_path.exists() {
                config = Self::from_file(&global_path)?;
            }
        }
        
        // Then try project config (overrides global)
        let project_path = Self::project_config_path();
        if project_path.exists() {
            let project_config = Self::from_file(&project_path)?;
            config = config.merge(project_config);
        }
        
        // Then try explicitly specified config (overrides project)
        if let Some(path) = cli_config_path {
            if path.exists() {
                let file_config = Self::from_file(path)?;
                config = config.merge(file_config);
            }
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
        if other.export.subtitles { self.export.subtitles = true; }
        if other.export.chapters { self.export.chapters = true; }
        if other.export.fcpxml { self.export.fcpxml = true; }
        if other.export.edl { self.export.edl = true; }
        
        self
    }
    
    /// Generate a default config file content
    pub fn generate_default_toml() -> Result<String> {
        let config = Config::default();
        toml::to_string_pretty(&config).context("Failed to serialize default config")
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
            Some(-50.0),  // cli_threshold
            Some(1.0),    // cli_duration
            Some(0.2),    // cli_padding
            true,         // cli_speedup
        ).unwrap();
        
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
