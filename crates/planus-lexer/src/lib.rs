//! Library for lexing flatbuffer files.
//!
//! This library is an internal implementation
//! detail of [planus-cli](https://docs.rs/planus-cli).
//!
//! Feel free to use it, however there are no stability guarantees.

mod error;
mod full_lexer;
mod raw_lexer;
mod text_lexer;

use codespan::ByteIndex;
pub use error::LexicalError;
pub use full_lexer::{Comment, CommentBlock, TokenMetadata, TokenWithMetadata};
pub use raw_lexer::{CommentKind, Keyword, Symbol, Token};

pub fn lexer(
    s: &str,
) -> impl Iterator<Item = Result<(ByteIndex, TokenWithMetadata<'_>, ByteIndex), LexicalError>> {
    full_lexer::Lexer::new(s)
}
