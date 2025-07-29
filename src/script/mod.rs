//! Script parsing and execution module
//! 
//! This module handles parsing and executing GNU Typist lesson script files (.typ).
//! It replicates the functionality of the C implementation's script.c

pub mod commands;
pub mod parser;
pub mod executor;

use std::collections::HashMap;
use std::io;
use thiserror::Error;

/// Script parsing and execution errors
#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Invalid command format in line {line}: {content}")]
    InvalidCommand { line: usize, content: String },
    
    #[error("Label not found: {label}")]
    LabelNotFound { label: String },
    
    #[error("Invalid script format: {message}")]
    InvalidFormat { message: String },
    
    #[error("UTF-8 encoding error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
}

/// Result type for script operations
pub type ScriptResult<T> = Result<T, ScriptError>;

/// A parsed lesson script with indexed labels
#[derive(Debug, Clone)]
pub struct Script {
    /// Script file path
    pub path: String,
    /// All parsed commands in order
    pub commands: Vec<commands::Command>,
    /// Label index for fast navigation (label -> command index)
    pub labels: HashMap<String, usize>,
    /// Current execution position
    pub position: usize,
}

impl Script {
    /// Create a new script from a file path
    pub fn from_file(path: &str) -> ScriptResult<Self> {
        parser::parse_script_file(path)
    }
    
    /// Jump to a specific label
    pub fn goto_label(&mut self, label: &str) -> ScriptResult<()> {
        if let Some(&pos) = self.labels.get(label) {
            self.position = pos;
            Ok(())
        } else {
            Err(ScriptError::LabelNotFound { 
                label: label.to_string() 
            })
        }
    }
    
    /// Get the current command
    pub fn current_command(&self) -> Option<&commands::Command> {
        self.commands.get(self.position)
    }
    
    /// Advance to next command
    pub fn next(&mut self) -> Option<&commands::Command> {
        self.position += 1;
        self.current_command()
    }
    
    /// Check if we're at the end of the script
    pub fn is_finished(&self) -> bool {
        self.position >= self.commands.len()
    }
}