//! 语法解析
//!
//! ```bnf
//! program       ::= decl*
//!
//! decl          ::= signature
//!                 | definition
//!                 | typed_def
//!
//! signature     ::= IDENT ':' expr
//!
//! definition    ::= IDENT ':≡' expr
//!
//! typed_def     ::= IDENT ':' expr ':≡' expr
//!
//! expr          ::= 'λ' IDENT '.' expr
//!                 | infix_expr
//!
//! infix_expr    ::= app_expr
//!                 | app_expr -> app_expr
//!
//! app_expr      ::= atom+
//!
//! atom          ::= IDENT | '(' expr ')'
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
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        let mut stream = Lexer::new(source);
        let current = stream.next_token();
        Self {
            stream,
            current,
            previous: Token::new(TokenType::Eof, Span::new(0, 0), ""),
        }
    }

    fn check(&self, class: TokenType) -> bool {
        self.current.class == class
    }

    fn advance(&mut self) -> Token {
        self.previous = std::mem::replace(&mut self.current, self.stream.next_token());
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

    // Basic parsing.

    fn parse_name(&mut self) -> Result<Name, ParseError> {
        let token = self.expect(TokenType::Ident)?;
        Ok(Name::with_span(token.literal, token.span))
    }

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

            let class = ExprType::Lam(
                Name::raw("_".into()),
                Box::new(Expr {
                    class: parameter,
                    span,
                }),
                Box::new(body),
            );
            return Ok(Expr { class, span });
        }

        let name = self.parse_name()?;
        return Ok(Expr::var(name, start_span));
    }
}
