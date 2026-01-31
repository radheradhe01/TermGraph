//! Icon management and rendering via terminal graphics protocols

use image::{DynamicImage, RgbaImage, Rgba, imageops::FilterType};
use std::collections::HashMap;
use std::path::Path;

use crate::graphics::{GraphicsBackend, GraphicsProtocol};

/// Default icon size in terminal cells
pub const ICON_SIZE: u32 = 16;

/// Icon manager - handles loading, caching, and rendering file type icons
pub struct IconManager {
    /// Cached rendered icon escape sequences
    cache: HashMap<String, String>,
    /// Graphics backend
    backend: GraphicsBackend,
}

impl IconManager {
    pub fn new(backend: GraphicsBackend) -> Self {
        Self {
            cache: HashMap::new(),
            backend,
        }
    }

    /// Get the icon escape sequence for a file type
    /// Returns empty string if graphics not supported (use emoji fallback)
    pub fn get_icon_sequence(&mut self, file_type: &str, is_dir: bool) -> String {
        if !self.backend.supports_images() {
            return String::new();
        }

        let key = if is_dir { "folder" } else { file_type }.to_string();
        
        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }

        // Generate icon image
        let icon = self.generate_icon(file_type, is_dir);
        
        // Encode to terminal graphics protocol
        let sequence = self.encode_icon(&icon);
        
        self.cache.insert(key, sequence.clone());
        sequence
    }

    /// Generate a simple colored icon image
    fn generate_icon(&self, file_type: &str, is_dir: bool) -> RgbaImage {
        let mut img = RgbaImage::new(ICON_SIZE, ICON_SIZE);
        
        // Choose color based on file type
        let color = if is_dir {
            Rgba([100, 180, 255, 255]) // Blue for folders
        } else {
            match file_type {
                "rs" => Rgba([255, 100, 50, 255]),   // Orange for Rust
                "py" => Rgba([50, 150, 255, 255]),  // Blue for Python
                "js" | "ts" => Rgba([255, 220, 50, 255]), // Yellow for JS
                "md" => Rgba([100, 200, 100, 255]), // Green for markdown
                "toml" | "json" | "yaml" => Rgba([200, 150, 255, 255]), // Purple for config
                "png" | "jpg" | "gif" => Rgba([255, 100, 150, 255]), // Pink for images
                _ => Rgba([180, 180, 180, 255]),    // Gray for unknown
            }
        };

        // Draw a simple filled rectangle with rounded look
        for y in 2..ICON_SIZE-2 {
            for x in 2..ICON_SIZE-2 {
                img.put_pixel(x, y, color);
            }
        }

        // Add folder tab for directories
        if is_dir {
            for x in 2..8 {
                img.put_pixel(x, 1, color);
            }
        }

        img
    }

    /// Encode icon to terminal graphics escape sequence
    fn encode_icon(&self, img: &RgbaImage) -> String {
        match self.backend.protocol {
            GraphicsProtocol::Kitty => self.encode_kitty(img),
            GraphicsProtocol::ITerm2 => self.encode_iterm2(img),
            _ => String::new(),
        }
    }

    /// Encode image using Kitty graphics protocol
    fn encode_kitty(&self, img: &RgbaImage) -> String {
        use base64::{Engine, engine::general_purpose::STANDARD};
        
        // Encode PNG
        let mut png_data = Vec::new();
        {
            use std::io::Cursor;
            let mut cursor = Cursor::new(&mut png_data);
            img.write_to(&mut cursor, image::ImageFormat::Png).ok();
        }
        
        let encoded = STANDARD.encode(&png_data);
        
        // Kitty inline image: f=100 (PNG), a=T (transmit+display), c=2 (2 columns), r=1 (1 row)
        // Use virtual placement for inline rendering
        if encoded.len() <= 4096 {
            format!("\x1b_Gf=100,a=T,t=d,c=2,r=1;{}\x1b\\", encoded)
        } else {
            // Chunked transmission for larger images
            let mut result = String::new();
            let chunks: Vec<&[u8]> = encoded.as_bytes().chunks(4096).collect();
            
            for (i, chunk) in chunks.iter().enumerate() {
                let is_last = i == chunks.len() - 1;
                let m = if is_last { 0 } else { 1 };
                let chunk_str = std::str::from_utf8(chunk).unwrap_or("");
                
                if i == 0 {
                    result.push_str(&format!("\x1b_Gf=100,a=T,t=d,c=2,r=1,m={};{}\x1b\\", m, chunk_str));
                } else {
                    result.push_str(&format!("\x1b_Gm={};{}\x1b\\", m, chunk_str));
                }
            }
            result
        }
    }

    /// Encode image using iTerm2 protocol
    fn encode_iterm2(&self, img: &RgbaImage) -> String {
        use base64::{Engine, engine::general_purpose::STANDARD};
        
        let mut png_data = Vec::new();
        {
            use std::io::Cursor;
            let mut cursor = Cursor::new(&mut png_data);
            img.write_to(&mut cursor, image::ImageFormat::Png).ok();
        }
        
        let encoded = STANDARD.encode(&png_data);
        format!("\x1b]1337;File=inline=1;width=2;height=1:{}\x07", encoded)
    }
}

/// Get file extension from filename
pub fn get_extension(filename: &str) -> &str {
    filename.rsplit('.').next().unwrap_or("")
}
