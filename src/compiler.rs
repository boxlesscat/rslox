use crate::chunk::Chunk;
use crate::chunk::OpCode;
use crate::scanner::Scanner;
use crate::scanner::Token;
use crate::scanner::TokenType;
use crate::value::Value;
use std::mem;

#[derive(Default)]
struct Compiler {
    chunk: Chunk,
}

#[derive(Default)]
pub struct Parser<'a> {
    current: Token<'a>,
    previous: Token<'a>,
    had_error: bool,
    panic_mode: bool,
    compiler: Compiler,
    scanner: Scanner<'a>,
}

#[repr(u8)]
#[derive(PartialEq, PartialOrd)]
pub enum Precedence {
    None,
    Assignment,  // =
    Or,          // or
    And,         // and
    Equality,    // == !=
    Comparision, // < > <= >=
    Term,        // + 1
    Factor,      // * /
    Unary,       // ! -
    Call,        // . ()
    Primary,
}

impl std::ops::Add<u8> for Precedence {
    type Output = Self;

    fn add(self, rhs: u8) -> Self::Output {
        unsafe { mem::transmute((self as u8 + rhs) % 11) }
    }
}

type ParseFn<'a> = fn(&mut Parser<'a>);

struct ParseRule<'a> {
    prefix: Option<ParseFn<'a>>,
    infix: Option<ParseFn<'a>>,
    precedence: Precedence,
}

impl<'a> ParseRule<'a> {
    fn new(
        prefix: Option<ParseFn<'a>>,
        infix: Option<ParseFn<'a>>,
        precedence: Precedence,
    ) -> Self {
        Self {
            prefix,
            infix,
            precedence,
        }
    }

    fn get_rule(token_type: TokenType) -> Self {
        use TokenType::*;
        match token_type {
            LeftParen       => Self::new(Some(Parser::grouping),    None,                       Precedence::None),
            RightParen      => Self::new(None,                      None,                       Precedence::None),
            LeftBrace       => Self::new(None,                      None,                       Precedence::None),
            RightBrace      => Self::new(None,                      None,                       Precedence::None),
            Comma           => Self::new(None,                      None,                       Precedence::None),
            Dot             => Self::new(None,                      None,                       Precedence::None),
            Minus           => Self::new(Some(Parser::unary),       Some(Parser::binary),       Precedence::Term),
            Plus            => Self::new(None,                      Some(Parser::binary),       Precedence::Term),
            Semicolon       => Self::new(None,                      None,                       Precedence::None),
            Slash           => Self::new(None,                      Some(Parser::binary),       Precedence::Factor),
            Star            => Self::new(None,                      Some(Parser::binary),       Precedence::Factor),
            Bang            => Self::new(Some(Parser::unary),       None,                       Precedence::None),
            BangEqual       => Self::new(None,                      Some(Parser::binary),       Precedence::Equality),
            Equal           => Self::new(None,                      None,                       Precedence::None),
            EqualEqual      => Self::new(None,                      Some(Parser::binary),       Precedence::Equality),
            Greater         => Self::new(None,                      Some(Parser::binary),       Precedence::Comparision),
            GreaterEqual    => Self::new(None,                      Some(Parser::binary),       Precedence::Comparision),
            Less            => Self::new(None,                      Some(Parser::binary),       Precedence::Comparision),
            LessEqual       => Self::new(None,                      Some(Parser::binary),       Precedence::Comparision),
            Idenitifier     => Self::new(None,                      None,                       Precedence::None),
            String          => Self::new(None,                      None,                       Precedence::None),
            Number          => Self::new(Some(Parser::number),      None,                       Precedence::None),
            And             => Self::new(None,                      None,                       Precedence::None),
            Class           => Self::new(None,                      None,                       Precedence::None),
            Else            => Self::new(None,                      None,                       Precedence::None),
            False           => Self::new(Some(Parser::literal),     None,                       Precedence::None),
            For             => Self::new(None,                      None,                       Precedence::None),
            Fun             => Self::new(None,                      None,                       Precedence::None),
            If              => Self::new(None,                      None,                       Precedence::None),
            Nil             => Self::new(Some(Parser::literal),     None,                       Precedence::None),
            Or              => Self::new(None,                      None,                       Precedence::None),
            Print           => Self::new(None,                      None,                       Precedence::None),
            Return          => Self::new(None,                      None,                       Precedence::None),
            Super           => Self::new(None,                      None,                       Precedence::None),
            This            => Self::new(None,                      None,                       Precedence::None),
            True            => Self::new(Some(Parser::literal),     None,                       Precedence::None),
            Var             => Self::new(None,                      None,                       Precedence::None),
            While           => Self::new(None,                      None,                       Precedence::None),
            Error           => Self::new(None,                      None,                       Precedence::None),
            EOF             => Self::new(None,                      None,                       Precedence::None),
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            scanner: Scanner::new(source),
            ..Default::default()
        }
    }

    pub fn compile(&mut self) -> Result<Chunk, ()> {
        self.advance();
        self.expression();
        self.consume(TokenType::EOF, "Expect end of expression.");
        self.end_compiler();
        if self.had_error {
            Err(())
        } else {
            Ok(mem::take(&mut self.compiler.chunk))
        }
    }

    fn advance(&mut self) {
        self.previous = mem::take(&mut self.current);
        loop {
            self.current = self.scanner.scan_token();
            if self.current.token_type != TokenType::Error {
                break;
            }
            self.error_at_current(self.current.value);
        }
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.current.clone(), message);
    }

    fn error(&mut self, message: &str) {
        self.error_at(self.previous.clone(), message);
    }

    fn error_at(&mut self, token: Token, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        eprint!("[line {}] Error", token.line);
        if token.token_type == TokenType::EOF {
            eprint!(" at end");
        } else if token.token_type == TokenType::Error {
        } else {
            eprint!(" at '{}'", token.value);
        }
        eprintln!(": {message}");
        self.had_error = true;
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.current.token_type == token_type {
            self.advance();
        } else {
            self.error_at_current(message);
        }
    }

    fn emit_byte(&mut self, op_code: OpCode) {
        self.compiler.chunk.write(op_code, self.previous.line);
    }

    fn emit_bytes(&mut self, op_code1: OpCode, op_code2: OpCode) {
        self.compiler.chunk.write(op_code1, self.previous.line);
        self.compiler.chunk.write(op_code2, self.previous.line);
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        #[cfg(feature = "debug_print_code")]
        {
            use crate::debug::Disassembler;
            if !self.had_error {
                let disassembler = Disassembler::new(&self.compiler.chunk);
                disassembler.disassemble_chunk("code");
            }
        }
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return);
    }

    fn expression(&mut self) {
        self.parser_precendence(Precedence::Assignment);
    }

    fn number(&mut self) {
        let value = Value::Number(self.previous.value.parse().unwrap());
        self.emit_constant(value);
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self) {
        let operator_type = self.previous.token_type;
        self.parser_precendence(Precedence::Unary);
        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate),
            TokenType::Bang => self.emit_byte(OpCode::Not),
            _ => (),
        }
    }

    fn binary(&mut self) {
        let operator_type = self.previous.token_type;
        let rule = ParseRule::get_rule(operator_type);
        self.parser_precendence(rule.precedence + 1);
        use TokenType::*;
        match operator_type {
            BangEqual       => self.emit_bytes(OpCode::Equal, OpCode::Not),
            EqualEqual      => self.emit_byte (OpCode::Equal),
            Greater         => self.emit_byte (OpCode::Greater),
            GreaterEqual    => self.emit_bytes(OpCode::Less, OpCode::Not),
            Less            => self.emit_byte (OpCode::Less),
            LessEqual       => self.emit_bytes(OpCode::Greater, OpCode::Not),
            Plus            => self.emit_byte (OpCode::Add),
            Minus           => self.emit_byte (OpCode::Subtract),
            Star            => self.emit_byte (OpCode::Multiply),
            Slash           => self.emit_byte (OpCode::Divide),
            _               => (),
        }
    }

    fn literal(&mut self) {
        match self.previous.token_type {
            TokenType::False => self.emit_byte(OpCode::False),
            TokenType::Nil =>   self.emit_byte(OpCode::Nil),
            TokenType::True =>  self.emit_byte(OpCode::True),
            _ => (),
        }
    }

    fn parser_precendence(&mut self, precendence: Precedence) {
        self.advance();
        let prefix_rule = ParseRule::get_rule(self.previous.token_type).prefix;
        match prefix_rule {
            None => return self.error("Expect expression."),
            Some(rule) => rule(self),
        }
        while precendence <= ParseRule::get_rule(self.current.token_type).precedence {
            self.advance();
            let infix_rule = ParseRule::get_rule(self.previous.token_type).infix.unwrap();
            infix_rule(self);
        }
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.compiler
            .chunk
            .write(OpCode::Constant(constant), self.previous.line);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.compiler.chunk.add_constant(value);
        if constant > u8::MAX as usize {
            self.error("Too many constants in one chunk.");
            0
        } else {
            constant as u8
        }
    }
}
