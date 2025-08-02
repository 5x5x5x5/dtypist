//! Script parsing and execution module
//! 
//! This module handles parsing and executing GNU Typist lesson script files (.typ).
//! It replicates the functionality of the C implementation's script.c

pub mod commands;
pub mod parser;
pub mod executor;

use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::fs;
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
    
    #[error("File error: {0}")]
    FileError(String),
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

/// Load text content from a file, resolving the path relative to the script directory
pub fn load_text_file(file_path: &str, script_path: &str) -> ScriptResult<String> {
    // Get the directory containing the script file
    let script_dir = Path::new(script_path)
        .parent()
        .unwrap_or_else(|| Path::new("."));
    
    // Resolve the text file path relative to the script directory
    let full_path = script_dir.join(file_path);
    
    // Security check: ensure the resolved path doesn't escape the script directory
    let canonical_script_dir = script_dir.canonicalize()
        .map_err(|e| ScriptError::FileError(format!("Cannot access script directory: {}", e)))?;
    
    let canonical_file_path = full_path.canonicalize()
        .map_err(|e| ScriptError::FileError(format!("Cannot access file '{}': {}", file_path, e)))?;
    
    if !canonical_file_path.starts_with(&canonical_script_dir) {
        return Err(ScriptError::FileError(format!(
            "File path '{}' is outside the script directory", 
            file_path
        )));
    }
    
    // Check file size to prevent memory issues (limit to 1MB)
    const MAX_FILE_SIZE: u64 = 1024 * 1024; // 1MB
    let metadata = fs::metadata(&full_path)
        .map_err(|e| ScriptError::FileError(format!("Cannot read file metadata for '{}': {}", file_path, e)))?;
    
    if metadata.len() > MAX_FILE_SIZE {
        return Err(ScriptError::FileError(format!(
            "File '{}' is too large ({} bytes). Maximum size is {} bytes.",
            file_path, metadata.len(), MAX_FILE_SIZE
        )));
    }
    
    // Read and return the file content
    let content = fs::read_to_string(&full_path)
        .map_err(|e| ScriptError::FileError(format!("Cannot read file '{}': {}", file_path, e)))?;
    
    Ok(content)
}