//! Script command definitions
//! 
//! Defines all the lesson script commands supported by GNU Typist,
//! matching the C implementation in script.h

use serde::{Deserialize, Serialize};

/// All supported script commands
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
    /// Comment line - ignored during execution
    Comment { content: String },
    
    /// Label definition (*:LABEL_NAME)
    Label { name: String },
    
    /// Tutorial text display (T:text)
    Tutorial { text: String },
    
    /// Instruction display (I:text) 
    Instruction { text: String },
    
    /// Clear screen and set banner (B:banner_text)
    Clear { banner: Option<String> },
    
    /// Jump to label (G:LABEL_NAME)
    Goto { label: String },
    
    /// Exit script (X:)
    Exit,
    
    /// Query/question (Q:question_text)
    Query { text: String },
    
    /// Conditional goto if yes (Y:LABEL_NAME)
    YesGoto { label: String },
    
    /// Conditional goto if no (N:LABEL_NAME)  
    NoGoto { label: String },
    
    /// Drill exercise (D:text_to_type)
    Drill { 
        text: String,
        practice_only: bool,
    },
    
    /// Speed test exercise (S:text_to_type)
    SpeedTest { 
        text: String,
        practice_only: bool,
    },
    
    /// Tutorial exercise from file (t:filename.txt)
    TutorialFile { path: String },
    
    /// Drill exercise from file (f:filename.txt)
    DrillFile { 
        path: String,
        practice_only: bool,
    },
    
    /// Speed test exercise from file (z:filename.txt)
    SpeedTestFile { 
        path: String,
        practice_only: bool,
    },
    
    /// Key binding (K:key_sequence)
    KeyBind { sequence: String },
    
    /// Set maximum error percentage (E:percentage)
    ErrorMaxSet { percentage: f32 },
    
    /// Set failure label (F:LABEL_NAME)
    OnFailureSet { label: String },
    
    /// Menu definition (M:title)
    Menu { 
        title: String,
        items: Vec<MenuItem>,
    },
}

/// Menu item definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MenuItem {
    pub label: String,
    pub description: String,
}

/// Command character constants matching C implementation
pub mod chars {
    pub const COMMENT: char = '#';
    pub const ALT_COMMENT: char = '!';
    pub const SEP: char = ':';
    pub const CONT: char = ' ';
    pub const LABEL: char = '*';
    pub const TUTORIAL: char = 'T';
    pub const INSTRUCTION: char = 'I';
    pub const CLEAR: char = 'B';
    pub const GOTO: char = 'G';
    pub const EXIT: char = 'X';
    pub const QUERY: char = 'Q';
    pub const YES_GOTO: char = 'Y';
    pub const NO_GOTO: char = 'N';
    pub const DRILL: char = 'D';
    pub const DRILL_PRACTICE_ONLY: char = 'd';
    pub const SPEEDTEST: char = 'S';
    pub const SPEEDTEST_PRACTICE_ONLY: char = 's';
    pub const TUTORIAL_FILE: char = 't';
    pub const DRILL_FILE: char = 'f';
    pub const DRILL_FILE_PRACTICE: char = 'p';
    pub const SPEEDTEST_FILE: char = 'z';
    pub const SPEEDTEST_FILE_PRACTICE: char = 'w';
    pub const KEYBIND: char = 'K';
    pub const ERROR_MAX_SET: char = 'E';
    pub const ON_FAILURE_SET: char = 'F';
    pub const MENU: char = 'M';
}

impl Command {
    /// Parse a single line into a command
    pub fn parse_line(line: &str, line_number: usize) -> Result<Option<Command>, crate::script::ScriptError> {
        let line = line.trim();
        
        // Skip empty lines
        if line.is_empty() {
            return Ok(None);
        }
        
        // Skip comments
        if line.starts_with(chars::COMMENT) || line.starts_with(chars::ALT_COMMENT) {
            return Ok(Some(Command::Comment { 
                content: line.to_string() 
            }));
        }
        
        // Must have at least "X:" format
        if line.len() < 2 {
            return Err(crate::script::ScriptError::InvalidCommand {
                line: line_number,
                content: line.to_string(),
            });
        }
        
        let command_char = line.chars().nth(0).unwrap();
        let separator = line.chars().nth(1).unwrap();
        
        if separator != chars::SEP {
            return Err(crate::script::ScriptError::InvalidCommand {
                line: line_number,
                content: line.to_string(),
            });
        }
        
        let data = if line.len() > 2 { &line[2..] } else { "" };
        
        let command = match command_char {
            chars::LABEL => Command::Label { 
                name: data.to_string() 
            },
            chars::TUTORIAL => Command::Tutorial { 
                text: data.to_string() 
            },
            chars::INSTRUCTION => Command::Instruction { 
                text: data.to_string() 
            },
            chars::CLEAR => Command::Clear { 
                banner: if data.is_empty() { None } else { Some(data.to_string()) }
            },
            chars::GOTO => Command::Goto { 
                label: data.to_string() 
            },
            chars::EXIT => Command::Exit,
            chars::QUERY => Command::Query { 
                text: data.to_string() 
            },
            chars::YES_GOTO => Command::YesGoto { 
                label: data.to_string() 
            },
            chars::NO_GOTO => Command::NoGoto { 
                label: data.to_string() 
            },
            chars::DRILL => Command::Drill { 
                text: data.to_string(),
                practice_only: false,
            },
            chars::DRILL_PRACTICE_ONLY => Command::Drill { 
                text: data.to_string(),
                practice_only: true,
            },
            chars::SPEEDTEST => Command::SpeedTest { 
                text: data.to_string(),
                practice_only: false,
            },
            chars::SPEEDTEST_PRACTICE_ONLY => Command::SpeedTest { 
                text: data.to_string(),
                practice_only: true,
            },
            chars::TUTORIAL_FILE => Command::TutorialFile { 
                path: data.to_string() 
            },
            chars::DRILL_FILE => Command::DrillFile { 
                path: data.to_string(),
                practice_only: false,
            },
            chars::DRILL_FILE_PRACTICE => Command::DrillFile { 
                path: data.to_string(),
                practice_only: true,
            },
            chars::SPEEDTEST_FILE => Command::SpeedTestFile { 
                path: data.to_string(),
                practice_only: false,
            },
            chars::SPEEDTEST_FILE_PRACTICE => Command::SpeedTestFile { 
                path: data.to_string(),
                practice_only: true,
            },
            chars::KEYBIND => Command::KeyBind { 
                sequence: data.to_string() 
            },
            chars::ERROR_MAX_SET => {
                // Handle special cases like "default"
                if data.trim() == "default" {
                    Command::ErrorMaxSet { percentage: 0.0 }
                } else {
                    // Remove % sign and any trailing characters, parse as float
                    let percentage_str = data.trim()
                        .trim_end_matches('*')
                        .trim_end_matches('%')
                        .trim();
                    let percentage = percentage_str.parse::<f32>().map_err(|_| {
                        crate::script::ScriptError::InvalidCommand {
                            line: line_number,
                            content: line.to_string(),
                        }
                    })?;
                    Command::ErrorMaxSet { percentage }
                }
            },
            chars::ON_FAILURE_SET => Command::OnFailureSet { 
                label: data.to_string() 
            },
            chars::MENU => {
                // Remove quotes and leading/trailing spaces from menu title
                let cleaned_data = data.trim();
                let title = if cleaned_data.starts_with('"') && cleaned_data.ends_with('"') && cleaned_data.len() >= 2 {
                    cleaned_data[1..cleaned_data.len()-1].to_string()
                } else {
                    cleaned_data.to_string()
                };
                Command::Menu { 
                    title,
                    items: Vec::new(), // Will be populated by parser
                }
            },
            _ => {
                return Err(crate::script::ScriptError::InvalidCommand {
                    line: line_number,
                    content: line.to_string(),
                });
            }
        };
        
        Ok(Some(command))
    }
}