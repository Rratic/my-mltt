//! 表面语法

use crate::definitions::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprType {
    // 简单类型论
    Var(Name),
    Lam(Name, Box<Expr>, Box<Expr>),
    App(Name, Box<Expr>, Box<Expr>),

    Anno(Box<Expr>, Box<Expr>), // 类型标注
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub class: ExprType,
    pub span: Span,
}

impl Expr {
    pub fn var(name: Name, span: Span) -> Self {
        Self {
            class: ExprType::Var(name),
            span,
        }
    }
}
