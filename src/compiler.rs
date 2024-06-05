use crate::scanner::Scanner;
use crate::scanner::TokenType;

pub fn compile(source: &str) {
    let mut scanner = Scanner::new(&source);
    let mut token;
    let mut line = usize::MAX;
    loop {
        token = scanner.scan_token();
        if token.line != line {
            print!("{:4} ", token.line);
            line = token.line;
        } else {
            print!("   | ");
        }
        println!("{:#?} '{}'", token.token_type, token.value);
        if token.token_type == TokenType::EOF {
            break;
        }
    }
}