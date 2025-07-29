//! Exercise execution engine
//! 
//! Implements the three core exercise types: Tutorial, Drill, and Speed Test
//! This replicates the functionality from the C implementation's do_tutorial, 
//! do_drill, and do_speedtest functions.

use std::time::{Duration, Instant};
use crossterm::{
    cursor, execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
};
use std::io::{stdout, Write};
use crate::performance::{PerformanceTracker, ExerciseResult};

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
        
        // Clear screen and display text
        execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        
        // Display the tutorial text
        println!("{}", self.text);
        println!("\nPress SPACE to continue, ESC to quit...");
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
        
        // Clear screen and display exercise setup
        execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        
        println!("=== {} ===", if self.practice_only { "DRILL PRACTICE" } else { "DRILL" });
        println!("Type the following text. Press ESC to quit, Ctrl+R to retry.\n");
        
        // Display target text
        println!("Target:");
        println!("{}\n", self.text);
        
        println!("Your typing:");
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
        
        // Display typed characters with error highlighting
        for (i, ch) in typed_text.chars().enumerate() {
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
        println!("\n=== RESULTS ===");
        println!("Characters typed: {}", result.total_chars);
        println!("Correct: {}", result.correct_chars);
        println!("Errors: {}", result.errors);
        println!("Accuracy: {:.1}%", 100.0 - result.error_rate);
        println!("Speed: {:.1} WPM", result.wpm);
        println!("Time: {:.1}s", result.duration.as_secs_f32());
        println!("\nPress any key to continue...");
        
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
        
        // Clear screen and display exercise setup
        execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        
        println!("=== {} ===", if self.practice_only { "SPEED TEST PRACTICE" } else { "SPEED TEST" });
        if let Some(time_limit) = self.time_limit {
            println!("Time limit: {} seconds", time_limit.as_secs());
        }
        println!("Type as fast and accurately as possible. Press ESC to quit.\n");
        
        // Display target text  
        println!("Text to type:");
        println!("{}\n", self.text);
        
        println!("Press any key to start...");
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
        
        // Display typed text with highlighting
        print!("Typed: ");
        for (i, ch) in typed_text.chars().enumerate() {
            if i < target_chars.len() {
                if ch == target_chars[i] {
                    queue!(stdout, SetForegroundColor(Color::Green), Print(ch))?;
                } else {
                    queue!(stdout, SetForegroundColor(Color::Red), Print(ch))?;
                }
            }
        }
        
        queue!(stdout, ResetColor)?;
        stdout.flush()?;
        
        Ok(())
    }
    
    fn display_speed_results(&self, result: &ExerciseResult) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n=== SPEED TEST RESULTS ===");
        println!("Characters typed: {}", result.total_chars);
        println!("Correct characters: {}", result.correct_chars);
        println!("Errors: {}", result.errors);
        println!("Accuracy: {:.1}%", 100.0 - result.error_rate);
        println!("Speed: {:.1} WPM", result.wpm);
        println!("Time: {:.1} seconds", result.duration.as_secs_f32());
        
        // Grade the performance
        if result.wpm >= 40.0 && result.error_rate <= 5.0 {
            println!("Excellent typing!");
        } else if result.wpm >= 25.0 && result.error_rate <= 10.0 {
            println!("Good job!");
        } else {
            println!("Keep practicing!");
        }
        
        println!("\nPress any key to continue...");
        read()?;
        Ok(())
    }
}