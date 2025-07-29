//! GNU Typist - Rust Implementation
//! 
//! A Rust port of the GNU Typist typing tutor program.
//! This library provides the core functionality for parsing and executing
//! typing lesson scripts.

pub mod script;

pub use script::{Script, ScriptError, ScriptResult};
pub use script::commands::Command;
pub use script::executor::{Executor, ExecutionResult};
