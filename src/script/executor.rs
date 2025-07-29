//! Script command executor
//! 
//! Handles execution of parsed script commands.
//! This will be expanded in later phases of the migration.

use crate::script::{Script, ScriptResult};
use crate::script::commands::Command;

/// Script executor state
pub struct Executor {
    pub script: Script,
    pub error_percentage: f32,
    pub failure_label: Option<String>,
}

impl Executor {
    /// Create a new executor for a script
    pub fn new(script: Script) -> Self {
        Self {
            script,
            error_percentage: 0.0,
            failure_label: None,
        }
    }
    
    /// Execute the next command in the script
    pub fn execute_next(&mut self) -> ScriptResult<ExecutionResult> {
        if let Some(command) = self.script.current_command() {
            let result = self.execute_command(command.clone())?;
            if !matches!(result, ExecutionResult::Jump(_)) {
                self.script.next();
            }
            Ok(result)
        } else {
            Ok(ExecutionResult::Finished)
        }
    }
    
    /// Execute a specific command
    fn execute_command(&mut self, command: Command) -> ScriptResult<ExecutionResult> {
        match command {
            Command::Comment { .. } => Ok(ExecutionResult::Continue),
            
            Command::Label { .. } => Ok(ExecutionResult::Continue),
            
            Command::Tutorial { text } => {
                // TODO: Implement tutorial display
                println!("TUTORIAL: {}", text);
                Ok(ExecutionResult::Continue)
            },
            
            Command::Instruction { text } => {
                // TODO: Implement instruction display  
                println!("INSTRUCTION: {}", text);
                Ok(ExecutionResult::Continue)
            },
            
            Command::Clear { banner } => {
                // TODO: Implement screen clearing
                if let Some(banner_text) = banner {
                    println!("BANNER: {}", banner_text);
                }
                Ok(ExecutionResult::Continue)
            },
            
            Command::Goto { label } => {
                Ok(ExecutionResult::Jump(label))
            },
            
            Command::Exit => Ok(ExecutionResult::Exit),
            
            Command::Drill { text, practice_only } => {
                // TODO: Implement drill exercise
                println!("DRILL{}: {}", if practice_only { " (practice)" } else { "" }, text);
                Ok(ExecutionResult::Continue)
            },
            
            Command::SpeedTest { text, practice_only } => {
                // TODO: Implement speed test exercise
                println!("SPEEDTEST{}: {}", if practice_only { " (practice)" } else { "" }, text);
                Ok(ExecutionResult::Continue)
            },
            
            Command::ErrorMaxSet { percentage } => {
                self.error_percentage = percentage;
                Ok(ExecutionResult::Continue)
            },
            
            Command::OnFailureSet { label } => {
                self.failure_label = Some(label);
                Ok(ExecutionResult::Continue)
            },
            
            _ => {
                // TODO: Implement remaining commands
                println!("TODO: Implement command {:?}", command);
                Ok(ExecutionResult::Continue)
            }
        }
    }
}

/// Result of executing a command
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    /// Continue to next command
    Continue,
    /// Jump to a specific label
    Jump(String),
    /// Exit the script
    Exit,
    /// Script execution finished
    Finished,
    /// Wait for user input
    WaitForInput,
}