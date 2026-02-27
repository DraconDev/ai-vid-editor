use std::path::PathBuf;
use std::process::Command;

pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn test_video_path() -> PathBuf {
    let path = fixtures_dir().join("test_video_temp.mp4");
    if !path.exists() {
        create_test_video_with_silence(&path, 6);
    }
    path
}

pub fn create_test_video_with_silence(output_path: &std::path::Path, duration_secs: u32) -> bool {
    let status = Command::new("ffmpeg")
        .args([
            "-f",
            "lavfi",
            "-i",
            &format!("sine=frequency=1000:duration={}", duration_secs),
            "-f",
            "lavfi",
            "-i",
            &format!("color=c=black:s=320x240:d={}", duration_secs),
            "-af",
            &format!("volume=0:enable='between(t,1,2)+between(t,4,5)'"),
            "-c:v",
            "libx264",
            "-c:a",
            "aac",
            "-shortest",
            "-y",
            output_path.to_str().unwrap(),
        ])
        .status()
        .is_ok();

    status && output_path.exists()
}

pub fn has_ffmpeg() -> bool {
    Command::new("ffmpeg").arg("-version").status().is_ok()
}

pub fn has_ffprobe() -> bool {
    Command::new("ffprobe").arg("-version").status().is_ok()
}
