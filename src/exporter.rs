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

    for (i, seg) in segments.iter().enumerate() {
        // EDL format is complex, using a simplified version for now
        edl.push_str(&format!(
            "{:03}  AX       V     C        {:02}:{:02}:{:02}:{:02} {:02}:{:02}:{:02}:{:02}\n",
            i + 1,
            0, 0, 0, 0, // Placeholder for source TC
            0, 0, 0, 0  // Placeholder for record TC
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

fn format_srt_time(seconds: f32) -> String {
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;
    let millis = ((seconds % 1.0) * 1000.0) as u32;
    format!("{:02}:{:02}:{:02},{:03}", hours, minutes, secs, millis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_export_fcpxml_structure() -> Result<()> {
        let dir = tempdir()?;
        let output_xml = dir.path().join("output.xml");
        let segments = vec![
            ProcessedSegment { start: 0.0, end: 1.0, speed: 1.0 },
            ProcessedSegment { start: 2.0, end: 4.0, speed: 1.0 },
        ];
        let input_path = Path::new("test.mp4");

        export_fcpxml(&segments, input_path, &output_xml)?;

        let content = fs::read_to_string(output_xml)?;
        assert!(content.contains("<?xml"));
        assert!(content.contains("<fcpxml"));
        assert!(content.contains("test.mp4"));

        Ok(())
    }

    #[test]
    fn test_export_srt() -> Result<()> {
        let dir = tempdir()?;
        let output_srt = dir.path().join("output.srt");
        let transcript = vec![
            TranscriptSegment { start: 1.5, end: 3.2, text: "Hello world".to_string(), confidence: 1.0 },
        ];

        export_srt(&transcript, &output_srt)?;

        let content = fs::read_to_string(output_srt)?;
        assert!(content.contains("1"));
        assert!(content.contains("00:00:01,500 --> 00:00:03,200"));
        assert!(content.contains("Hello world"));

        Ok(())
    }
}
