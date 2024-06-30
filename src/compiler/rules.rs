use crate::compiler::Parser;
use crate::compiler::Precedence;
use crate::compiler::TokenType;


type ParseFn<'a> = fn(&mut Parser<'a>, bool);

pub struct ParseRule<'a> {
    pub prefix:     Option<ParseFn<'a>>,
    pub infix:      Option<ParseFn<'a>>,
    pub precedence: Precedence,
}

impl<'a> ParseRule<'a> {
    fn new(
        prefix:     Option<ParseFn<'a>>,
        infix:      Option<ParseFn<'a>>,
        precedence: Precedence,
    ) -> Self {
        Self {
            prefix,
            infix,
            precedence,
        }
    }

    pub fn get_rule(token_type: TokenType) -> Self {
        use TokenType::*;
        match token_type {
            LeftParen       => Self::new(Some(Parser::grouping),    Some(Parser::call),         Precedence::Call),
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