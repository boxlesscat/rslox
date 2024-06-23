use crate::chunk::Chunk;
use crate::chunk::OpCode;
use crate::scanner::Scanner;
use crate::scanner::Token;
use crate::scanner::TokenType;
use crate::value::Value;
use std::mem;

#[derive(Default)]
struct Compiler <'a> {
    chunk: Chunk,
    locals: Vec<Local<'a>>,
    scope_depth: i32,
}

#[derive(Default)]
struct Local <'a> {
    name: Token<'a>,
    depth: i32,
}

#[derive(Default)]
pub struct Parser<'a> {
    current: Token<'a>,
    previous: Token<'a>,
    had_error: bool,
    panic_mode: bool,
    compiler: Compiler<'a>,
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

type ParseFn<'a> = fn(&mut Parser<'a>, bool);

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
            Idenitifier     => Self::new(Some(Parser::variable),    None,                       Precedence::None),
            String          => Self::new(Some(Parser::string),      None,                       Precedence::None),
            Number          => Self::new(Some(Parser::number),      None,                       Precedence::None),
            And             => Self::new(None,                      Some(Parser::and),          Precedence::And),
            Class           => Self::new(None,                      None,                       Precedence::None),
            Else            => Self::new(None,                      None,                       Precedence::None),
            False           => Self::new(Some(Parser::literal),     None,                       Precedence::None),
            For             => Self::new(None,                      None,                       Precedence::None),
            Fun             => Self::new(None,                      None,                       Precedence::None),
            If              => Self::new(None,                      None,                       Precedence::None),
            Nil             => Self::new(Some(Parser::literal),     None,                       Precedence::None),
            Or              => Self::new(None,                      Some(Parser::or),           Precedence::Or),
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
        while !self.r#match(TokenType::EOF) {
            self.declaration();
        }
        self.end_compiler();
        if self.had_error {
            Err(())
        } else {
            Ok(mem::take(&mut self.compiler.chunk))
        }
    }

    fn declaration(&mut self) {
        if self.r#match(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }
        if self.panic_mode {
            self.synchronize();
        }
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");
        if self.r#match(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil);
        }
        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration.");
        self.define_variable(global);
    }

    fn parse_variable(&mut self, error_message: &str) -> u8 {
        self.consume(TokenType::Idenitifier, error_message);
        self.declare_variable();
        if self.compiler.scope_depth > 0 {
            return 0;
        }
        self.idenitifier_constant(self.previous.clone())
    }

    fn declare_variable(&mut self) {
        if self.compiler.scope_depth == 0 {
            return;
        }
        let name = self.previous.clone();
        for i in (0..self.compiler.locals.len()).rev() {
            let local = &self.compiler.locals[i];
            if local.depth != -1 && local.depth < self.compiler.scope_depth {
                break;
            }
            if name.value == local.name.value {
                self.error("A variable with this name in this scope already exists.");
            }
        }
        self.add_local(name);
    }

    fn add_local(&mut self, name: Token<'a>) {
        self.compiler.locals.push(Local { name, depth: -1, });
    }

    fn idenitifier_constant(&mut self, token: Token) -> u8 {
        self.make_constant(Value::String(token.value.to_string()))
    }

    fn define_variable(&mut self, global: u8) {
        if self.compiler.scope_depth > 0 {
            self.mark_initialized();
            return;
        }
        self.emit_byte(OpCode::DefineGlobal(global));
    }

    fn mark_initialized(&mut self) {
        self.compiler.locals.last_mut().unwrap().depth = self.compiler.scope_depth;
    }

    fn synchronize(&mut self) {
        use TokenType::*;
        self.panic_mode = false;
        while self.current.token_type != EOF {
            if self.previous.token_type == Semicolon {
                return;
            }
            match self.current.token_type {
                Class | Fun | Var | For | If | While | Print | Return => return,
                _ => ()
            }
            self.advance();
        }
    }

    fn statement(&mut self) {
        if self.r#match(TokenType::Print) {
            self.print_statement();
        } else if self.r#match(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else if self.r#match(TokenType::If) {
            self.if_statement();
        } else if self.r#match(TokenType::While) {
            self.while_statement();
        } else if self.r#match(TokenType::For) {
            self.for_statement();
        } else {
            self.expression_statement();
        }
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");
        if self.r#match(TokenType::Semicolon) {

        } else if self.r#match(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }
        let mut loop_start = self.compiler.chunk.code().len();
        let mut exit_jump = -1;
        if !self.r#match(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition");
            exit_jump = self.emit_jump(OpCode::JumpIfFalse(0)) as i32;
            self.emit_byte(OpCode::Pop);
        }
        if !self.r#match(TokenType::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump(0));
            let increment_start = self.compiler.chunk.code().len();
            self.expression();
            self.emit_byte(OpCode::Pop);
            self.consume(TokenType::RightParen, "Expect ')' after for clauses");
            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }
        self.statement();
        self.emit_loop(loop_start);
        if exit_jump != -1 {
            self.patch_jump(exit_jump as usize);
            self.emit_byte(OpCode::Pop);
        }
        self.end_scope();
    }

    fn while_statement(&mut self) {
        let loop_start = self.compiler.chunk.code().len();
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");
        let exit_jump = self.emit_jump(OpCode::JumpIfFalse(0));
        self.emit_byte(OpCode::Pop);
        self.statement();
        self.emit_loop(loop_start);
        self.patch_jump(exit_jump);
        self.emit_byte(OpCode::Pop);
    }

    fn emit_loop(&mut self, loop_start: usize) {
        let offset = self.compiler.chunk.code().len() - loop_start + 1;
        if offset as u16 > u16::MAX {
            self.error("Loop body too large");
        }
        self.emit_byte(OpCode::Loop(offset as u16));
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after 'if'.");
    
        let then_jump = self.emit_jump(OpCode::JumpIfFalse(0));
        self.emit_byte(OpCode::Pop);
        self.statement();
        let else_jump = self.emit_jump(OpCode::Jump(0));
        self.patch_jump(then_jump);
        self.emit_byte(OpCode::Pop);
        if self.r#match(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn emit_jump(&mut self, opcode: OpCode) -> usize {
        self.emit_byte(opcode);
        return self.compiler.chunk.code().len() - 1;
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.compiler.chunk.code().len() - offset - 1;
        if jump as u16 > u16::MAX {
            self.error("Too much code to jump over.");
        }
        let opcode = &mut self.compiler.chunk.code()[offset];
        *opcode = match opcode {
            OpCode::JumpIfFalse(_) => OpCode::JumpIfFalse(jump as u16),
            OpCode::Jump(_) => OpCode::Jump(jump as u16),
            _ => return,
        };
    }
 
    fn begin_scope(&mut self) {
        self.compiler.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.compiler.scope_depth -= 1;
        while let Some(val) = self.compiler.locals.last() {
            if val.depth > self.compiler.scope_depth {
                self.emit_byte(OpCode::Pop);
                self.compiler.locals.pop();
            }
        }
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::EOF) {
            self.declaration();
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }


    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop);
    }

    fn r#match(&mut self, token_type: TokenType) -> bool {
        if !self.check(token_type) {
            return false;
        }
        self.advance();
        return true;
    }

    fn check(&mut self, token_type: TokenType) -> bool {
        return self.current.token_type == token_type;
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print);
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
                let mut disassembler = Disassembler::new(&mut self.compiler.chunk);
                disassembler.disassemble_chunk("code");
            }
        }
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return);
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.previous.clone(), can_assign);
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let arg = self.resolve_local(&name);
        let get_op: OpCode;
        let set_op: OpCode;
        if arg != -1 {
            get_op = OpCode::GetLocal(arg as u8);
            set_op = OpCode::SetLocal(arg as u8);
        } else {
            let arg = self.idenitifier_constant(name);
            get_op = OpCode::GetGlobal(arg);
            set_op = OpCode::SetGlobal(arg);
        }
        if can_assign && self.r#match(TokenType::Equal) {
            self.expression();
            self.emit_byte(set_op);
        } else {
            self.emit_byte(get_op);
        }
    }

    fn resolve_local(&mut self, name: &Token) -> i32 {
        for i in (0..self.compiler.locals.len()).rev() {
            let local = &self.compiler.locals[i as usize];
            if local.name.value == name.value {
                if local.depth == -1 {
                    self.error("Can't read local variable in its own initializer.");
                }
                return i as i32;
            }
        }
        return -1;
    }

    fn expression(&mut self) {
        self.parse_precendence(Precedence::Assignment);
    }

    fn number(&mut self, _can_assign: bool) {
        let value = Value::Number(self.previous.value.parse().unwrap());
        self.emit_constant(value);
    }

    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self, _can_assign: bool) {
        let operator_type = self.previous.token_type;
        self.parse_precendence(Precedence::Unary);
        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate),
            TokenType::Bang => self.emit_byte(OpCode::Not),
            _ => (),
        }
    }

    fn binary(&mut self, _can_assign: bool) {
        let operator_type = self.previous.token_type;
        let rule = ParseRule::get_rule(operator_type);
        self.parse_precendence(rule.precedence + 1);
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

    fn literal(&mut self, _can_assign: bool) {
        match self.previous.token_type {
            TokenType::False => self.emit_byte(OpCode::False),
            TokenType::Nil =>   self.emit_byte(OpCode::Nil),
            TokenType::True =>  self.emit_byte(OpCode::True),
            _ => (),
        }
    }

    fn parse_precendence(&mut self, precendence: Precedence) {
        self.advance();
        let prefix_rule = ParseRule::get_rule(self.previous.token_type).prefix;
        let can_assign = precendence <= Precedence::Assignment;
        match prefix_rule {
            None => return self.error("Expect expression."),
            Some(rule) => rule(self, can_assign),
        }
        while precendence <= ParseRule::get_rule(self.current.token_type).precedence {
            self.advance();
            let infix_rule = ParseRule::get_rule(self.previous.token_type).infix.unwrap();
            infix_rule(self, can_assign);
        }
        if can_assign && self.r#match(TokenType::Equal){
            self.error("Invalid assignment target.");
        }
    }

    fn string(&mut self, _can_assign: bool) {
        let s = String::from(self.previous.value);
        self.emit_constant(Value::String(s[1..s.len() - 1].to_string()));
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

    fn and(&mut self, _can_assign: bool) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse(0));
        self.emit_byte(OpCode::Pop);
        self.parse_precendence(Precedence::And);
        self.patch_jump(end_jump);
    }

    fn or(&mut self, _can_assign: bool) {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse(0));
        let end_jump = self.emit_jump(OpCode::Jump(0));
        self.patch_jump(else_jump);
        self.emit_byte(OpCode::Pop);
        self.parse_precendence(Precedence::Or);
        self.patch_jump(end_jump);
    }
}
