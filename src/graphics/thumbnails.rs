//! Thumbnail generation and caching

use image::{DynamicImage, imageops::FilterType};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use base64::{Engine, engine::general_purpose::STANDARD};

use crate::graphics::{GraphicsBackend, GraphicsProtocol};

/// Maximum thumbnail size (in pixels)
pub const THUMBNAIL_SIZE: u32 = 200;

/// Thumbnail cache
pub struct ThumbnailCache {
    /// Map from file path to base64-encoded thumbnail data
    cache: HashMap<PathBuf, String>,
    /// Graphics backend for protocol-specific encoding
    backend: GraphicsBackend,
}

impl ThumbnailCache {
    pub fn new(backend: GraphicsBackend) -> Self {
        Self {
            cache: HashMap::new(),
            backend,
        }
    }

    /// Get thumbnail for an image file
    /// Returns the escape sequence to render the image, or None if not an image
    pub fn get_thumbnail(&mut self, path: &Path) -> Option<String> {
        // Check if it's an image file
        if !Self::is_image_file(path) {
            return None;
        }

        // Check cache
        if let Some(cached) = self.cache.get(path) {
            return Some(cached.clone());
        }

        // Load and resize image
        let img = image::open(path).ok()?;
        let thumbnail = self.create_thumbnail(&img);
        
        // Encode for terminal
        let sequence = self.encode_thumbnail(&thumbnail);
        
        // Cache it
        self.cache.insert(path.to_path_buf(), sequence.clone());
        
        Some(sequence)
    }

    /// Check if file is a supported image format
    fn is_image_file(path: &Path) -> bool {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();
        
        matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico")
    }

    /// Create a resized thumbnail
    fn create_thumbnail(&self, img: &DynamicImage) -> DynamicImage {
        // Calculate aspect-preserving dimensions
        let (w, h) = (img.width(), img.height());
        let (new_w, new_h) = if w > h {
            (THUMBNAIL_SIZE, (THUMBNAIL_SIZE as f32 * h as f32 / w as f32) as u32)
        } else {
            ((THUMBNAIL_SIZE as f32 * w as f32 / h as f32) as u32, THUMBNAIL_SIZE)
        };
        
        img.resize(new_w, new_h, FilterType::Triangle)
    }

    /// Encode thumbnail for terminal display
    fn encode_thumbnail(&self, img: &DynamicImage) -> String {
        match self.backend.protocol {
            GraphicsProtocol::Kitty => self.encode_kitty(img),
            GraphicsProtocol::ITerm2 => self.encode_iterm2(img),
            _ => String::new(), // No graphics support
        }
    }

    /// Encode using Kitty graphics protocol
    fn encode_kitty(&self, img: &DynamicImage) -> String {
        let rgba = img.to_rgba8();
        let (w, h) = (rgba.width(), rgba.height());
        
        // Encode PNG
        let mut png_data = Vec::new();
        {
            use std::io::Cursor;
            let mut cursor = Cursor::new(&mut png_data);
            rgba.write_to(&mut cursor, image::ImageFormat::Png).ok();
        }
        
        let encoded = STANDARD.encode(&png_data);
        
        // Calculate cell dimensions (assume ~12 pixels per cell)
        let cols = (w as f32 / 12.0).ceil() as u16;
        let rows = (h as f32 / 24.0).ceil() as u16;
        
        // Kitty graphics escape sequence
        // f=100 (PNG), a=T (transmit+display), t=d (direct data)
        // c=columns, r=rows
        if encoded.len() <= 4096 {
            format!("\x1b_Gf=100,a=T,t=d,c={},r={};{}\x1b\\", cols, rows, encoded)
        } else {
            // Chunked transmission
            let mut result = String::new();
            let chunks: Vec<&[u8]> = encoded.as_bytes().chunks(4096).collect();
            
            for (i, chunk) in chunks.iter().enumerate() {
                let is_last = i == chunks.len() - 1;
                let m = if is_last { 0 } else { 1 };
                let chunk_str = std::str::from_utf8(chunk).unwrap_or("");
                
                if i == 0 {
                    result.push_str(&format!("\x1b_Gf=100,a=T,t=d,c={},r={},m={};{}\x1b\\", cols, rows, m, chunk_str));
                } else {
                    result.push_str(&format!("\x1b_Gm={};{}\x1b\\", m, chunk_str));
                }
            }
            result
        }
    }

    /// Encode using iTerm2 protocol
    fn encode_iterm2(&self, img: &DynamicImage) -> String {
        let rgba = img.to_rgba8();
        
        let mut png_data = Vec::new();
        {
            use std::io::Cursor;
            let mut cursor = Cursor::new(&mut png_data);
            rgba.write_to(&mut cursor, image::ImageFormat::Png).ok();
        }
        
        let encoded = STANDARD.encode(&png_data);
        format!("\x1b]1337;File=inline=1;preserveAspectRatio=1:{}\x07", encoded)
    }

    /// Get image dimensions as string
    pub fn get_image_info(path: &Path) -> Option<String> {
        let img = image::open(path).ok()?;
        Some(format!("{}×{} px", img.width(), img.height()))
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

/// Check if a file is an image
pub fn is_image_file(path: &Path) -> bool {
    ThumbnailCache::is_image_file(path)
}
