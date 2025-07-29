use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gtypist_rs::{Script, Executor};
use std::fs;
use tempfile::NamedTempFile;
use std::io::Write;

fn benchmark_script_parsing(c: &mut Criterion) {
    let script_content = fs::read_to_string("lessons/demo.typ")
        .unwrap_or_else(|_| {
            // Fallback if demo.typ not available
            r#"
*:START
T:Welcome to the tutorial
D:Type this sample text for practice
S:Speed test text here
G:END
*:END
X:
"#.to_string()
        });
    
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(script_content.as_bytes()).unwrap();
    let path = temp_file.path().to_str().unwrap().to_string();
    
    c.bench_function("parse_script", |b| {
        b.iter(|| Script::from_file(black_box(&path)))
    });
}

fn benchmark_command_execution(c: &mut Criterion) {
    let script_content = r#"
*:START
T:Tutorial text
I:Instruction text
B:Banner text
G:END
*:END
X:
"#;
    
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(script_content.as_bytes()).unwrap();
    
    let script = Script::from_file(temp_file.path().to_str().unwrap()).unwrap();
    
    c.bench_function("execute_commands", |b| {
        b.iter(|| {
            let mut executor = Executor::new(script.clone());
            while !executor.script.is_finished() {
                let _ = executor.execute_next();
            }
        })
    });
}

criterion_group!(benches, benchmark_script_parsing, benchmark_command_execution);
criterion_main!(benches);