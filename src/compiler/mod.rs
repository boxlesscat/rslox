use crate::compiler::rules::ParseRule;
use crate::chunk::OpCode;
use crate::scanner::Scanner;
use crate::scanner::Token;
use crate::scanner::TokenType;
use crate::value::Function;
use crate::value::Value;

use std::mem;
use std::rc::Rc;

mod rules;


const U8_COUNT: usize = u8::MAX as usize + 1;

#[derive(Default)]
enum FunctionType {
    #[default]
    Function,
    Script,    
}

struct Compiler <'a> {
    enclosing:      Option<Box<Compiler<'a>>>,
    function:       Function,
    #[allow(dead_code)]
    function_type:  FunctionType,
    locals:         [Local<'a>; U8_COUNT],
    local_count:    usize,
    scope_depth:    i32,
    upvalues:       [Upvalue; U8_COUNT]
}

impl <'a> Compiler<'a> {
    fn new(function_type: FunctionType) -> Self {
        let mut compiler = Self {
            enclosing: None,
            function: Function::new(),
            function_type,
            locals: unsafe {
                #[allow(invalid_value)]
                mem::MaybeUninit::uninit().assume_init()
            },
            local_count: 0,
            scope_depth: 0,
            upvalues: [Upvalue::default(); 256]
        };
        let local = &mut compiler.locals[0];
        compiler.local_count += 1;
        local.depth = 0;
        local.name = Token::default();
        local.name.value = "this";
        local.is_captured = false;
        compiler
    }
    
    fn resolve_local(&mut self, name: &Token) -> Result<i32, &'static str> {
        for i in (0..self.local_count).rev() {
            let local = &self.locals[i as usize];
            if local.name.value == name.value {
                if local.depth == -1 {
                    return Err("Can't read local variable in its own initializer.");
                }
                return Ok(i as i32);
            }
        }
        return Ok(-1);
    }

    fn resolve_upvalue(&mut self, name: &Token) -> Result<i32, &'static str> {
        if self.enclosing.is_none() {
            return Ok(-1);
        }
        let local = self.enclosing.as_mut().unwrap().resolve_local(name)?;
        if local != -1 {
            self.enclosing.as_mut().unwrap().locals[local as usize].is_captured = true;
            return Ok(self.add_upvalue(local as u8, true)?);
        } 
        let upvalue = self.enclosing.as_mut().unwrap().resolve_upvalue(name)?;
        if upvalue != -1 {
            return Ok(self.add_upvalue(upvalue as u8, false)?);
        }
        Ok(-1)
    }

    fn add_upvalue(&mut self, index: u8, is_local: bool) -> Result<i32, &'static str> {
        let upvalue_count = self.function.upvalue_count;
        for i in 0..upvalue_count {
            let upvalue = &self.upvalues[i];
            if upvalue.index == index && upvalue.is_local == is_local {
                return Ok(i as i32);
            }
        }
        if upvalue_count == U8_COUNT {
            return Err("Too many closures variables in function.")
        }
        self.upvalues[upvalue_count].is_local = is_local;
        self.upvalues[upvalue_count].index = index;
        self.function.upvalue_count += 1;
        Ok(upvalue_count as i32)
    }

}

impl <'a> Default for Compiler<'a> {
    fn default() -> Self {
        Self::new(FunctionType::Script)
    }
}

#[derive(Default)]
struct Local <'a> {
    name:   Token<'a>,
    depth:  i32,
    is_captured: bool,
}

#[derive(Debug, Default, Clone, Copy)]
struct Upvalue {
    index: u8,
    is_local: bool,
}

#[derive(Default)]
pub struct Parser<'a> {
    current:    Token<'a>,
    previous:   Token<'a>,
    had_error:  bool,
    panic_mode: bool,
    compiler:   Box<Compiler<'a>>,
    scanner:    Scanner<'a>,
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
        unsafe {
            mem::transmute((self as u8 + rhs) % 11)
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            scanner: Scanner::new(source),
            compiler: Box::new(Compiler::new(FunctionType::Script)),
            ..Default::default()
        }
    }

    fn init_compiler(&mut self, function_type: FunctionType) {
        let compiler = mem::replace(&mut self.compiler, Box::new(Compiler::new(function_type)));
        self.compiler.enclosing = Some(compiler);
        self.compiler.function.name = self.previous.value.to_string();
    }

    pub fn compile(&mut self) -> Option<Rc<Function>> {
        self.advance();
        while !self.r#match(TokenType::EOF) {
            self.declaration();
        }
        let function = self.end_compiler(false);
        if self.had_error {
            None
        } else {
            Some(function)
        }
    }

    fn call(&mut self, _can_assign: bool) {
        let arg_count = self.argument_list();
        self.emit_bytes(OpCode::Call, arg_count);
    }

    fn argument_list(&mut self) -> u8 {
        let mut arg_count: u8 = 0;
        if !self.check(TokenType::RightParen) {
            loop {
                self.expression();
                if arg_count == 255 {
                    self.error("Can't have more than 255 arguments.");
                }
                arg_count += 1;
                if !self.r#match(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after arguments.");
        arg_count
    }

    fn declaration(&mut self) {
        if self.r#match(TokenType::Fun) {
            self.fun_declaration();
        } else if self.r#match(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }
        if self.panic_mode {
            self.synchronize();
        }
    }

    fn fun_declaration(&mut self) {
        let global = self.parse_variable("Expect function name.");
        self.mark_initialized();
        self.function(FunctionType::Function);
        self.define_variable(global);
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
        for i in (0..self.compiler.local_count).rev() {
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
        if self.compiler.local_count == U8_COUNT {
            self.error("Too many local variables in function.");
            return;
        }
        self.compiler.locals[self.compiler.local_count] = Local {
            name,
            depth: -1,
            is_captured: false,
        };
        self.compiler.local_count += 1;
    }

    fn idenitifier_constant(&mut self, token: Token) -> u8 {
        self.make_constant(Value::String(Rc::new(token.value.to_string())))
    }

    fn define_variable(&mut self, global: u8) {
        if self.compiler.scope_depth > 0 {
            self.mark_initialized();
            return;
        }
        self.emit_bytes(OpCode::DefineGlobal, global);
    }

    fn mark_initialized(&mut self) {
        if self.compiler.scope_depth == 0 {
            return;
        }
        self.compiler.locals[self.compiler.local_count - 1].depth = self.compiler.scope_depth;
    }

    fn function(&mut self, function_type: FunctionType) {
        self.init_compiler(function_type);
        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expect '(' after function name.");
        if !self.check(TokenType::RightParen) {
            loop {
                self.compiler.function.arity += 1;
                if self.compiler.function.arity > 255 {
                    self.error_at_current("Can't have more than 255 parameters.");
                }
                let constant = self.parse_variable("Expect parameter name.");
                self.define_variable(constant);
                if !self.r#match(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.");
        self.consume(TokenType::LeftBrace, "Expect '{' before function body.");
        self.block();
        self.end_compiler(true);
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
        } else if self.r#match(TokenType::Return) {
            self.return_statement();
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
            ()
        } else if self.r#match(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }
        let mut loop_start = self.compiler.function.chunk.code.len();
        let mut exit_jump = -1;
        if !self.r#match(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition");
            exit_jump = self.emit_jump(OpCode::JumpIfFalse) as i32;
            self.emit_byte(OpCode::Pop);
        }
        if !self.r#match(TokenType::RightParen) {
            let body_jump = self.emit_jump(OpCode::Jump);
            let increment_start = self.compiler.function.chunk.code.len();
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
        let loop_start = self.compiler.function.chunk.code.len();
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");
        let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);
        self.statement();
        self.emit_loop(loop_start);
        self.patch_jump(exit_jump);
        self.emit_byte(OpCode::Pop);
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_byte(OpCode::Loop);
        let offset = self.compiler.function.chunk.code.len() - loop_start + 2;
        if offset as u16 > u16::MAX {
            self.error("Loop body too large");
        }
        self.emit_byte(((offset >> 8) & 0xff) as u8);
        self.emit_byte((offset & 0xff) as u8);
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after 'if'.");
    
        let then_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);
        self.statement();
        let else_jump = self.emit_jump(OpCode::Jump);
        self.patch_jump(then_jump);
        self.emit_byte(OpCode::Pop);
        if self.r#match(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn emit_jump<T: Into<OpCode>>(&mut self, instruction: T) -> usize {
        self.emit_byte(instruction);
        self.emit_byte(0xff);
        self.emit_byte(0xff);
        return self.compiler.function.chunk.code.len() - 2;
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.compiler.function.chunk.code.len() - offset - 2;
        if jump as u16 > u16::MAX {
            self.error("Too much code to jump over.");
        }
        self.compiler.function.chunk.code[offset] = (((jump >> 8) & 0xff) as u8).into();
        self.compiler.function.chunk.code[offset + 1] = ((jump & 0xff) as u8).into();
    }
 
    fn begin_scope(&mut self) {
        self.compiler.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.compiler.scope_depth -= 1;
        while self.compiler.local_count > 0 && self.compiler.locals[self.compiler.local_count - 1].depth > self.compiler.scope_depth {
            if self.compiler.locals[self.compiler.local_count - 1].is_captured {
                self.emit_byte(OpCode::CloseUpvalue);
            } else {
                self.emit_byte(OpCode::Pop);
            }
            self.compiler.local_count -= 1;
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
        self.current.token_type == token_type
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print);
    }

    fn return_statement(&mut self) {
        if matches!(self.compiler.function_type, FunctionType::Script) {
            self.error("Can't return from top-level code.");
        }
        if self.r#match(TokenType::Semicolon) {
            self.emit_return();
        } else {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after return value.");
            self.emit_byte(OpCode::Return);
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
            ()
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

    fn emit_byte<T: Into<OpCode>>(&mut self, byte: T) {
        self.compiler.function.chunk.write(byte, self.previous.line);
    }

    fn emit_bytes<T: Into<OpCode>, U: Into<OpCode>>(&mut self, byte1: T, byte2: U) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn end_compiler(&mut self, from_function: bool) -> Rc<Function> {
        self.emit_return();
        let function = mem::take(&mut self.compiler.function);
        let function = Rc::new(function);
        #[cfg(feature = "debug_print_code")]
        {
            use crate::debug::Disassembler;
            if !self.had_error {
                let disassembler = Disassembler::new(&function.chunk);
                disassembler.disassemble_chunk(
                    if matches!(self.compiler.function_type, FunctionType::Script) {
                        "<script>"
                    } else {
                        &function.name
                    });
            }
        }
        if let Some(enclosing) = self.compiler.enclosing.take() {
            let compiler = mem::replace(&mut self.compiler, enclosing);
            if from_function {
                let constant = self.make_constant(Value::Function(Rc::clone(&function)));
                self.emit_bytes(OpCode::Closure, constant);
                for i in 0..function.upvalue_count {
                    self.emit_byte(if compiler.upvalues[i].is_local { 1 } else { 0 });
                    self.emit_byte(compiler.upvalues[i].index);
                }
            }
        }
        function
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Nil);
        self.emit_byte(OpCode::Return);
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.previous.clone(), can_assign);
    }

    fn unwrap_err(&mut self, res: Result<i32, &'static str>) -> i32 {
        match res {
            Ok(i) => i,
            Err(msg) => {
                self.error(&msg);
                0
            }
        }
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let res = self.compiler.resolve_local(&name);
        let mut arg = self.unwrap_err(res);
        let get_op: OpCode;
        let set_op: OpCode;
        if arg != -1 {
            get_op = OpCode::GetLocal;
            set_op = OpCode::SetLocal;
        } else {
            let res = self.compiler.resolve_upvalue(&name); 
            arg = self.unwrap_err(res);
            if arg != -1 {
                get_op = OpCode::GetUpvalue;
                set_op = OpCode::SetUpvalue;
            } else {
                arg = self.idenitifier_constant(name) as i32;
                get_op = OpCode::GetGlobal;
                set_op = OpCode::SetGlobal;
            }
        }
        if can_assign && self.r#match(TokenType::Equal) {
            self.expression();
            self.emit_bytes(set_op, arg as u8);
        } else {
            self.emit_bytes(get_op, arg as u8);
        }
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
        if can_assign && self.r#match(TokenType::Equal) {
            self.error("Invalid assignment target.");
        }
    }

    fn string(&mut self, _can_assign: bool) {
        let s = String::from(self.previous.value);
        self.emit_constant(Value::String(Rc::new(s[1..s.len() - 1].to_string())));
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(OpCode::Constant, constant);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.compiler.function.chunk.add_constant(value);
        if constant > u8::MAX as usize {
            self.error("Too many constants in one chunk.");
            0
        } else {
            constant as u8
        }
    }

    fn and(&mut self, _can_assign: bool) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::Pop);
        self.parse_precendence(Precedence::And);
        self.patch_jump(end_jump);
    }

    fn or(&mut self, _can_assign: bool) {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse);
        let end_jump = self.emit_jump(OpCode::Jump);
        self.patch_jump(else_jump);
        self.emit_byte(OpCode::Pop);
        self.parse_precendence(Precedence::Or);
        self.patch_jump(end_jump);
    }
}
