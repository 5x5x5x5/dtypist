//! GNU Typist - Rust Implementation
//! 
//! Main binary for the Rust port of GNU Typist

use clap::{Arg, Command as ClapCommand};
use gtypist_rs::{Script, Executor, ExecutionResult};
use std::process;

fn main() {
    let matches = ClapCommand::new("gtypist")
        .version("0.1.0")
        .about("GNU Typist - A typing tutor program (Rust implementation)")
        .author("Daniel Siegle <daniel.siegle@gmail.com>")
        .arg(
            Arg::new("script")
                .help("Script file to run")
                .required(false)
                .index(1)
        )
        .arg(
            Arg::new("list-lessons")
                .long("list-lessons")
                .short('l')
                .action(clap::ArgAction::SetTrue)
                .help("List available lessons")
        )
        .get_matches();

    if matches.get_flag("list-lessons") {
        list_lessons();
        return;
    }

    let default_script = "lessons/gtypist.typ".to_string();
    let script_file = matches.get_one::<String>("script")
        .unwrap_or(&default_script);

    println!("GNU Typist - Rust Implementation");
    println!("Loading script: {}", script_file);

    match run_script(script_file) {
        Ok(_) => println!("Script completed successfully"),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn run_script(script_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let script = Script::from_file(script_file)?;
    let mut executor = Executor::new(script);
    
    println!("Found {} commands and {} labels", 
             executor.script.commands.len(), 
             executor.script.labels.len());

    // Simple execution loop (will be enhanced in later phases)
    loop {
        match executor.execute_next()? {
            ExecutionResult::Continue => continue,
            ExecutionResult::Jump(label) => {
                executor.script.goto_label(&label)?;
            },
            ExecutionResult::Exit | ExecutionResult::Finished => break,
            ExecutionResult::WaitForInput => {
                // TODO: Implement user input handling
                println!("Press Enter to continue...");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
            }
        }
    }
    
    Ok(())
}

fn list_lessons() {
    println!("Available lessons:");
    
    let lesson_files = [
        ("gtypist.typ", "Main lesson selection menu"),
        ("demo.typ", "Demonstration of commands and features"),
        ("q.typ", "QWERTY basic lessons"),
        ("r.typ", "QWERTY review lessons"),
        ("t.typ", "QWERTY touch typing"),
        ("ktdvorak.typ", "Dvorak keyboard lessons"),
    ];
    
    for (file, description) in lesson_files {
        println!("  {:<15} - {}", file, description);
    }
}