//! GNU Typist - Rust Implementation
//! 
//! A Rust port of the GNU Typist typing tutor program.
//! This library provides the core functionality for parsing and executing
//! typing lesson scripts.

pub mod script;
pub mod exercises;
pub mod performance;

pub use script::{Script, ScriptError, ScriptResult};
pub use script::commands::Command;
pub use script::executor::{Executor, ExecutionResult};
pub use exercises::{TutorialExercise, DrillExercise, SpeedTestExercise, ExerciseOutcome};
pub use performance::{PerformanceTracker, ExerciseResult, PerformanceGrade, SpeedRecords};
