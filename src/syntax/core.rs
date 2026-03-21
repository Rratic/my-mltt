//! 核心语法

use crate::definitions::*;

// ============ 核心类型 ============

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    // ============ 简单类型论 ============
    Var(i32),     // 使用 de Bruijn 索引
    Global(Name), // 全局变量
    Func(Box<Term>),
    App(Box<Term>, Box<Term>),
    Anno(Box<Term>, Box<Term>),

    // ============ 扩展类型 ============
    Universe(usize),
    FuncType(Box<Term>, Box<Term>),
}

impl Term {
    // ============ 变量替换 ============
    pub fn shift(&self, cutoff: i32, amount: i32) -> Term {
        match self {
            Term::Var(i) => {
                if *i >= cutoff {
                    Term::Var(*i + amount)
                } else {
                    Term::Var(*i)
                }
            }
            Term::Global(name) => Term::Global(name.clone()),
            Term::Func(body) => Term::Func(Box::new(body.shift(cutoff + 1, amount))),
            Term::App(func, arg) => Term::App(
                Box::new(func.shift(cutoff, amount)),
                Box::new(arg.shift(cutoff, amount)),
            ),
            Term::Anno(term, ty) => Term::Anno(
                Box::new(term.shift(cutoff, amount)),
                Box::new(ty.shift(cutoff, amount)),
            ),
            Term::Universe(level) => Term::Universe(*level),
            Term::FuncType(dom, codom) => Term::FuncType(
                Box::new(dom.shift(cutoff, amount)),
                Box::new(codom.shift(cutoff, amount)),
            ),
        }
    }

    pub fn subst(&self, index: i32, term: &Term) -> Term {
        match self {
            Term::Var(i) => match (*i).cmp(&index) {
                std::cmp::Ordering::Equal => term.clone(),
                std::cmp::Ordering::Greater => Term::Var(*i - 1),
                std::cmp::Ordering::Less => Term::Var(*i),
            },
            Term::Global(name) => Term::Global(name.clone()),
            Term::Func(body) => {
                let shifted = term.shift(0, 1);
                Term::Func(Box::new(body.subst(index + 1, &shifted)))
            }
            Term::App(func, arg) => Term::App(
                Box::new(func.subst(index, term)),
                Box::new(arg.subst(index, term)),
            ),
            Term::Anno(term, ty) => Term::Anno(
                Box::new(term.subst(index, term)),
                Box::new(ty.subst(index, term)),
            ),
            Term::Universe(level) => Term::Universe(*level),
            Term::FuncType(dom, codom) => Term::FuncType(
                Box::new(dom.subst(index, term)),
                Box::new(codom.subst(index, term)),
            ),
        }
    }
}

impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Var(i) => write!(f, "#{}", i),
            Term::Global(name) => write!(f, "{}", name.text),
            Term::Func(body) => write!(f, "(λ. {})", body),
            Term::App(func, arg) => write!(f, "({} {})", func, arg),
            Term::Anno(term, ty) => write!(f, "({}: {})", term, ty),
            Term::Universe(level) => write!(f, "𝒰 {}", level),
            Term::FuncType(dom, codom) => write!(f, "({} -> {})", dom, codom),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shift() {
        // λ. #0
        let t = Term::Func(Box::new(Term::Var(0)));
        assert_eq!(t.shift(0, 1), Term::Func(Box::new(Term::Var(0))));

        // #0
        let t = Term::Var(0);
        assert_eq!(t.shift(0, 1), Term::Var(1));
    }

    #[test]
    fn test_subst() {
        // (λ. #0) with #0 :≡ y
        let t = Term::Func(Box::new(Term::Var(0)));
        let s = Term::Var(42);
        assert_eq!(t.subst(0, &s), Term::Func(Box::new(Term::Var(0))));

        // #0 with #0 :≡ y
        let t = Term::Var(0);
        let s = Term::Var(42);
        assert_eq!(t.subst(0, &s), Term::Var(42));
    }
}
