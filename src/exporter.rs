use crate::analyzer::ProcessedSegment;
use crate::stt_analyzer::TranscriptSegment;
use std::path::Path;
use anyhow::{Result, Context};
use std::fs;

pub fn export_fcpxml(segments: &[ProcessedSegment], input_path: &Path, output_path: &Path) -> Result<()> {
    let filename = input_path.file_name().and_then(|s| s.to_str()).unwrap_or("video.mp4");
    
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<!DOCTYPE fcpxml>\n");
    xml.push_str("<fcpxml version=\"1.8\">\n");
    xml.push_str("  <resources>\n");
    xml.push_str(&format!("    <asset id=\"r1\" name=\"{}\" src=\"file://{}\" />\n", filename, input_path.to_str().unwrap()));
    xml.push_str("  </resources>\n");
    xml.push_str("  <library>\n");
    xml.push_str("    <event name=\"Automated Cuts\">\n");
    xml.push_str("      <project name=\"Edited Timeline\">\n");
    xml.push_str("        <sequence duration=\"3600/1s\" format=\"r1\">\n");
    xml.push_str("          <spine>\n");

    let mut start_offset = 0.0;
    for seg in segments {
        let duration = seg.end - seg.start;
        xml.push_str(&format!(
            "            <video name=\"{}\" offset=\"{}s\" ref=\"r1\" start=\"{}s\" duration=\"{}s\" role=\"video\" />\n",
            filename, start_offset, seg.start, duration
        ));
        start_offset += duration;
    }

    xml.push_str("          </spine>\n");
    xml.push_str("        </sequence>\n");
    xml.push_str("      </project>\n");
    xml.push_str("    </event>\n");
    xml.push_str("  </library>\n");
    xml.push_str("</fcpxml>\n");

    fs::write(output_path, xml).context("failed to write XML file")?;
    Ok(())
}

pub fn export_edl(segments: &[ProcessedSegment], input_path: &Path, output_path: &Path) -> Result<()> {
    let filename = input_path.file_name().and_then(|s| s.to_str()).unwrap_or("video.mp4");
    let mut edl = String::new();
    edl.push_str("TITLE: Edited Timeline\n");
    edl.push_str("FCM: NON-DROP FRAME\n\n");

    for (i, _seg) in segments.iter().enumerate() {
        edl.push_str(&format!(
            "{:03}  AX       V     C        {:02}:{:02}:{:02}:{:02} {:02}:{:02}:{:02}:{:02}\n",
            i + 1,
            0, 0, 0, 0, 
            0, 0, 0, 0 
        ));
        edl.push_str(&format!("* FROM CLIP NAME: {}\n\n", filename));
    }

    fs::write(output_path, edl).context("failed to write EDL file")?;
    Ok(())
}

pub fn export_srt(transcript: &[TranscriptSegment], output_path: &Path) -> Result<()> {
    let mut srt = String::new();
    for (i, seg) in transcript.iter().enumerate() {
        srt.push_str(&format!("{}\n", i + 1));
        srt.push_str(&format!("{} --> {}\n", format_srt_time(seg.start), format_srt_time(seg.end)));
        srt.push_str(&format!("{}\n\n", seg.text.trim()));
    }

    fs::write(output_path, srt).context("failed to write SRT file")?;
    Ok(())
}

pub fn export_youtube_chapters(transcript: &[TranscriptSegment], output_path: &Path) -> Result<()> {
    let mut chapters = String::new();
    chapters.push_str("00:00 Intro\n");
    
    // Simplistic logic: Every 5 minutes or significant topic change (TODO)
    // For now, let's just generate a few placeholders based on transcript
    for seg in transcript {
        if seg.text.to_lowercase().contains("chapter") || seg.text.to_lowercase().contains("topic") {
             chapters.push_str(&format!("{} {}\n", format_youtube_time(seg.start), seg.text.trim()));
        }
    }

    fs::write(output_path, chapters).context("failed to write chapters file")?;
    Ok(())
}

/// Thumbnail moment types for the guide
#[derive(Debug, Clone, serde::Serialize)]
pub struct ThumbnailMoment {
    /// Timestamp in seconds
    pub time: f32,
    /// Suggested text for thumbnail overlay
    pub text: String,
    /// Type of moment (hook, action, climax, chapter)
    #[serde(rename = "type")]
    pub moment_type: String,
}

/// Thumbnail guide output structure
#[derive(Debug, Clone, serde::Serialize)]
pub struct ThumbnailGuide {
    /// Suggested video titles
    pub title_suggestions: Vec<String>,
    /// Best moments for thumbnails
    pub thumbnail_moments: Vec<ThumbnailMoment>,
    /// Key quotes for overlay text
    pub key_quotes: Vec<String>,
}

/// Generate a thumbnail guide from transcript
/// 
/// Analyzes transcript for:
/// - Hooks (questions, bold statements)
/// - Chapter starts
/// - Key quotes for overlay text
/// - Title suggestions
pub fn generate_thumbnail_guide(transcript: &[TranscriptSegment]) -> ThumbnailGuide {
    let mut title_suggestions: Vec<String> = Vec::new();
    let mut thumbnail_moments: Vec<ThumbnailMoment> = Vec::new();
    let mut key_quotes: Vec<String> = Vec::new();

    // Hook patterns that indicate good thumbnail moments
    let hook_patterns = [
        "secret", "how to", "why", "what", "the best", "amazing", "incredible",
        "you won't believe", "this is", "watch this", "here's", "tip", "trick",
        "mistake", "wrong", "right", "important", "key", "step"
    ];

    // Question words indicate hooks
    let question_patterns = ["?", "how do", "what is", "why does", "can you", "should i"];

    for seg in transcript {
        let text_lower = seg.text.to_lowercase();
        
        // Detect hooks (questions and bold statements)
        let is_hook = question_patterns.iter().any(|p| text_lower.contains(p)) ||
            hook_patterns.iter().any(|p| text_lower.contains(p));
        
        if is_hook {
            // Extract a short phrase for the thumbnail text
            let text = seg.text.trim();
            let display_text = if text.len() > 40 {
                // Try to find a good breaking point
                if let Some(pos) = text[..40].rfind(' ') {
                    text[..pos].to_string() + "..."
                } else {
                    text[..40].to_string() + "..."
                }
            } else {
                text.to_string()
            };

            thumbnail_moments.push(ThumbnailMoment {
                time: seg.start,
                text: display_text.clone(),
                moment_type: "hook".to_string(),
            });

            // Also add as potential key quote if it's impactful
            if text.len() > 20 && text.len() < 100 {
                key_quotes.push(text.to_string());
            }
        }

        // Detect chapter/section starts
        let chapter_patterns = ["chapter", "section", "part", "step 1", "step 2", "step 3", 
            "first", "second", "third", "finally", "next", "now let's", "moving on"];
        let is_chapter = chapter_patterns.iter().any(|p| text_lower.contains(p));
        
        if is_chapter {
            thumbnail_moments.push(ThumbnailMoment {
                time: seg.start,
                text: seg.text.trim().to_string(),
                moment_type: "chapter".to_string(),
            });
        }
    }

    // Generate title suggestions from first few segments
    let first_text: String = transcript.iter()
        .take(5)
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    
    // Simple title extraction (look for capitalized phrases, key topics)
    if first_text.contains("how to") || first_text.contains("How to") {
        if let Some(pos) = first_text.to_lowercase().find("how to") {
            let remainder = &first_text[pos..];
            if let Some(end) = remainder.find('.') {
                title_suggestions.push(remainder[..end].to_string());
            } else {
                title_suggestions.push(remainder.to_string());
            }
        }
    }

    // Add generic title suggestions based on content
    if title_suggestions.is_empty() {
        title_suggestions.push("How to [Topic from Video]".to_string());
        title_suggestions.push("[Number] Tips for [Topic]".to_string());
        title_suggestions.push("The Secret to [Topic]".to_string());
    }

    // Limit results
    thumbnail_moments.truncate(10);
    key_quotes.truncate(5);
    title_suggestions.truncate(3);

    ThumbnailGuide {
        title_suggestions,
        thumbnail_moments,
        key_quotes,
    }
}

/// Export thumbnail guide to JSON file
pub fn export_thumbnail_guide(transcript: &[TranscriptSegment], output_path: &Path) -> Result<()> {
    let guide = generate_thumbnail_guide(transcript);
    let json = serde_json::to_string_pretty(&guide)
        .context("failed to serialize thumbnail guide")?;
    fs::write(output_path, json).context("failed to write thumbnail guide")?;
    Ok(())
}

fn format_srt_time(seconds: f32) -> String {
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;
    let millis = ((seconds % 1.0) * 1000.0) as u32;
    format!("{:02}:{:02}:{:02},{:03}", hours, minutes, secs, millis)
}

fn format_youtube_time(seconds: f32) -> String {
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;
    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_export_youtube_chapters() -> Result<()> {
        let dir = tempdir()?;
        let output_chapters = dir.path().join("chapters.txt");
        let transcript = vec![
            TranscriptSegment { start: 65.0, end: 70.0, text: "New Topic: AI features".to_string(), confidence: 1.0 },
        ];

        export_youtube_chapters(&transcript, &output_chapters)?;

        let content = fs::read_to_string(output_chapters)?;
        assert!(content.contains("00:00 Intro"));
        assert!(content.contains("01:05 New Topic: AI features"));

        Ok(())
    }
}
