use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn find_video_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut video_files = Vec::new();
    let supported_extensions = ["mp4", "mov", "avi"];

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                if supported_extensions.contains(&extension.to_lowercase().as_str()) {
                    video_files.push(path.to_path_buf());
                }
            }
        }
    }
    Ok(video_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_find_video_files() -> Result<()> {
        let dir = tempdir()?;
        let video1 = dir.path().join("video1.mp4");
        let video2 = dir.path().join("subdir/video2.mov");
        let text_file = dir.path().join("text.txt");
        let unsupported_video = dir.path().join("unsupported.mkv");

        fs::write(&video1, "dummy video content")?;
        fs::create_dir(dir.path().join("subdir"))?;
        fs::write(&video2, "dummy video content")?;
        fs::write(&text_file, "dummy text content")?;
        fs::write(&unsupported_video, "dummy video content")?;

        let found_files = find_video_files(dir.path())?;
        assert_eq!(found_files.len(), 2);
        assert!(found_files.contains(&video1));
        assert!(found_files.contains(&video2));
        assert!(!found_files.contains(&text_file));
        assert!(!found_files.contains(&unsupported_video));

        Ok(())
    }
}
