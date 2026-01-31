//! GraphTerm - Terminal-Native Graphical File Manager
//! 
//! A mouse-first file manager that renders pixel graphics inside the terminal
//! using Kitty/Sixel protocols, while maintaining full SSH compatibility.

mod app;
mod ui;
mod graphics;
mod fs;

use anyhow::Result;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the application
    let mut app = App::new()?;
    
    // Run the main event loop
    app.run().await?;
    
    Ok(())
}
