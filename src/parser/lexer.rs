use super::span::*;
use std::{iter::Peekable, str::CharIndices};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    // 标识符和字面量
    Ident,
    Number,

    // 运算符
    Colon,      // :
    Assign,     // :≡
    To,         // ->
    FatArrow,   // =>
    Eq,         // =
    Comma,      // ,
    Product,    // ×
    LeftPar,    // (
    RightPar,   // )
    LeftBrace,  // {
    RightBrace, // }

    // 特殊
    Eof,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub class: TokenType,
    pub span: Span,
    pub literal: String,
}

impl Token {
    pub fn new(class: TokenType, span: Span, literal: &str) -> Self {
        Self {
            class,
            span,
            literal: literal.into(),
        }
    }
}

#[derive(Clone)]
pub struct Lexer<'src> {
    /// 解析的文本
    literal: &'src str,
    /// 字符迭代器
    chars: Peekable<CharIndices<'src>>,
    /// 当前位置
    pos: usize,
}

fn is_legal_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl<'src> Lexer<'src> {
    pub fn new(literal: &'src str) -> Self {
        Self {
            literal,
            chars: literal.char_indices().peekable(),
            pos: 0,
        }
    }

    /// 预览字符
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    /// 消费下一个字符
    fn advance(&mut self) -> Option<char> {
        self.chars.next().and_then(|(pos, c)| {
            self.pos = pos + c.len_utf8();
            Some(c)
        })
    }

    /// 将迭代器恢复到之前的位置
    fn restore_pos(&mut self, pos: usize) {
		// 这套 API 似乎不支持 O(1) 的位置设置
		// TODO: 用好一点的 API 优化
        self.chars = self.literal.char_indices().peekable();
        while self.chars.peek().is_some_and(|(p, _)| *p < pos) {
            self.chars.next();
        }
		self.pos = pos;
    }

    /// 跳过空白和注释
    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                Some(' ' | '\t' | '\r' | '\n') => {
                    self.advance();
                }
                Some('/') => {
                    let start_pos = self.pos;
                    self.advance();
                    if self.peek() == Some('/') {
                        // 行注释
                        while self.peek().is_some_and(|c| c != '\n') {
                            self.advance();
                        }
                    } else if self.peek() == Some('*') {
                        // 块注释
                        self.advance(); // 不允许 /*/
                        while let Some(c) = self.peek() {
                            self.advance();
                            if c == '*' {
                                self.advance();
                                if self.peek() == Some('/') {
                                    self.advance();
                                    break;
                                }
                            }
                        }
                    } else {
                        // 不是注释
                        self.restore_pos(start_pos);
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn read_ident(&mut self, start: usize) -> Token {
        while self.peek().is_some_and(|c| is_legal_ident_char(c)) {
            self.advance();
        }

        let literal = &self.literal[start..self.pos];
        let span = Span::new(start, self.pos);

        // 匹配关键字
        let class = match literal {
            _ => TokenType::Ident,
        };

        Token::new(class, span, literal)
    }

    pub fn read_number(&mut self, start: usize) -> Token {
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
        }

        let literal = &self.literal[start..self.pos];
        let span = Span::new(start, self.pos);

        Token::new(TokenType::Number, span, literal)
    }

    /// 获取下一个 Token
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace(); // 跳过空白和注释

        let start = self.pos;

        let Some(c) = self.advance() else {
            return Token::new(TokenType::Eof, Span::new(start, start), "");
        };

        let span = Span::new(start, self.pos);

        match c {
            // 单字符
            ',' => Token::new(TokenType::Comma, span, ","),
            '×' => Token::new(TokenType::Product, span, "×"),
            '(' => Token::new(TokenType::LeftPar, span, "("),
            ')' => Token::new(TokenType::RightPar, span, ")"),
            '{' => Token::new(TokenType::LeftBrace, span, "{"),
            '}' => Token::new(TokenType::RightBrace, span, "}"),

            // 多字符；使用最大匹配原则
            ':' => {
                if self.peek() == Some('≡') {
                    self.advance();
                    Token::new(TokenType::Assign, span, ":≡")
                } else {
                    Token::new(TokenType::Colon, span, ":")
                }
            }

            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    Token::new(TokenType::To, span, "->")
                } else {
                    Token::new(TokenType::Error, span, "-")
                }
            }
            '=' => {
                if self.peek() == Some('>') {
                    self.advance();
                    Token::new(TokenType::FatArrow, span, "=>")
                } else {
                    Token::new(TokenType::Eq, span, "=")
                }
            }

			// 数字字面量
            _ if c.is_ascii_digit() => self.read_number(start),

            // 标识符和关键字
            _ if is_legal_ident_char(c) => self.read_ident(start),

            // 无法识别
            _ => Token::new(TokenType::Error, Span::new(start, self.pos), &c.to_string()),
        }
    }
}
