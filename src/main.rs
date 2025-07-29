//! GNU Typist - Rust Implementation
//! 
//! A typing tutor program that teaches touch typing through structured lessons.
//! This is a Rust port of the original C implementation of GNU Typist.

use clap::{App, Arg, ArgMatches};
use gtypist_rs::{Script, Executor, ExecutionResult};
use std::path::Path;
use std::process;
use crossterm::{
    execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
    cursor,
};
use std::io::{stdout, Write};

fn main() {
    let matches = create_cli().get_matches();
    
    // Run application (raw mode will be enabled when needed)
    let result = run_application(&matches);
    
    // Cleanup terminal (in case raw mode was enabled)
    let _ = disable_raw_mode();
    let _ = execute!(stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0));
    
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn create_cli() -> App<'static, 'static> {
    App::new("gtypist")
        .version("2.10.1-rust")
        .author("GNU Typist Team <bug-gtypist@gnu.org>")
        .about("A typing tutor program that teaches touch typing")
        .arg(Arg::with_name("lesson")
            .help("Lesson file to load (.typ)")
            .required(false)
            .index(1))
        .arg(Arg::with_name("label")
            .short("l")
            .long("label")
            .value_name("LABEL")
            .help("Start at specific label in lesson")
            .takes_value(true))
        .arg(Arg::with_name("personal-best")
            .short("p")
            .long("personal-best")
            .help("Show personal best times"))
        .arg(Arg::with_name("silent")
            .short("s")
            .long("silent")
            .help("Silent mode - no sound"))
        .arg(Arg::with_name("colours")
            .short("c")
            .long("colours")
            .help("Use colours in terminal"))
        .arg(Arg::with_name("no-colours")
            .long("no-colours")
            .help("Disable colours in terminal"))
}

fn run_application(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let lesson_file = matches.value_of("lesson").unwrap_or("lessons/gtypist.typ");
    let start_label = matches.value_of("label");
    
    // Check if lesson file exists
    if !Path::new(lesson_file).exists() {
        return Err(format!("Lesson file not found: {}", lesson_file).into());
    }
    
    // Display welcome message (before enabling raw mode)
    display_welcome()?;
    
    // Now enable raw mode for interactive parts
    if let Err(e) = enable_raw_mode() {
        return Err(format!("Failed to enable terminal raw mode: {}", e).into());
    }
    
    // Parse and execute the lesson script
    let script = Script::from_file(lesson_file)?;
    let mut executor = Executor::new(script);
    
    // Jump to start label if specified
    if let Some(label) = start_label {
        executor.script.goto_label(label)?;
    }
    
    // Main execution loop
    loop {
        match executor.execute_next()? {
            ExecutionResult::Continue => {
                // Continue to next command
            },
            ExecutionResult::Exit => {
                display_goodbye()?;
                break;
            },
            ExecutionResult::Finished => {
                display_completion()?;
                break;
            },
            ExecutionResult::WaitForInput => {
                // Wait for user input
                use crossterm::event::{read, Event, KeyCode, KeyEvent};
                loop {
                    match read()? {
                        Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                            display_goodbye()?;
                            return Ok(());
                        },
                        Event::Key(_) => break, // Any other key continues
                        _ => continue,
                    }
                }
            },
            ExecutionResult::Jump(_) => {
                // This should not happen at this level since it's handled in execute_next
                continue;
            },
        }
        
        // Check if script is finished
        if executor.script.is_finished() {
            display_completion()?;
            break;
        }
    }
    
    Ok(())
}

/// Helper function to center text in terminal
fn center_text(text: &str) -> String {
    // Try to get terminal size, fall back to reasonable defaults
    let width = match crossterm::terminal::size() {
        Ok((w, _)) => w as usize,
        Err(_) => {
            // If we can't get terminal size, try environment variables or use 80
            std::env::var("COLUMNS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(80)
        }
    };
    
    let text_len = text.chars().count(); // Use char count for proper Unicode handling
    if text_len >= width {
        return text.to_string(); // Don't try to center if text is too long
    }
    
    let padding = (width - text_len) / 2;
    format!("{}{}", " ".repeat(padding), text)
}

fn display_welcome() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    
    // Clear screen but don't use raw mode yet
    print!("\x1B[2J\x1B[1;1H"); // ANSI escape codes for clear screen and move cursor
    
    println!();
    println!("{}", center_text("=== GNU Typist - Rust Implementation ===")); 
    println!("{}", center_text("Version 2.10.1"));
    println!();
    println!("{}", center_text("A typing tutor to help you learn touch typing."));
    println!("{}", center_text("Press ESC at any time to exit."));
    println!();
    println!("{}", center_text("Press any key to continue..."));
    println!();
    stdout.flush()?;
    
    // Wait for user input
    use crossterm::event::{read, Event, KeyCode, KeyEvent};
    loop {
        match read()? {
            Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                return Err("User cancelled".into());
            },
            Event::Key(_) => break,
            _ => continue,
        }
    }
    
    Ok(())
}

fn display_goodbye() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    
    println!();
    println!("{}", center_text("=== GNU Typist ==="));
    println!();
    println!("{}", center_text("Thanks for using GNU Typist!"));
    println!("{}", center_text("Keep practicing to improve your typing skills."));
    println!();
    println!("{}", center_text("Press any key to exit..."));
    println!();
    stdout.flush()?;
    
    // Wait for user input
    use crossterm::event::{read, Event, KeyEvent};
    loop {
        match read()? {
            Event::Key(KeyEvent { .. }) => break,
            _ => continue,
        }
    }
    
    Ok(())
}

fn display_completion() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    
    println!();
    println!("{}", center_text("=== Lesson Complete ==="));
    println!();
    println!("{}", center_text("Congratulations! You have completed this lesson."));
    println!("{}", center_text("Continue practicing to improve your typing skills."));
    println!();
    println!("{}", center_text("Press any key to exit..."));
    println!();
    stdout.flush()?;
    
    // Wait for user input
    use crossterm::event::{read, Event, KeyEvent};
    loop {
        match read()? {
            Event::Key(KeyEvent { .. }) => break,
            _ => continue,
        }
    }
    
    Ok(())
}