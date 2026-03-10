//! 词法分析

use crate::definitions::Span;
use std::{iter::Peekable, str::CharIndices};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    // 标识符和字面量
    Ident,
    Number,

    // 关键字
    Lambda, // λ

    // 运算符
    Colon,      // :
    Assign,     // :≡
    To,         // ->
    FatArrow,   // =>
    Eq,         // =
    Dot,        // .
    Comma,      // ,
    Product,    // ×
    LeftPar,    // (
    RightPar,   // )
    LeftBrace,  // {
    RightBrace, // }

    // 特殊
    Newline, // \n
    Eof,
    Error,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Ident => write!(f, "identifier"),
            TokenType::Number => write!(f, "number"),
            TokenType::Lambda => write!(f, "'λ'"),
            TokenType::Colon => write!(f, "':'"),
            TokenType::Assign => write!(f, "':≡'"),
            TokenType::To => write!(f, "'->'"),
            TokenType::FatArrow => write!(f, "'=>'"),
            TokenType::Eq => write!(f, "'='"),
            TokenType::Dot => write!(f, "'.'"),
            TokenType::Comma => write!(f, "','"),
            TokenType::Product => write!(f, "'×'"),
            TokenType::LeftPar => write!(f, "'('"),
            TokenType::RightPar => write!(f, "')'"),
            TokenType::LeftBrace => write!(f, "'{{'"),
            TokenType::RightBrace => write!(f, "'}}'"),
            TokenType::Newline => write!(f, "new line"),
            TokenType::Eof => write!(f, "end of file"),
            TokenType::Error => write!(f, "unresolved symbol"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub class: TokenType,
    pub span: Span,
    pub literal: String,
}

impl Token {
    pub fn new(class: TokenType, span: Span, literal: impl Into<String>) -> Self {
        Self {
            class,
            span,
            literal: literal.into(),
        }
    }
}

pub trait TokenStream<'src> {
    /// 获取下一个 Token
    fn next_token(&mut self) -> Token;
}

#[derive(Clone)]
pub struct Lexer<'src> {
    /// 解析的文本
    literal: &'src str,
    /// 字符迭代器
    chars: Peekable<CharIndices<'src>>,
    /// 当前位置
    pos: usize,
    /// 是否经过换行，因为是换行敏感的
    crossed_newline: bool,
    /// 是否跳过了单个 '/' 防止使用 O(n) 的位置设置
    skipped_slash: bool,
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
            crossed_newline: false,
            skipped_slash: false,
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
    // fn restore_pos(&mut self, pos: usize) {
    //     self.chars = self.literal.char_indices().peekable();
    //     while self.chars.peek().is_some_and(|(p, _)| *p < pos) {
    //         self.chars.next();
    //     }
    //     self.pos = pos;
    // }

    /// 跳过空白和注释
    fn skip_whitespace(&mut self) {
        self.crossed_newline = false;
        self.skipped_slash = false;
        loop {
            match self.peek() {
                Some(' ' | '\t' | '\r') => {
                    self.advance();
                }
                Some('\n') => {
                    self.advance();
                    self.crossed_newline = true;
                }
                Some('/') => {
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
                                if self.peek() == Some('/') {
                                    self.advance();
                                    // 不算作经过换行
                                    // 不接受 a : T /* */ a = expr
                                    break;
                                }
                            }
                        }
                    } else {
                        // 不是注释
                        self.skipped_slash = true;
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
            "λ" => TokenType::Lambda,
            _ => TokenType::Ident,
        };

        Token::new(class, span, literal)
    }

    fn read_number(&mut self, start: usize) -> Token {
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
        }

        let literal = &self.literal[start..self.pos];
        let span = Span::new(start, self.pos);

        Token::new(TokenType::Number, span, literal)
    }
}

impl<'src> TokenStream<'src> for Lexer<'src> {
    fn next_token(&mut self) -> Token {
        self.skip_whitespace(); // 跳过空白和注释

        if self.crossed_newline {
            return Token::new(TokenType::Newline, Span::new(self.pos, self.pos), "\n");
        }

        if self.skipped_slash {
            return Token::new(TokenType::Error, Span::new(self.pos - 1, self.pos), "/");
        }

        let start = self.pos;

        let Some(c) = self.advance() else {
            return Token::new(TokenType::Eof, Span::new(start, start), "");
        };

        let span = Span::new(start, self.pos);

        match c {
            // 单字符
            '.' => Token::new(TokenType::Dot, span, "."),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(source: &str) -> Vec<TokenType> {
        let mut lexer = Lexer::new(source);
        std::iter::repeat_with(|| lexer.next_token())
            .take_while(|t| t.class != TokenType::Eof)
            .map(|token| token.class)
            .collect()
    }

    #[test]
    fn test_lexer() {
        assert_eq!(
            tokens("λ (x : T) . E"),
            vec![
                TokenType::Lambda,
                TokenType::LeftPar,
                TokenType::Ident,
                TokenType::Colon,
                TokenType::Ident,
                TokenType::RightPar,
                TokenType::Dot,
                TokenType::Ident,
            ]
        );
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(
            tokens("left /* not \r\n= expr */ right"),
            vec![TokenType::Ident, TokenType::Ident]
        );

        assert_eq!(
            tokens("left / right"),
            vec![TokenType::Ident, TokenType::Error, TokenType::Ident]
        );

        assert_eq!(
            tokens("left // commented \n \t right"),
            vec![TokenType::Ident, TokenType::Newline, TokenType::Ident]
        );
    }
}
