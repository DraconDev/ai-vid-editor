use crate::analyzer::ProcessedSegment;
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
        assert!(content.contains("duration=\"1s\""));
        assert!(content.contains("duration=\"2s\""));

        Ok(())
    }
}
