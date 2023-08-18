use crate::{chunk::Chunk, scanner::{Scanner, Token, TokenKind}, vm::InterpretError};

struct Parser<'a> {
    scanner: Scanner<'a>,
    previous: Token<'a>,
    current: Token<'a>,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Parser<'a> {
        Parser {
            scanner: Scanner::new(source),
            previous: Token::default(),
            current: Token::default(),
        }
    }
    
    fn advance(&'a mut self) -> Token<'a> {
        self.previous = self.current;
        'skip_errors: while let x = self.scanner.scan_token() {
            self.current = x;
            if x.kind() != TokenKind::Error {
                break 'skip_errors;
            }
        }
        self.current
    }
}

pub fn compile(source: &str) -> Result<Chunk, InterpretError>{
    let chunk = Chunk::new();
    let mut parser = Parser::new(source);
    parser.advance();
    todo!()
}
