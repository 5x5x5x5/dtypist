//! Performance tracking and statistics
//! 
//! Handles WPM/CPM calculations, error rates, and typing statistics.
//! Replicates the functionality from speedbox.c in the C implementation.

use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Performance tracking for typing exercises
#[derive(Debug, Clone)]
pub struct PerformanceTracker {
    correct_chars: usize,
    errors: usize,
    backspaces: usize,
    start_time: Option<Instant>,
    duration: Option<Duration>,
}

impl PerformanceTracker {
    /// Create a new performance tracker
    pub fn new() -> Self {
        Self {
            correct_chars: 0,
            errors: 0,
            backspaces: 0,
            start_time: None,
            duration: None,
        }
    }
    
    /// Start timing (automatically called on first character)
    pub fn start(&mut self) {
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }
    }
    
    /// Record a correctly typed character
    pub fn record_correct_char(&mut self) {
        self.start();
        self.correct_chars += 1;
    }
    
    /// Record an error (incorrect character)
    pub fn record_error(&mut self) {
        self.start();
        self.errors += 1;
    }
    
    /// Record a backspace operation
    pub fn record_backspace(&mut self) {
        self.backspaces += 1;
    }
    
    /// Set the final duration (for completed exercises)
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = Some(duration);
    }
    
    /// Get elapsed time since start
    pub fn elapsed(&self) -> Duration {
        match (self.start_time, self.duration) {
            (_, Some(duration)) => duration,
            (Some(start), None) => start.elapsed(),
            (None, None) => Duration::from_secs(0),
        }
    }
    
    /// Calculate words per minute (WPM)
    /// Uses the standard convention: 5 characters = 1 word
    pub fn words_per_minute(&self) -> f32 {
        let elapsed_minutes = self.elapsed().as_secs_f32() / 60.0;
        if elapsed_minutes <= 0.0 {
            return 0.0;
        }
        
        let total_words = self.correct_chars as f32 / 5.0;
        total_words / elapsed_minutes
    }
    
    /// Calculate characters per minute (CPM)
    pub fn characters_per_minute(&self) -> f32 {
        let elapsed_minutes = self.elapsed().as_secs_f32() / 60.0;
        if elapsed_minutes <= 0.0 {
            return 0.0;
        }
        
        self.correct_chars as f32 / elapsed_minutes
    }
    
    /// Calculate error rate as percentage
    pub fn error_rate(&self) -> f32 {
        let total_chars = self.correct_chars + self.errors;
        if total_chars == 0 {
            return 0.0;
        }
        
        (self.errors as f32 / total_chars as f32) * 100.0
    }
    
    /// Calculate accuracy as percentage
    pub fn accuracy(&self) -> f32 {
        100.0 - self.error_rate()
    }
    
    /// Get correct character count
    pub fn correct_chars(&self) -> usize {
        self.correct_chars
    }
    
    /// Get error count
    pub fn errors(&self) -> usize {
        self.errors
    }
    
    /// Get backspace count  
    pub fn backspaces(&self) -> usize {
        self.backspaces
    }
    
    /// Get total keystrokes (including errors and backspaces)
    pub fn total_keystrokes(&self) -> usize {
        self.correct_chars + self.errors + self.backspaces
    }
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Results from a completed exercise
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExerciseResult {
    pub total_chars: usize,
    pub correct_chars: usize,
    pub errors: usize,
    pub duration: Duration,
    pub wpm: f32,
    pub error_rate: f32,
}

impl ExerciseResult {
    /// Create a default result (for tutorials)
    pub fn default() -> Self {
        Self {
            total_chars: 0,
            correct_chars: 0,
            errors: 0,
            duration: Duration::from_secs(0),
            wpm: 0.0,
            error_rate: 0.0,
        }
    }
    
    /// Calculate accuracy percentage
    pub fn accuracy(&self) -> f32 {
        100.0 - self.error_rate
    }
    
    /// Calculate characters per minute
    pub fn cpm(&self) -> f32 {
        let elapsed_minutes = self.duration.as_secs_f32() / 60.0;
        if elapsed_minutes <= 0.0 {
            return 0.0;
        }
        
        self.correct_chars as f32 / elapsed_minutes
    }
    
    /// Grade the performance
    pub fn grade(&self) -> PerformanceGrade {
        match (self.wpm, self.error_rate) {
            (wpm, err) if wpm >= 60.0 && err <= 3.0 => PerformanceGrade::Excellent,
            (wpm, err) if wpm >= 40.0 && err <= 5.0 => PerformanceGrade::Good,
            (wpm, err) if wpm >= 25.0 && err <= 10.0 => PerformanceGrade::Fair,
            _ => PerformanceGrade::NeedsImprovement,
        }
    }
}

/// Performance grading levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PerformanceGrade {
    Excellent,
    Good,
    Fair,
    NeedsImprovement,
}

impl PerformanceGrade {
    /// Get a descriptive message for the grade
    pub fn message(&self) -> &'static str {
        match self {
            PerformanceGrade::Excellent => "Excellent typing! You're a speed demon!",
            PerformanceGrade::Good => "Good job! Your typing skills are solid.",
            PerformanceGrade::Fair => "Not bad! Keep practicing to improve.",
            PerformanceGrade::NeedsImprovement => "Keep practicing - you'll get there!",
        }
    }
}

/// Persistent speed records and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedRecords {
    pub best_wpm: f32,
    pub best_accuracy: f32,
    pub total_exercises: usize,
    pub total_time_practiced: Duration,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl SpeedRecords {
    /// Create new empty records
    pub fn new() -> Self {
        Self {
            best_wpm: 0.0,
            best_accuracy: 0.0,
            total_exercises: 0,
            total_time_practiced: Duration::from_secs(0),
            last_updated: chrono::Utc::now(),
        }
    }
    
    /// Update records with a new exercise result
    pub fn update(&mut self, result: &ExerciseResult) {
        if result.wpm > self.best_wpm {
            self.best_wpm = result.wpm;
        }
        
        let accuracy = result.accuracy();
        if accuracy > self.best_accuracy {
            self.best_accuracy = accuracy;
        }
        
        self.total_exercises += 1;
        self.total_time_practiced += result.duration;
        self.last_updated = chrono::Utc::now();
    }
    
    /// Get average practice time per exercise
    pub fn average_exercise_time(&self) -> Duration {
        if self.total_exercises == 0 {
            return Duration::from_secs(0);
        }
        
        self.total_time_practiced / self.total_exercises as u32
    }
}

impl Default for SpeedRecords {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_wpm_calculation() {
        let mut tracker = PerformanceTracker::new();
        
        // Type 50 characters (10 words) in 1 minute
        for _ in 0..50 {
            tracker.record_correct_char();
        }
        tracker.set_duration(Duration::from_secs(60));
        
        assert_eq!(tracker.words_per_minute(), 10.0);
    }
    
    #[test]
    fn test_error_rate() {
        let mut tracker = PerformanceTracker::new();
        
        // 8 correct, 2 errors = 20% error rate
        for _ in 0..8 {
            tracker.record_correct_char();
        }
        for _ in 0..2 {
            tracker.record_error();
        }
        
        assert_eq!(tracker.error_rate(), 20.0);
        assert_eq!(tracker.accuracy(), 80.0);
    }
    
    #[test]
    fn test_performance_grades() {
        let excellent = ExerciseResult {
            total_chars: 100,
            correct_chars: 98,
            errors: 2,
            duration: Duration::from_secs(60),
            wpm: 60.0,
            error_rate: 2.0,
        };
        
        assert_eq!(excellent.grade(), PerformanceGrade::Excellent);
        
        let needs_improvement = ExerciseResult {
            total_chars: 50,
            correct_chars: 40,
            errors: 10,
            duration: Duration::from_secs(120),
            wpm: 15.0,
            error_rate: 20.0,
        };
        
        assert_eq!(needs_improvement.grade(), PerformanceGrade::NeedsImprovement);
    }
}