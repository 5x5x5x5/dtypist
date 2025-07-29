//! Test program to verify text centering and display formatting

use crossterm::{
    execute,
    terminal::{Clear, ClearType},
    cursor,
};
use std::io::{stdout, Write};

/// Helper function to center text in terminal
fn center_text(text: &str) -> String {
    let (width, _) = crossterm::terminal::size().unwrap_or((80, 24));
    let padding = (width as usize).saturating_sub(text.len()) / 2;
    format!("{}{}", " ".repeat(padding), text)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    
    // Test terminal size detection
    let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
    println!("Terminal size: {}x{}", width, height);
    println!();
    
    // Test centering with different text lengths
    let test_texts = vec![
        "=== GNU Typist - Rust Implementation ===",
        "Version 2.10.1", 
        "A typing tutor to help you learn touch typing.",
        "Press ESC at any time to exit.",
        "Press any key to continue...",
    ];
    
    println!("Testing text centering:");
    println!("{}|", "-".repeat(width as usize));
    
    for text in &test_texts {
        let centered = center_text(text);
        println!("{}|", centered);
        
        // Debug info
        let actual_padding = centered.len() - text.len();
        let expected_padding = (width as usize).saturating_sub(text.len()) / 2;
        println!("  Text: '{}' (len={})", text, text.len());
        println!("  Padding: actual={}, expected={}", actual_padding, expected_padding);
        println!();
    }
    
    println!("{}|", "-".repeat(width as usize));
    
    // Test what the actual welcome screen would look like
    println!("\nActual welcome screen preview:");
    println!("{}|", "=".repeat(width as usize));
    
    println!();
    println!("{}", center_text("=== GNU Typist - Rust Implementation ===")); 
    println!("{}", center_text("Version 2.10.1"));
    println!();
    println!("{}", center_text("A typing tutor to help you learn touch typing."));
    println!("{}", center_text("Press ESC at any time to exit."));
    println!();
    println!("{}", center_text("Press any key to continue..."));
    println!();
    
    println!("{}|", "=".repeat(width as usize));
    
    Ok(())
}