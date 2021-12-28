use codespan::{ByteIndex, Span};

pub fn sp(l: ByteIndex, r: ByteIndex) -> Span {
    Span::new(l, r)
}
