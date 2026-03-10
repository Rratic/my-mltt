/// 源代码位置信息
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(&self, latter: Self) -> Self {
        Self {
            start: self.start,
            end: latter.end,
        }
    }
}

/// 名称信息
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name {
    pub text: String,
    pub span: Option<Span>,
}

impl Name {
    pub fn raw(text: String) -> Self {
        Self { text, span: None }
    }

    pub fn with_span(text: String, span: Span) -> Self {
        Self {
            text,
            span: Some(span),
        }
    }
}
