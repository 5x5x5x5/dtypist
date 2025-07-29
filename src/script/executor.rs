//! Script command executor
//! 
//! Handles execution of parsed script commands with integrated exercise engine.

use crate::script::{Script, ScriptResult};
use crate::script::commands::Command;
use crate::exercises::{TutorialExercise, DrillExercise, SpeedTestExercise, ExerciseOutcome};
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
    cursor,
};
use std::io::{stdout, Write};

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
                let exercise = TutorialExercise::new(text);
                match exercise.execute() {
                    Ok(ExerciseOutcome::Completed(_)) => Ok(ExecutionResult::Continue),
                    Ok(ExerciseOutcome::Quit) => Ok(ExecutionResult::Exit),
                    Ok(ExerciseOutcome::Retry) => Ok(ExecutionResult::Continue), // Retry the same command
                    Ok(ExerciseOutcome::Failed) => Ok(ExecutionResult::Continue), // Continue for tutorials
                    Err(_) => Ok(ExecutionResult::Continue), // Handle errors gracefully
                }
            },
            
            Command::Instruction { text } => {
                let mut stdout = stdout();
                execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0)).ok();
                println!("{}", text);
                println!("\nPress any key to continue...");
                stdout.flush().ok();
                Ok(ExecutionResult::WaitForInput)
            },
            
            Command::Clear { banner } => {
                let mut stdout = stdout();
                execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0)).ok();
                if let Some(banner_text) = banner {
                    println!("=== {} ===\n", banner_text);
                }
                stdout.flush().ok();
                Ok(ExecutionResult::Continue)
            },
            
            Command::Goto { label } => {
                Ok(ExecutionResult::Jump(label))
            },
            
            Command::Exit => Ok(ExecutionResult::Exit),
            
            Command::Drill { text, practice_only } => {
                let exercise = DrillExercise::new(text, practice_only, self.error_percentage);
                match exercise.execute() {
                    Ok(ExerciseOutcome::Completed(_)) => Ok(ExecutionResult::Continue),
                    Ok(ExerciseOutcome::Quit) => Ok(ExecutionResult::Exit),
                    Ok(ExerciseOutcome::Retry) => Ok(ExecutionResult::Continue), // Retry the same command
                    Ok(ExerciseOutcome::Failed) => {
                        // Jump to failure label if set, otherwise continue
                        if let Some(ref label) = self.failure_label {
                            Ok(ExecutionResult::Jump(label.clone()))
                        } else {
                            Ok(ExecutionResult::Continue)
                        }
                    },
                    Err(_) => Ok(ExecutionResult::Continue), // Handle errors gracefully
                }
            },
            
            Command::SpeedTest { text, practice_only } => {
                let exercise = SpeedTestExercise::new(text, practice_only, None); // No time limit by default
                match exercise.execute() {
                    Ok(ExerciseOutcome::Completed(_)) => Ok(ExecutionResult::Continue),
                    Ok(ExerciseOutcome::Quit) => Ok(ExecutionResult::Exit),
                    Ok(ExerciseOutcome::Retry) => Ok(ExecutionResult::Continue), // Retry the same command
                    Ok(ExerciseOutcome::Failed) => Ok(ExecutionResult::Continue), // Speed tests don't typically fail
                    Err(_) => Ok(ExecutionResult::Continue), // Handle errors gracefully
                }
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