use std::path::Path;
use anyhow::{Result, Context};
use candle_core::{Device, Tensor, DType};
use candle_transformers::models::whisper::{Config, model::Whisper, audio};
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;

#[derive(Debug, PartialEq, Clone)]
pub struct TranscriptSegment {
    pub start: f32,
    pub end: f32,
    pub text: String,
    pub confidence: f32,
}

pub trait VideoSttAnalyzer {
    fn transcribe(&self, audio_path: &Path) -> Result<Vec<TranscriptSegment>>;
}

pub struct CandleSttAnalyzer;

impl VideoSttAnalyzer for CandleSttAnalyzer {
    fn transcribe(&self, audio_path: &Path) -> Result<Vec<TranscriptSegment>> {
        let device = Device::Cpu; 
        
        // Load model and tokenizer from HF hub
        let api = Api::new().context("failed to create hf-hub api")?;
        let repo = api.repo(Repo::new("openai/whisper-tiny".to_string(), RepoType::Model));
        
        let config_filename = repo.get("config.json").context("failed to get config.json")?;
        let tokenizer_filename = repo.get("tokenizer.json").context("failed to get tokenizer.json")?;
        let weights_filename = repo.get("model.safetensors").context("failed to get model.safetensors")?;

        let config: Config = serde_json::from_str(&std::fs::read_to_string(config_filename)?)
            .context("failed to parse config")?;
        let tokenizer = Tokenizer::from_file(tokenizer_filename)
            .map_err(anyhow::Error::msg).context("failed to load tokenizer")?;
        
        let vb = unsafe {
            candle_nn::VarBuilder::from_mmaped_safetensors(
                &[weights_filename],
                DType::F32,
                &device,
            )?
        };
        
        let model = Whisper::load(&vb, config.clone()).context("failed to load model")?;

        // Load audio using ffmpeg
        let audio_data = load_audio_as_f32(audio_path)?;
        
        // Convert to mel spectrogram using candle's audio module
        let mel = audio::pcm_to_mel(&config, &audio_data, audio::mel_filters(&config))?;
        let mel_len = mel.len();
        let mel = Tensor::from_vec(mel, (1, config.num_mel_bins, mel_len / config.num_mel_bins), &device)?;

        // Decode using greedy search
        let segments = decode_greedy(&model, &tokenizer, &mel, &config)?;

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

/// Greedy decoding - simple but effective for transcription
fn decode_greedy(
    model: &Whisper,
    tokenizer: &Tokenizer,
    mel: &Tensor,
    config: &Config,
) -> Result<Vec<TranscriptSegment>> {
    // Token IDs
    let sot_token = tokenizer.token_to_id("<|startoftranscript|>")
        .context("missing sot token")?;
    let eot_token = tokenizer.token_to_id("<|endoftext|>")
        .context("missing eot token")?;
    let transcribe_token = tokenizer.token_to_id("<|transcribe|>")
        .context("missing transcribe token")?;
    
    // Process in 30-second chunks
    let mel_len = mel.dims()[2];
    let chunk_size = 3000; // 30 seconds at 100 frames/sec
    let mut segments = Vec::new();
    
    for chunk_start in (0..mel_len).step_by(chunk_size) {
        let chunk_end = (chunk_start + chunk_size).min(mel_len);
        let chunk_mel = mel.narrow(2, chunk_start, chunk_end - chunk_start)?;
        
        // Encode this chunk
        let chunk_encoder_output = model.encoder.forward(&chunk_mel)?;
        
        // Initialize with start tokens
        let mut tokens = vec![sot_token, transcribe_token];
        let mut token_probs = Vec::new();
        
        // Greedy decode up to max tokens
        for _ in 0..config.max_target_positions {
            let input = Tensor::new(tokens.clone(), mel.device())?
                .unsqueeze(0)?;
            
            let logits = model.decoder.forward(&input, &chunk_encoder_output)?;
            let next_token_logits = logits.get((0, tokens.len() - 1))?;
            
            // Greedy: pick highest probability token
            let next_token = next_token_logits.argmax(0)?.to_scalar::<u32>()?;
            
            if next_token == eot_token {
                break;
            }
            
            // Get probability for confidence
            let probs = candle_nn::ops::softmax(&next_token_logits, 0)?;
            let prob = probs.get(next_token as usize)?.to_scalar::<f32>()?;
            token_probs.push(prob);
            
            tokens.push(next_token);
        }
        
        // Decode tokens to text
        let text_tokens: Vec<u32> = tokens[2..].to_vec(); // Skip sot and transcribe tokens
        if text_tokens.is_empty() {
            continue;
        }
        
        let text = tokenizer.decode(&text_tokens, true)
            .map_err(anyhow::Error::msg)?;
        
        if text.is_empty() || text.trim().is_empty() {
            continue;
        }
        
        // Calculate time bounds
        let time_start = (chunk_start as f32 / 100.0) as f32;
        let time_end = (chunk_end as f32 / 100.0) as f32;
        
        // Average confidence
        let confidence = if token_probs.is_empty() {
            0.5
        } else {
            token_probs.iter().sum::<f32>() / token_probs.len() as f32
        };
        
        segments.push(TranscriptSegment {
            start: time_start,
            end: time_end,
            text: text.trim().to_string(),
            confidence,
        });
    }
    
    // If no segments were produced, return a placeholder
    if segments.is_empty() {
        segments.push(TranscriptSegment {
            start: 0.0,
            end: 30.0,
            text: "[No speech detected]".to_string(),
            confidence: 0.0,
        });
    }
    
    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcription_structure() {
        let _analyzer = CandleSttAnalyzer;
    }
}