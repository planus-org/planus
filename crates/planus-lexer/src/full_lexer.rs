use codespan::{ByteIndex, Span};
use logos::Logos;

use super::{
    raw_lexer::{CommentKind, Token},
    text_lexer::Text,
};
use crate::error::LexicalError;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Comment<'input> {
    pub span: Span,
    pub kind: CommentKind,
    pub content: &'input str,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CommentBlock<'a>(pub Vec<Comment<'a>>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenMetadata<'a> {
    pub pre_comment_blocks: Vec<CommentBlock<'a>>,
    pub token_begins_paragraph: bool,
    pub post_comment: Option<Comment<'a>>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct TokenWithMetadata<'a>(pub Token<'a>, pub TokenMetadata<'a>);

impl<'a> std::fmt::Debug for TokenWithMetadata<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct Lexer<'input> {
    // Ideally this should not need to be an Option, and
    // will always be Some outside of the `with_text_lexer`
    // function. The reason it needs to be an options, is
    // because the `morph` function from logos requires an
    // owned value.
    lex: Option<logos::Lexer<'input, Token<'input>>>,

    // We sometimes need to be able to take a peek at a token
    // and put it back if we do not want it. We cannot simply
    // use Peekable, because it interferes with our ability
    // to morph the Lexer.
    saved_token: Option<Token<'input>>,

    end_of_stream_reached: bool,
    pre_comment_blocks: Vec<CommentBlock<'input>>,
    current_comment_block: CommentBlock<'input>,
}

impl<'input> Lexer<'input> {
    pub fn new(s: &'input str) -> Self {
        Self {
            lex: Some(Token::lexer(s)),
            saved_token: None,
            end_of_stream_reached: false,
            pre_comment_blocks: Vec::new(),
            current_comment_block: CommentBlock::default(),
        }
    }

    fn with_text_lexer<F, R>(&mut self, f: F) -> R
    where
        for<'a> F: FnOnce(&'a mut logos::Lexer<'input, Text>) -> R,
    {
        let mut lex = self.lex.take().unwrap().morph();
        let result = f(&mut lex);
        self.lex = Some(lex.morph());
        result
    }

    fn consume_string_token(&mut self) -> Result<(Span, Token<'input>), LexicalError> {
        let end_quote_type = self.lex().slice().chars().next().unwrap();
        let mut error = None;
        let start = self.span().start();
        let out = self.with_text_lexer(|lex| Text::run_lexer(lex, end_quote_type, &mut error));
        let end = self.span().end();
        if let Some(e) = error {
            Err(e)
        } else {
            Ok((Span::new(start, end), Token::StringLiteral(out)))
        }
    }

    fn lex(&mut self) -> &mut logos::Lexer<'input, Token<'input>> {
        self.lex.as_mut().unwrap()
    }

    fn span(&self) -> Span {
        let span = self.lex.as_ref().unwrap().span();
        Span::new(span.start as u32, span.end as u32)
    }

    fn next_raw_token(&mut self) -> Option<Token<'input>> {
        if let Some(token) = self.saved_token.take() {
            Some(token)
        } else {
            self.lex().next()
        }
    }

    fn next_token(&mut self) -> Option<Result<(Span, bool, Token<'input>), LexicalError>> {
        let mut newlines = 0;
        loop {
            let token_begins_paragraph = newlines >= 2;
            let saved_newlines = newlines;
            newlines = 0;

            return match self.next_raw_token() {
                Some(Token::Newline) => {
                    newlines = saved_newlines + 1;
                    continue;
                }
                Some(Token::Comment(c)) => {
                    if token_begins_paragraph && !self.current_comment_block.0.is_empty() {
                        self.pre_comment_blocks
                            .push(std::mem::take(&mut self.current_comment_block));
                    }

                    self.current_comment_block.0.push(Comment {
                        span: self.span(),
                        kind: c.kind,
                        content: c.content,
                    });
                    continue;
                }
                Some(Token::UnexpectedToken) => {
                    let err = format!("Unknown token {}", self.lex().slice());
                    Some(Err(LexicalError::new(err, self.span())))
                }
                Some(Token::StringLiteral(_)) => Some(
                    self.consume_string_token()
                        .map(|(span, t)| (span, token_begins_paragraph, t)),
                ),
                Some(t) => Some(Ok((self.span(), token_begins_paragraph, t))),
                None if !self.end_of_stream_reached => {
                    self.end_of_stream_reached = true;
                    Some(Ok((
                        self.span(),
                        token_begins_paragraph,
                        Token::EndOfStream,
                    )))
                }
                None => None,
            };
        }
    }

    fn next_post_comment(&mut self) -> Option<Comment<'input>> {
        match self.next_raw_token()? {
            Token::Comment(c) => Some(Comment {
                span: self.span(),
                kind: c.kind,
                content: c.content.trim_end(),
            }),
            token => {
                self.saved_token = Some(token);
                None
            }
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<(ByteIndex, TokenWithMetadata<'input>, ByteIndex), LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token()? {
            Ok((span, token_begins_paragraph, token)) => {
                let mut pre_comment_blocks = std::mem::take(&mut self.pre_comment_blocks);
                if !self.current_comment_block.0.is_empty() {
                    pre_comment_blocks.push(std::mem::take(&mut self.current_comment_block));
                }
                let metadata = TokenMetadata {
                    pre_comment_blocks,
                    token_begins_paragraph,
                    post_comment: self.next_post_comment(),
                };
                Some(Ok((
                    span.start(),
                    TokenWithMetadata(token, metadata),
                    span.end(),
                )))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer() {
        let tokenizer = Lexer::new(
            r#"
            "foo\x41\n\r\t\b\f\"\'\/"
            "\k"
            "\uffff"
            "
            ""#,
        );
        let tokens = tokenizer
            .map(|v| v.map(|(_start, tok, _end)| tok))
            .collect::<Vec<_>>();

        for token in tokens {
            println!("{token:?}");
        }
    }
}
