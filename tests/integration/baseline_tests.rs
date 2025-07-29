use std::process::Command;
use std::fs;
use tempfile::TempDir;

/// Test that captures the expected output format from the C version
/// This serves as our baseline for ensuring Rust compatibility
#[test]
fn test_c_version_help_output() {
    // Skip if C version not built
    if !std::path::Path::new("./gtypist").exists() {
        println!("Skipping baseline test - C version not built");
        return;
    }
    
    let output = Command::new("./gtypist")
        .arg("--help")
        .output()
        .expect("Failed to execute C gtypist");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Capture key help text patterns that Rust version should match
    assert!(stdout.contains("GNU Typist"));
    assert!(stdout.contains("typing tutor"));
    
    // Save baseline for comparison
    fs::write("tests/baseline_help.txt", &stdout)
        .expect("Failed to write baseline help output");
}

#[test]
fn test_c_version_demo_lesson_structure() {
    // Test that we can parse the demo.typ file structure
    let demo_content = fs::read_to_string("lessons/demo.typ")
        .expect("Failed to read demo.typ");
    
    // Verify key lesson structure elements exist
    assert!(demo_content.contains("*:MENU"));
    assert!(demo_content.contains("M: \"Demonstration"));
    assert!(demo_content.contains("T:"));  // Tutorial command
    assert!(demo_content.contains("D:"));  // Drill command  
    assert!(demo_content.contains("S:"));  // Speed test command
    assert!(demo_content.contains("B:"));  // Banner command
    
    // Save demo structure for Rust parser validation
    fs::write("tests/baseline_demo_structure.txt", &demo_content)
        .expect("Failed to write baseline demo structure");
}

/// Test script command parsing patterns from C implementation
#[test]
fn test_script_command_patterns() {
    let demo_content = fs::read_to_string("lessons/demo.typ")
        .expect("Failed to read demo.typ");
    
    let lines: Vec<&str> = demo_content.lines().collect();
    let mut commands = Vec::new();
    
    for line in lines {
        if line.len() >= 2 && line.chars().nth(1) == Some(':') {
            let command = line.chars().nth(0).unwrap();
            commands.push((command, line.to_string()));
        }
    }
    
    // Verify we found expected command types
    let command_chars: Vec<char> = commands.iter().map(|(c, _)| *c).collect();
    assert!(command_chars.contains(&'*'));  // Label
    assert!(command_chars.contains(&'M'));  // Menu
    assert!(command_chars.contains(&'T'));  // Tutorial
    assert!(command_chars.contains(&'D'));  // Drill
    assert!(command_chars.contains(&'S'));  // Speed test
    assert!(command_chars.contains(&'B'));  // Banner
    
    println!("Found {} script commands in demo.typ", commands.len());
}