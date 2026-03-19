use crate::analyzer::ProcessedSegment;
use crate::stt_analyzer::TranscriptSegment;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn export_fcpxml(
    segments: &[ProcessedSegment],
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    let filename = input_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("video.mp4");

    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<!DOCTYPE fcpxml>\n");
    xml.push_str("<fcpxml version=\"1.8\">\n");
    xml.push_str("  <resources>\n");
    xml.push_str(&format!(
        "    <asset id=\"r1\" name=\"{}\" src=\"file://{}\" />\n",
        filename,
        input_path.to_str().unwrap()
    ));
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

pub fn export_edl(
    segments: &[ProcessedSegment],
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    let filename = input_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("video.mp4");
    let mut edl = String::new();
    edl.push_str("TITLE: Edited Timeline\n");
    edl.push_str("FCM: NON-DROP FRAME\n\n");

    for (i, _seg) in segments.iter().enumerate() {
        edl.push_str(&format!(
            "{:03}  AX       V     C        {:02}:{:02}:{:02}:{:02} {:02}:{:02}:{:02}:{:02}\n",
            i + 1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0
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
        srt.push_str(&format!(
            "{} --> {}\n",
            format_srt_time(seg.start),
            format_srt_time(seg.end)
        ));
        srt.push_str(&format!("{}\n\n", seg.text.trim()));
    }

    fs::write(output_path, srt).context("failed to write SRT file")?;
    Ok(())
}

pub fn export_youtube_chapters(transcript: &[TranscriptSegment], output_path: &Path) -> Result<()> {
    let mut chapters = String::new();
    chapters.push_str("00:00 Intro\n");

    // Group transcript segments into chapters every ~3 minutes
    // Whisper returns ~30-second chunks, so we group by ~6 segments per chapter
    let chapter_interval_secs = 180.0; // 3 minutes
    let mut chapter_start = 0.0;
    let mut chapter_texts: Vec<String> = Vec::new();

    for seg in transcript {
        if seg.start >= chapter_start + chapter_interval_secs {
            // Time to start a new chapter
            if !chapter_texts.is_empty() {
                // Use first meaningful text as chapter title (first 50 chars)
                let joined = chapter_texts.join(" ");
                let title = joined.trim();
                let title = if title.len() > 50 {
                    &title[..50]
                } else {
                    title
                };
                let title = title.replace('\n', " ").replace('\r', "");
                chapters.push_str(&format!(
                    "{} {}\n",
                    format_youtube_time(chapter_start),
                    title
                ));
            }
            chapter_start = seg.start;
            chapter_texts.clear();
        }
        // Collect non-empty text
        let text = seg.text.trim();
        if !text.is_empty() && text != "[No speech detected]" {
            chapter_texts.push(text.to_string());
        }
    }

    // Don't forget the last chapter
    if !chapter_texts.is_empty() {
        let joined = chapter_texts.join(" ");
        let title = joined.trim();
        let title = if title.len() > 50 {
            &title[..50]
        } else {
            title
        };
        let title = title.replace('\n', " ").replace('\r', "");
        chapters.push_str(&format!(
            "{} {}\n",
            format_youtube_time(chapter_start),
            title
        ));
    }

    fs::write(output_path, chapters).context("failed to write chapters file")?;
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
        // Whisper returns ~30-second chunks, so simulate a 10-minute video with multiple chunks
        let transcript = vec![
            TranscriptSegment {
                start: 0.0,
                end: 30.0,
                text: "Welcome everyone to today's video".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 30.0,
                end: 60.0,
                text: "We're going to talk about AI video editing".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 60.0,
                end: 90.0,
                text: "Let's start with the introduction".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 90.0,
                end: 120.0,
                text: "First, I'll show you the basic setup".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 200.0,
                end: 230.0,
                text: "Now let's look at the advanced features".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 230.0,
                end: 260.0,
                text: "This is where it gets really powerful".to_string(),
                confidence: 1.0,
            },
        ];

        export_youtube_chapters(&transcript, &output_chapters)?;

        let content = fs::read_to_string(output_chapters)?;
        assert!(content.contains("00:00 Intro"));
        // First chapter should be at ~3 min mark (180s)
        assert!(content.contains("03:00") || content.contains("03:20"));

        Ok(())
    }

    #[test]
    fn test_export_srt() -> Result<()> {
        let dir = tempdir()?;
        let output_srt = dir.path().join("subtitles.srt");
        let transcript = vec![
            TranscriptSegment {
                start: 0.0,
                end: 5.0,
                text: "Hello world".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 5.0,
                end: 10.0,
                text: "This is a test".to_string(),
                confidence: 1.0,
            },
        ];

        export_srt(&transcript, &output_srt)?;

        let content = fs::read_to_string(output_srt)?;
        assert!(content.contains("1\n"));
        assert!(content.contains("Hello world"));
        assert!(content.contains("2\n"));
        assert!(content.contains("This is a test"));
        assert!(content.contains("00:00:00,000 --> 00:00:05,000"));

        Ok(())
    }
}
