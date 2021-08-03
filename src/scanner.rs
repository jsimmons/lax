use crate::Lax;

use fast_float;

#[derive(Copy, Clone, Debug)]
enum TokenType<'text> {
    // Single-character tokens.
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
    String(&'text [u8]),
    Number(f64),

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
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

    EOF,
}

#[derive(Clone)]
pub struct Token<'text> {
    token_type: TokenType<'text>,
    lexeme: &'text [u8],
    line: usize,
}

impl<'text> std::fmt::Debug for Token<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.token_type.fmt(f)
    }
}

pub struct Scanner<'lax, 'chunk, 'text, 'report> {
    lax: &'lax mut Lax<'chunk, 'report>,
    source: &'text [u8],
    start: usize,
    current: usize,
    line: usize,
    tokens: Vec<Token<'text>>,
}

impl<'lax, 'chunk, 'text, 'report> Scanner<'lax, 'chunk, 'text, 'report> {
    pub fn new(lax: &'lax mut Lax<'chunk, 'report>, source: &'text [u8]) -> Self {
        Self {
            lax,
            source,
            start: 0,
            current: 0,
            line: 0,
            tokens: Vec::new(),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek(&self) -> u8 {
        if self.is_at_end() {
            b'\0'
        } else {
            self.source[self.current]
        }
    }

    fn peek_next(&self) -> u8 {
        if self.current + 1 >= self.source.len() {
            b'\0'
        } else {
            self.source[self.current + 1]
        }
    }

    fn advance(&mut self) -> u8 {
        let position = self.current;
        self.current += 1;
        self.source[position]
    }

    fn check_next(&mut self, expected: u8) -> bool {
        if self.is_at_end() || self.peek() != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn add_token(&mut self, token_type: TokenType<'text>) {
        let lexeme = &self.source[self.start..self.current];
        self.tokens.push(Token {
            token_type,
            lexeme,
            line: self.line,
        })
    }

    pub fn scan_tokens(&mut self) -> Vec<Token<'text>> {
        self.tokens.clear();
        'next_token: while !self.is_at_end() {
            self.start = self.current;
            match self.advance() {
                b'(' => self.add_token(TokenType::LeftParen),
                b')' => self.add_token(TokenType::RightParen),
                b'{' => self.add_token(TokenType::LeftBrace),
                b'}' => self.add_token(TokenType::RightBrace),
                b',' => self.add_token(TokenType::Comma),
                b'.' => self.add_token(TokenType::Dot),
                b'-' => self.add_token(TokenType::Minus),
                b'+' => self.add_token(TokenType::Plus),
                b';' => self.add_token(TokenType::Semicolon),
                b'*' => self.add_token(TokenType::Star),
                b'!' => {
                    if self.check_next(b'=') {
                        self.add_token(TokenType::BangEqual)
                    } else {
                        self.add_token(TokenType::Bang)
                    }
                }
                b'=' => {
                    if self.check_next(b'=') {
                        self.add_token(TokenType::EqualEqual)
                    } else {
                        self.add_token(TokenType::Equal)
                    }
                }
                b'<' => {
                    if self.check_next(b'=') {
                        self.add_token(TokenType::LessEqual)
                    } else {
                        self.add_token(TokenType::Less)
                    }
                }
                b'>' => {
                    if self.check_next(b'=') {
                        self.add_token(TokenType::GreaterEqual)
                    } else {
                        self.add_token(TokenType::Greater)
                    }
                }
                b'/' => {
                    if self.check_next(b'/') {
                        while self.peek() != b'\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        self.add_token(TokenType::Slash)
                    }
                }
                b' ' | b'\r' | b'\t' => continue 'next_token,
                b'\n' => {
                    self.line += 1;
                }
                b'"' => {
                    while self.peek() != b'"' && !self.is_at_end() {
                        if self.peek() == b'\n' {
                            self.line += 1;
                        }
                        self.advance();
                    }

                    if self.is_at_end() {
                        self.lax.error(self.line, "unterminated string");
                        break;
                    }

                    self.advance(); // consume the "

                    self.add_token(TokenType::String(
                        &self.source[self.start + 1..self.current - 1],
                    ));
                }
                b'0'..=b'9' => {
                    let is_digit = |c| match c {
                        b'0'..=b'9' => true,
                        _ => false,
                    };
                    while is_digit(self.peek()) {
                        self.advance();
                    }
                    if self.peek() == b'.' && is_digit(self.peek_next()) {
                        self.advance();
                        while is_digit(self.peek()) {
                            self.advance();
                        }
                    }
                    match fast_float::parse(&self.source[self.start..self.current]) {
                        Ok(number) => self.add_token(TokenType::Number(number)),
                        Err(_) => self.lax.error(self.line, "could not parse number"),
                    }
                }
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                    let is_alpha_numeric = |c| match c {
                        b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9' => true,
                        _ => false,
                    };
                    while is_alpha_numeric(self.peek()) {
                        self.advance();
                    }
                    match &self.source[self.start..self.current] {
                        b"and" => self.add_token(TokenType::And),
                        b"class" => self.add_token(TokenType::Class),
                        b"else" => self.add_token(TokenType::Else),
                        b"false" => self.add_token(TokenType::False),
                        b"for" => self.add_token(TokenType::For),
                        b"fun" => self.add_token(TokenType::Fun),
                        b"if" => self.add_token(TokenType::If),
                        b"nil" => self.add_token(TokenType::Nil),
                        b"or" => self.add_token(TokenType::Or),
                        b"print" => self.add_token(TokenType::Print),
                        b"return" => self.add_token(TokenType::Return),
                        b"super" => self.add_token(TokenType::Super),
                        b"this" => self.add_token(TokenType::This),
                        b"true" => self.add_token(TokenType::True),
                        b"var" => self.add_token(TokenType::Var),
                        b"while" => self.add_token(TokenType::While),
                        _ => self.add_token(TokenType::Identifier),
                    }
                }
                _ => {
                    self.lax.error(self.line, "unexpected character");
                }
            }
        }

        self.tokens.push(Token {
            token_type: TokenType::EOF,
            lexeme: &[],
            line: self.line,
        });

        self.tokens.clone()
    }
}
