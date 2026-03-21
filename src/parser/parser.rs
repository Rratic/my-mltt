//! 语法解析
//!
//! ```bnf
//! program       ::= decl*
//!
//! decl          ::= signature
//!                 | definition
//!
//! signature     ::= IDENT ':' expr
//!
//! definition    ::= IDENT ':≡' expr
//!
//! expr          ::= 'λ' IDENT '.' expr
//!                 | infix_expr
//!
//! infix_expr    ::= app_expr -> app_expr
//!                 | app_expr × app_expr
//!                 | app_expr = app_expr
//!                 | app_expr
//!
//! app_expr      ::= atom+
//!
//! atom          ::= IDENT | '(' expr ')' | '𝒰' NUMBER
//! ```

use crate::definitions::*;
use crate::parser::lexer::*;
use crate::syntax::surface::*;

#[derive(Debug)]
pub struct ParseError {
    message: String,
    span: Span,
}

pub struct Parser<'src> {
    stream: Lexer<'src>,
    current: Token,
    previous: Token,
    /// 当前 Token 与上一个 Token 之间是否有换行
    at_newline: bool,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        let mut stream = Lexer::new(source);
        let current = stream.next_token();
        Self {
            stream,
            current,
            previous: Token::new(TokenType::Eof, Span::new(0, 0), ""),
            at_newline: false,
        }
    }

    fn check(&self, class: TokenType) -> bool {
        self.current.class == class
    }

    fn advance(&mut self) -> Token {
        self.previous = std::mem::replace(&mut self.current, self.stream.next_token());
        self.at_newline = self.stream.crossed_newline();
        self.previous.clone()
    }

    fn expect(&mut self, class: TokenType) -> Result<Token, ParseError> {
        if self.check(class) {
            Ok(self.advance())
        } else {
            Err(ParseError {
                message: format!("Expected {}, found {}", class, self.current.class),
                span: self.current.span.clone(),
            })
        }
    }

    fn try_match(&mut self, class: TokenType) -> bool {
        if self.check(class) {
            self.advance();
            true
        } else {
            false
        }
    }

    // ============ 解析辅助 ============

    fn parse_name(&mut self) -> Result<Name, ParseError> {
        let token = self.expect(TokenType::Ident)?;
        Ok(Name::with_span(token.literal, token.span))
    }

    // ============ 表达式解析 ============

    pub fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current.span;

        if self.try_match(TokenType::Lambda) {
            let parameter = if self.try_match(TokenType::LeftPar) {
                // λ (x : A) . e
                let mut name = self.parse_name()?;
                self.expect(TokenType::Colon)?;
                let typ = self.parse_expr()?;
                self.expect(TokenType::RightPar)?;
                let span = std::mem::replace(&mut name.span, None);
                ExprType::Anno(Box::new(Expr::var(name, span.unwrap())), Box::new(typ))
            } else {
                // λ x . e
                let name = self.parse_name()?;
                ExprType::Var(name)
            };

            self.expect(TokenType::Dot)?;
            let body = self.parse_expr()?;
            let span = start_span.merge(body.span);

            let expr = Expr::unnamed_func(
                Expr {
                    class: parameter,
                    span,
                },
                body,
                span,
            );
            return Ok(expr);
        }

        self.parse_infix_expr()
    }

    fn parse_infix_expr(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current.span;
        let expr = self.parse_app_expr()?;
        if self.try_match(TokenType::To) {
            let expr2 = self.parse_app_expr()?;
            let span = start_span.merge(expr2.span);
            Ok(Expr {
                class: ExprType::FuncType(Box::new(expr), Box::new(expr2)),
                span,
            })
        } else if self.try_match(TokenType::Product) {
            let expr2 = self.parse_app_expr()?;
            let span = start_span.merge(expr2.span);
            Ok(Expr {
                class: ExprType::PairType(Box::new(expr), Box::new(expr2)),
                span,
            })
        } else if self.try_match(TokenType::Eq) {
            let expr2 = self.parse_app_expr()?;
            let span = start_span.merge(expr2.span);
            Ok(Expr {
                class: ExprType::EqType(Box::new(expr), Box::new(expr2)),
                span,
            })
        } else {
            Ok(expr)
        }
    }

    fn parse_app_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_atom()?;

        while self.is_atom_start() {
            let arg = self.parse_atom()?;
            let span = expr.span.merge(arg.span);
            expr = Expr::app(expr, arg, span);
        }

        Ok(expr)
    }

    fn is_atom_start(&self) -> bool {
        self.check(TokenType::LeftPar) || self.check(TokenType::Ident)
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        if self.try_match(TokenType::LeftPar) {
            let expr = self.parse_expr()?;
            self.expect(TokenType::RightPar)?;
            Ok(expr)
        } else if self.try_match(TokenType::Universe) {
            let number = self.expect(TokenType::Number)?;
            Ok(Expr::universe(number.literal, number.span))
        } else {
            let name = self.parse_name()?;
            Ok(Expr::var(name, self.current.span))
        }
    }

    // ============ 程序解析 ============

    pub fn parse_program(&mut self) -> Result<Vec<Decl>, ParseError> {
        let mut decls = Vec::<Decl>::new();

        while self.current.class != TokenType::Eof {
            decls.push(self.parse_decl()?);
        }

        Ok(decls)
    }

    fn parse_decl(&mut self) -> Result<Decl, ParseError> {
        let start_span = self.current.span;

        let name = self.parse_name()?;

        if self.try_match(TokenType::Colon) {
            let typ = self.parse_expr()?;
            let span = start_span.merge(typ.span);
            Ok(Decl {
                class: DeclType::Sig(name, typ),
                span,
            })
        } else {
            self.expect(TokenType::Assign)?;
            let body = self.parse_expr()?;
            let span = start_span.merge(body.span);
            Ok(Decl {
                class: DeclType::Def(name, body),
                span,
            })
        }
    }
}
