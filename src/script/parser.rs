//! Script file parser
//! 
//! Handles parsing lesson script files and building label indices,
//! replicating the functionality from the C implementation's build_label_index()

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::script::{Script, ScriptResult};
use crate::script::commands::{Command, MenuItem};

/// Parse a script file and build the complete Script structure
pub fn parse_script_file(path: &str) -> ScriptResult<Script> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    let mut commands = Vec::new();
    let mut labels = HashMap::new();
    let mut line_number = 0;
    let mut pending_menu: Option<(usize, String)> = None; // (command_index, title)
    
    for line in reader.lines() {
        line_number += 1;
        let line = line?;
        
        // Handle continuation lines (starting with space)
        if line.starts_with(' ') {
            if let Some((menu_idx, _)) = &pending_menu {
                // This is a menu item line
                parse_menu_item(&line, &mut commands, *menu_idx)?;
            } else {
                // This is a tutorial/instruction continuation line, append to last command
                append_continuation_line(&line, &mut commands)?;
            }
            continue;
        } else {
            pending_menu = None;
        }
        
        if let Some(command) = Command::parse_line(&line, line_number)? {
            let command_index = commands.len();
            
            // Index labels for fast navigation
            if let Command::Label { ref name } = command {
                labels.insert(name.clone(), command_index);
            }
            
            // Track menus for processing continuation lines
            if let Command::Menu { ref title, .. } = command {
                pending_menu = Some((command_index, title.clone()));
            }
            
            commands.push(command);
        }
    }
    
    Ok(Script {
        path: path.to_string(),
        commands,
        labels,
        position: 0,
    })
}

/// Parse a menu item line (format: " :LABEL  \"Description\"")
fn parse_menu_item(line: &str, commands: &mut Vec<Command>, menu_index: usize) -> ScriptResult<()> {
    let line = line.trim();
    
    // Skip empty continuation lines or lines with just ":"
    if line.is_empty() || line == ":" {
        return Ok(());
    }
    
    // Parse format: ":LABEL  \"Description\""
    if !line.starts_with(':') {
        return Ok(()); // Not a menu item, ignore
    }
    
    // Handle case where there's just ":" on the line
    if line == ":" {
        return Ok(());
    }
    
    let content = &line[1..]; // Remove leading ':'
    
    // Look for quoted description
    if let Some(quote_start) = content.find('"') {
        let label = content[..quote_start].trim().to_string();
        
        if let Some(quote_end) = content[quote_start + 1..].find('"') {
            let description = content[quote_start + 1..quote_start + 1 + quote_end].to_string();
            
            // Add the menu item to the existing menu command
            if let Some(Command::Menu { ref mut items, .. }) = commands.get_mut(menu_index) {
                items.push(MenuItem { label, description });
            }
        }
    }
    
    Ok(())
}

/// Append a continuation line to the last command's text
fn append_continuation_line(line: &str, commands: &mut Vec<Command>) -> ScriptResult<()> {
    let line = line.trim();
    
    // Skip empty continuation lines
    if line.is_empty() || line == ":" {
        return Ok(());
    }
    
    // Remove the leading ":" if present
    let text_to_append = if line.starts_with(':') {
        &line[1..]
    } else {
        line
    };
    
    // Find the last command that can accept continuation text
    if let Some(last_cmd) = commands.last_mut() {
        match last_cmd {
            Command::Tutorial { ref mut text } => {
                if !text.is_empty() {
                    text.push(' ');
                }
                text.push_str(text_to_append);
            },
            Command::Instruction { ref mut text } => {
                if !text.is_empty() {
                    text.push(' ');
                }
                text.push_str(text_to_append);
            },
            Command::Drill { ref mut text, .. } => {
                if !text.is_empty() {
                    text.push(' ');
                }
                text.push_str(text_to_append);
            },
            Command::SpeedTest { ref mut text, .. } => {
                if !text.is_empty() {
                    text.push(' ');
                }
                text.push_str(text_to_append);
            },
            _ => {
                // Ignore continuation lines for commands that don't support them
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_parse_simple_script() {
        let script_content = r#"
# Comment line
*:START
T:Welcome to the tutorial
D:Type this text
G:END
*:END
X:
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(script_content.as_bytes()).unwrap();
        
        let script = parse_script_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(script.commands.len(), 6);
        assert!(script.labels.contains_key("START"));
        assert!(script.labels.contains_key("END"));
        assert_eq!(script.labels["START"], 1); // Index of *:START command
        assert_eq!(script.labels["END"], 4);   // Index of *:END command
    }
    
    #[test]
    fn test_parse_menu_with_items() {
        let script_content = r#"
*:MENU
M: "Main Menu"
 :LESSON1  "Basic typing lesson"
 :LESSON2  "Advanced lesson"
 :EXIT     "Exit program"
*:LESSON1
T:This is lesson 1
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(script_content.as_bytes()).unwrap();
        
        let script = parse_script_file(temp_file.path().to_str().unwrap()).unwrap();
        
        // Find the menu command
        let menu_cmd = script.commands.iter().find(|cmd| {
            matches!(cmd, Command::Menu { .. })
        }).unwrap();
        
        if let Command::Menu { title, items } = menu_cmd {
            assert_eq!(title, "Main Menu");
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].label, "LESSON1");
            assert_eq!(items[0].description, "Basic typing lesson");
        }
    }
    
    #[test]
    fn test_command_parsing() {
        assert!(matches!(
            Command::parse_line("T:Hello world", 1).unwrap(),
            Some(Command::Tutorial { text }) if text == "Hello world"
        ));
        
        assert!(matches!(
            Command::parse_line("D:Type this", 1).unwrap(),
            Some(Command::Drill { text, practice_only: false }) if text == "Type this"
        ));
        
        assert!(matches!(
            Command::parse_line("d:Practice only", 1).unwrap(),
            Some(Command::Drill { text, practice_only: true }) if text == "Practice only"
        ));
        
        assert!(matches!(
            Command::parse_line("*:LABEL_NAME", 1).unwrap(),
            Some(Command::Label { name }) if name == "LABEL_NAME"
        ));
    }
}