//! ML-based video processing features
//!
//! This module provides:
//! - Face detection for auto-reframe
//! - Person segmentation for background blur
//!
//! Models are lazy-loaded to minimize memory usage when features aren't used.

use anyhow::Result;
use image::GenericImageView;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use tract_onnx::prelude::*;

/// Frame extraction utilities
pub struct FrameExtractor;

impl FrameExtractor {
    /// Extract frames from video at specified intervals
    /// Returns paths to extracted frame images
    pub fn extract_frames(
        video_path: &Path,
        output_dir: &Path,
        interval_fps: f32,
    ) -> Result<Vec<std::path::PathBuf>> {
        std::fs::create_dir_all(output_dir)?;

        // Extract frames at specified rate (e.g., 1 fps = 1 frame per second)
        let status = Command::new("ffmpeg")
            .args([
                "-i",
                video_path.to_str().unwrap_or(""),
                "-vf",
                &format!("fps={}", interval_fps),
                "-y",
                &format!("{}/frame_%04d.png", output_dir.to_str().unwrap_or("")),
            ])
            .status()?;

        if !status.success() {
            anyhow::bail!("Failed to extract frames from video");
        }

        // Collect extracted frame paths
        let mut frames = vec![];
        for entry in std::fs::read_dir(output_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "png").unwrap_or(false) {
                frames.push(path);
            }
        }

        frames.sort();
        Ok(frames)
    }

    /// Get video dimensions (width, height)
    pub fn get_video_dimensions(video_path: &Path) -> Result<(u32, u32)> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=width,height",
                "-of",
                "csv=p=0",
                video_path.to_str().unwrap_or(""),
            ])
            .output()?;

        let dims = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = dims.trim().split(',').collect();

        if parts.len() == 2 {
            let width: u32 = parts[0].parse()?;
            let height: u32 = parts[1].parse()?;
            Ok((width, height))
        } else {
            anyhow::bail!("Failed to parse video dimensions");
        }
    }

    /// Get video duration in seconds
    pub fn get_video_duration(video_path: &Path) -> Result<f32> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                video_path.to_str().unwrap_or(""),
            ])
            .output()?;

        let duration = String::from_utf8_lossy(&output.stdout);
        duration
            .trim()
            .parse::<f32>()
            .map_err(|e| anyhow::anyhow!("Failed to parse duration: {}", e))
    }
}

/// Model IDs on HuggingFace Hub
/// Using existing public models instead of custom uploads
const FACE_MODEL_ID: &str = "onnx-models/ultra-light-face-detector";
const FACE_MODEL_FILE: &str = "version-RFB-320.onnx";
const SEGMENT_MODEL_ID: &str = "dhkim2810/MODNet";
const SEGMENT_MODEL_FILE: &str = "modnet.onnx";

/// Type alias for the ONNX model
type OnnxModel = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

/// Face detector using ONNX model
pub struct FaceDetector {
    model: Arc<OnnxModel>,
}

impl FaceDetector {
    /// Load the face detection model
    ///
    /// Model is downloaded on first use if not present.
    /// Uses MediaPipe or similar lightweight face detection model.
    pub fn load() -> Result<Self> {
        // For now, we'll use a placeholder
        // In production, download from HuggingFace or bundle with the binary
        let model_path = Self::get_model_path()?;

        if !model_path.exists() {
            Self::download_model(&model_path)?;
        }

        let model = tract_onnx::onnx()
            .model_for_path(&model_path)?
            .into_optimized()?
            .into_runnable()?;

        Ok(Self {
            model: Arc::new(model),
        })
    }

    /// Get the path where the model is stored
    fn get_model_path() -> Result<std::path::PathBuf> {
        let cache_dir = directories::ProjectDirs::from("com", "ai-vid-editor", "ai-vid-editor")
            .map(|dirs| dirs.cache_dir().to_path_buf())
            .unwrap_or_else(std::env::temp_dir);

        Ok(cache_dir.join("face_detection.onnx"))
    }

    /// Download the model if not present
    fn download_model(path: &Path) -> Result<()> {
        // Create parent directory
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        println!("Downloading face detection model from HuggingFace...");

        // Use hf-hub to download the model
        let api = hf_hub::api::sync::Api::new()?;
        let repo = api.model(FACE_MODEL_ID.to_string());
        let downloaded = repo.get(FACE_MODEL_FILE)?;

        // Copy to cache location
        std::fs::copy(&downloaded, path)?;

        println!("Model downloaded to: {:?}", path);
        Ok(())
    }

    /// Detect faces in a frame
    ///
    /// Returns a list of bounding boxes (x, y, width, height) normalized to 0-1
    pub fn detect(&self, frame: &image::DynamicImage) -> Result<Vec<FaceBox>> {
        // Preprocess image for the model
        let input = Self::preprocess(frame)?;

        // Run inference
        let result = self.model.run(tvec!(input.into()))?;

        // Parse output into face boxes
        Self::parse_output(&result)
    }

    /// Preprocess image for the model
    fn preprocess(image: &image::DynamicImage) -> Result<Tensor> {
        // Resize to model input size (typically 320x320 or 640x480)
        let resized = image.resize_exact(320, 320, image::imageops::FilterType::Triangle);

        // Convert to RGB and normalize
        let rgb = resized.to_rgb8();
        let data: Vec<f32> = rgb
            .pixels()
            .flat_map(|p| p.0.iter().map(|&v| v as f32 / 255.0))
            .collect();

        // Create tensor with shape [1, 3, 320, 320]
        let tensor = Tensor::from_shape(&[1, 3, 320, 320], &data)?;

        Ok(tensor)
    }

    /// Parse model output into face boxes
    fn parse_output(output: &[TValue]) -> Result<Vec<FaceBox>> {
        // Ultra-light-face-detector outputs:
        // - scores: [1, num_anchors] or [1, num_anchors, 1]
        // - boxes: [1, num_anchors, 4] in [x1, y1, x2, y2] normalized format

        if output.len() < 2 {
            return Ok(vec![]);
        }

        let scores = output[0].to_array_view::<f32>()?;
        let boxes = output[1].to_array_view::<f32>()?;

        let confidence_threshold = 0.5;
        let mut faces = Vec::new();

        // Determine score tensor shape
        let score_dims = scores.shape();
        let num_faces = if score_dims.len() == 2 || score_dims.len() == 3 {
            score_dims[1]
        } else {
            return Ok(vec![]);
        };

        // Boxes shape: [1, num_faces, 4] or flattened
        let box_dims = boxes.shape();
        let boxes_are_flat = box_dims.len() == 2 && box_dims[1] == num_faces * 4;

        for i in 0..num_faces {
            let score = scores[i];
            if score < confidence_threshold {
                continue;
            }

            let (x1, y1, x2, y2) = if boxes_are_flat {
                (
                    boxes[i * 4],
                    boxes[i * 4 + 1],
                    boxes[i * 4 + 2],
                    boxes[i * 4 + 3],
                )
            } else if box_dims.len() == 3 {
                (
                    boxes[i * 4],
                    boxes[i * 4 + 1],
                    boxes[i * 4 + 2],
                    boxes[i * 4 + 3],
                )
            } else {
                continue;
            };

            // Convert [x1, y1, x2, y2] to [x, y, width, height] normalized
            let x = x1.clamp(0.0, 1.0);
            let y = y1.clamp(0.0, 1.0);
            let width = (x2 - x1).clamp(0.0, 1.0 - x);
            let height = (y2 - y1).clamp(0.0, 1.0 - y);

            faces.push(FaceBox {
                x,
                y,
                width,
                height,
                confidence: score,
            });
        }

        // Sort by confidence (highest first)
        faces.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(faces)
    }
}

/// Bounding box for a detected face
#[derive(Debug, Clone, Copy)]
pub struct FaceBox {
    /// X coordinate (0-1, normalized)
    pub x: f32,
    /// Y coordinate (0-1, normalized)
    pub y: f32,
    /// Width (0-1, normalized)
    pub width: f32,
    /// Height (0-1, normalized)
    pub height: f32,
    /// Confidence score (0-1)
    pub confidence: f32,
}

impl FaceBox {
    /// Get the center of the face box
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

/// Person segmentation for background blur
pub struct PersonSegmenter {
    model: Arc<OnnxModel>,
}

impl PersonSegmenter {
    /// Load the segmentation model
    pub fn load() -> Result<Self> {
        let model_path = Self::get_model_path()?;

        if !model_path.exists() {
            Self::download_model(&model_path)?;
        }

        let model = tract_onnx::onnx()
            .model_for_path(&model_path)?
            .into_optimized()?
            .into_runnable()?;

        Ok(Self {
            model: Arc::new(model),
        })
    }

    fn get_model_path() -> Result<std::path::PathBuf> {
        let cache_dir = directories::ProjectDirs::from("com", "ai-vid-editor", "ai-vid-editor")
            .map(|dirs| dirs.cache_dir().to_path_buf())
            .unwrap_or_else(std::env::temp_dir);

        Ok(cache_dir.join("person_segmentation.onnx"))
    }

    fn download_model(path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        println!("Downloading person segmentation model from HuggingFace...");

        // Use hf-hub to download the model
        let api = hf_hub::api::sync::Api::new()?;
        let repo = api.model(SEGMENT_MODEL_ID.to_string());
        let downloaded = repo.get(SEGMENT_MODEL_FILE)?;

        // Copy to cache location
        std::fs::copy(&downloaded, path)?;

        println!("Model downloaded to: {:?}", path);
        Ok(())
    }

    /// Segment person from background
    ///
    /// Returns a mask where 1.0 = person, 0.0 = background
    pub fn segment(&self, frame: &image::DynamicImage) -> Result<SegmentationMask> {
        let input = Self::preprocess(frame)?;
        let result = self.model.run(tvec!(input.into()))?;
        Self::parse_output(&result, frame.width(), frame.height())
    }

    fn preprocess(image: &image::DynamicImage) -> Result<Tensor> {
        let resized = image.resize_exact(512, 512, image::imageops::FilterType::Triangle);
        let rgb = resized.to_rgb8();
        let data: Vec<f32> = rgb
            .pixels()
            .flat_map(|p| p.0.iter().map(|&v| v as f32 / 255.0))
            .collect();

        let tensor = Tensor::from_shape(&[1, 3, 512, 512], &data)?;
        Ok(tensor)
    }

    fn parse_output(output: &[TValue], width: u32, height: u32) -> Result<SegmentationMask> {
        // MODNet outputs a matte/mask tensor: [1, 1, H, W] or [1, H, W]
        // Values are 0.0 (background) to 1.0 (foreground/person)

        if output.is_empty() {
            return Ok(SegmentationMask {
                data: vec![0.0; (width * height) as usize],
                width,
                height,
            });
        }

        let mask_tensor = output[0].to_array_view::<f32>()?;
        let dims = mask_tensor.shape();

        // Determine mask dimensions from tensor
        let (mask_h, mask_w) = match dims.len() {
            4 => (dims[2], dims[3]),
            3 => (dims[1], dims[2]),
            2 => (dims[0], dims[1]),
            _ => {
                return Ok(SegmentationMask {
                    data: vec![0.0; (width * height) as usize],
                    width,
                    height,
                });
            }
        };

        // Resize mask to original frame dimensions
        // Simple bilinear interpolation
        let mut data = vec![0.0; (width * height) as usize];

        let scale_x = mask_w as f32 / width as f32;
        let scale_y = mask_h as f32 / height as f32;

        for y in 0..height {
            for x in 0..width {
                let src_x = (x as f32 * scale_x).min(mask_w as f32 - 1.0) as usize;
                let src_y = (y as f32 * scale_y).min(mask_h as f32 - 1.0) as usize;

                let src_idx = src_y * mask_w + src_x;
                let value = if src_idx < mask_tensor.len() {
                    mask_tensor[src_idx]
                } else {
                    0.0
                };

                data[(y * width + x) as usize] = value.clamp(0.0, 1.0);
            }
        }

        Ok(SegmentationMask {
            data,
            width,
            height,
        })
    }
}

/// Segmentation mask for person/background separation
pub struct SegmentationMask {
    /// Mask data (0.0 = background, 1.0 = person)
    pub data: Vec<f32>,
    /// Width of the mask
    pub width: u32,
    /// Height of the mask
    pub height: u32,
}

impl SegmentationMask {
    /// Get the value at a specific pixel
    pub fn get(&self, x: u32, y: u32) -> f32 {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize]
        } else {
            0.0
        }
    }
}

/// Crop region for auto-reframe
#[derive(Debug, Clone, Copy)]
pub struct CropRegion {
    /// X offset (0-1, normalized to video width)
    pub x: f32,
    /// Y offset (0-1, normalized to video height)
    pub y: f32,
    /// Width of crop (0-1)
    pub width: f32,
    /// Height of crop (0-1)
    pub height: f32,
}

impl CropRegion {
    /// Create a center crop for 9:16 aspect ratio from 16:9 video
    pub fn center_crop_9_16() -> Self {
        // For 16:9 -> 9:16, we crop to 9/16 of the width
        let crop_width = 9.0 / 16.0; // ~0.56 of original width
        Self {
            x: (1.0 - crop_width) / 2.0, // Center horizontally
            y: 0.0,
            width: crop_width,
            height: 1.0,
        }
    }

    /// Create crop region following a face
    pub fn from_face(face: &FaceBox, video_aspect: f32) -> Self {
        // video_aspect = width / height (e.g., 16/9 = 1.78)
        // Target aspect = 9/16 = 0.5625

        let target_aspect = 9.0 / 16.0;
        let crop_width = target_aspect / video_aspect;

        // Center crop on face X position
        let face_center_x = face.x + face.width / 2.0;

        // Calculate crop X to center on face
        let mut crop_x = face_center_x - crop_width / 2.0;

        // Clamp to valid range
        crop_x = crop_x.max(0.0).min(1.0 - crop_width);

        Self {
            x: crop_x,
            y: 0.0,
            width: crop_width,
            height: 1.0,
        }
    }
}

/// Auto-reframe processor
pub struct AutoReframeProcessor {
    detector: FaceDetector,
}

impl AutoReframeProcessor {
    /// Create a new auto-reframe processor
    pub fn new() -> Result<Self> {
        let detector = FaceDetector::load()?;
        Ok(Self { detector })
    }

    /// Analyze video and generate crop regions for each frame
    pub fn analyze_video(
        &self,
        video_path: &Path,
        sample_fps: f32,
    ) -> Result<Vec<(f32, CropRegion)>> {
        let temp_dir = std::env::temp_dir().join("ai-vid-editor-frames");
        let frames = FrameExtractor::extract_frames(video_path, &temp_dir, sample_fps)?;

        let _video_duration = FrameExtractor::get_video_duration(video_path)?;
        let (video_width, video_height) = FrameExtractor::get_video_dimensions(video_path)?;
        let video_aspect = video_width as f32 / video_height as f32;

        let mut crop_regions = Vec::new();

        for (i, frame_path) in frames.iter().enumerate() {
            let timestamp = (i as f32) / sample_fps;

            // Load frame
            let frame = image::open(frame_path)?;

            // Detect faces
            let faces = self.detector.detect(&frame)?;

            // Determine crop region
            let crop = if let Some(main_face) = faces.first() {
                CropRegion::from_face(main_face, video_aspect)
            } else {
                // No face detected, use center crop
                CropRegion::center_crop_9_16()
            };

            crop_regions.push((timestamp, crop));
        }

        // Cleanup temp frames
        for frame in &frames {
            let _ = std::fs::remove_file(frame);
        }

        Ok(crop_regions)
    }

    /// Generate ffmpeg filter for smooth crop following faces
    pub fn generate_crop_filter(
        &self,
        crop_regions: &[(f32, CropRegion)],
        video_width: u32,
        video_height: u32,
    ) -> String {
        if crop_regions.is_empty() {
            // Fallback to center crop
            return "crop=ih*9/16:ih,scale=1080:1920".to_string();
        }

        // For simplicity, use the first detected crop region
        // Full implementation would interpolate between regions
        let region = &crop_regions[0].1;

        let crop_w = (region.width * video_width as f32) as u32;
        let crop_h = video_height;
        let crop_x = (region.x * video_width as f32) as u32;
        let crop_y = 0u32;

        format!(
            "crop={}:{}:{}:{},scale=1080:1920",
            crop_w, crop_h, crop_x, crop_y
        )
    }
}

/// Background blur processor
pub struct BackgroundBlurProcessor {
    segmenter: PersonSegmenter,
}

impl BackgroundBlurProcessor {
    /// Create a new background blur processor
    pub fn new() -> Result<Self> {
        let segmenter = PersonSegmenter::load()?;
        Ok(Self { segmenter })
    }

    /// Process a single frame, returning the blurred version
    pub fn process_frame(
        &self,
        frame: &image::DynamicImage,
        blur_strength: u32,
    ) -> Result<image::DynamicImage> {
        // Get segmentation mask
        let mask = self.segmenter.segment(frame)?;

        // Apply blur to the entire frame
        let blurred = frame.blur(blur_strength as f32);

        // Composite: person from original, background from blurred
        let mut result = frame.to_rgb8();
        let blurred_rgb = blurred.to_rgb8();

        for y in 0..frame.height() {
            for x in 0..frame.width() {
                let mask_val = mask.get(x, y);
                let original = frame.get_pixel(x, y);
                let blurred_px = blurred_rgb.get_pixel(x, y);

                // Blend based on mask (1.0 = person, 0.0 = background)
                let r = (original.0[0] as f32 * mask_val
                    + blurred_px.0[0] as f32 * (1.0 - mask_val)) as u8;
                let g = (original.0[1] as f32 * mask_val
                    + blurred_px.0[1] as f32 * (1.0 - mask_val)) as u8;
                let b = (original.0[2] as f32 * mask_val
                    + blurred_px.0[2] as f32 * (1.0 - mask_val)) as u8;

                result.put_pixel(x, y, image::Rgb([r, g, b]));
            }
        }

        Ok(image::DynamicImage::ImageRgb8(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face_box_center() {
        let face = FaceBox {
            x: 0.1,
            y: 0.2,
            width: 0.3,
            height: 0.4,
            confidence: 0.9,
        };

        let (cx, cy) = face.center();
        assert!((cx - 0.25).abs() < 0.001);
        assert!((cy - 0.4).abs() < 0.001);
    }
}
