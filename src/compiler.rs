use crate::{
    chunk::{operations::OpCode, Chunk},
    object::Object,
    scanner::{Scanner, Token, TokenKind},
    value::copy_string,
    value::Value,
    vm::InterpretError,
};

use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl Precedence {
    fn next(&self) -> Self {
        match self {
            Self::None => Self::Assignment,
            Self::Assignment => Self::Or,
            Self::Or => Self::And,
            Self::And => Self::Equality,
            Self::Equality => Self::Comparison,
            Self::Comparison => Self::Term,
            Self::Term => Self::Factor,
            Self::Factor => Self::Unary,
            Self::Unary => Self::Call,
            Self::Call => Self::Primary,
            Self::Primary => Self::Primary,
        }
    }
}

struct ParseRule<'a, 'b> {
    prefix: Option<&'a dyn Fn(&'a mut Parser<'b>, bool) -> ()>,
    infix: Option<&'a dyn Fn(&'a mut Parser<'b>, bool) -> ()>,
    precedence: Precedence,
}

fn get_rule<'a, 'b>(kind: TokenKind) -> ParseRule<'a, 'b> {
    match kind {
        TokenKind::LeftParen => ParseRule {
            prefix: Some(&Parser::grouping),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Bang => ParseRule {
            prefix: Some(&Parser::unary),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Minus => ParseRule {
            prefix: Some(&Parser::unary),
            infix: Some(&Parser::binary),
            precedence: Precedence::Term,
        },
        TokenKind::Plus => ParseRule {
            prefix: None,
            infix: Some(&Parser::binary),
            precedence: Precedence::Term,
        },
        TokenKind::Slash => ParseRule {
            prefix: None,
            infix: Some(&Parser::binary),
            precedence: Precedence::Factor,
        },
        TokenKind::Star => ParseRule {
            prefix: None,
            infix: Some(&Parser::binary),
            precedence: Precedence::Factor,
        },
        TokenKind::BangEqual => ParseRule {
            prefix: None,
            infix: Some(&Parser::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::EqualEqual => ParseRule {
            prefix: None,
            infix: Some(&Parser::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::Greater => ParseRule {
            prefix: None,
            infix: Some(&Parser::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::GreaterEqual => ParseRule {
            prefix: None,
            infix: Some(&Parser::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::Less => ParseRule {
            prefix: None,
            infix: Some(&Parser::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::LessEqual => ParseRule {
            prefix: None,
            infix: Some(&Parser::binary),
            precedence: Precedence::Comparison,
        },
        TokenKind::Number => ParseRule {
            prefix: Some(&Parser::number),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::True | TokenKind::False | TokenKind::Nil => ParseRule {
            prefix: Some(&Parser::literal),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::Identifier => ParseRule {
            prefix: Some(&Parser::variable),
            infix: None,
            precedence: Precedence::None,
        },
        TokenKind::String => ParseRule {
            prefix: Some(&Parser::string),
            infix: None,
            precedence: Precedence::None,
        },
        _ => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    }
}

fn error(token: Token, message: &str, had_error: &mut bool, panic_mode: &mut bool) {
    if *panic_mode {
        return;
    }
    *panic_mode = true;
    *had_error = true;
    eprint!("[line {}] Error", token.line());
    match token.kind() {
        TokenKind::Error => (),
        _ => {
            eprint!(" at '{}': ", token.as_str());
        }
    }

    eprintln!("{}", message);
}
struct Parser<'a> {
    scanner: Scanner<'a>,
    previous: Token<'a>,
    current: Token<'a>,
    chunk: Chunk,
    strings: &'a mut HashSet<Box<str>>,
    objects: &'a mut *mut Object,
    panic_mode: bool,
    had_error: bool,
}

impl<'a> Parser<'a> {
    fn new(
        source: &'a str,
        strings: &'a mut HashSet<Box<str>>,
        objects: &'a mut *mut Object,
    ) -> Parser<'a> {
        Parser {
            scanner: Scanner::new(source),
            previous: Token::default(),
            current: Token::default(),
            chunk: Chunk::new(),
            strings,
            objects,
            panic_mode: false,
            had_error: false,
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.current.kind() == kind
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) {
        let Self {
            scanner,
            previous,
            current,
            panic_mode,
            had_error,
            ..
        } = self;
        *previous = *current;
        'skip_errors: for token in scanner {
            *current = token;
            if current.kind() != TokenKind::Error {
                break 'skip_errors;
            }
            error(token, token.as_str(), had_error, panic_mode);
        }
    }

    fn consume(&mut self, expected: TokenKind, error_message: &str) {
        if self.current.kind() == expected {
            self.advance();
        } else {
            error(
                self.current,
                error_message,
                &mut self.had_error,
                &mut self.panic_mode,
            )
        }
    }

    fn emit_byte(&mut self, byte: u8) {
        self.chunk.add_byte(byte, self.current.line());
    }

    fn emit_byte_pair(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.chunk.add_constant(value);
        if constant > 255 {
            error(
                self.current,
                "too many constants in one chunk.",
                &mut self.had_error,
                &mut self.panic_mode,
            );
            return;
        }
        self.emit_byte_pair(OpCode::Constant.into(), constant as u8);
    }

    fn number(&mut self, _: bool) {
        let value = Value::Number(self.previous.as_str().parse::<f64>().unwrap());
        self.emit_constant(value);
    }

    fn literal(&mut self, _: bool) {
        match self.previous.kind() {
            TokenKind::False => self.emit_byte(OpCode::False.into()),
            TokenKind::True => self.emit_byte(OpCode::True.into()),
            TokenKind::Nil => self.emit_byte(OpCode::Nil.into()),
            _ => unreachable!(),
        }
    }

    fn string(&mut self, _: bool) {
        let string = self.previous.as_str();
        let value = copy_string(
            string.trim_start_matches('"').trim_end_matches('"'),
            self.strings,
            self.objects,
        );
        let index = self.chunk.add_constant(value);
        self.emit_byte_pair(OpCode::Constant.into(), index as u8);
    }

    fn named_variable(&mut self, can_assign: bool) {
        let arg = self.identifier_constant();
        if can_assign && self.match_token(TokenKind::Equal) {
            self.expression();
            self.emit_byte_pair(OpCode::SetGlobal.into(), arg);
        } else {
            self.emit_byte_pair(OpCode::GetGlobal.into(), arg);
        }
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(can_assign);
    }

    fn grouping(&mut self, _: bool) {
        self.expression();
        self.consume(TokenKind::RightParen, "Expected ')' after expression.");
    }

    fn unary(&mut self, _: bool) {
        let operator_kind = self.previous.kind();
        self.parse_precedence(Precedence::Unary);

        match operator_kind {
            TokenKind::Minus => self.emit_byte(OpCode::Negate.into()),
            TokenKind::Bang => self.emit_byte(OpCode::Not.into()),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self, _: bool) {
        let operator_kind = self.previous.kind();
        let parse_rule = get_rule(operator_kind);
        self.parse_precedence(parse_rule.precedence.next());

        match operator_kind {
            TokenKind::Plus => self.emit_byte(OpCode::Add.into()),
            TokenKind::Minus => self.emit_byte(OpCode::Subtract.into()),
            TokenKind::Star => self.emit_byte(OpCode::Multiply.into()),
            TokenKind::Slash => self.emit_byte(OpCode::Divide.into()),
            TokenKind::BangEqual => self.emit_byte_pair(OpCode::Equal.into(), OpCode::Not.into()),
            TokenKind::EqualEqual => self.emit_byte(OpCode::Equal.into()),
            TokenKind::Greater => self.emit_byte(OpCode::Greater.into()),
            TokenKind::GreaterEqual => {
                self.emit_byte_pair(OpCode::Less.into(), OpCode::Negate.into())
            }
            TokenKind::Less => self.emit_byte(OpCode::Less.into()),
            TokenKind::LessEqual => {
                self.emit_byte_pair(OpCode::Greater.into(), OpCode::Negate.into())
            }
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let can_assign = precedence <= Precedence::Assignment;
        let prefix_rule = get_rule(self.previous.kind()).prefix;
        match prefix_rule {
            None => error(
                self.previous,
                "Expect expression.",
                &mut self.had_error,
                &mut self.panic_mode,
            ),
            Some(prefix_rule) => prefix_rule(self, can_assign),
        }

        while precedence <= get_rule(self.current.kind()).precedence {
            self.advance();
            let infix_rule = get_rule(self.previous.kind()).infix;
            match infix_rule {
                None => {
                    println!("no infix rule for {:?}", self.previous.kind());
                }
                Some(infix_rule) => infix_rule(self, can_assign),
            }

            if can_assign && self.match_token(TokenKind::Equal) {
                error(self.previous, "Invalid assignment target.", &mut self.had_error, &mut self.panic_mode);
            }
        }
    }

    fn identifier_constant(&mut self) -> u8 {
        return self.chunk.add_constant(Value::Obj(Object::new_string(self.previous.as_str(), self.strings, &mut self.objects))) as u8
    }

    fn parse_variable(&mut self, error_message: &str) -> u8 {
        self.consume(TokenKind::Identifier, error_message);
        return self.identifier_constant();
    }

    fn define_variable(&mut self, global: u8) {
        self.emit_byte_pair(OpCode::DefineGlobal.into(), global);
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenKind::Semicolon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print.into());
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenKind::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop.into());
    }

    fn statement(&mut self) {
        if self.match_token(TokenKind::Print) {
            self.print_statement();
        }
        else {
            self.expression_statement();
        }
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;
        'sync: while !self.scanner.is_at_end() {
            if self.previous.kind() == TokenKind::Semicolon {
                return;
            }
            match self.current.kind() {
                TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return => break 'sync,
                _ => ()
            }
            self.advance();
        }
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.match_token(TokenKind::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil.into());
        }

        self.consume(TokenKind::Semicolon, "Expect ';' after variable declaration.");

        self.define_variable(global);
    }

    fn declaration(&mut self) {
        if self.match_token(TokenKind::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }
        if self.panic_mode {
            self.synchronize();
        }
    }

    fn end(&mut self) {
        self.emit_byte(OpCode::Return.into())
    }
}

pub fn compile<'a>(
    source: &str,
    strings: &'a mut HashSet<Box<str>>,
    objects: &'a mut *mut Object,
) -> Result<Chunk, InterpretError> {
    let mut parser = Parser::new(source, strings, objects);
    parser.advance();
    while !parser.scanner.is_at_end() {
        parser.declaration();
    }
    parser.end();
    match parser.had_error {
        false => {
            #[cfg(debug_assertions)]
            parser.chunk.disassemble();
            Ok(parser.chunk)
        }
        true => Err(InterpretError::Compile),
    }
}
