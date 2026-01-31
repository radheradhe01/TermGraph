//! UI rendering and layout

mod layout;

pub use layout::*;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::fs::FileSystem;
use crate::graphics::GraphicsBackend;

/// UI state
pub struct Ui {
    /// Currently selected file index
    pub selected_index: usize,
    /// Scroll offset for file list
    pub scroll_offset: usize,
    /// Whether context menu is visible
    pub show_context_menu: bool,
    /// Context menu position (x, y)
    pub context_menu_pos: (u16, u16),
    /// Selected context menu item
    pub context_menu_selected: usize,
    /// Whether sidebar is visible
    pub show_sidebar: bool,
    /// Whether preview pane is visible
    pub show_preview: bool,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            scroll_offset: 0,
            show_context_menu: false,
            context_menu_pos: (0, 0),
            context_menu_selected: 0,
            show_sidebar: true,
            show_preview: true,
        }
    }

    /// Render the entire UI
    pub fn render(&self, frame: &mut Frame, fs: &FileSystem, graphics: &GraphicsBackend, thumbnail: Option<&str>) {
        let size = frame.area();

        // Create main layout: Header | Main Content | Status Bar
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header/toolbar
                Constraint::Min(10),    // Main content
                Constraint::Length(1),  // Status bar
            ])
            .split(size);

        self.render_header(frame, main_chunks[0], fs);
        
        // Three-pane layout for main content
        if self.show_sidebar || self.show_preview {
            let mut constraints = Vec::new();
            
            if self.show_sidebar {
                constraints.push(Constraint::Percentage(20)); // Sidebar
            }
            constraints.push(Constraint::Min(30)); // Main files
            if self.show_preview {
                constraints.push(Constraint::Percentage(25)); // Preview
            }
            
            let content_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(main_chunks[1]);
            
            let mut idx = 0;
            
            if self.show_sidebar {
                self.render_sidebar(frame, content_chunks[idx], fs);
                idx += 1;
            }
            
            self.render_file_grid(frame, content_chunks[idx], fs, graphics);
            idx += 1;
            
            if self.show_preview {
                self.render_preview(frame, content_chunks[idx], fs, thumbnail);
            }
        } else {
            self.render_file_grid(frame, main_chunks[1], fs, graphics);
        }
        
        self.render_status_bar(frame, main_chunks[2], fs);

        // Render context menu if visible
        if self.show_context_menu {
            self.render_context_menu(frame);
        }
    }

    /// Render the header with path and navigation
    fn render_header(&self, frame: &mut Frame, area: Rect, fs: &FileSystem) {
        let path_display = fs.current_path.display().to_string();
        
        let header = Paragraph::new(Line::from(vec![
            Span::styled(" 📁 ", Style::default().fg(Color::Yellow)),
            Span::styled(&path_display, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("[F1:Sidebar]", Style::default().fg(Color::DarkGray)),
            Span::raw(" "),
            Span::styled("[F2:Preview]", Style::default().fg(Color::DarkGray)),
        ]))
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" GraphTerm ")
            .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)));

        frame.render_widget(header, area);
    }

    /// Render the sidebar with bookmarks
    fn render_sidebar(&self, frame: &mut Frame, area: Rect, fs: &FileSystem) {
        let home = dirs::home_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "~".to_string());
        
        let bookmarks = vec![
            ("🏠", "Home", home.clone()),
            ("📄", "Documents", format!("{}/Documents", home)),
            ("⬇️", "Downloads", format!("{}/Downloads", home)),
            ("🖼️", "Pictures", format!("{}/Pictures", home)),
            ("🎵", "Music", format!("{}/Music", home)),
            ("💻", "Desktop", format!("{}/Desktop", home)),
        ];

        let items: Vec<ListItem> = bookmarks
            .iter()
            .map(|(icon, name, path)| {
                let is_current = fs.current_path.to_string_lossy().starts_with(path);
                let style = if is_current {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                
                ListItem::new(Line::from(vec![
                    Span::raw(format!(" {} ", icon)),
                    Span::styled(*name, style),
                ]))
            })
            .collect();

        let sidebar = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" Bookmarks ")
                .title_style(Style::default().fg(Color::Yellow)));

        frame.render_widget(sidebar, area);
    }

    /// Render the preview pane
    fn render_preview(&self, frame: &mut Frame, area: Rect, fs: &FileSystem, thumbnail: Option<&str>) {
        let content = if let Some(entry) = fs.get_selected(self.selected_index) {
            let name = entry.name.clone();
            let path = entry.path.display().to_string();
            let is_image = crate::graphics::is_image_file(&entry.path);
            
            if entry.is_dir {
                // Show directory info
                vec![
                    Line::from(vec![
                        Span::styled("📁 Directory", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
                        Span::raw(name),
                    ]),
                    Line::from(vec![
                        Span::styled("Path: ", Style::default().fg(Color::DarkGray)),
                        Span::raw(path),
                    ]),
                ]
            } else if is_image {
                // Show image info with thumbnail placeholder
                let size = crate::fs::format_size(entry.size);
                let dimensions = crate::graphics::thumbnails::ThumbnailCache::get_image_info(&entry.path)
                    .unwrap_or_else(|| "Unknown".to_string());
                
                let mut lines = vec![
                    Line::from(vec![
                        Span::styled("🖼️ Image", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
                        Span::raw(name),
                    ]),
                    Line::from(vec![
                        Span::styled("Size: ", Style::default().fg(Color::DarkGray)),
                        Span::raw(size),
                    ]),
                    Line::from(vec![
                        Span::styled("Dimensions: ", Style::default().fg(Color::DarkGray)),
                        Span::raw(dimensions),
                    ]),
                ];
                
                // If we have a thumbnail, add placeholder for where it will render
                if thumbnail.is_some() {
                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![
                        Span::styled("[Thumbnail Below]", Style::default().fg(Color::Green)),
                    ]));
                }
                
                lines
            } else {
                // Show file info
                let size = crate::fs::format_size(entry.size);
                let ext = entry.name.rsplit('.').next().unwrap_or("").to_uppercase();
                
                vec![
                    Line::from(vec![
                        Span::styled(ext, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Span::raw(" File"),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
                        Span::raw(name),
                    ]),
                    Line::from(vec![
                        Span::styled("Size: ", Style::default().fg(Color::DarkGray)),
                        Span::raw(size),
                    ]),
                    Line::from(vec![
                        Span::styled("Path: ", Style::default().fg(Color::DarkGray)),
                        Span::raw(path),
                    ]),
                ]
            }
        } else {
            vec![Line::from(Span::styled("No file selected", Style::default().fg(Color::DarkGray)))]
        };

        let preview = Paragraph::new(content)
            .wrap(Wrap { trim: true })
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" Preview ")
                .title_style(Style::default().fg(Color::Green)));

        frame.render_widget(preview, area);
        
        // Render thumbnail after the widget if we have one
        // The thumbnail escape sequence is written directly to stdout after ratatui draws
        if let Some(thumb_seq) = thumbnail {
            // Store the sequence to be written after frame render
            // Note: This is a simplified approach - in production you'd use cursor positioning
            if let Some(entry) = fs.get_selected(self.selected_index) {
                if crate::graphics::is_image_file(&entry.path) {
                    // We'll print the thumbnail sequence after the frame
                    // The position would be calculated based on preview pane location
                    let _ = thumb_seq; // Sequence will be rendered by app after frame
                }
            }
        }
    }

    /// Render the file grid
    fn render_file_grid(&self, frame: &mut Frame, area: Rect, fs: &FileSystem, _graphics: &GraphicsBackend) {
        let visible_height = area.height.saturating_sub(2) as usize; // Account for borders
        
        let items: Vec<ListItem> = fs.entries
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(visible_height)
            .map(|(index, entry)| {
                let icon = if entry.is_dir { "📁" } else { Self::get_file_icon(&entry.name) };
                let is_selected = index == self.selected_index;
                
                // Selection indicator
                let indicator = if is_selected { "▶" } else { " " };
                
                let style = if is_selected {
                    Style::default()
                        .bg(Color::Rgb(80, 80, 160))
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(if entry.is_dir { Color::Cyan } else { Color::White })
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", indicator), Style::default().fg(Color::Yellow)),
                    Span::raw(format!("{} ", icon)),
                    Span::styled(&entry.name, style),
                ]))
                .style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(format!(" Files ({}) ", fs.entries.len()))
                .title_style(Style::default().fg(Color::Green)));

        frame.render_widget(list, area);
    }

    /// Render the status bar
    fn render_status_bar(&self, frame: &mut Frame, area: Rect, fs: &FileSystem) {
        let selected_info = if let Some(entry) = fs.get_selected(self.selected_index) {
            if entry.is_dir {
                format!("📁 {}", entry.name)
            } else {
                format!("{} ({})", entry.name, crate::fs::format_size(entry.size))
            }
        } else {
            String::new()
        };

        let status = Paragraph::new(Line::from(vec![
            Span::styled(
                format!(" {} items ", fs.entries.len()),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw(" | "),
            Span::styled(
                &selected_info,
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" | "),
            Span::styled(
                "↑↓:Nav Enter:Open Bksp:Back q:Quit",
                Style::default().fg(Color::DarkGray),
            ),
        ]));

        frame.render_widget(status, area);
    }

    /// Render context menu
    fn render_context_menu(&self, frame: &mut Frame) {
        let menu_items = vec![
            ("📂", "Open"),
            ("📋", "Copy"),
            ("✂️", "Cut"),
            ("📄", "Paste"),
            ("🗑️", "Delete"),
            ("✏️", "Rename"),
        ];

        let menu_height = menu_items.len() as u16 + 2;
        let menu_width = 16;

        let area = Rect::new(
            self.context_menu_pos.0.min(frame.area().width.saturating_sub(menu_width)),
            self.context_menu_pos.1.min(frame.area().height.saturating_sub(menu_height)),
            menu_width,
            menu_height,
        );

        let items: Vec<ListItem> = menu_items
            .iter()
            .enumerate()
            .map(|(i, (icon, label))| {
                let style = if i == self.context_menu_selected {
                    Style::default().bg(Color::Rgb(80, 80, 160)).fg(Color::White)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(format!(" {} {} ", icon, label))).style(style)
            })
            .collect();

        let menu = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Rgb(40, 40, 60))));

        // Clear the area first
        frame.render_widget(ratatui::widgets::Clear, area);
        frame.render_widget(menu, area);
    }

    /// Get file icon based on extension
    fn get_file_icon(filename: &str) -> &'static str {
        let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "rs" => "🦀",
            "py" => "🐍",
            "js" | "ts" => "📜",
            "tsx" | "jsx" => "⚛️",
            "md" => "📝",
            "toml" | "yaml" | "yml" | "json" => "⚙️",
            "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => "🖼️",
            "mp4" | "mov" | "avi" | "mkv" => "🎬",
            "mp3" | "wav" | "flac" | "ogg" => "🎵",
            "zip" | "tar" | "gz" | "rar" | "7z" => "📦",
            "pdf" => "📕",
            "doc" | "docx" => "📘",
            "xls" | "xlsx" => "📗",
            "ppt" | "pptx" => "📙",
            "html" | "css" => "🌐",
            "sh" | "bash" | "zsh" => "🖥️",
            "lock" => "🔒",
            "gitignore" | "git" => "📋",
            "dockerfile" | "docker" => "🐳",
            "log" => "📃",
            "txt" => "📄",
            _ => "📄",
        }
    }

    /// Move selection up/down
    pub fn move_selection(&mut self, delta: i32, total_items: usize) {
        let new_index = self.selected_index as i32 + delta;
        if new_index >= 0 && new_index < total_items as i32 {
            self.selected_index = new_index as usize;
        }
    }

    /// Scroll the view
    pub fn scroll(&mut self, delta: i32) {
        let new_offset = self.scroll_offset as i32 + delta;
        if new_offset >= 0 {
            self.scroll_offset = new_offset as usize;
        }
    }

    /// Toggle sidebar visibility
    pub fn toggle_sidebar(&mut self) {
        self.show_sidebar = !self.show_sidebar;
    }

    /// Toggle preview pane visibility
    pub fn toggle_preview(&mut self) {
        self.show_preview = !self.show_preview;
    }

    /// Get item index at mouse position
    pub fn get_item_at_position(&self, row: u16, _column: u16) -> Option<usize> {
        // Layout:
        // Row 0-2: Header (3 lines with borders)
        // Row 3: Files panel top border
        // Row 4+: File items (first item at row 4)
        if row >= 4 {
            let index = (row - 4) as usize + self.scroll_offset;
            Some(index)
        } else {
            None
        }
    }
}
