use crate::vm::InterpretResult;
use crate::vm::VM;

use std::io;
use std::io::Write;
use std::process::exit;

pub mod chunk;
pub mod compiler;
pub mod debug;
pub mod scanner;
pub mod value;
pub mod vm;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        repl();
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        eprintln!("Usage: rslox [path]");
        exit(64);
    }
}

fn repl() {
    let mut vm = VM::new();
    let mut line = String::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        std::io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");
        vm.interpret(&line);
        line.clear();
    }
}

fn run_file(path: &str) {
    let mut vm = VM::new();
    let source = std::fs::read_to_string(path).expect("Could not open file.");
    match vm.interpret(&source) {
        InterpretResult::CompileError   => exit(65),
        InterpretResult::RuntimeError   => exit(70),
        InterpretResult::Ok             => exit(0),
    }
}
