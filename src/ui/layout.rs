//! Layout utilities for positioning components

use ratatui::layout::Rect;

/// Calculate grid layout for file icons
pub fn calculate_grid_layout(area: Rect, item_width: u16, item_height: u16) -> Vec<Rect> {
    let mut positions = Vec::new();
    
    let cols = (area.width / item_width).max(1);
    let rows = (area.height / item_height).max(1);
    
    for row in 0..rows {
        for col in 0..cols {
            let x = area.x + col * item_width;
            let y = area.y + row * item_height;
            
            if x + item_width <= area.x + area.width && y + item_height <= area.y + area.height {
                positions.push(Rect::new(x, y, item_width, item_height));
            }
        }
    }
    
    positions
}

/// Hit test - find which grid cell contains the given coordinates
pub fn hit_test_grid(positions: &[Rect], x: u16, y: u16) -> Option<usize> {
    positions.iter().position(|rect| {
        x >= rect.x && x < rect.x + rect.width &&
        y >= rect.y && y < rect.y + rect.height
    })
}
