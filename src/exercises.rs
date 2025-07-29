//! Exercise execution engine
//! 
//! Implements the three core exercise types: Tutorial, Drill, and Speed Test
//! This replicates the functionality from the C implementation's do_tutorial, 
//! do_drill, and do_speedtest functions.

use std::time::{Duration, Instant};
use crossterm::{
    cursor, execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
};
use std::io::{stdout, Write};
use crate::performance::{PerformanceTracker, ExerciseResult};

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

/// Exercise execution results
#[derive(Debug, Clone, PartialEq)]
pub enum ExerciseOutcome {
    /// Exercise completed successfully
    Completed(ExerciseResult),
    /// User quit the exercise
    Quit,
    /// Exercise failed (too many errors)
    Failed,
    /// User requested retry
    Retry,
}

/// Tutorial exercise - display-only, no user input required
#[derive(Debug, Clone)]
pub struct TutorialExercise {
    pub text: String,
}

impl TutorialExercise {
    pub fn new(text: String) -> Self {
        Self { text }
    }
    
    /// Execute tutorial - just display text and wait for user
    pub fn execute(&self) -> Result<ExerciseOutcome, Box<dyn std::error::Error>> {
        let mut stdout = stdout();
        
        // Clear screen with direct ANSI codes
        print!("\x1B[2J\x1B[1;1H");
        
        println!();
        println!("{}", center_text("=== TUTORIAL ==="));
        println!();
        
        // Display the tutorial text (truncated to prevent excessive output)
        const MAX_TUTORIAL_DISPLAY: usize = 2000;
        let display_text = if self.text.len() > MAX_TUTORIAL_DISPLAY {
            format!("{}...", &self.text[..MAX_TUTORIAL_DISPLAY])
        } else {
            self.text.clone()
        };
        
        // Word wrap the text properly
        let width = match crossterm::terminal::size() {
            Ok((w, _)) => (w as usize).saturating_sub(4),
            Err(_) => 76,
        };
        
        for line in display_text.split('\n') {
            if line.len() <= width {
                println!("  {}", line);
            } else {
                // Simple word wrapping
                let words: Vec<&str> = line.split_whitespace().collect();
                let mut current_line = String::new();
                
                for word in words {
                    if current_line.len() + word.len() + 1 <= width {
                        if !current_line.is_empty() {
                            current_line.push(' ');
                        }
                        current_line.push_str(word);
                    } else {
                        if !current_line.is_empty() {
                            println!("  {}", current_line);
                            current_line.clear();
                        }
                        current_line.push_str(word);
                    }
                }
                if !current_line.is_empty() {
                    println!("  {}", current_line);
                }
            }
        }
        
        println!();
        println!("{}", center_text("Press SPACE to continue, ESC to quit..."));
        stdout.flush()?;
        
        // Wait for user input
        loop {
            match read()? {
                Event::Key(KeyEvent { code: KeyCode::Char(' '), .. }) => {
                    return Ok(ExerciseOutcome::Completed(ExerciseResult::default()));
                },
                Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                    return Ok(ExerciseOutcome::Quit);
                },
                _ => continue,
            }
        }
    }
}

/// Drill exercise - typing practice with error tracking
#[derive(Debug, Clone)]
pub struct DrillExercise {
    pub text: String,
    pub practice_only: bool,
    pub max_error_rate: f32,
}

impl DrillExercise {
    pub fn new(text: String, practice_only: bool, max_error_rate: f32) -> Self {
        Self { 
            text, 
            practice_only, 
            max_error_rate: if max_error_rate <= 0.0 { 100.0 } else { max_error_rate }
        }
    }
    
    /// Execute drill exercise with real-time feedback
    pub fn execute(&self) -> Result<ExerciseOutcome, Box<dyn std::error::Error>> {
        let mut stdout = stdout();
        let mut tracker = PerformanceTracker::new();
        let target_chars: Vec<char> = self.text.chars().collect();
        let mut position = 0;
        let mut typed_text = String::new();
        
        // Clear screen with direct ANSI codes
        print!("\x1B[2J\x1B[1;1H");
        
        println!();
        println!("{}", center_text(&format!("=== {} ===", 
            if self.practice_only { "DRILL PRACTICE" } else { "DRILL" })));
        println!();
        println!("{}", center_text("Type the following text. Press ESC to quit, Ctrl+R to retry."));
        println!();
        
        // Display target text (truncated to prevent excessive output)
        const MAX_TARGET_DISPLAY: usize = 500;
        let display_text = if self.text.len() > MAX_TARGET_DISPLAY {
            format!("{}...", &self.text[..MAX_TARGET_DISPLAY])
        } else {
            self.text.clone()
        };
        
        println!("{}", center_text("Target:"));
        println!("  {}", display_text);
        println!();
        println!("{}", center_text("Your typing:"));
        stdout.flush()?;
        
        let start_time = Instant::now();
        
        loop {
            match read()? {
                Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                    return Ok(ExerciseOutcome::Quit);
                },
                Event::Key(KeyEvent { 
                    code: KeyCode::Char('r'), 
                    modifiers: KeyModifiers::CONTROL,
                    .. 
                }) => {
                    return Ok(ExerciseOutcome::Retry);
                },
                Event::Key(KeyEvent { code: KeyCode::Char(ch), .. }) => {
                    if position < target_chars.len() {
                        let expected = target_chars[position];
                        typed_text.push(ch);
                        
                        if ch == expected {
                            tracker.record_correct_char();
                        } else {
                            tracker.record_error();
                        }
                        
                        position += 1;
                        
                        // Display progress after each character
                        self.display_progress(&mut stdout, &typed_text, &target_chars, position)?;
                        
                        // Check if exercise is complete
                        if position >= target_chars.len() {
                            break;
                        }
                        
                        // Check error rate if not practice mode
                        if !self.practice_only && tracker.error_rate() > self.max_error_rate {
                            println!("\nToo many errors! Try again.");
                            return Ok(ExerciseOutcome::Failed);
                        }
                    }
                },
                Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
                    if position > 0 && !typed_text.is_empty() {
                        position -= 1;
                        typed_text.pop();
                        tracker.record_backspace();
                        self.display_progress(&mut stdout, &typed_text, &target_chars, position)?;
                    }
                },
                _ => continue,
            }
        }
        
        let duration = start_time.elapsed();
        tracker.set_duration(duration);
        
        let result = ExerciseResult {
            total_chars: target_chars.len(),
            correct_chars: tracker.correct_chars(),
            errors: tracker.errors(),
            duration,
            wpm: tracker.words_per_minute(),
            error_rate: tracker.error_rate(),
        };
        
        // Display final results
        self.display_results(&result)?;
        
        Ok(ExerciseOutcome::Completed(result))
    }
    
    fn display_progress(
        &self, 
        stdout: &mut std::io::Stdout,
        typed_text: &str,
        target_chars: &[char],
        position: usize
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Move cursor to typing area
        execute!(stdout, cursor::MoveTo(0, 6))?;
        
        // Limit display to prevent excessive output that could cause RangeError
        const MAX_DISPLAY_CHARS: usize = 1000;
        let display_limit = typed_text.len().min(MAX_DISPLAY_CHARS);
        
        // Display typed characters with error highlighting
        for (i, ch) in typed_text.chars().take(display_limit).enumerate() {
            if i < target_chars.len() {
                if ch == target_chars[i] {
                    queue!(stdout, SetForegroundColor(Color::Green), Print(ch))?;
                } else {
                    queue!(stdout, SetForegroundColor(Color::Red), Print(ch))?;
                }
            }
        }
        
        // Show cursor position
        if position < target_chars.len() {
            queue!(stdout, SetForegroundColor(Color::Yellow), Print('|'))?;
        }
        
        queue!(stdout, ResetColor)?;
        stdout.flush()?;
        
        Ok(())
    }
    
    fn display_results(&self, result: &ExerciseResult) -> Result<(), Box<dyn std::error::Error>> {
        // Clear screen with direct ANSI codes
        print!("\x1B[2J\x1B[1;1H");
        
        println!();
        println!("{}", center_text("=== RESULTS ==="));
        println!();
        println!("{}", center_text(&format!("Characters typed: {}", result.total_chars)));
        println!("{}", center_text(&format!("Correct: {}", result.correct_chars)));
        println!("{}", center_text(&format!("Errors: {}", result.errors)));
        println!("{}", center_text(&format!("Accuracy: {:.1}%", 100.0 - result.error_rate)));
        println!("{}", center_text(&format!("Speed: {:.1} WPM", result.wpm)));
        println!("{}", center_text(&format!("Time: {:.1}s", result.duration.as_secs_f32())));
        println!();
        println!("{}", center_text("Press any key to continue..."));
        
        read()?;
        Ok(())
    }
}

/// Speed test exercise - timed typing with WPM calculation
#[derive(Debug, Clone)]
pub struct SpeedTestExercise {
    pub text: String,
    pub practice_only: bool,
    pub time_limit: Option<Duration>,
}

impl SpeedTestExercise {
    pub fn new(text: String, practice_only: bool, time_limit: Option<Duration>) -> Self {
        Self { 
            text, 
            practice_only,
            time_limit 
        }
    }
    
    /// Execute speed test with timer
    pub fn execute(&self) -> Result<ExerciseOutcome, Box<dyn std::error::Error>> {
        let mut stdout = stdout();
        let mut tracker = PerformanceTracker::new();
        let target_chars: Vec<char> = self.text.chars().collect();
        let mut position = 0;
        let mut typed_text = String::new();
        
        // Clear screen with direct ANSI codes
        print!("\x1B[2J\x1B[1;1H");
        
        println!();
        println!("{}", center_text(&format!("=== {} ===", 
            if self.practice_only { "SPEED TEST PRACTICE" } else { "SPEED TEST" })));
        
        if let Some(time_limit) = self.time_limit {
            println!("{}", center_text(&format!("Time limit: {} seconds", time_limit.as_secs())));
        }
        println!();
        println!("{}", center_text("Type as fast and accurately as possible. Press ESC to quit."));
        println!();
        
        // Display target text (truncated to prevent excessive output)
        const MAX_TARGET_DISPLAY: usize = 500;
        let display_text = if self.text.len() > MAX_TARGET_DISPLAY {
            format!("{}...", &self.text[..MAX_TARGET_DISPLAY])
        } else {
            self.text.clone()
        };
        
        println!("{}", center_text("Text to type:"));
        println!("  {}", display_text);
        println!();
        println!("{}", center_text("Press any key to start..."));
        stdout.flush()?;
        
        // Wait for start signal
        read()?;
        
        let start_time = Instant::now();
        
        loop {
            // Check time limit
            if let Some(time_limit) = self.time_limit {
                if start_time.elapsed() >= time_limit {
                    println!("\nTime's up!");
                    break;
                }
            }
            
            match read()? {
                Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                    return Ok(ExerciseOutcome::Quit);
                },
                Event::Key(KeyEvent { code: KeyCode::Char(ch), .. }) => {
                    if position < target_chars.len() {
                        let expected = target_chars[position];
                        typed_text.push(ch);
                        
                        if ch == expected {
                            tracker.record_correct_char();
                        } else {
                            tracker.record_error();
                        }
                        
                        position += 1;
                        
                        // Display progress after each character
                        self.display_speed_progress(&mut stdout, &typed_text, &target_chars, position, start_time)?;
                        
                        // Check if test is complete
                        if position >= target_chars.len() {
                            break;
                        }
                    }
                },
                Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
                    if position > 0 && !typed_text.is_empty() {
                        position -= 1;  
                        typed_text.pop();
                        tracker.record_backspace();
                        self.display_speed_progress(&mut stdout, &typed_text, &target_chars, position, start_time)?;
                    }
                },
                _ => continue,
            }
        }
        
        let duration = start_time.elapsed();
        tracker.set_duration(duration);
        
        let result = ExerciseResult {
            total_chars: position,
            correct_chars: tracker.correct_chars(),
            errors: tracker.errors(),
            duration,
            wpm: tracker.words_per_minute(),
            error_rate: tracker.error_rate(),
        };
        
        // Display final results
        self.display_speed_results(&result)?;
        
        Ok(ExerciseOutcome::Completed(result))
    }
    
    fn display_speed_progress(
        &self,
        stdout: &mut std::io::Stdout,
        typed_text: &str,
        target_chars: &[char],
        position: usize,
        start_time: Instant
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Move to progress area
        execute!(stdout, cursor::MoveTo(0, 5))?;
        
        // Display timer
        let elapsed = start_time.elapsed().as_secs_f32();
        println!("Time: {:.1}s", elapsed);
        
        // Calculate real-time WPM
        if elapsed > 0.0 {
            let chars_per_minute = (position as f32 / elapsed) * 60.0;
            let wpm = chars_per_minute / 5.0; // Standard: 5 chars = 1 word
            println!("Current WPM: {:.1}", wpm);
        }
        
        println!("Progress: {}/{} characters\n", position, target_chars.len());
        
        // Limit display to prevent excessive output that could cause RangeError
        const MAX_DISPLAY_CHARS: usize = 1000;
        let display_limit = typed_text.len().min(MAX_DISPLAY_CHARS);
        
        // Display typed text with highlighting
        print!("Typed: ");
        for (i, ch) in typed_text.chars().take(display_limit).enumerate() {
            if i < target_chars.len() {
                if ch == target_chars[i] {
                    queue!(stdout, SetForegroundColor(Color::Green), Print(ch))?;
                } else {
                    queue!(stdout, SetForegroundColor(Color::Red), Print(ch))?;
                }
            }
        }
        
        // Add indication if text was truncated
        if typed_text.len() > MAX_DISPLAY_CHARS {
            queue!(stdout, SetForegroundColor(Color::Blue), Print("..."))?;
        }
        
        queue!(stdout, ResetColor)?;
        stdout.flush()?;
        
        Ok(())
    }
    
    fn display_speed_results(&self, result: &ExerciseResult) -> Result<(), Box<dyn std::error::Error>> {
        // Clear screen with direct ANSI codes
        print!("\x1B[2J\x1B[1;1H");
        
        println!();
        println!("{}", center_text("=== SPEED TEST RESULTS ==="));
        println!();
        println!("{}", center_text(&format!("Characters typed: {}", result.total_chars)));
        println!("{}", center_text(&format!("Correct characters: {}", result.correct_chars)));
        println!("{}", center_text(&format!("Errors: {}", result.errors)));
        println!("{}", center_text(&format!("Accuracy: {:.1}%", 100.0 - result.error_rate)));
        println!("{}", center_text(&format!("Speed: {:.1} WPM", result.wpm)));
        println!("{}", center_text(&format!("Time: {:.1} seconds", result.duration.as_secs_f32())));
        println!();
        
        // Grade the performance
        let grade = if result.wpm >= 40.0 && result.error_rate <= 5.0 {
            "Excellent typing!"
        } else if result.wpm >= 25.0 && result.error_rate <= 10.0 {
            "Good job!"
        } else {
            "Keep practicing!"
        };
        println!("{}", center_text(grade));
        println!();
        println!("{}", center_text("Press any key to continue..."));
        
        read()?;
        Ok(())
    }
}