//! Integration tests for GNU Typist Rust implementation

use gtypist_rs::{Script, Executor};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_basic_script_parsing() {
    let script_content = r#"
# Test script
*:START
T:Hello world
D:Type this text
G:END
*:END
X:
"#;
    
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(script_content.as_bytes()).unwrap();
    
    let script = Script::from_file(temp_file.path().to_str().unwrap()).unwrap();
    
    assert_eq!(script.commands.len(), 7);
    assert!(script.labels.contains_key("START"));
    assert!(script.labels.contains_key("END"));
}

#[test]
fn test_command_parsing() {
    use gtypist_rs::script::commands::Command;
    
    // Test tutorial command
    let cmd = Command::parse_line("T:Hello world", 1).unwrap().unwrap();
    match cmd {
        Command::Tutorial { text } => assert_eq!(text, "Hello world"),
        _ => panic!("Expected Tutorial command"),
    }
    
    // Test drill command
    let cmd = Command::parse_line("D:Type this", 1).unwrap().unwrap();
    match cmd {
        Command::Drill { text, practice_only } => {
            assert_eq!(text, "Type this");
            assert!(!practice_only);
        },
        _ => panic!("Expected Drill command"),
    }
    
    // Test label command
    let cmd = Command::parse_line("*:MYLABEL", 1).unwrap().unwrap();
    match cmd {
        Command::Label { name } => assert_eq!(name, "MYLABEL"),
        _ => panic!("Expected Label command"),
    }
}

#[test]
fn test_executor_creation() {
    let script_content = r#"
*:START
T:Test
X:
"#;
    
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(script_content.as_bytes()).unwrap();
    
    let script = Script::from_file(temp_file.path().to_str().unwrap()).unwrap();
    let executor = Executor::new(script);
    
    assert_eq!(executor.error_percentage, 0.0);
    assert!(executor.failure_label.is_none());
}

#[test]
fn test_menu_parsing() {
    let script_content = r#"
M: "Test Menu"
 :ITEM1  "First item"
 :ITEM2  "Second item"
"#;
    
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(script_content.as_bytes()).unwrap();
    
    let script = Script::from_file(temp_file.path().to_str().unwrap()).unwrap();
    
    // Should have one menu command
    assert_eq!(script.commands.len(), 1);
    
    match &script.commands[0] {
        gtypist_rs::script::commands::Command::Menu { title, items } => {
            assert_eq!(title, "Test Menu");
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].label, "ITEM1");
            assert_eq!(items[0].description, "First item");
        },
        _ => panic!("Expected Menu command"),
    }
}

#[test]
fn test_error_percentage_parsing() {
    use gtypist_rs::script::commands::Command;
    
    // Test percentage with % sign
    let cmd = Command::parse_line("E:5%", 1).unwrap().unwrap();
    match cmd {
        Command::ErrorMaxSet { percentage } => assert_eq!(percentage, 5.0),
        _ => panic!("Expected ErrorMaxSet command"),
    }
    
    // Test percentage with %* suffix
    let cmd = Command::parse_line("E:10%*", 1).unwrap().unwrap();
    match cmd {
        Command::ErrorMaxSet { percentage } => assert_eq!(percentage, 10.0),
        _ => panic!("Expected ErrorMaxSet command"),
    }
    
    // Test default keyword
    let cmd = Command::parse_line("E:default", 1).unwrap().unwrap();
    match cmd {
        Command::ErrorMaxSet { percentage } => assert_eq!(percentage, 0.0),
        _ => panic!("Expected ErrorMaxSet command"),
    }
}

#[test]
fn test_continuation_lines() {
    let script_content = r#"
T:This is line one
 :and this is continuation
I:Instruction line
 :with continuation
"#;
    
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(script_content.as_bytes()).unwrap();
    
    let script = Script::from_file(temp_file.path().to_str().unwrap()).unwrap();
    
    assert_eq!(script.commands.len(), 2);
    
    match &script.commands[0] {
        gtypist_rs::script::commands::Command::Tutorial { text } => {
            assert!(text.contains("This is line one"));
            assert!(text.contains("and this is continuation"));
        },
        _ => panic!("Expected Tutorial command"),
    }
    
    match &script.commands[1] {
        gtypist_rs::script::commands::Command::Instruction { text } => {
            assert!(text.contains("Instruction line"));
            assert!(text.contains("with continuation"));
        },
        _ => panic!("Expected Instruction command"),
    }
}

#[test]
fn test_label_navigation() {
    let script_content = r#"
*:START
T:First
G:END
T:Never reached
*:END
T:End reached
"#;
    
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(script_content.as_bytes()).unwrap();
    
    let mut script = Script::from_file(temp_file.path().to_str().unwrap()).unwrap();
    
    // Test jumping to END label
    script.goto_label("END").unwrap();
    
    // Should be at the END label command
    match script.current_command().unwrap() {
        gtypist_rs::script::commands::Command::Label { name } => {
            assert_eq!(name, "END");
        },
        _ => panic!("Expected to be at END label"),
    }
}