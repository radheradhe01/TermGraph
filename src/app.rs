//! Application state and main event loop

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};

use crate::ui::Ui;
use crate::fs::FileSystem;
use crate::graphics::{GraphicsBackend, ThumbnailCache};

/// Main application state
pub struct App {
    /// Terminal instance
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// UI state and rendering
    ui: Ui,
    /// Filesystem operations
    fs: FileSystem,
    /// Graphics backend (Kitty, Sixel, or Fallback)
    graphics: GraphicsBackend,
    /// Thumbnail cache for image previews
    thumbnails: ThumbnailCache,
    /// Whether the app should quit
    should_quit: bool,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Result<Self> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Detect graphics protocol
        let graphics = GraphicsBackend::detect();
        
        // Create thumbnail cache with the same backend
        let thumbnails = ThumbnailCache::new(graphics.clone());
        
        // Get current directory
        let current_dir = std::env::current_dir()?;
        let fs = FileSystem::new(current_dir);

        Ok(Self {
            terminal,
            ui: Ui::new(),
            fs,
            graphics,
            thumbnails,
            should_quit: false,
        })
    }


    /// Main event loop
    pub async fn run(&mut self) -> Result<()> {
        // Load initial directory
        self.fs.load_directory()?;

        loop {
            // Get thumbnail for current selection if it's an image
            let thumbnail = if let Some(entry) = self.fs.get_selected(self.ui.selected_index) {
                if crate::graphics::is_image_file(&entry.path) {
                    self.thumbnails.get_thumbnail(&entry.path)
                } else {
                    None
                }
            } else {
                None
            };
            
            // Render UI
            self.terminal.draw(|frame| {
                self.ui.render(frame, &self.fs, &self.graphics, thumbnail.as_deref());
            })?;
            
            // After frame render, output thumbnail escape sequence for Kitty
            if let Some(ref thumb_seq) = thumbnail {
                // Position cursor at preview pane location and output image
                // For now, output after the frame (Kitty will handle positioning)
                use std::io::Write;
                let _ = std::io::stdout().write_all(thumb_seq.as_bytes());
                let _ = std::io::stdout().flush();
            }

            // Handle events
            if event::poll(std::time::Duration::from_millis(16))? {
                match event::read()? {
                    Event::Key(key) => self.handle_key(key.code),
                    Event::Mouse(mouse) => self.handle_mouse(mouse),
                    Event::Resize(_, _) => {} // Ratatui handles resize automatically
                    _ => {}
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// Handle keyboard input
    fn handle_key(&mut self, key: KeyCode) {
        // Close context menu on any key if open
        if self.ui.show_context_menu {
            match key {
                KeyCode::Esc => self.ui.show_context_menu = false,
                KeyCode::Up => {
                    if self.ui.context_menu_selected > 0 {
                        self.ui.context_menu_selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if self.ui.context_menu_selected < 5 {
                        self.ui.context_menu_selected += 1;
                    }
                }
                KeyCode::Enter => {
                    // TODO: Execute selected action
                    self.ui.show_context_menu = false;
                }
                _ => {}
            }
            return;
        }

        let total = self.fs.entries.len();
        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Up | KeyCode::Char('k') => self.ui.move_selection(-1, total),
            KeyCode::Down | KeyCode::Char('j') => self.ui.move_selection(1, total),
            KeyCode::F(1) => self.ui.toggle_sidebar(),
            KeyCode::F(2) => self.ui.toggle_preview(),
            KeyCode::Enter => {
                if let Some(entry) = self.fs.get_selected(self.ui.selected_index) {
                    if entry.is_dir {
                        let name = entry.name.clone();
                        let _ = self.fs.enter_directory(&name);
                        self.ui.selected_index = 0;
                    }
                }
            }
            KeyCode::Backspace => {
                let _ = self.fs.go_up();
                self.ui.selected_index = 0;
            }
            KeyCode::Home => self.ui.selected_index = 0,
            KeyCode::End => self.ui.selected_index = total.saturating_sub(1),
            _ => {}
        }
    }

    /// Handle mouse input
    fn handle_mouse(&mut self, mouse: event::MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(event::MouseButton::Left) => {
                // Calculate which file was clicked based on mouse position
                let clicked_index = self.ui.get_item_at_position(mouse.row, mouse.column);
                if let Some(index) = clicked_index {
                    self.ui.selected_index = index;
                }
            }
            MouseEventKind::Down(event::MouseButton::Right) => {
                // TODO: Context menu
                self.ui.show_context_menu = true;
                self.ui.context_menu_pos = (mouse.column, mouse.row);
            }
            MouseEventKind::ScrollUp => self.ui.scroll(-3),
            MouseEventKind::ScrollDown => self.ui.scroll(3),
            _ => {}
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // Restore terminal state
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}
