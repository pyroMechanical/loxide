use crate::{chunk::{Chunk, operations::OpCode}, scanner::{Scanner, Token, TokenKind}, vm::InterpretError, value::Value};
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
    Primary
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
            Self::Primary => Self::Primary
        }
    }
}

struct ParseRule<'a, 'b> {
    prefix: Option<&'a dyn Fn(&'a mut Parser<'b>) -> ()>,
    infix: Option<&'a dyn Fn(&'a mut Parser<'b>) -> ()>,
    precedence: Precedence
}

fn get_rule<'a, 'b>(kind: TokenKind) -> ParseRule<'a, 'b> {
    match kind {
        TokenKind::LeftParen => ParseRule{prefix: Some(&Parser::grouping), infix: None, precedence: Precedence::None},
        TokenKind::Bang => ParseRule {prefix: Some(&Parser::unary), infix: None, precedence: Precedence::None},
        TokenKind::Minus => ParseRule{prefix:Some(&Parser::unary), infix: Some(&Parser::binary), precedence: Precedence::Term},
        TokenKind::Plus => ParseRule{prefix: None, infix: Some(&Parser::binary), precedence: Precedence::Term},
        TokenKind::Slash => ParseRule{prefix: None, infix: Some(&Parser::binary), precedence: Precedence::Factor},
        TokenKind::Star => ParseRule{prefix: None, infix: Some(&Parser::binary), precedence: Precedence::Factor},
        TokenKind::BangEqual => ParseRule{prefix: None, infix: Some(&Parser::binary), precedence: Precedence::Comparison},
        TokenKind::EqualEqual => ParseRule{prefix: None, infix: Some(&Parser::binary), precedence: Precedence::Comparison},
        TokenKind::Greater => ParseRule{prefix: None, infix: Some(&Parser::binary), precedence: Precedence::Comparison},
        TokenKind::GreaterEqual => ParseRule{prefix: None, infix: Some(&Parser::binary), precedence: Precedence::Comparison},
        TokenKind::Less => ParseRule{prefix: None, infix: Some(&Parser::binary), precedence: Precedence::Comparison},
        TokenKind::LessEqual => ParseRule{prefix: None, infix: Some(&Parser::binary), precedence: Precedence::Comparison},
        TokenKind::Number => ParseRule{prefix: Some(&Parser::number), infix: None, precedence: Precedence::None},
        TokenKind::True | TokenKind::False | TokenKind::Nil => ParseRule{prefix: Some(&Parser::literal), infix: None, precedence: Precedence::None},
        _ => ParseRule{prefix: None, infix: None, precedence: Precedence::None},
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
    panic_mode: bool,
    had_error: bool,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Parser<'a> {
        Parser {
            scanner: Scanner::new(source),
            previous: Token::default(),
            current: Token::default(),
            chunk: Chunk::new(),
            panic_mode: false,
            had_error: false,
        }
    }
    
    fn advance(&mut self) {
        let Self{scanner, previous, current, panic_mode, had_error, ..} = self;
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
        }
        else {
            error(self.current, error_message, &mut self.had_error, &mut self.panic_mode)
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
            error(self.current, "too many constants in one chunk.", &mut self.had_error, &mut self.panic_mode);
            return;
        }
        self.emit_byte_pair(OpCode::Constant.into(), constant as u8);
    }

    fn number(&mut self) {
        let value = Value::Number(self.previous.as_str().parse::<f64>().unwrap());
        self.emit_constant(value);
    }

    fn literal(&mut self) {
        match self.previous.kind() {
            TokenKind::False => self.emit_byte(OpCode::False.into()),
            TokenKind::True => self.emit_byte(OpCode::True.into()),
            TokenKind::Nil => self.emit_byte(OpCode::Nil.into()),
            _ => unreachable!()
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenKind::RightParen, "Expected ')' after expression.");
    }

    fn unary(&mut self) {
        let operator_kind = self.previous.kind();
        self.parse_precedence(Precedence::Unary);

        match operator_kind {
            TokenKind::Minus => self.emit_byte(OpCode::Negate.into()),
            TokenKind::Bang => self.emit_byte(OpCode::Not.into()),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self) {
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
            TokenKind::GreaterEqual => self.emit_byte_pair(OpCode::Less.into(), OpCode::Negate.into()),
            TokenKind::Less => self.emit_byte(OpCode::Less.into()),
            TokenKind::LessEqual => self.emit_byte_pair(OpCode::Greater.into(), OpCode::Negate.into()),
            _ => unreachable!()
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = get_rule(self.previous.kind()).prefix;
        match prefix_rule {
            None => error(self.previous, "Expect expression.", &mut self.had_error, &mut self.panic_mode),
            Some(prefix_rule) => prefix_rule(self),
        }

        while precedence <= get_rule(self.current.kind()).precedence {
            self.advance();
            let infix_rule = get_rule(self.previous.kind()).infix;
            match infix_rule {
                None => {
                    println!("no infix rule for {:?}", self.previous.kind());
                },
                Some(infix_rule) => infix_rule(self)
            }
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn end(&mut self) {
        self.emit_byte(OpCode::Return.into())
    }
}

pub fn compile(source: &str) -> Result<Chunk, InterpretError>{
    let mut parser = Parser::new(source);
    parser.advance();
    parser.expression();
    parser.end();
    #[cfg(debug_assertions)]
    if !parser.had_error {
        parser.chunk.disassemble();
    }
    Ok(parser.chunk)
}
