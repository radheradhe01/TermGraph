//! Terminal graphics abstraction layer
//! 
//! Detects terminal capabilities and provides unified API for rendering images.

mod protocol;
mod kitty;
mod sixel;
pub mod icons;
pub mod thumbnails;

pub use protocol::*;
pub use thumbnails::{ThumbnailCache, is_image_file};

/// Graphics backend type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GraphicsProtocol {
    /// Kitty graphics protocol (best support)
    Kitty,
    /// Sixel graphics (legacy but widely supported)
    Sixel,
    /// iTerm2 inline images (macOS)
    ITerm2,
    /// Fallback to Unicode block art
    Fallback,
}

/// Graphics backend for rendering images in terminal
#[derive(Clone)]
pub struct GraphicsBackend {
    pub protocol: GraphicsProtocol,
}

impl GraphicsBackend {
    /// Detect the best available graphics protocol
    pub fn detect() -> Self {
        let protocol = Self::detect_protocol();
        eprintln!("[GraphTerm] Detected graphics protocol: {:?}", protocol);
        Self { protocol }
    }

    fn detect_protocol() -> GraphicsProtocol {
        // Check for Kitty
        if std::env::var("KITTY_WINDOW_ID").is_ok() {
            return GraphicsProtocol::Kitty;
        }

        // Check TERM variable for kitty
        if let Ok(term) = std::env::var("TERM") {
            if term.contains("kitty") {
                return GraphicsProtocol::Kitty;
            }
        }

        // Check for iTerm2
        if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
            if term_program == "iTerm.app" {
                return GraphicsProtocol::ITerm2;
            }
        }

        // Check for WezTerm (supports Kitty protocol)
        if std::env::var("WEZTERM_PANE").is_ok() {
            return GraphicsProtocol::Kitty;
        }

        // TODO: Query terminal for Sixel support via escape sequences
        // For now, fallback
        GraphicsProtocol::Fallback
    }

    /// Render an image at the specified position
    /// 
    /// # Arguments
    /// * `x` - Column position (0-indexed)
    /// * `y` - Row position (0-indexed)  
    /// * `width` - Width in terminal cells
    /// * `height` - Height in terminal cells
    /// * `image_data` - Raw RGBA image data
    pub fn render_image(&self, x: u16, y: u16, width: u16, height: u16, image_data: &[u8]) -> String {
        match self.protocol {
            GraphicsProtocol::Kitty => kitty::render(x, y, width, height, image_data),
            GraphicsProtocol::Sixel => sixel::render(x, y, width, height, image_data),
            GraphicsProtocol::ITerm2 => self.render_iterm2(x, y, width, height, image_data),
            GraphicsProtocol::Fallback => String::new(), // Use Unicode in UI layer
        }
    }

    fn render_iterm2(&self, _x: u16, _y: u16, _width: u16, _height: u16, image_data: &[u8]) -> String {
        // iTerm2 protocol: OSC 1337 ; File=inline=1;size=<bytes>:<base64 data> ST
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, image_data);
        format!("\x1b]1337;File=inline=1;size={}:{}\x07", image_data.len(), encoded)
    }

    /// Check if real graphics are supported
    pub fn supports_images(&self) -> bool {
        self.protocol != GraphicsProtocol::Fallback
    }
}
