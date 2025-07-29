//! Menu system for lesson selection
//! 
//! Implements interactive menus that allow users to select lessons
//! and navigate through the typing tutor interface.

use crossterm::{
    cursor, execute, queue,
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    event::{read, Event, KeyCode, KeyEvent},
};
use std::io::{stdout, Write};

/// Menu item representing a selectable option
#[derive(Debug, Clone, PartialEq)]
pub struct MenuItem {
    pub label: String,
    pub title: String,
}

/// Interactive menu display and selection
#[derive(Debug, Clone)]
pub struct Menu {
    pub title: String,
    pub items: Vec<MenuItem>,
}

impl Menu {
    /// Create a new menu with title
    pub fn new(title: String) -> Self {
        Self {
            title,
            items: Vec::new(),
        }
    }
    
    /// Add a menu item
    pub fn add_item(&mut self, label: String, title: String) {
        self.items.push(MenuItem { label, title });
    }
    
    /// Display menu and handle user selection
    pub fn display(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let mut stdout = stdout();
        let mut selected = 0;
        
        loop {
            // Clear screen and display menu
            execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
            
            // Display title
            println!("=== {} ===\n", self.title);
            
            // Display menu items
            for (i, item) in self.items.iter().enumerate() {
                if i == selected {
                    // Highlight selected item
                    queue!(stdout, SetForegroundColor(Color::Yellow))?;
                    println!("  > {:<10} {}", item.label, item.title);
                    queue!(stdout, ResetColor)?;
                } else {
                    println!("    {:<10} {}", item.label, item.title);
                }
            }
            
            println!("\nUse UP/DOWN arrows to navigate, ENTER to select, ESC to quit");
            stdout.flush()?;
            
            // Handle user input
            match read()? {
                Event::Key(KeyEvent { code: KeyCode::Up, .. }) => {
                    if selected > 0 {
                        selected -= 1;
                    }
                },
                Event::Key(KeyEvent { code: KeyCode::Down, .. }) => {
                    if selected < self.items.len().saturating_sub(1) {
                        selected += 1;
                    }
                },
                Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                    if selected < self.items.len() {
                        return Ok(Some(self.items[selected].label.clone()));
                    }
                },
                Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                    return Ok(None);
                },
                Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) |
                Event::Key(KeyEvent { code: KeyCode::Char('Q'), .. }) => {
                    return Ok(None);
                },
                // Allow numeric selection
                Event::Key(KeyEvent { code: KeyCode::Char(ch), .. }) if ch.is_ascii_digit() => {
                    let digit = ch.to_digit(10).unwrap() as usize;
                    if digit > 0 && digit <= self.items.len() {
                        return Ok(Some(self.items[digit - 1].label.clone()));
                    }
                },
                _ => continue,
            }
        }
    }
}

/// Parse menu items from script commands
/// Handles the format: " :LABEL  "description""
pub fn parse_menu_item_line(line: &str) -> Option<MenuItem> {
    let line = line.trim();
    
    // Skip empty lines and non-menu lines
    if line.is_empty() || !line.starts_with(':') {
        return None;
    }
    
    // Format: " :LABEL  "description""
    let parts: Vec<&str> = line[1..].splitn(2, '"').collect();
    if parts.len() != 2 {
        return None;
    }
    
    let label = parts[0].trim().to_string();
    let title = parts[1].trim_end_matches('"').to_string();
    
    if label.is_empty() {
        return None;
    }
    
    Some(MenuItem { label, title })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_menu_item() {
        let line = " :DEMO_0  \"B:         (banner)\"";
        let item = parse_menu_item_line(line).unwrap();
        assert_eq!(item.label, "DEMO_0");
        assert_eq!(item.title, "B:         (banner)");
    }
    
    #[test]
    fn test_parse_menu_item_invalid() {
        assert!(parse_menu_item_line("").is_none());
        assert!(parse_menu_item_line("not a menu item").is_none());
        assert!(parse_menu_item_line(":LABEL_NO_DESCRIPTION").is_none());
    }
    
    #[test]
    fn test_menu_creation() {
        let mut menu = Menu::new("Test Menu".to_string());
        menu.add_item("ITEM1".to_string(), "First Item".to_string());
        menu.add_item("ITEM2".to_string(), "Second Item".to_string());
        
        assert_eq!(menu.title, "Test Menu");
        assert_eq!(menu.items.len(), 2);
        assert_eq!(menu.items[0].label, "ITEM1");
        assert_eq!(menu.items[1].title, "Second Item");
    }
}