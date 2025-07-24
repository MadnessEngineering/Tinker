//! Visual testing tools for screenshot capture and comparison

use std::path::Path;
use std::io::Cursor;
use image::{ImageBuffer, RgbaImage, DynamicImage, ImageFormat, GenericImageView};
use imageproc::rect::Rect;
use base64::{Engine as _, engine::general_purpose};
use serde::{Serialize, Deserialize};
use tracing::{debug, error, info};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotOptions {
    /// File format for the screenshot
    pub format: ScreenshotFormat,
    /// Quality for JPEG format (1-100)
    pub quality: Option<u8>,
    /// Capture area (None for full page)
    pub area: Option<CaptureArea>,
    /// Include browser chrome in screenshot
    pub include_chrome: bool,
    /// Scale factor for high-DPI displays
    pub scale_factor: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScreenshotFormat {
    PNG,
    JPEG,
    WebP,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureArea {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotResult {
    /// Base64 encoded image data
    pub data: String,
    /// Image dimensions
    pub width: u32,
    pub height: u32,
    /// Format used
    pub format: ScreenshotFormat,
    /// File size in bytes
    pub size: usize,
    /// Timestamp when screenshot was taken
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualComparisonResult {
    /// Percentage difference (0.0 = identical, 1.0 = completely different)
    pub difference_percentage: f64,
    /// Number of differing pixels
    pub differing_pixels: u32,
    /// Total pixels compared
    pub total_pixels: u32,
    /// Areas where differences were found
    pub difference_areas: Vec<CaptureArea>,
    /// Base64 encoded diff image (optional)
    pub diff_image: Option<String>,
}

impl Default for ScreenshotOptions {
    fn default() -> Self {
        Self {
            format: ScreenshotFormat::PNG,
            quality: Some(90),
            area: None,
            include_chrome: false,
            scale_factor: None,
        }
    }
}

pub struct VisualTester {
    screenshot_dir: String,
}

impl VisualTester {
    pub fn new(screenshot_dir: String) -> Self {
        // Create screenshot directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(&screenshot_dir) {
            error!("Failed to create screenshot directory {}: {}", screenshot_dir, e);
        }
        
        Self { screenshot_dir }
    }

    /// Capture screenshot from raw image data
    pub fn capture_from_data(&self, image_data: &[u8], width: u32, height: u32, options: ScreenshotOptions) -> Result<ScreenshotResult> {
        debug!("Capturing screenshot from raw data: {}x{}", width, height);
        
        // Create image from raw RGBA data
        let img_buffer = ImageBuffer::from_raw(width, height, image_data.to_vec())
            .ok_or_else(|| anyhow::anyhow!("Failed to create image buffer from raw data"))?;
        
        let dynamic_img = DynamicImage::ImageRgba8(img_buffer);
        
        // Apply capture area if specified
        let final_img = if let Some(area) = &options.area {
            debug!("Cropping to area: {}x{} at ({}, {})", area.width, area.height, area.x, area.y);
            dynamic_img.crop_imm(area.x, area.y, area.width, area.height)
        } else {
            dynamic_img
        };

        // Apply scale factor if specified
        let final_img = if let Some(scale) = options.scale_factor {
            if scale != 1.0 {
                let new_width = (final_img.width() as f32 * scale) as u32;
                let new_height = (final_img.height() as f32 * scale) as u32;
                debug!("Scaling image by {}: {}x{} -> {}x{}", scale, final_img.width(), final_img.height(), new_width, new_height);
                final_img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
            } else {
                final_img
            }
        } else {
            final_img
        };

        // Encode to specified format
        let mut buffer = Vec::new();
        let format = match options.format {
            ScreenshotFormat::PNG => {
                final_img.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)?;
                ImageFormat::Png
            },
            ScreenshotFormat::JPEG => {
                // Convert RGBA to RGB for JPEG
                let rgb_img = DynamicImage::ImageRgb8(final_img.to_rgb8());
                rgb_img.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Jpeg)?;
                ImageFormat::Jpeg
            },
            ScreenshotFormat::WebP => {
                // Note: WebP support may require additional features
                final_img.write_to(&mut Cursor::new(&mut buffer), ImageFormat::WebP)?;
                ImageFormat::WebP
            },
        };

        // Encode to base64
        let base64_data = general_purpose::STANDARD.encode(&buffer);
        
        let result = ScreenshotResult {
            data: base64_data,
            width: final_img.width(),
            height: final_img.height(),
            format: options.format,
            size: buffer.len(),
            timestamp: chrono::Utc::now(),
        };

        info!("Screenshot captured: {}x{} {} ({} bytes)", 
              result.width, result.height, 
              format!("{:?}", result.format).to_lowercase(), 
              result.size);

        Ok(result)
    }

    /// Save screenshot to file
    pub fn save_screenshot(&self, screenshot: &ScreenshotResult, filename: &str) -> Result<String> {
        let extension = match screenshot.format {
            ScreenshotFormat::PNG => "png",
            ScreenshotFormat::JPEG => "jpg",
            ScreenshotFormat::WebP => "webp",
        };
        
        let full_filename = if filename.ends_with(&format!(".{}", extension)) {
            filename.to_string()
        } else {
            format!("{}.{}", filename, extension)
        };
        
        let file_path = Path::new(&self.screenshot_dir).join(&full_filename);
        
        // Decode base64 and save
        let image_data = general_purpose::STANDARD.decode(&screenshot.data)?;
        std::fs::write(&file_path, image_data)?;
        
        let path_str = file_path.to_string_lossy().to_string();
        info!("Screenshot saved to: {}", path_str);
        Ok(path_str)
    }

    /// Compare two screenshots and return difference analysis
    pub fn compare_screenshots(&self, img1: &ScreenshotResult, img2: &ScreenshotResult, tolerance: f64) -> Result<VisualComparisonResult> {
        debug!("Comparing screenshots: {}x{} vs {}x{}", img1.width, img1.height, img2.width, img2.height);
        
        // Decode base64 images
        let data1 = general_purpose::STANDARD.decode(&img1.data)?;
        let data2 = general_purpose::STANDARD.decode(&img2.data)?;
        
        // Load images
        let image1 = image::load_from_memory(&data1)?;
        let image2 = image::load_from_memory(&data2)?;
        
        // Ensure images are same size
        if image1.dimensions() != image2.dimensions() {
            return Err(anyhow::anyhow!("Images have different dimensions: {:?} vs {:?}", 
                                     image1.dimensions(), image2.dimensions()));
        }

        let (width, height) = image1.dimensions();
        let total_pixels = (width * height) as u32;
        
        // Convert to RGBA for comparison
        let rgba1 = image1.to_rgba8();
        let rgba2 = image2.to_rgba8();
        
        let mut differing_pixels = 0u32;
        let mut difference_areas = Vec::new();
        let mut diff_image = RgbaImage::new(width, height);
        
        // Compare pixel by pixel
        for y in 0..height {
            for x in 0..width {
                let pixel1 = rgba1.get_pixel(x, y);
                let pixel2 = rgba2.get_pixel(x, y);
                
                // Calculate pixel difference
                let diff = pixel_difference(pixel1, pixel2);
                
                if diff > tolerance {
                    differing_pixels += 1;
                    // Mark difference in red
                    diff_image.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
                } else {
                    // Keep original pixel but dimmed
                    let orig = rgba1.get_pixel(x, y);
                    diff_image.put_pixel(x, y, image::Rgba([
                        orig[0] / 2,
                        orig[1] / 2, 
                        orig[2] / 2,
                        orig[3]
                    ]));
                }
            }
        }
        
        let difference_percentage = (differing_pixels as f64) / (total_pixels as f64);
        
        // Generate diff image as base64
        let diff_image_data = if differing_pixels > 0 {
            let mut buffer = Vec::new();
            DynamicImage::ImageRgba8(diff_image)
                .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)?;
            Some(general_purpose::STANDARD.encode(&buffer))
        } else {
            None
        };
        
        let result = VisualComparisonResult {
            difference_percentage,
            differing_pixels,
            total_pixels,
            difference_areas,
            diff_image: diff_image_data,
        };
        
        info!("Visual comparison complete: {:.2}% different ({}/{} pixels)", 
              difference_percentage * 100.0, differing_pixels, total_pixels);
        
        Ok(result)
    }

    /// Create a baseline screenshot for future comparisons
    pub fn create_baseline(&self, screenshot: &ScreenshotResult, test_name: &str) -> Result<String> {
        let filename = format!("baseline_{}", test_name);
        self.save_screenshot(screenshot, &filename)
    }

    /// Load a baseline screenshot for comparison
    pub fn load_baseline(&self, test_name: &str) -> Result<ScreenshotResult> {
        let filename = format!("baseline_{}.png", test_name);
        let file_path = Path::new(&self.screenshot_dir).join(filename);
        
        let image_data = std::fs::read(file_path)?;
        let base64_data = general_purpose::STANDARD.encode(&image_data);
        
        let img = image::load_from_memory(&image_data)?;
        
        Ok(ScreenshotResult {
            data: base64_data,
            width: img.width(),
            height: img.height(),
            format: ScreenshotFormat::PNG,
            size: image_data.len(),
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Calculate the difference between two pixels
fn pixel_difference(pixel1: &image::Rgba<u8>, pixel2: &image::Rgba<u8>) -> f64 {
    let r_diff = (pixel1[0] as f64 - pixel2[0] as f64).abs();
    let g_diff = (pixel1[1] as f64 - pixel2[1] as f64).abs();
    let b_diff = (pixel1[2] as f64 - pixel2[2] as f64).abs();
    let a_diff = (pixel1[3] as f64 - pixel2[3] as f64).abs();
    
    // Calculate normalized Euclidean distance
    ((r_diff * r_diff + g_diff * g_diff + b_diff * b_diff + a_diff * a_diff).sqrt()) / (255.0 * 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_difference() {
        let black = image::Rgba([0, 0, 0, 255]);
        let white = image::Rgba([255, 255, 255, 255]);
        let red = image::Rgba([255, 0, 0, 255]);
        
        // Same pixels should have no difference
        assert_eq!(pixel_difference(&black, &black), 0.0);
        
        // Different pixels should have measurable difference
        assert!(pixel_difference(&black, &white) > 0.5);
        assert!(pixel_difference(&black, &red) > 0.0);
    }

    #[test]
    fn test_screenshot_options_default() {
        let options = ScreenshotOptions::default();
        assert!(matches!(options.format, ScreenshotFormat::PNG));
        assert_eq!(options.quality, Some(90));
        assert!(options.area.is_none());
        assert!(!options.include_chrome);
    }
}