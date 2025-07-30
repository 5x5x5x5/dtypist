# Feature Plan: Source Text File Support

## Overview
Add the ability to read external text files as target content for typing exercises, extending beyond the current inline text approach.

## Current State Analysis

### Text Handling
- Script commands like `D:` (drill) and `S:` (speed test) currently store text directly inline in lesson files
- Parser: `commands.rs:164-177` handles `D:` and `S:` commands by storing the `text` field as a String
- Exercises: `exercises.rs` uses the text directly from command structures
- CLI: `main.rs:38-41` already accepts lesson files but no source text option

### Integration Points Identified
- `commands.rs:164-177` - Command parsing for drill and speed test commands
- `exercises.rs:DrillExercise::execute()` and `SpeedTestExercise::execute()` - Exercise execution
- `parser.rs:parse_script_file()` - Script file parsing
- `main.rs:create_cli()` - Command-line interface

## Implementation Strategy

### 1. New Command Syntax
Add new command variants that reference external files:
- `DF:filename.txt` - Drill with file content  
- `SF:filename.txt` - Speed test with file content
- `df:filename.txt` - Practice drill with file content
- `sf:filename.txt` - Practice speed test with file content

**Rationale**: Uses existing command prefixes with 'F' suffix to indicate "File" variant, maintaining consistency with current syntax patterns.

### 2. Command Structure Updates
Extend `Command` enum in `commands.rs` to support file references:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TextSource {
    Inline(String),
    File(String), // file path
}

// Update existing commands:
Drill { 
    text_source: TextSource,
    practice_only: bool,
},
SpeedTest { 
    text_source: TextSource,
    practice_only: bool,
}
```

### 3. Parser Integration
- Modify `commands.rs:parse_line()` to detect file-based commands (`DF:`, `SF:`, etc.)
- Add new command character constants:
  ```rust
  pub const DRILL_FILE: char = 'F'; // Following 'D'
  pub const SPEEDTEST_FILE: char = 'F'; // Following 'S'
  ```
- Handle two-character command prefixes (`DF`, `SF`, `df`, `sf`)

### 4. Exercise Execution Updates
- Update `exercises.rs` drill/speed test implementations to resolve file content
- Add file reading logic with proper error handling:
  ```rust
  fn resolve_text_content(source: &TextSource, base_dir: Option<&Path>) -> Result<String, Error>
  ```
- Cache file contents to avoid repeated I/O during exercise execution

### 5. CLI Enhancement
Add optional flag for specifying source text directory:
```bash
gtypist lesson.typ --source-dir ./texts/
gtypist lesson.typ -t ./texts/
```

Update `main.rs:create_cli()`:
```rust
.arg(Arg::with_name("source-dir")
    .short("t")
    .long("source-dir")
    .value_name("DIR")
    .help("Directory containing source text files")
    .takes_value(true))
```

### 6. Error Handling Strategy
Comprehensive error handling for:
- **File not found**: Clear error message with file path
- **Permission denied**: Helpful message about file permissions
- **Invalid UTF-8 encoding**: Suggest file encoding issues
- **Empty files**: Warning but allow continuation
- **Files too large**: Memory protection with configurable limits
- **Relative vs absolute paths**: Proper path resolution

### 7. Text Processing Features
- **Line ending normalization**: Convert Windows/Mac line endings to Unix
- **Unicode handling**: Proper UTF-8 support with BOM detection
- **Whitespace handling**: Configurable trimming and normalization
- **Chunk processing**: Support for large files with memory-efficient reading
- **File format support**: Plain text files (.txt) initially, extensible for other formats

### 8. Implementation Phases

#### Phase 1: Core Infrastructure
1. Add `TextSource` enum and update `Command` structures
2. Extend parser to recognize file-based commands
3. Implement basic file reading in exercises

#### Phase 2: CLI and Path Handling  
1. Add `--source-dir` command-line option
2. Implement path resolution logic (relative/absolute paths)
3. Add comprehensive error handling

#### Phase 3: Text Processing
1. Add text normalization features
2. Implement file caching for performance
3. Add support for large file handling

#### Phase 4: Testing and Documentation
1. Create comprehensive test suite
2. Add example source text files
3. Update documentation and help text

## Backward Compatibility
- Existing lesson files continue to work unchanged
- Current `D:` and `S:` commands remain fully functional
- No breaking changes to existing API or file formats

## Benefits
- **Modularity**: Separate lesson logic from content text
- **Reusability**: Same text files can be used across multiple lessons
- **Scalability**: Support large texts without bloating lesson files
- **Flexibility**: Easy content updates without modifying lesson scripts
- **Organization**: Clean separation of concerns between structure and content

## Example Usage

### Lesson File (lesson.typ)
```
*:START
T:Welcome to the typing lesson
DF:sample_text.txt
G:END
*:END
X:
```

### Source Text File (sample_text.txt)
```
The quick brown fox jumps over the lazy dog.
This pangram contains every letter of the alphabet.
```

### Command Line Usage
```bash
# Use default directory (same as lesson file)
gtypist lesson.typ

# Specify custom source directory
gtypist lesson.typ --source-dir ./practice_texts/

# Start at specific label with source directory
gtypist lesson.typ -l START --source-dir ./texts/
```

## Risk Assessment
- **Low risk**: Implementation builds on existing, stable architecture
- **No breaking changes**: Fully backward compatible
- **Incremental deployment**: Can be implemented and tested in phases
- **Rollback capability**: Feature can be disabled without affecting core functionality

## Success Criteria
1. All existing lesson files continue to work without modification
2. New file-based commands successfully load external text content
3. Proper error handling for all file-related issues
4. Performance impact is minimal (file caching prevents repeated I/O)
5. Clear, helpful error messages guide users to resolve issues
6. Documentation and examples enable easy adoption