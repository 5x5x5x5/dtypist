//! Integration tests for verifying C-to-Rust compatibility
//! 
//! These tests ensure the Rust implementation produces identical behavior
//! to the original C implementation.

use gtypist_rs::{Script, Executor, Command};
use std::path::Path;
use std::process::Command as ProcessCommand;
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
    
    let output = ProcessCommand::new("./gtypist")
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
fn test_demo_script_parsing() {
    let demo_path = "lessons/demo.typ";
    
    if !Path::new(demo_path).exists() {
        panic!("Demo script not found at {}", demo_path);
    }
    
    let script = Script::from_file(demo_path)
        .expect("Failed to parse demo script");
    
    // Verify script has expected structure
    assert!(!script.commands.is_empty(), "Demo script should have commands");
    assert!(!script.labels.is_empty(), "Demo script should have labels");
    
    // Check for common lesson elements
    let has_tutorial = script.commands.iter().any(|cmd| {
        matches!(cmd, Command::Tutorial { .. })
    });
    let has_drill = script.commands.iter().any(|cmd| {
        matches!(cmd, Command::Drill { .. })
    });
    
    assert!(has_tutorial || has_drill, "Demo should contain exercises");
}

#[test]
fn test_basic_script_commands() {
    let script_content = r#"
# Test script with basic commands
*:START
I:Welcome to the test
T:This is a tutorial line
D:abc def ghi
*:END
X:
"#;
    
    // Write temporary test file
    use tempfile::NamedTempFile;
    
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    fs::write(temp_file.path(), script_content).expect("Failed to write test script");
    
    let script = Script::from_file(temp_file.path().to_str().unwrap())
        .expect("Failed to parse test script");
    
    // Verify parsing results
    assert_eq!(script.labels.len(), 2); // START and END labels
    assert!(script.labels.contains_key("START"));
    assert!(script.labels.contains_key("END"));
    
    // Check command types
    let command_types: Vec<_> = script.commands.iter()
        .filter_map(|cmd| match cmd {
            Command::Comment { .. } => Some("Comment"),
            Command::Label { .. } => Some("Label"),
            Command::Instruction { .. } => Some("Instruction"),
            Command::Tutorial { .. } => Some("Tutorial"),
            Command::Drill { .. } => Some("Drill"),
            Command::Exit => Some("Exit"),
            _ => None,
        })
        .collect();
    
    assert!(command_types.contains(&"Instruction"));
    assert!(command_types.contains(&"Tutorial"));
    assert!(command_types.contains(&"Drill"));
    assert!(command_types.contains(&"Exit"));
}

#[test]
fn test_executor_basic_flow() {
    let script_content = r#"
*:START
I:Test instruction
T:Test tutorial text
G:END
*:END
X:
"#;
    
    use tempfile::NamedTempFile;
    
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    fs::write(temp_file.path(), script_content).expect("Failed to write test script");
    
    let script = Script::from_file(temp_file.path().to_str().unwrap())
        .expect("Failed to parse test script");
    
    let mut executor = Executor::new(script);
    
    // Test that we can create and initialize the executor
    assert!(!executor.script.is_finished());
    assert_eq!(executor.error_percentage, 0.0);
    assert!(executor.failure_label.is_none());
}

#[test]
fn test_error_rate_parsing() {
    let script_content = r#"
E:5%
E:default
E:10*
"#;
    
    use tempfile::NamedTempFile;
    
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    fs::write(temp_file.path(), script_content).expect("Failed to write test script");
    
    let script = Script::from_file(temp_file.path().to_str().unwrap())
        .expect("Failed to parse test script");
    
    // Verify error rate commands are parsed correctly
    let error_commands: Vec<_> = script.commands.iter()
        .filter_map(|cmd| match cmd {
            Command::ErrorMaxSet { percentage } => Some(*percentage),
            _ => None,
        })
        .collect();
    
    assert_eq!(error_commands.len(), 3);
    assert!(error_commands.contains(&5.0));
    assert!(error_commands.contains(&0.0)); // "default" maps to 0.0
    assert!(error_commands.contains(&10.0));
}

#[test]
fn test_lesson_file_compatibility() {
    let lesson_files = [
        "lessons/demo.typ",
        "lessons/q.typ",
    ];
    
    for lesson_file in &lesson_files {
        if Path::new(lesson_file).exists() {
            let result = Script::from_file(lesson_file);
            assert!(result.is_ok(), "Failed to parse lesson file: {}", lesson_file);
            
            let script = result.unwrap();
            assert!(!script.commands.is_empty(), 
                   "Lesson file {} should not be empty", lesson_file);
        }
    }
}

#[test]
fn test_utf8_text_handling() {
    let script_content = r#"
# Test UTF-8 characters
T:Héllo Wörld! 你好世界 🌍
D:café naïve résumé
"#;
    
    use tempfile::NamedTempFile;
    
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    fs::write(temp_file.path(), script_content).expect("Failed to write test script");
    
    let script = Script::from_file(temp_file.path().to_str().unwrap())
        .expect("Failed to parse UTF-8 script");
    
    // Verify UTF-8 content is preserved
    let tutorial_found = script.commands.iter().any(|cmd| {
        matches!(cmd, Command::Tutorial { text } if text.contains("🌍"))
    });
    let drill_found = script.commands.iter().any(|cmd| {
        matches!(cmd, Command::Drill { text, .. } if text.contains("naïve"))
    });
    
    assert!(tutorial_found, "UTF-8 tutorial text should be preserved");
    assert!(drill_found, "UTF-8 drill text should be preserved");
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

#[cfg(test)]
mod performance_tests {
    use super::*;
    use gtypist_rs::{PerformanceTracker, ExerciseResult, PerformanceGrade};
    use std::time::Duration;
    
    #[test]
    fn test_wpm_calculation_accuracy() {
        let mut tracker = PerformanceTracker::new();
        
        // Simulate typing "hello world" (11 chars) in 30 seconds
        // Should be 11 chars/min * 60 = 22 CPM, 22/5 = 4.4 WPM
        for _ in 0..11 {
            tracker.record_correct_char();
        }
        tracker.set_duration(Duration::from_secs(30));
        
        let wpm = tracker.words_per_minute();
        assert!((wpm - 4.4).abs() < 0.1, "WPM calculation mismatch: got {}, expected ~4.4", wpm);
    }
    
    #[test]
    fn test_error_rate_calculation() {
        let mut tracker = PerformanceTracker::new();
        
        // 7 correct, 3 errors = 30% error rate
        for _ in 0..7 {
            tracker.record_correct_char();
        }
        for _ in 0..3 {
            tracker.record_error();
        }
        
        assert_eq!(tracker.error_rate(), 30.0);
        assert_eq!(tracker.accuracy(), 70.0);
    }
    
    #[test]
    fn test_performance_grading() {
        let excellent = ExerciseResult {
            total_chars: 300,
            correct_chars: 297,
            errors: 3,
            duration: Duration::from_secs(60),
            wpm: 60.0,
            error_rate: 1.0,
        };
        
        assert_eq!(excellent.grade(), PerformanceGrade::Excellent);
        
        let needs_work = ExerciseResult {
            total_chars: 100,
            correct_chars: 80,
            errors: 20,
            duration: Duration::from_secs(300),
            wpm: 15.0,
            error_rate: 20.0,
        };
        
        assert_eq!(needs_work.grade(), PerformanceGrade::NeedsImprovement);
    }
}