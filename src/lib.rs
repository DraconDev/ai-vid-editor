#![allow(clippy::too_many_arguments)]
#![allow(clippy::should_implement_trait)]

pub mod analyzer;
pub mod batch_processor;
pub mod config;
pub mod editor;
pub mod exporter;
pub mod ml;
pub mod stt_analyzer;
pub mod utils;

pub use analyzer::FfmpegAnalyzer;
pub use batch_processor::{
    FfmpegDurationGetter, ProcessingProgress, process_batch_dir, process_single_file,
    process_single_file_with_intro_outro, process_single_file_with_intro_outro_progress,
};
pub use config::{
    Config, FolderSettings, JoinMode, Preset, ProcessingConfig, SilenceMode, WatchFolder,
};
pub use editor::FfmpegEditor;
pub use ml::{AutoReframeProcessor, FaceDetector, FrameExtractor, PersonSegmenter};
