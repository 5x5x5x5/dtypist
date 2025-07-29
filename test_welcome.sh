#!/bin/bash

echo "Testing welcome screen display..."
echo "Terminal size: $(tput cols)x$(tput lines)"
echo "COLUMNS env var: ${COLUMNS:-not set}"
echo ""

# Create a simple test program that just shows the welcome screen
cat > temp_test.rs << 'EOF'
use std::io::{stdout, Write};

/// Helper function to center text in terminal
fn center_text(text: &str) -> String {
    // Try to get terminal size, fall back to reasonable defaults
    let width = std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80);
    
    let text_len = text.chars().count();
    if text_len >= width {
        return text.to_string();
    }
    
    let padding = (width - text_len) / 2;
    format!("{}{}", " ".repeat(padding), text)
}

fn main() {
    println!("Terminal width detected: {}", std::env::var("COLUMNS").unwrap_or_else(|_| "80 (default)".to_string()));
    println!();
    
    println!("{}", center_text("=== GNU Typist - Rust Implementation ===")); 
    println!("{}", center_text("Version 2.10.1"));
    println!();
    println!("{}", center_text("A typing tutor to help you learn touch typing."));
    println!("{}", center_text("Press ESC at any time to exit."));
    println!();
    println!("{}", center_text("Press any key to continue..."));
    println!();
}
EOF

rustc temp_test.rs -o temp_test
./temp_test
rm temp_test.rs temp_test