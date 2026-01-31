//! Kitty Graphics Protocol implementation
//! 
//! Reference: https://sw.kovidgoyal.net/kitty/graphics-protocol/

use base64::{Engine, engine::general_purpose::STANDARD};

/// Render image using Kitty graphics protocol
/// 
/// The Kitty protocol uses escape sequences to transmit image data:
/// ESC_G<control data>;<payload>ESC\
pub fn render(x: u16, y: u16, _width: u16, _height: u16, image_data: &[u8]) -> String {
    // Encode image data as base64
    let encoded = STANDARD.encode(image_data);
    
    // Kitty graphics protocol escape sequence
    // f=100 means PNG format
    // a=T means transmit and display
    // t=d means direct (data follows inline)
    // C=1 means place cursor after image
    
    // For images larger than 4096 bytes, we need chunked transmission
    if encoded.len() <= 4096 {
        format!(
            "\x1b_Gf=100,a=T,t=d,x={},y={};{}\x1b\\",
            x, y, encoded
        )
    } else {
        // Chunked transmission for large images
        let mut result = String::new();
        let chunks: Vec<&str> = encoded.as_bytes()
            .chunks(4096)
            .map(|c| std::str::from_utf8(c).unwrap_or(""))
            .collect();
        
        for (i, chunk) in chunks.iter().enumerate() {
            let is_last = i == chunks.len() - 1;
            let m = if is_last { 0 } else { 1 }; // m=1 means more chunks coming
            
            if i == 0 {
                result.push_str(&format!(
                    "\x1b_Gf=100,a=T,t=d,x={},y={},m={};{}\x1b\\",
                    x, y, m, chunk
                ));
            } else {
                result.push_str(&format!(
                    "\x1b_Gm={};{}\x1b\\",
                    m, chunk
                ));
            }
        }
        
        result
    }
}

/// Clear an image at the specified position
pub fn clear(_x: u16, _y: u16) -> String {
    // a=d means delete, d=a means delete all images
    "\x1b_Ga=d,d=a\x1b\\".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_image_encoding() {
        let data = vec![0u8; 100];
        let result = render(0, 0, 10, 10, &data);
        assert!(result.starts_with("\x1b_G"));
        assert!(result.ends_with("\x1b\\"));
    }
}
