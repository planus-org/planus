mod error;
mod full_lexer;
mod raw_lexer;
mod text_lexer;

pub use error::LexicalError;
pub use full_lexer::{Comment, CommentBlock, Lexer, TokenMetadata, TokenWithMetadata};
pub use raw_lexer::{CommentKind, Keyword, Symbol, Token};
