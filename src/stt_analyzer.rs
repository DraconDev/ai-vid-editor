use std::path::Path;
use anyhow::{Result, Context};
use whisper_rs::{WhisperContext, FullParams, SamplingStrategy, WhisperContextParameters};

#[derive(Debug, PartialEq, Clone)]
pub struct TranscriptSegment {
    pub start: f32,
    pub end: f32,
    pub text: String,
    pub confidence: f32,
}

pub trait VideoSttAnalyzer {
    fn transcribe(&self, audio_path: &Path, model_path: &Path) -> Result<Vec<TranscriptSegment>>;
}

pub struct WhisperSttAnalyzer;

impl VideoSttAnalyzer for WhisperSttAnalyzer {
    fn transcribe(&self, audio_path: &Path, model_path: &Path) -> Result<Vec<TranscriptSegment>> {
        let ctx = WhisperContext::new_with_params(
            model_path.to_str().context("invalid model path")?,
            WhisperContextParameters::default()
        ).context("failed to load whisper model")?;

        let mut state = ctx.create_state().context("failed to create state")?;

        // Load and decode audio using ffmpeg (whisper needs 16kHz mono f32)
        let audio_data = load_audio_as_f32(audio_path)?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        state.full(params, &audio_data).context("failed to run stt")?;

        let mut segments = Vec::new();
        let num_segments = state.full_n_segments().context("failed to get segments")?;

        for i in 0..num_segments {
            let text = state.full_get_segment_text(i).context("failed to get segment text")?;
            let start = state.full_get_segment_t0(i).context("failed to get start time")? as f32 / 100.0;
            let end = state.full_get_segment_t1(i).context("failed to get end time")? as f32 / 100.0;
            
            // Note: whisper-rs doesn't directly expose confidence per segment in a simple f32 way 
            // without more complex access, using 1.0 as placeholder for now.
            segments.push(TranscriptSegment {
                start,
                end,
                text,
                confidence: 1.0,
            });
        }

        Ok(segments)
    }
}

fn load_audio_as_f32(path: &Path) -> Result<Vec<f32>> {
    let output = std::process::Command::new("ffmpeg")
        .args([
            "-i", path.to_str().context("invalid path")?,
            "-ar", "16000",
            "-ac", "1",
            "-f", "f32le",
            "-",
        ])
        .output()
        .context("failed to extract audio with ffmpeg")?;

    let bytes = output.stdout;
    let samples: Vec<f32> = bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
        .collect();

    Ok(samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcription_structure() {
        // Structural test only, real integration needs a model file
        let _analyzer = WhisperSttAnalyzer;
    }
}
