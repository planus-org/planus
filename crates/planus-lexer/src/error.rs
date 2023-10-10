use codespan::Span;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LexicalError {
    pub err: String,
    pub span: Span,
}

impl std::fmt::Display for LexicalError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.span.start().0 == 0 && self.span.end().0 == 0 {
            write!(fmt, "{}", self.err)
        } else {
            write!(fmt, "{} at {:?}", self.err, self.span)
        }
    }
}

impl LexicalError {
    pub fn new<E: ToString>(err: E, span: Span) -> Self {
        LexicalError {
            err: err.to_string(),
            span,
        }
    }
}
