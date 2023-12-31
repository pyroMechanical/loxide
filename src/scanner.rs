use std::str::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
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
    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    Identifier,
    String,
    Number,
    // Keywords.
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

#[derive(Copy, Clone, Debug)]
pub struct Token<'a> {
    kind: TokenKind,
    line: u32,
    string: &'a str,
}

impl<'a> Token<'a> {
    pub fn synthetic_new(string: &'a str) -> Token<'a> {
        Token{kind: TokenKind::Identifier, line: 0, string}
    }
    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    pub fn as_str(&self) -> &'a str {
        self.string
    }

    pub fn line(&self) -> u32 {
        self.line
    }
}

impl Default for Token<'static> {
    fn default() -> Token<'static> {
        Token {
            kind: TokenKind::Error,
            line: 0,
            string: "",
        }
    }
}

fn check_keyword(string: &str, keyword: &str, kind: TokenKind) -> TokenKind {
    if string == keyword {
        return kind;
    } else {
        return TokenKind::Identifier;
    }
}

#[derive(Clone)]
pub struct Scanner<'a> {
    string: &'a str,
    source: CharIndices<'a>,
    start: usize,
    current: usize,
    line: u32,
}
impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            string: source,
            source: source.char_indices(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn is_at_end(&self) -> bool {
        return self.source.as_str() == "";
    }

    pub fn peek(&self) -> Option<char> {
        let mut peekable = self.source.clone();
        peekable.nth(0).map(|(_, c)| c)
    }

    fn peek_next(&self) -> Option<char> {
        let mut peekable = self.source.clone();
        peekable.nth(1).map(|(_, c)| c)
    }

    fn advance(&mut self) -> Option<char> {
        let (_, c) = self.source.next()?;
        self.current = self.string.len() - self.source.as_str().len();
        Some(c)
    }

    fn match_char(&mut self, expected: char) -> bool {
        match self.peek() {
            None => false,
            Some(c) => {
                if c == expected {
                    self.advance();
                    true
                } else {
                    false
                }
            }
        }
    }

    fn make_token(&self, kind: TokenKind) -> Token<'a> {
        Token {
            kind,
            line: self.line,
            string: &self.string[self.start..self.current],
        }
    }

    fn error_token(&self, msg: &'a str) -> Token<'a> {
        Token {
            kind: TokenKind::Error,
            line: self.line,
            string: msg,
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek();
            match c {
                None => break,
                Some(c) => match c {
                    ' ' | '\r' | '\t' => {
                        self.advance();
                    }
                    '\n' => {
                        self.line += 1;
                        self.advance();
                    }
                    '/' => {
                        if let Some('/') = self.peek_next() {
                            loop {
                                match self.peek() {
                                    None => break,
                                    Some(c) => {
                                        if c != '\n' {
                                            self.advance();
                                        } else {
                                            break;
                                        }
                                    }
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    _ => break,
                },
            }
        }
        self.start = self.current;
    }

    fn identifier_kind(&self) -> TokenKind {
        let token = &self.string[self.start..self.current];
        let mut chars = token.chars();
        match chars.next() {
            None => TokenKind::Identifier,
            Some(c) => match c {
                'a' => check_keyword(chars.as_str(), "nd", TokenKind::And),
                'c' => check_keyword(chars.as_str(), "lass", TokenKind::Class),
                'e' => check_keyword(chars.as_str(), "lse", TokenKind::Else),
                'f' => match chars.next() {
                    None => TokenKind::Identifier,
                    Some(c) => match c {
                        'a' => check_keyword(chars.as_str(), "lse", TokenKind::False),
                        'o' => check_keyword(chars.as_str(), "r", TokenKind::For),
                        'u' => check_keyword(chars.as_str(), "n", TokenKind::Fun),
                        _ => TokenKind::Identifier,
                    },
                },
                'i' => check_keyword(chars.as_str(), "f", TokenKind::If),
                'n' => check_keyword(chars.as_str(), "il", TokenKind::Nil),
                'o' => check_keyword(chars.as_str(), "r", TokenKind::Or),
                'p' => check_keyword(chars.as_str(), "rint", TokenKind::Print),
                'r' => check_keyword(chars.as_str(), "eturn", TokenKind::Return),
                's' => check_keyword(chars.as_str(), "uper", TokenKind::Super),
                't' => match chars.next() {
                    None => TokenKind::Identifier,
                    Some(c) => match c {
                        'h' => check_keyword(chars.as_str(), "is", TokenKind::This),
                        'r' => check_keyword(chars.as_str(), "ue", TokenKind::True),
                        _ => TokenKind::Identifier,
                    },
                },
                'v' => check_keyword(chars.as_str(), "ar", TokenKind::Var),
                'w' => check_keyword(chars.as_str(), "hile", TokenKind::While),
                _ => TokenKind::Identifier,
            },
        }
    }

    fn identifier(&mut self) -> Token<'a> {
        'identifier: loop {
            match self.peek() {
                None => break 'identifier,
                Some(c) => match c {
                    '0'..='9' | 'a'..='z' | 'A'..='Z'| '_' => {
                        self.advance();
                    }
                    _ => break 'identifier,
                },
            };
        }
        self.make_token(self.identifier_kind())
    }

    fn number(&mut self) -> Token<'a> {
        'integer: loop {
            match self.peek() {
                None => break 'integer,
                Some(c) => match c {
                    '0'..='9' => {
                        self.advance();
                    }
                    _ => break 'integer,
                },
            }
        }
        if self.peek() == Some('.') && self.peek_next().is_some_and(|c| ('0'..='9').contains(&c)) {
            self.advance();
            'fraction: loop {
                match self.peek() {
                    None => break 'fraction,
                    Some(c) => match c {
                        '0'..='9' => {
                            self.advance();
                        }
                        _ => break 'fraction,
                    },
                }
            }
        }
        self.make_token(TokenKind::Number)
    }

    fn string(&mut self) -> Token<'a> {
        loop {
            match self.advance() {
                None => return self.error_token("Unterminated String."),
                Some(c) => {
                    if c == '\n' {
                        self.line += 1;
                    } else if c == '"' {
                        return self.make_token(TokenKind::String);
                    }
                }
            }
        }
    }

    pub fn scan_token(&mut self) -> Token<'a> {
        self.start = self.current;
        self.skip_whitespace();
        let c = self.advance();
        let token = match c {
            None => return self.make_token(TokenKind::EOF),
            Some(c) => match c {
                '(' => self.make_token(TokenKind::LeftParen),
                ')' => self.make_token(TokenKind::RightParen),
                '{' => self.make_token(TokenKind::LeftBrace),
                '}' => self.make_token(TokenKind::RightBrace),
                ';' => self.make_token(TokenKind::Semicolon),
                ',' => self.make_token(TokenKind::Comma),
                '.' => self.make_token(TokenKind::Dot),
                '-' => self.make_token(TokenKind::Minus),
                '+' => self.make_token(TokenKind::Plus),
                '/' => self.make_token(TokenKind::Slash),
                '*' => self.make_token(TokenKind::Star),
                '"' => self.string(),
                '=' => {
                    let kind = if self.match_char('=') {
                        TokenKind::EqualEqual
                    } else {
                        TokenKind::Equal
                    };
                    self.make_token(kind)
                }
                '!' => {
                    let kind = if self.match_char('=') {
                        TokenKind::BangEqual
                    } else {
                        TokenKind::Bang
                    };
                    self.make_token(kind)
                }
                '>' => {
                    let kind = if self.match_char('=') {
                        TokenKind::GreaterEqual
                    } else {
                        TokenKind::Greater
                    };
                    self.make_token(kind)
                }
                '<' => {
                    let kind = if self.match_char('=') {
                        TokenKind::LessEqual
                    } else {
                        TokenKind::Less
                    };
                    self.make_token(kind)
                }
                '0'..='9' => self.number(),
                'a'..='z' | 'A'..='Z'| '_' => self.identifier(),
                _ => self.error_token("Unexpected character."),
            },
        };
        token
    }
}
