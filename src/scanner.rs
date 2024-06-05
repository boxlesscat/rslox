#[derive(Debug)]
pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: usize,
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Idenitifier,
    String,
    Number,
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Error,
    EOF,
}

pub struct Token<'a> {
    pub token_type: TokenType,
    pub value: &'a str,
    pub line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: &source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token {
        use TokenType::*;
        self.start = self.current;
        if self.is_at_end() {
            return self.make_token(EOF);
        }
        self.current += 1;
        self.error_token("Unexpected Character")
    }

    fn is_at_end(&self) -> bool {
        self.current == self.source.len()
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        Token {
            token_type,
            value: &self.source[self.start..self.current],
            line: self.line,
        }
    }

    fn error_token(&self, message: &'a str) -> Token {
        Token {
            token_type: TokenType::Error,
            value: &message,
            line: self.line,
        }
    }

}
