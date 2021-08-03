// Crafting Interpreters.

use std::{
    io::{BufRead, Read},
    path::Path,
};

mod scanner;

use scanner::Scanner;

trait Reporter {
    fn report(&mut self, chunk_name: &str, line: usize, r#where: &str, message: &str);
}

struct StderrReporter {}

impl StderrReporter {
    fn new() -> Self {
        Self {}
    }
}

impl Reporter for StderrReporter {
    fn report(&mut self, chunk_name: &str, line: usize, r#where: &str, message: &str) {
        eprintln!("error{}: {}", r#where, message);
        eprintln!("\t{}:{}", chunk_name, line);
    }
}

pub struct Lax<'chunk, 'report> {
    reporter: &'report mut dyn Reporter,
    chunk_name: &'chunk str,
    had_error: bool,
}

impl<'chunk, 'report> Lax<'chunk, 'report> {
    fn new(reporter: &'report mut dyn Reporter, chunk_name: &'chunk str) -> Self {
        Self {
            reporter,
            chunk_name,
            had_error: false,
        }
    }

    fn had_error(&self) -> bool {
        self.had_error
    }

    fn error(&mut self, line: usize, message: &str) {
        self.reporter.report(self.chunk_name, line, "", message);
        self.had_error = true;
    }
}

impl<'chunk, 'report> Lax<'chunk, 'report> {
    fn run_file(path: &Path) -> std::io::Result<()> {
        let mut file = std::fs::File::open(path)?;
        let mut source = String::new();
        file.read_to_string(&mut source)?;
        Lax::run(path.to_string_lossy().as_ref(), &source);
        Ok(())
    }

    fn run_repl() -> std::io::Result<()> {
        for line in std::io::stdin().lock().lines() {
            let source = line?;
            Lax::run("repl", &source);
        }

        Ok(())
    }

    fn run(chunk_name: &str, source: &str) {
        let mut reporter = StderrReporter::new();
        let mut lax = Lax::new(&mut reporter, chunk_name);
        let mut scanner = Scanner::new(&mut lax, source.as_bytes());
        let lexemes = scanner.scan_tokens();
        println!("{:?}", lexemes);
    }
}

fn main() {
    Lax::run_repl().unwrap()
}
