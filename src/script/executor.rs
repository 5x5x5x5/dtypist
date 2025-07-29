//! Script command executor
//! 
//! Handles execution of parsed script commands with integrated exercise engine.

use crate::script::{Script, ScriptResult};
use crate::script::commands::Command;
use crate::exercises::{TutorialExercise, DrillExercise, SpeedTestExercise, ExerciseOutcome};
use crate::menu::Menu;
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
    pub last_query_response: Option<bool>, // Track Y/N responses for conditional jumps
}

impl Executor {
    /// Create a new executor for a script
    pub fn new(script: Script) -> Self {
        Self {
            script,
            error_percentage: 0.0,
            failure_label: None,
            last_query_response: None,
        }
    }
    
    /// Execute the next command in the script
    pub fn execute_next(&mut self) -> ScriptResult<ExecutionResult> {
        if let Some(command) = self.script.current_command() {
            let result = self.execute_command(command.clone())?;
            match result {
                ExecutionResult::Jump(ref label) => {
                    // Handle jump by updating script position
                    self.script.goto_label(label)?;
                    Ok(ExecutionResult::Continue)
                },
                _ => {
                    // Normal execution - advance to next command
                    if !matches!(result, ExecutionResult::Exit | ExecutionResult::Finished) {
                        self.script.next();
                    }
                    Ok(result)
                }
            }
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
            
            Command::Menu { title, items } => {
                let mut menu = Menu::new(title);
                for item in items {
                    menu.add_item(item.label, item.description);
                }
                
                match menu.display() {
                    Ok(Some(selected_label)) => Ok(ExecutionResult::Jump(selected_label)),
                    Ok(None) => Ok(ExecutionResult::Exit), // User quit menu
                    Err(_) => Ok(ExecutionResult::Continue), // Handle errors gracefully
                }
            },
            
            Command::Query { text } => {
                let mut stdout = stdout();
                execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0)).ok();
                println!("{}", text);
                println!("\nPress Y for yes, N for no, ESC to quit...");
                stdout.flush().ok();
                
                // Handle user input for query
                use crossterm::event::{read, Event, KeyCode, KeyEvent};
                loop {
                    match read() {
                        Ok(Event::Key(KeyEvent { code: KeyCode::Char('y'), .. })) |
                        Ok(Event::Key(KeyEvent { code: KeyCode::Char('Y'), .. })) => {
                            self.last_query_response = Some(true);
                            return Ok(ExecutionResult::Continue);
                        },
                        Ok(Event::Key(KeyEvent { code: KeyCode::Char('n'), .. })) |
                        Ok(Event::Key(KeyEvent { code: KeyCode::Char('N'), .. })) => {
                            self.last_query_response = Some(false);
                            return Ok(ExecutionResult::Continue);
                        },
                        Ok(Event::Key(KeyEvent { code: KeyCode::Esc, .. })) => {
                            return Ok(ExecutionResult::Exit);
                        },
                        _ => continue,
                    }
                }
            },
            
            Command::YesGoto { label } => {
                if let Some(true) = self.last_query_response {
                    Ok(ExecutionResult::Jump(label))
                } else {
                    Ok(ExecutionResult::Continue)
                }
            },
            
            Command::NoGoto { label } => {
                if let Some(false) = self.last_query_response {
                    Ok(ExecutionResult::Jump(label))
                } else {
                    Ok(ExecutionResult::Continue)
                }
            },
            
            Command::KeyBind { sequence } => {
                // TODO: Implement key binding functionality
                // For now, just log and continue
                println!("Key binding: {}", sequence);
                Ok(ExecutionResult::Continue)
            },
            
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