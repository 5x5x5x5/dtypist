//! Simple test for script parsing without UI components

use gtypist_rs::{Script, Executor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing script parser...");
    
    let script_file = std::env::args().nth(1).unwrap_or_else(|| {
        "test_script.typ".to_string()
    });
    
    println!("Loading script: {}", script_file);
    
    let script = Script::from_file(&script_file)?;
    println!("Successfully parsed script!");
    println!("Commands: {}", script.commands.len());
    println!("Labels: {}", script.labels.len());
    
    println!("\nLabel index:");
    for (label, pos) in &script.labels {
        println!("  {} -> command #{}", label, pos);
    }
    
    println!("\nCommands:");
    for (i, command) in script.commands.iter().enumerate() {
        println!("  {}: {:?}", i, command);
    }
    
    // Test executor creation
    let _executor = Executor::new(script);
    println!("\nExecutor created successfully!");
    
    Ok(())
}