use crate::error::LexicalError;
use codespan::Span;
use logos::Logos;
use std::convert::TryInto;

#[derive(Logos, Debug, PartialEq)]
pub(crate) enum Text {
    #[error]
    Error,
    // TODO: Implement unicode support
    #[regex(r#"[^"'\\]+"#)]
    Text,
    #[regex(r"\\.")]
    EscapeCharacter,
    #[regex(r"\\u[0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F]")]
    Codepoint,
    #[regex(r"\\x[0-7][0-9a-fA-F]")]
    Byte,
    #[token("\"")]
    #[token("\'")]
    Quote,
}

impl Text {
    pub(crate) fn run_lexer(
        lex: &mut logos::Lexer<'_, Self>,
        end_quote_type: char,
        error: &mut Option<LexicalError>,
    ) -> String {
        let mut out = String::new();
        while let Some(token) = lex.next() {
            let span = lex.span();
            let span = Span::new(span.start as u32, span.end as u32);
            match token {
                Text::Text => out += lex.slice(),
                Text::Byte => out.push(u8::from_str_radix(&lex.slice()[2..], 16).unwrap() as char),
                Text::Codepoint => {
                    let codepoint = u32::from_str_radix(&lex.slice()[2..], 16).unwrap();
                    // TODO: handle surrogate pairs
                    match codepoint.try_into() {
                        Ok(c) => out.push(c),
                        Err(_) => {
                            error.get_or_insert(LexicalError::new(
                                "Codepoint escape does not correspond to a valid character",
                                span,
                            ));
                        }
                    }
                }
                Text::EscapeCharacter => match &lex.slice()[1..] {
                    "n" => out.push('\n'),
                    "t" => out.push('\t'),
                    "r" => out.push('\r'),
                    "b" => out.push('\x08'),
                    "f" => out.push('\x0c'),
                    "\"" => out.push('"'),
                    "'" => out.push('\''),
                    "/" => out.push('/'),
                    _ => {
                        error.get_or_insert(LexicalError::new("Invalid escape sequence", span));
                    }
                },
                Text::Quote => {
                    let cur_quote_type = lex.slice().chars().next().unwrap();
                    if cur_quote_type == end_quote_type {
                        return out;
                    } else {
                        out.push(cur_quote_type);
                    }
                }
                Text::Error => {
                    error.get_or_insert(LexicalError::new("Invalid string", span));
                }
            }
        }
        let span = lex.span();
        let span = Span::new(span.start as u32, span.end as u32);
        error.get_or_insert(LexicalError::new("Unexpected end of string", span));
        out
    }
}

// TODO: tests
