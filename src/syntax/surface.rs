//! 表面语法

use crate::definitions::*;

// ============ 表达式类型 ============

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprType {
    // 简单类型论
    Var(Name),
    Func(Name, Box<Expr>, Box<Expr>),
    App(Box<Expr>, Box<Expr>),

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

    pub fn unnamed_func(parameter: Expr, body: Expr, span: Span) -> Self {
        Self {
            class: ExprType::Func(Name::raw("_".into()), Box::new(parameter), Box::new(body)),
            span: span,
        }
    }

    pub fn app(func: Expr, arg: Expr, span: Span) -> Self {
        Self {
            class: ExprType::App(Box::new(func), Box::new(arg)),
            span,
        }
    }
}

// ============ 顶层声明 ============

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclType {
    /// 类型签名
    Sig(Name, Expr),
    /// 定义
    Def(Name, Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decl {
    pub class: DeclType,
    pub span: Span,
}
